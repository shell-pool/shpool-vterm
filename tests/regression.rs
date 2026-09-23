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

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

frag! {
    reverse_index_partial_screen { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"),
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().reverse_index,
       term::Raw::from("XX")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("XX"),
            term::Crlf::default(),
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    insert_lines_partial_screen { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_lines(1),
       term::Raw::from("XX")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("XX"),
            term::Crlf::default(),
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    delete_lines_partial_screen { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

// DCH (Delete Character) with the cursor to the right of the last cell that
// has actually been written.
//
// A line only stores cells out to the last one written; trailing blanks are
// implicit. That makes `min(cells.len(), col + n) - col` in delete_character
// underflow whenever the cursor is parked past the end of a short line, which
// CUF, CHA and CUP all manage without writing anything. The sibling ops
// (insert_character, erase_character) guard against it by clamping to width
// and padding up to col first.

// Control case: the cursor sits exactly on cells.len(), the rightmost
// position that cannot underflow. It anchors the expected output of the cases
// below, showing that DCH pads the line out to the full width even when it
// has nothing real to delete.
frag! {
    delete_character_at_end_of_line { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("ab"),
       term::ControlCodes::delete_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab   "),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    delete_character_past_end_of_line_via_cursor_forward
        { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("ab"),
       term::ControlCodes::cursor_forward(2),
       term::ControlCodes::delete_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab   "),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    delete_character_past_end_of_line_via_horizontal_absolute
        { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("ab"),
       term::ControlCodes::cursor_horizontal_absolute(5),
       term::ControlCodes::delete_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab   "),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    delete_character_past_end_of_line_via_cursor_position
        { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("ab"),
       term::ControlCodes::cursor_position(1, 5),
       term::ControlCodes::delete_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab   "),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    delete_character_past_end_of_line_second_row
        { scrollback_lines: 100, width: 5, height: 10 }
    <= term::Raw::from("ab"), term::Crlf::default(),
       term::Raw::from("cd"),
       term::ControlCodes::cursor_position(2, 5),
       term::ControlCodes::delete_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab"),
            term::Crlf::default(),
            term::Raw::from("cd   "),
            term::ControlCodes::cursor_position(2, 5),
            term::control_codes().clear_attrs
}

//
// Zero width codepoints.
//

// Control case: the precomposed form of the same glyph, which takes the
// ordinary width-1 path today. It anchors the expected rendering of
// combining_mark_after_ascii below.
frag! {
    precomposed_char_needs_no_modifier { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\u{e9}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("\u{e9}"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// 'e' followed by a combining acute accent. One cell, one column.
frag! {
    combining_mark_after_ascii { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("e\u{301}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("e\u{301}"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// VS16 requests the emoji presentation of the preceding symbol. We keep the
// base char's width here rather than promoting the cell to width 2: terminals
// disagree about the promotion, and getting the bytes back out intact is what
// session restore actually needs.
frag! {
    variation_selector_after_symbol { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\u{2714}\u{fe0f}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("\u{2714}\u{fe0f}"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// A ZWJ sequence. The joiner attaches to the cell before it, and the emoji
// after the joiner starts a fresh width-2 cell, so the pair spans 4 columns.
frag! {
    zwj_emoji_sequence { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\u{1f9d1}\u{200d}\u{1f4bb}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("\u{1f9d1}\u{200d}\u{1f4bb}"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

// A combining mark with no cell to modify. There is nothing sensible to
// attach it to, so it gets dropped, but it must not take the session down.
frag! {
    combining_mark_with_no_preceding_cell { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\u{301}")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// Same, but the cursor has moved to a fresh row. The mark must not reach back
// up to the last cell of the previous line.
frag! {
    combining_mark_at_start_of_line { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ab"), term::Crlf::default(),
       term::Raw::from("\u{301}")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

//
// Scroll region bottom past the last row of the screen.
//
// DECSTBM stores the requested bottom verbatim, so `CSI 1;5r` on a three row
// screen leaves a region extending past the grid. LF only scrolls once the
// cursor reaches the bottom of the region, so from the last row it walks the
// cursor off the screen instead. Real terminals clamp the bottom to the
// screen height, which is what these tests assert.
//
// With the cursor below the last row, `bottom - cursor.row` underflows in
// both Scrollback::insert_lines and Scrollback::delete_lines. The SU is
// needed to get there: it pushes the grid start up into the scrollback so the
// off screen rows still resolve to lines, otherwise both bail out early.

// Control case: an in-range bottom, where LF scrolls and the cursor stays on
// the last row. Anchors the expected output of the three cases below.
frag! {
    scroll_region_bottom_at_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"), term::Crlf::default(),
       term::Raw::from("dd"), term::Crlf::default(),
       term::Raw::from("ee"),
       term::ControlCodes::scroll_up(2),
       term::ControlCodes::set_scroll_region(1, 3),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::control_codes().unset_scroll_region
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

// An out of range bottom must behave exactly like the control. Today nothing
// scrolls and the restore buffer ends with a CUP to row 5 of a three row
// screen, which is the mid screen cursor users see on reattach.
frag! {
    scroll_region_bottom_past_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"), term::Crlf::default(),
       term::Raw::from("dd"), term::Crlf::default(),
       term::Raw::from("ee"),
       term::ControlCodes::scroll_up(2),
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::control_codes().unset_scroll_region
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

// IL with the cursor below the last row. Once the cursor is back on the
// screen the insert has nothing to push down, so the output matches the
// control.
frag! {
    insert_lines_with_cursor_past_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"), term::Crlf::default(),
       term::Raw::from("dd"), term::Crlf::default(),
       term::Raw::from("ee"),
       term::ControlCodes::scroll_up(2),
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::control_codes().unset_scroll_region,
       term::ControlCodes::insert_lines(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

// The same for DL, which has its own copy of the underflowing arithmetic.
frag! {
    delete_lines_with_cursor_past_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"), term::Crlf::default(),
       term::Raw::from("dd"), term::Crlf::default(),
       term::Raw::from("ee"),
       term::ControlCodes::scroll_up(2),
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::control_codes().unset_scroll_region,
       term::ControlCodes::delete_lines(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

// The alt screen keeps its grid in a buffer exactly one screen tall, so an out
// of range bottom indexes off the end of it rather than underflowing. The
// expected output here is what the in-range `CSI 1;3r` produces.
frag! {
    alt_screen_scroll_region_past_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"),
       term::ControlCodes::set_scroll_region(1, 9),
       term::ControlCodes::scroll_down(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(1, 3),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

// Clamping the bottom can pull it up to or above the top, leaving a region
// that describes no rows at all. There is nothing to scroll, so it is dropped.
frag! {
    scroll_region_clamped_away { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::set_scroll_region(5, 9)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// A shrinking resize strands a region that was in range when it was set, so
// the clamp has to be reapplied there too. shpool resizes the spool on every
// reattach, which is how a long lived session gets into this state.
#[test]
fn scroll_region_clamped_on_shrinking_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::ControlCodes::set_scroll_region(1, 6).term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 6 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::ControlCodes::set_scroll_region(1, 3).term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Same, on the alt screen, where the stranded region also takes the session
// down on the next scroll.
#[test]
fn alt_screen_scroll_region_clamped_on_shrinking_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::control_codes().enable_alt_screen.term_input_into(&mut input);
    term::ControlCodes::set_scroll_region(1, 6).term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 6 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut scroll = vec![];
    term::ControlCodes::scroll_down(1).term_input_into(&mut scroll);
    term.process(scroll.as_slice());

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    crate::support::frag::empty_scrollback.term_input_into(&mut expected);
    term::control_codes().enable_alt_screen.term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::ControlCodes::set_scroll_region(1, 3).term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Enabling origin mode homes the cursor, so it has to be restored before the
// cursor position, which is then relative to the top of the scroll region.
frag! {
    origin_mode_restored_before_cursor { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Raw::from("22222"),
            term::Crlf::default(),
            term::Raw::from("33333"),
            term::Crlf::default(),
            term::Raw::from("44444"),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

// A dump has to land correctly on a terminal in any state, so the prefix
// clears margins and origin mode before it homes the cursor and erases.
//
// Spelled out rather than built from `reset_codes` so that the helper cannot
// quietly track a regression here.
#[test]
fn dump_prefix_clears_scroll_region_and_origin_mode() {
    use shpool_vterm::term::AsTermInput;

    let mut expected = vec![];
    term::control_codes().end_link.term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);
    term::control_codes().unset_scroll_region.term_input_into(&mut expected);
    term::control_codes().disable_scroll_region_origin_mode.term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_screen.term_input_into(&mut expected);
    term::Raw::from("hi").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    crate::support::frag::round_trip_frag(
        b"hi",
        expected.as_slice(),
        100,
        shpool_vterm::Size { width: 5, height: 3 },
        ContentRegion::All,
    );
}

// Reflow rebuilds the scrollback out of logical lines, and a blank line is a
// logical line with nothing in it. Dropping those silently deletes rows out of
// the middle of the screen and slides everything below them up, while the
// cursor stays on the row it was already on.
//
// shpool resizes the spool on every reattach whether or not the size actually
// changed, so a resize that does nothing still has to leave the dump alone.
#[test]
fn blank_lines_survive_a_noop_resize() {
    use shpool_vterm::term::AsTermInput;

    let size = shpool_vterm::Size { width: 5, height: 8 };

    let mut input = vec![];
    term::Raw::from("aa").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("bb").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("cc").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, size);
    term.process(input.as_slice());
    term.resize(size);

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("aa").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("bb").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("cc").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(5, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    // Screen rather than All because that is the region shpool restores.
    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// The same, for a resize that actually reflows. The widths are picked so that
// "abcdef" occupies two grid lines either side of the resize, which keeps the
// row count stable and isolates the blank line from cursor re-anchoring.
#[test]
fn blank_lines_survive_a_reflowing_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::Raw::from("abcdef").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("gh").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 4, height: 8 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 3, height: 8 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("abc").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("def").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("gh").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(4, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// The other side of the same coin: a logical line that ends exactly on the
// width boundary fills its last grid line completely and must not pick up a
// trailing blank one. Reflow cannot simply emit a row for every logical line
// it starts.
#[test]
fn full_width_line_does_not_gain_a_blank_on_reflow() {
    use shpool_vterm::term::AsTermInput;

    let size = shpool_vterm::Size { width: 3, height: 5 };

    let mut input = vec![];
    term::Raw::from("abc").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("d").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, size);
    term.process(input.as_slice());
    term.resize(size);

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("abc").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("d").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(2, 2).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}
