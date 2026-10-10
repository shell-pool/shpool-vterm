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

//! Tests for left/right margins (DECLRMM and DECSLRM), which tmux and neovim
//! use to scroll one of two panes that sit side by side.
//!
//! Most of these check that some input leaves the terminal in the same state
//! as input that draws the same thing without relying on the margins, and
//! then puts the same margins in place.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion, Size, Term};

fn term_after(size: Size, input: &[u8]) -> Term {
    let mut term = Term::new(100, size);
    term.process(input);
    term
}

fn dump(term: &Term) -> String {
    String::from_utf8_lossy(&term.contents(ContentRegion::All)).into_owned()
}

/// Check that `input` leaves a terminal of the given size in the same state
/// as `want`.
fn assert_same(size: Size, input: &[u8], want: &[u8]) {
    assert_eq!(dump(&term_after(size, input)), dump(&term_after(size, want)));
}

frag! {
    margins_get_restored { scrollback_lines: 10, width: 6, height: 3 }
    <= term::control_codes().enable_left_right_margin_mode,
       term::ControlCodes::set_left_right_margins(2, 4),
       term::ControlCodes::cursor_position(1, 3),
       term::Raw::from("x")
    => ContentRegion::All =>
        reset_codes,
        term::Raw::from("  x"),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(2, 4),
        term::ControlCodes::cursor_position(1, 4),
        term::control_codes().clear_attrs
}

frag! {
    margin_mode_gets_restored { scrollback_lines: 10, width: 6, height: 3 }
    <= term::control_codes().enable_left_right_margin_mode
    => ContentRegion::All =>
        reset_codes,
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::cursor_position(1, 1),
        term::control_codes().clear_attrs
}

// Without left/right margin mode, `CSI l ; r s` is SCOSC, which saves the
// cursor.
#[test]
fn margins_need_margin_mode() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::Raw::from("ab"),
            term::ControlCodes::set_left_right_margins(2, 4),
            term::ControlCodes::cursor_position(2, 1),
            term::Raw::from("cd"),
            term::control_codes().restore_cursor_position,
            term::Raw::from("!"),
        ],
        &input![
            term::Raw::from("ab"),
            term::control_codes().save_cursor_position,
            term::ControlCodes::cursor_position(2, 1),
            term::Raw::from("cd"),
            term::control_codes().restore_cursor_position,
            term::Raw::from("!"),
        ],
    );
}

// Text wraps at the right margin, onto the left margin of the next row.
#[test]
fn text_wraps_between_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("abcdef"),
        ],
        &input![
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(2, 2),
            term::Raw::from("de"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::ControlCodes::cursor_position(2, 4),
            term::Raw::from("f"),
        ],
    );
}

// The rows that text wrapped over between the margins do not make up a line,
// so a resize does not join them back up.
#[test]
fn text_wrapped_between_the_margins_does_not_reflow() {
    let (size, wider) = (Size { width: 6, height: 2 }, Size { width: 12, height: 2 });
    let mut got = term_after(
        size,
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 6),
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("abcdefgh"),
            term::control_codes().disable_left_right_margin_mode,
        ],
    );
    got.resize(wider);
    let mut want = term_after(
        size,
        &input![
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("abcde"),
            term::ControlCodes::cursor_position(2, 2),
            term::Raw::from("fgh"),
        ],
    );
    want.resize(wider);
    assert_eq!(dump(&got), dump(&want));
}

// A linefeed on the bottom row only scrolls what is between the margins, and
// nothing goes into the scrollback.
#[test]
fn linefeed_scrolls_between_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
            term::ControlCodes::cursor_position(3, 1),
            term::Raw::from("\n"),
        ],
        &input![
            term::Raw::from("bbbaaa\r\ncccbbb\r\n"),
            term::ControlCodes::cursor_horizontal_absolute(4),
            term::Raw::from("ccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
            term::ControlCodes::cursor_position(3, 1),
        ],
    );
}

// Outside of the margins, the cursor has nowhere to go on the bottom row.
#[test]
fn linefeed_outside_of_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
            term::ControlCodes::cursor_position(3, 5),
            term::Raw::from("\n"),
        ],
        &input![
            term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
            term::ControlCodes::cursor_position(3, 5),
        ],
    );
}

// So do SU and SD.
#[test]
fn scroll_up_and_down_between_the_margins() {
    let size = Size { width: 4, height: 2 };
    assert_same(
        size,
        &input![
            term::Raw::from("aaaa\r\nbbbb"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 3),
            term::ControlCodes::scroll_up(1),
        ],
        &input![
            term::Raw::from("abba\r\nb"),
            term::ControlCodes::cursor_horizontal_absolute(4),
            term::Raw::from("b"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 3),
        ],
    );
    assert_same(
        size,
        &input![
            term::Raw::from("aaaa\r\nbbbb"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 3),
            term::ControlCodes::scroll_down(1),
        ],
        &input![
            term::Raw::from("a"),
            term::ControlCodes::cursor_horizontal_absolute(4),
            term::Raw::from("a\r\nbaab"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 3),
        ],
    );
}

