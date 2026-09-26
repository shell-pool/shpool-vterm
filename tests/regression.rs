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
// These used to prime the buffer with an SU, back when SU moved the viewport
// rather than the grid, so that rows below the screen still resolved to lines
// and `bottom - cursor.row` underflowed in Scrollback::insert_lines and
// Scrollback::delete_lines. The viewport is gone, so the grid always starts at
// the bottom of the buffer and that underflow is no longer reachable.

// Control case: an in-range bottom, where LF scrolls and the cursor stays on
// the last row. Anchors the expected output of the three cases below.
frag! {
    scroll_region_bottom_at_last_row { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"), term::Crlf::default(),
       term::Raw::from("dd"), term::Crlf::default(),
       term::Raw::from("ee"),
       term::ControlCodes::set_scroll_region(1, 3),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default()
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("ee"),
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
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default()
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("ee"),
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
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("ee"),
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
       term::ControlCodes::set_scroll_region(1, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::Crlf::default(),
       term::Crlf::default(),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("ee"),
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
       term::ControlCodes::scroll_up(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// Clamping the bottom can pull it up to or above the top, leaving a region
// that describes no rows at all. Like in xterm, the whole command is ignored.
frag! {
    scroll_region_clamped_away { scrollback_lines: 100, width: 5, height: 3 }
    <= term::ControlCodes::set_scroll_region(5, 9)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// A shrinking resize used to strand a region that was in range when it was
// set. shpool resizes the spool on every reattach, which is how a long lived
// session got into that state. Like in other terminals, a resize now drops
// the region altogether.
#[test]
fn scroll_region_dropped_on_shrinking_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::ControlCodes::set_scroll_region(2, 6).term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 6 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Same, on the alt screen, where a stranded region used to take the session
// down on the next scroll.
#[test]
fn alt_screen_scroll_region_dropped_on_shrinking_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::control_codes().enable_alt_screen.term_input_into(&mut input);
    term::ControlCodes::set_scroll_region(2, 6).term_input_into(&mut input);

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
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Growing the screen drops the region too. Origin mode stays on, but with no
// region it just addresses the whole screen.
#[test]
fn scroll_region_dropped_on_growing_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::ControlCodes::set_scroll_region(2, 3).term_input_into(&mut input);
    term::control_codes().enable_scroll_region_origin_mode.term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 4 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 6 });

    let mut more = vec![];
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut more);
    term::Raw::from("X").term_input_into(&mut more);
    term.process(more.as_slice());

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("X").term_input_into(&mut expected);
    term::control_codes().enable_scroll_region_origin_mode.term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 2).term_input_into(&mut expected);
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
// switches off every mode the dump might replay and clears margins and
// origin mode before it homes the cursor and erases.
//
// Spelled out byte for byte rather than built from `reset_codes` so that the
// helper cannot quietly track a regression here.
#[test]
fn dump_prefix_resets_terminal_modes() {
    let prefix = [
        "\x1b]8;;\x1b\\",                            // end any link
        "\x1b[?1049l",                               // leave the alt screen
        "\x1b[m",                                    // reset attrs
        "\x1b[r",                                    // clear the scroll region
        "\x1b[?6l",                                  // origin mode off
        "\x1b[4l",                                   // insert mode off
        "\x1b[?7h",                                  // auto-wrap on
        "\x1b(B\x1b)B\x1b*B\x1b+B\x0f",              // G0-G3 ascii, G0 in use
        "\x1b[?25h",                                 // show the cursor
        "\x1b[?1l",                                  // normal cursor keys
        "\x1b>",                                     // normal keypad
        "\x1b[?1000;1002;1003;1005;1006;1015;1016l", // mouse reporting off
        "\x1b[?1004l",                               // focus reporting off
        "\x1b[?2004l",                               // bracketed paste off
        "\x1b[H",                                    // home
        "\x1b7",                                     // save the reset cursor
        "\x1b[J",                                    // erase
    ];
    let expected = format!("{}hi\x1b[1;3H\x1b[m", prefix.concat());

    crate::support::frag::round_trip_frag(
        b"hi",
        expected.as_bytes(),
        100,
        shpool_vterm::Size { width: 5, height: 3 },
        ContentRegion::All,
    );
}

// The terminal a restore gets painted into is often still in whatever state
// the previous session left it in, e.g. because the connection dropped while
// a curses app was running. Restoring into it has to end up in the same state
// as restoring into a fresh terminal.
#[test]
fn restore_resets_modes_left_over_in_the_client() {
    let size = shpool_vterm::Size { width: 20, height: 5 };
    let mut session = shpool_vterm::Term::new(100, size);
    session.process(b"$ ls\r\nfoo  bar\r\n$ ");

    let mut client = shpool_vterm::Term::new(100, size);
    client.process(b"\x1b[3;3H\x1b[?1049h"); // saved cursor, alt screen
    client.process(b"\x1b[1;31m\x1b[2;4r\x1b[?6h"); // attrs, margins, DECOM
    client.process(b"\x1b[4h\x1b[?7l"); // insert mode, no auto-wrap
    client.process(b"\x1b[?25l\x1b[?1h\x1b="); // hidden cursor, app keys
    client.process(b"\x1b[?1002;1006h\x1b[?1004h\x1b[?2004h"); // reporting
    client.process(b"\x1b(0\x1b)0\x1b*0\x1b+0\x0e"); // line drawing
    client.process(&session.contents(ContentRegion::All));

    assert_eq!(client.contents(ContentRegion::All), session.contents(ContentRegion::All));
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

// The row a line occupies is derived, not stored: it falls out of the buffer
// length and the height. `cursor.row` is an absolute row, so a resize that
// moves every line has to move the cursor with them. Clamping to the new size
// is not enough, because the cursor can stay in range while the line it was
// sitting on slides out from under it.
//
// Here the buffer is longer than the screen, so it is anchored to the bottom
// and growing the height pulls scrollback down into view. The cursor is on the
// last line and has to stay there.
#[test]
fn cursor_follows_its_row_when_the_height_grows() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 3 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 5 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("11").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("22").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("33").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("44").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("55").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(5, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// The height is not the only input that moves lines around. Here the screen is
// anchored to the top and the height never changes, but narrowing splits
// "abcdef" across two rows, which pushes "gh" and the cursor on it down one.
#[test]
fn cursor_follows_its_row_when_reflow_adds_rows() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::Raw::from("abcdef").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("gh").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 6, height: 8 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 3, height: 8 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("abc").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("def").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("gh").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(3, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// Shrinking the height walks the bottom anchored screen the other way, and the
// clamp already lands the cursor in the right place. Pinned so that teaching
// resize to re-anchor does not overshoot the case it happens to get right.
#[test]
fn cursor_stays_on_the_bottom_row_when_the_height_shrinks() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 5 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("33").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("44").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("55").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(3, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// Clearing the screen erases it and homes the cursor, which leaves the prompt
// on the top row with a screen full of blank rows under it. Shrinking the
// height has to drop those rows rather than push the prompt up into the
// scrollback, which used to leave the cursor on an empty screen.
#[test]
fn shrinking_a_cleared_screen_keeps_the_prompt_on_it() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut input);
    term::control_codes().erase_screen.term_input_into(&mut input);
    term::Raw::from("$ ").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 5 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("$ ").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Only as many blank rows go as it takes to keep what was on the screen on
// it, and never the ones the cursor is on or above.
#[test]
fn shrinking_drops_blank_rows_from_below_the_cursor() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);
    term::ControlCodes::cursor_position(2, 3).term_input_into(&mut input);
    term::control_codes().erase_to_end.term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 5 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 2 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("11").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("22").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(2, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// Text below the cursor stays, and so do the blank rows above it. The screen
// gets pushed up into the scrollback like before.
#[test]
fn shrinking_keeps_text_below_the_cursor() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", ""] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);
    term::ControlCodes::cursor_position(3, 1).term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 5 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("33").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("55").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// Narrowing splits long lines into more rows, which pushes the screen up just
// like shrinking the height does, and the blank rows at the bottom make up
// for it the same way.
#[test]
fn narrowing_drops_blank_rows_from_below_the_cursor() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    term::Raw::from("abcdef").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("gh").term_input_into(&mut input);
    term::Crlf::default().term_input_into(&mut input);
    term::Raw::from("ij").term_input_into(&mut input);
    term::ControlCodes::cursor_position(2, 3).term_input_into(&mut input);
    term::control_codes().erase_to_end.term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 6, height: 3 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 3, height: 3 });

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("abc").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("def").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("gh").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(3, 3).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);
}

