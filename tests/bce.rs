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

//! Background color erase (bce). Erasing, inserting blanks and scrolling
//! leave blanks behind that are painted with the current background color,
//! and nothing but the background color.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion, Size, Term};

/// A run of `n` blanks painted blue, the color all of these tests erase with.
fn blue(n: usize) -> term::Raw {
    term::Raw::from(format!("\x1b[44m{}\x1b[49m", " ".repeat(n)).as_str())
}

frag! {
    el_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_to_end_of_line
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            blue(4),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    el_to_start_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abcd"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_to_start_of_line
    => ContentRegion::All =>
            reset_codes,
            blue(3),
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    el_whole_line_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_line
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

// Nothing is stored for a row that has never been written to, but a painted
// row has to be.
frag! {
    el_below_content_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_line
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

// Only the background color carries over. The blanks don't get underlined
// just because underline was on when they got erased.
frag! {
    el_only_takes_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::fgcolor_idx(1),
       term::control_codes().underline,
       term::control_codes().inverse,
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_to_end_of_line
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            blue(4),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::Raw::from("\x1b[31;44;4;7m")
}

frag! {
    el_without_bgcolor_leaves_plain_blanks { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::fgcolor_idx(1),
       term::control_codes().erase_to_end_of_line
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::fgcolor_idx(1)
}

frag! {
    ed_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(2, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_to_end
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::Crlf::default(),
            term::Raw::from(" "),
            blue(4),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    ed_from_start_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::cursor_position(2, 3),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_from_start
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::Crlf::default(),
            blue(3),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

// This is how full screen programs paint their background.
frag! {
    ed_whole_screen_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::Crlf::default(),
            blue(5),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    ech_below_content_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::cursor_position(2, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::erase_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from(" "),
            blue(2),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    ich_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::insert_character(2)
    => ContentRegion::All =>
            reset_codes,
            blue(2),
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    il_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::Crlf::default(),
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    dl_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("b"),
            term::Crlf::default(),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    dl_in_scroll_region_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb\r\nc"),
       term::ControlCodes::set_scroll_region(1, 2),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("b"),
            term::Crlf::default(),
            blue(5),
            term::Crlf::default(),
            term::Raw::from("c"),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    su_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb\r\nc"),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::Crlf::default(),
            term::Raw::from("c"),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    sd_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb\r\nc"),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::scroll_down(1)
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::Crlf::default(),
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    linefeed_scroll_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb\r\nc"),
       term::ControlCodes::bgcolor_idx(4),
       term::Raw::from("\n")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::Crlf::default(),
            term::Raw::from("c"),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    reverse_index_scroll_paints_bgcolor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("a\r\nb"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().reverse_index
    => ContentRegion::All =>
            reset_codes,
            blue(5),
            term::Crlf::default(),
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

// Wrapping onto a new line at the bottom of the screen scrolls just like LF.
frag! {
    wrap_scroll_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::Raw::from("abcdef")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("\x1b[44mabcde\x1b[49m"),
            term::Crlf::default(),
            term::Raw::from("\x1b[44mf    \x1b[49m"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_ed_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            blue(5),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_el_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("abc"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::control_codes().erase_to_end_of_line
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("a"),
            blue(4),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_scroll_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("a\r\nb"),
       term::ControlCodes::bgcolor_idx(4),
       term::Raw::from("\n")
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("b"),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_il_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("a"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            blue(5),
            term::Crlf::default(),
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_dl_paints_bgcolor { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("a\r\nb"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("b"),
            term::Crlf::default(),
            blue(5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

/// Run `input` through a term of `size`, resize it to `new_size` and return
/// what it dumps.
fn resized_dump(size: Size, input: &[u8], new_size: Size) -> Vec<u8> {
    let mut term = Term::new(100, size);
    term.process(input);
    term.resize(new_size);
    term.contents(ContentRegion::All)
}

/// What a term of `size` dumps after running `input`.
fn dump(size: Size, input: &[u8]) -> Vec<u8> {
    let mut term = Term::new(100, size);
    term.process(input);
    term.contents(ContentRegion::All)
}

// A line painted out to the right edge stays painted out to the edge at the
// new width, rather than getting chopped up into extra rows of blanks.
#[test]
fn painted_line_reaches_the_edge_when_narrowed() {
    let got = resized_dump(
        Size { width: 10, height: 3 },
        b"ab\x1b[44m\x1b[K",
        Size { width: 5, height: 3 },
    );
    assert_eq!(got, dump(Size { width: 5, height: 3 }, b"ab\x1b[44m\x1b[K"));
}

#[test]
fn painted_line_reaches_the_edge_when_widened() {
    let got = resized_dump(
        Size { width: 5, height: 3 },
        b"ab\x1b[44m\x1b[K",
        Size { width: 10, height: 3 },
    );
    assert_eq!(got, dump(Size { width: 10, height: 3 }, b"ab\x1b[44m\x1b[K"));
}

#[test]
fn painted_screen_keeps_its_rows_when_narrowed() {
    let got = resized_dump(
        Size { width: 10, height: 3 },
        b"\x1b[44m\x1b[2J",
        Size { width: 5, height: 3 },
    );
    assert_eq!(got, dump(Size { width: 5, height: 3 }, b"\x1b[44m\x1b[2J"));
}

// Blanks that only cover part of the line are content like any other, and
// don't spread when the line gets wider.
#[test]
fn partly_painted_line_keeps_its_blanks_when_widened() {
    let got = resized_dump(
        Size { width: 5, height: 3 },
        b"ab\x1b[44m\x1b[2X",
        Size { width: 10, height: 3 },
    );
    assert_eq!(got, dump(Size { width: 10, height: 3 }, b"ab\x1b[44m\x1b[2X"));
}

#[test]
fn bgcolor_blanks_survive_a_restore() {
    let size = Size { width: 10, height: 4 };
    let mut session = Term::new(100, size);
    session.process(b"\x1b[42m\x1b[2J\x1b[Hstatus\x1b[44m\x1b[K\r\n\x1b[41m\x1b[2X\x1b[m$ ");

    let mut client = Term::new(100, size);
    client.process(&session.contents(ContentRegion::All));

    assert_eq!(client.contents(ContentRegion::All), session.contents(ContentRegion::All));
}