// And IL and DL, which also move the cursor to the left margin.
#[test]
fn insert_and_delete_lines_between_the_margins() {
    let size = Size { width: 6, height: 4 };
    let setup = term::Raw::new(input![
        term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc\r\ndddddd"),
        term::ControlCodes::set_scroll_region(2, 3),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(2, 4),
        term::ControlCodes::cursor_position(2, 3),
    ]);
    let margins = term::Raw::new(input![
        term::ControlCodes::set_scroll_region(2, 3),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(2, 4),
        term::ControlCodes::cursor_position(2, 2),
    ]);
    assert_same(
        size,
        &input![setup, term::ControlCodes::insert_lines(1)],
        &input![
            term::Raw::from("aaaaaa\r\nb"),
            term::ControlCodes::cursor_horizontal_absolute(5),
            term::Raw::from("bb\r\ncbbbcc\r\ndddddd"),
            margins,
        ],
    );
    assert_same(
        size,
        &input![setup, term::ControlCodes::delete_lines(1)],
        &input![
            term::Raw::from("aaaaaa\r\nbcccbb\r\nc"),
            term::ControlCodes::cursor_horizontal_absolute(5),
            term::Raw::from("cc\r\ndddddd"),
            margins,
        ],
    );
}

// Outside of the margins, IL does nothing at all, not even move the cursor.
#[test]
fn insert_lines_outside_of_the_margins() {
    let setup = term::Raw::new(input![
        term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(2, 4),
        term::ControlCodes::cursor_position(2, 6),
    ]);
    assert_same(
        Size { width: 6, height: 3 },
        &input![setup, term::ControlCodes::insert_lines(1)],
        &input![setup],
    );
}

// Scrolling between the margins blanks out the wide chars that straddle one,
// since only half of each would move.
#[test]
fn scrolling_between_the_margins_splits_wide_chars() {
    assert_same(
        Size { width: 6, height: 2 },
        &input![
            term::Raw::from("a中bcd\r\nefghij"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 6),
            term::ControlCodes::scroll_up(1),
        ],
        &input![
            term::Raw::from("a"),
            term::ControlCodes::cursor_horizontal_absolute(3),
            term::Raw::from("ghij\r\nef"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 6),
        ],
    );
}

// ICH and DCH only shift the cells up to the right margin.
#[test]
fn insert_and_delete_chars_before_the_right_margin() {
    let size = Size { width: 6, height: 1 };
    let margins = term::Raw::new(input![
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(2, 4),
        term::ControlCodes::cursor_position(1, 2),
    ]);
    assert_same(
        size,
        &input![term::Raw::from("abcdef"), margins, term::ControlCodes::insert_character(1)],
        &input![
            term::Raw::from("a"),
            term::ControlCodes::cursor_horizontal_absolute(3),
            term::Raw::from("bcef"),
            margins,
        ],
    );
    assert_same(
        size,
        &input![term::Raw::from("abcdef"), margins, term::ControlCodes::delete_character(1)],
        &input![
            term::Raw::from("acd"),
            term::ControlCodes::cursor_horizontal_absolute(5),
            term::Raw::from("ef"),
            margins,
        ],
    );
}

// And neither does insert mode.
#[test]
fn insert_mode_before_the_right_margin() {
    assert_same(
        Size { width: 6, height: 1 },
        &input![
            term::Raw::from("abcdef"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::control_codes().enable_insert_mode,
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("x"),
        ],
        &input![
            term::Raw::from("axbcef"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::control_codes().enable_insert_mode,
            term::ControlCodes::cursor_position(1, 3),
        ],
    );
}

// CR goes to the left margin, unless the cursor is left of it.
#[test]
fn carriage_return_goes_to_the_left_margin() {
    assert_same(
        Size { width: 6, height: 2 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 5),
            term::ControlCodes::cursor_position(1, 4),
            term::Raw::from("\rx"),
            term::ControlCodes::cursor_position(2, 2),
            term::Raw::from("\ry"),
        ],
        &input![
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(2, 1),
            term::Raw::from("y"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 5),
            term::ControlCodes::cursor_position(2, 2),
        ],
    );
}

// Moving sideways, the cursor stops at the margins, unless it starts out
// beyond them.
#[test]
fn cursor_movement_stops_at_the_margins() {
    assert_same(
        Size { width: 8, height: 1 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 5),
            term::ControlCodes::cursor_position(1, 4),
            term::ControlCodes::cursor_forward(9),
            term::Raw::from("x"),
            term::ControlCodes::cursor_backwards(9),
            term::Raw::from("y"),
            term::ControlCodes::cursor_position(1, 7),
            term::ControlCodes::cursor_forward(9),
            term::Raw::from("z"),
            term::ControlCodes::cursor_position(1, 2),
            term::ControlCodes::cursor_backwards(9),
            term::Raw::from("w"),
        ],
        &input![
            term::Raw::from("w"),
            term::ControlCodes::cursor_position(1, 3),
            term::Raw::from("y"),
            term::ControlCodes::cursor_position(1, 5),
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(1, 8),
            term::Raw::from("z"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(3, 5),
            term::ControlCodes::cursor_position(1, 2),
        ],
    );
}

// A tab stops in front of the right margin.
#[test]
fn tab_stops_at_the_right_margin() {
    assert_same(
        Size { width: 20, height: 1 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 5),
            term::Raw::from("\tx"),
        ],
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 5),
            term::ControlCodes::cursor_position(1, 5),
            term::Raw::from("x"),
        ],
    );
}