// DECSC stores a row too, so it has the same problem as the live cursor: a
// resize between the save and the restore moves the line out from under it.
// Save on "55", park the live cursor elsewhere so a fix cannot get this right
// by accident, grow the height, then restore and write.
#[test]
fn saved_cursor_follows_its_row_across_a_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);
    term::control_codes().save_cursor.term_input_into(&mut input);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 3 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 5 });

    let mut restore = vec![];
    term::control_codes().restore_cursor.term_input_into(&mut restore);
    term::Raw::from("X").term_input_into(&mut restore);
    term.process(restore.as_slice());

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("11").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("22").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("33").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("44").term_input_into(&mut expected);
    term::Crlf::default().term_input_into(&mut expected);
    term::Raw::from("55X").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(5, 3).term_input_into(&mut expected);
    term::control_codes().save_cursor.term_input_into(&mut expected);
    term::ControlCodes::cursor_position(5, 4).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// Restoring the cursor when nothing was saved homes it. The empty slot used
// to be a saved cursor in the top left corner, which then followed its row
// across a resize like any other saved cursor, here down onto "33".
#[test]
fn restoring_an_unsaved_cursor_homes_it_after_a_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut input = vec![];
    for row in ["11", "22", "33", "44"] {
        term::Raw::from(row).term_input_into(&mut input);
        term::Crlf::default().term_input_into(&mut input);
    }
    term::Raw::from("55").term_input_into(&mut input);

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 5, height: 3 });
    term.process(input.as_slice());
    term.resize(shpool_vterm::Size { width: 5, height: 5 });

    let mut restore = vec![];
    term::control_codes().restore_cursor.term_input_into(&mut restore);
    term::Raw::from("X").term_input_into(&mut restore);
    term.process(restore.as_slice());

    let mut expected = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut expected);
    term::Raw::from("X1").term_input_into(&mut expected);
    for row in ["22", "33", "44", "55"] {
        term::Crlf::default().term_input_into(&mut expected);
        term::Raw::from(row).term_input_into(&mut expected);
    }
    term::ControlCodes::cursor_position(1, 2).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::Screen), expected);
}

