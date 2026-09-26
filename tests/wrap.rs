// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for line wrapping, and in particular for the pending wrap state that
//! a terminal is in right after a char gets written into the last column.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion, Size, Term};

// Writing into the bottom right cell must not scroll. Full screen programs
// draw their status line all the way to the last column.
frag! {
    alt_screen_bottom_right_cell_does_not_scroll { scrollback_lines: 100, width: 3, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("abc"), term::Crlf::default(),
       term::Raw::from("def")
    => ContentRegion::Screen =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("def"),
            term::ControlCodes::cursor_position(2, 3),
            term::Raw::from("f"),
            term::control_codes().clear_attrs
}

frag! {
    crlf_after_a_full_line_does_not_add_a_blank_line { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"), term::Crlf::default(),
       term::Raw::from("d")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// The cursor is still on the last column, so backspace moves it to the one
// before that.
frag! {
    backspace_cancels_a_pending_wrap { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc\x08X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("aXc"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    cursor_movement_cancels_a_pending_wrap { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_forward(1),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abX"),
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("X"),
            term::control_codes().clear_attrs
}

// Attributes don't move the cursor, so they leave the wrap pending.
frag! {
    sgr_keeps_a_pending_wrap { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"),
       term::control_codes().bold,
       term::Raw::from("d")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::control_codes().bold,
            term::Raw::from("d"),
            term::control_codes().reset_font_weight,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::control_codes().bold
}

// Wrapping on the bottom row of the scroll region scrolls the region, not
// the whole screen.
frag! {
    wrap_at_the_bottom_of_the_scroll_region { scrollback_lines: 100, width: 3, height: 4 }
    <= term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4"),
       term::ControlCodes::set_scroll_region(2, 3),
       term::ControlCodes::cursor_position(3, 1),
       term::Raw::from("abcd")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("d"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

// Below the scroll region on the last row there is nowhere to wrap to, so
// the cursor goes back to the start of the same row.
frag! {
    wrap_below_the_scroll_region_on_the_last_row { scrollback_lines: 100, width: 3, height: 3 }
    <= term::ControlCodes::set_scroll_region(1, 2),
       term::ControlCodes::cursor_position(3, 1),
       term::Raw::from("abcd")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("dbc"),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

frag! {
    save_and_restore_cursor_keep_a_pending_wrap { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"),
       term::control_codes().save_cursor,
       term::ControlCodes::cursor_position(3, 1),
       term::control_codes().restore_cursor,
       term::Raw::from("d")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("d"),
            // The saved cursor still has the wrap pending.
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("c"),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// A combining mark describes the char the cursor is sitting on.
frag! {
    combining_mark_after_the_last_column { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abe\u{301}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abe\u{301}"),
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("e\u{301}"),
            term::control_codes().clear_attrs
}

// The reprint that restores the pending wrap needs the attrs of the cell it
// prints, and has to leave the attrs as it found them.
frag! {
    pending_wrap_restore_reprints_attrs { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("ab"),
       term::control_codes().italic,
       term::Raw::from("c"),
       term::control_codes().clear_attrs
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab"),
            term::control_codes().italic,
            term::Raw::from("c"),
            term::control_codes().undo_italic,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().italic,
            term::Raw::from("c"),
            term::control_codes().undo_italic,
            term::control_codes().clear_attrs
}

// Setting a tab stop with the cursor on the last column used to index past
// the end of the tab stops, since the cursor was parked one column past the
// edge of the screen.
frag! {
    tab_set_on_the_last_column { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"),
       term::control_codes().horizontal_tab_set
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().horizontal_tab_set,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("c"),
            term::control_codes().clear_attrs
}

/// Feed `input` to a terminal, restore its contents into a fresh terminal,
/// then feed `then` to both and check that they still agree.
fn assert_restores(size: Size, input: &[u8], then: &[u8]) {
    let mut term = Term::new(100, size);
    term.process(input);

    let mut restored = Term::new(100, size);
    restored.process(term.contents(ContentRegion::All).as_slice());

    term.process(then);
    restored.process(then);
    assert_eq!(
        String::from_utf8_lossy(restored.contents(ContentRegion::All).as_slice()),
        String::from_utf8_lossy(term.contents(ContentRegion::All).as_slice()),
    );
}

#[test]
fn pending_wrap_survives_a_restore() {
    assert_restores(Size { width: 3, height: 3 }, b"abc", b"d");
}

#[test]
fn pending_wrap_after_a_wide_char_survives_a_restore() {
    assert_restores(Size { width: 3, height: 3 }, "a😊".as_bytes(), b"d");
}

#[test]
fn pending_wrap_on_the_alt_screen_survives_a_restore() {
    assert_restores(Size { width: 3, height: 2 }, b"\x1b[?1049habc\r\ndef", b"g");
}

/// The contents of a terminal of the given size after processing `input`.
fn contents_after(size: Size, input: &[u8]) -> Vec<u8> {
    let mut term = Term::new(100, size);
    term.process(input);
    term.contents(ContentRegion::All)
}

// After a widening resize the line is no longer full, so the next char goes
// right after it.
#[test]
fn widening_resize_cancels_a_pending_wrap() {
    let mut term = Term::new(100, Size { width: 5, height: 3 });
    term.process(b"abcde");
    term.resize(Size { width: 10, height: 3 });
    term.process(b"f");

    assert_eq!(
        term.contents(ContentRegion::All),
        contents_after(Size { width: 10, height: 3 }, b"abcdef")
    );
}

// Reflowing a full line onto rows that are full too leaves the cursor
// waiting to wrap at the end of the last of them.
#[test]
fn narrowing_resize_keeps_a_pending_wrap() {
    let mut term = Term::new(100, Size { width: 4, height: 3 });
    term.process(b"abcd");
    term.resize(Size { width: 2, height: 3 });
    term.process(b"e");

    assert_eq!(
        term.contents(ContentRegion::All),
        contents_after(Size { width: 2, height: 3 }, b"abcde")
    );
}

// With autowrap (DECAWM) off, chars that run into the right edge keep
// overwriting the last column.
frag! {
    autowrap_off_overwrites_the_last_column { scrollback_lines: 100, width: 3, height: 3 }
    <= term::control_codes().disable_autowrap,
       term::Raw::from("abcde")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abe"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().disable_autowrap
}

frag! {
    autowrap_off_squeezes_a_wide_char_in_at_the_end { scrollback_lines: 100, width: 3, height: 3 }
    <= term::control_codes().disable_autowrap,
       term::Raw::from("abc😊")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a😊"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().disable_autowrap
}

frag! {
    autowrap_turned_off_with_a_wrap_pending { scrollback_lines: 100, width: 3, height: 3 }
    <= term::Raw::from("abc"),
       term::control_codes().disable_autowrap,
       term::Raw::from("d")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abd"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().disable_autowrap
}

frag! {
    autowrap_turned_back_on { scrollback_lines: 100, width: 3, height: 3 }
    <= term::control_codes().disable_autowrap,
       term::control_codes().enable_autowrap,
       term::Raw::from("abcd")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    decstr_turns_autowrap_back_on { scrollback_lines: 100, width: 3, height: 3 }
    <= term::control_codes().disable_autowrap,
       term::control_codes().soft_reset,
       term::Raw::from("abcd")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}