// RI on the top row only scrolls what is between the margins, and outside of
// them the cursor has nowhere to go.
#[test]
fn reverse_index_between_the_margins() {
    let size = Size { width: 6, height: 3 };
    let setup = term::Raw::new(input![
        term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(1, 3),
    ]);
    assert_same(
        size,
        &input![setup, term::control_codes().reverse_index],
        &input![
            term::ControlCodes::cursor_horizontal_absolute(4),
            term::Raw::from("aaa\r\naaabbb\r\nbbbccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
        ],
    );
    assert_same(
        size,
        &input![
            setup,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().reverse_index,
        ],
        &input![setup, term::ControlCodes::cursor_position(1, 5)],
    );
}

// In origin mode, cursor positions are relative to the margins, and the
// cursor cannot leave them.
#[test]
fn origin_mode_is_relative_to_the_margins() {
    let size = Size { width: 8, height: 4 };
    let margins = term::Raw::new(input![
        term::ControlCodes::set_scroll_region(2, 3),
        term::control_codes().enable_left_right_margin_mode,
        term::ControlCodes::set_left_right_margins(3, 6),
        term::control_codes().enable_scroll_region_origin_mode,
    ]);
    assert_same(
        size,
        &input![
            margins,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(9, 9),
            term::Raw::from("y"),
        ],
        &input![
            term::ControlCodes::cursor_position(2, 3),
            term::Raw::from("x"),
            margins,
            term::ControlCodes::cursor_position(2, 4),
            term::Raw::from("y"),
        ],
    );
    assert_same(
        size,
        &input![
            margins,
            term::ControlCodes::cursor_horizontal_absolute(2),
            term::Raw::from("x"),
            term::ControlCodes::vertical_position_absolute(2),
            term::Raw::from("y"),
        ],
        &input![
            term::ControlCodes::cursor_position(2, 4),
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(3, 5),
            term::Raw::from("y"),
            margins,
            term::ControlCodes::cursor_position(2, 4),
        ],
    );
}

// Turning left/right margin mode off drops the margins.
#[test]
fn margin_mode_off_drops_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc"),
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(1, 3),
            term::control_codes().disable_left_right_margin_mode,
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::cursor_position(3, 1),
            term::Raw::from("\n"),
        ],
        &input![
            term::Raw::from("aaaaaa\r\nbbbbbb\r\ncccccc\r\n"),
            term::control_codes().enable_left_right_margin_mode,
        ],
    );
}

// tmux drops the margins it set with a bare `CSI s`, which does not save the
// cursor while left/right margin mode is on.
#[test]
fn bare_csi_s_drops_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().unset_left_right_margins,
        ],
        &input![term::control_codes().enable_left_right_margin_mode],
    );
}

// Like the scroll region, the margins do not survive a resize, but left/right
// margin mode stays on.
#[test]
fn resize_drops_the_margins() {
    let (size, wider) = (Size { width: 6, height: 3 }, Size { width: 8, height: 3 });
    let margin_mode = input![term::control_codes().enable_left_right_margin_mode];
    let mut got = term_after(
        size,
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
        ],
    );
    got.resize(wider);
    let mut want = term_after(size, &margin_mode);
    want.resize(wider);
    assert_eq!(dump(&got), dump(&want));
    assert!(dump(&got).contains(std::str::from_utf8(&margin_mode).unwrap()));
}

// DECSTR turns left/right margin mode off, so `CSI s` saves the cursor again.
#[test]
fn soft_reset_turns_margin_mode_off() {
    assert_same(
        Size { width: 6, height: 3 },
        &input![
            term::control_codes().enable_left_right_margin_mode,
            term::ControlCodes::set_left_right_margins(2, 4),
            term::control_codes().soft_reset,
            term::ControlCodes::cursor_position(2, 3),
            term::ControlCodes::set_left_right_margins(2, 4),
            term::ControlCodes::cursor_position(1, 1),
        ],
        &input![
            term::control_codes().soft_reset,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().save_cursor_position,
            term::ControlCodes::cursor_position(1, 1),
        ],
    );
}