// SU (CSI S) moves the content toward the top of the screen and opens blank
// rows at the bottom. It is what terminfo calls `indn`, and it is what a pager
// emits to advance a page. The cursor does not move, so it ends up on a
// different line than the one it started on.
//
// These assert against the screen rather than the whole buffer, so they say
// nothing about where a scrolled off line is kept. That is up to the
// implementation.
frag! {
    scroll_up_moves_content_toward_the_top { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

// SD (CSI T) is the mirror image: content moves toward the bottom, blank rows
// open at the top, and whatever falls off the bottom is gone. This is `rin`.
frag! {
    scroll_down_moves_content_toward_the_bottom { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"),
       term::ControlCodes::scroll_down(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("aa"),
            term::Crlf::default(),
            term::Raw::from("bb"),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

// SU edits the grid, so a later write has to leave it alone. This is what the
// viewport implementation got wrong: it looked right until the next printed
// character snapped the window back to the bottom of the buffer.
frag! {
    scroll_up_is_not_undone_by_the_next_write { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"), term::Crlf::default(),
       term::Raw::from("cc"),
       term::ControlCodes::scroll_up(1),
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("X")
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("Xb"),
            term::Crlf::default(),
            term::Raw::from("cc"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// With DECSTBM set, SU scrolls the region and nothing else. The rows above and
// below it stay put.
frag! {
    scroll_up_only_touches_the_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// The main screen only stores the rows that have been written to, but the
// rows below them are still on the screen. A linefeed on the bottom row has
// to scroll everything up past those blank rows, not just open another blank
// row right below the content.
frag! {
    linefeed_at_the_bottom_scrolls_a_partially_filled_screen { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"),
       term::ControlCodes::cursor_position(3, 1),
       term::Raw::from("\nX")
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("aa"),
            term::Crlf::default(),
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

// Same for SU.
frag! {
    scroll_up_moves_a_partially_filled_screen { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

// Scrolling by more than a screenful blanks the screen. The lines that were
// on it go into the scrollback, but there is no need to follow them up with
// thousands of blank lines.
frag! {
    huge_scroll_up_only_pushes_a_screenful { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("aa"), term::Crlf::default(),
       term::Raw::from("bb"),
       term::ControlCodes::scroll_up(1000)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("aa"),
            term::Crlf::default(),
            term::Raw::from("bb"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}
