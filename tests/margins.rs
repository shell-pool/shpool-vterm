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

fn term_after(size: Size, input: &str) -> Term {
    let mut term = Term::new(100, size);
    term.process(input.as_bytes());
    term
}

fn dump(term: &Term) -> String {
    String::from_utf8_lossy(&term.contents(ContentRegion::All)).into_owned()
}

/// Check that `input` leaves a terminal of the given size in the same state
/// as `want`.
fn assert_same(size: Size, input: &str, want: &str) {
    assert_eq!(dump(&term_after(size, input)), dump(&term_after(size, want)));
}

frag! {
    margins_get_restored { scrollback_lines: 10, width: 6, height: 3 }
    <= term::Raw::from("\x1b[?69h\x1b[2;4s\x1b[1;3Hx")
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
        "ab\x1b[2;4s\x1b[2;1Hcd\x1b[u!",
        "ab\x1b[s\x1b[2;1Hcd\x1b[u!",
    );
}

// Text wraps at the right margin, onto the left margin of the next row.
#[test]
fn text_wraps_between_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        "\x1b[?69h\x1b[2;4s\x1b[1;2Habcdef",
        "\x1b[1;2Habc\x1b[2;2Hde\x1b[?69h\x1b[2;4s\x1b[2;4Hf",
    );
}

// The rows that text wrapped over between the margins do not make up a line,
// so a resize does not join them back up.
#[test]
fn text_wrapped_between_the_margins_does_not_reflow() {
    let (size, wider) = (Size { width: 6, height: 2 }, Size { width: 12, height: 2 });
    let mut got = term_after(size, "\x1b[?69h\x1b[2;6s\x1b[1;2Habcdefgh\x1b[?69l");
    got.resize(wider);
    let mut want = term_after(size, "\x1b[1;2Habcde\x1b[2;2Hfgh");
    want.resize(wider);
    assert_eq!(dump(&got), dump(&want));
}

// A linefeed on the bottom row only scrolls what is between the margins, and
// nothing goes into the scrollback.
#[test]
fn linefeed_scrolls_between_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[1;3s\x1b[3;1H\n",
        "bbbaaa\r\ncccbbb\r\n\x1b[4Gccc\x1b[?69h\x1b[1;3s\x1b[3;1H",
    );
}

// Outside of the margins, the cursor has nowhere to go on the bottom row.
#[test]
fn linefeed_outside_of_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[1;3s\x1b[3;5H\n",
        "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[1;3s\x1b[3;5H",
    );
}

// So do SU and SD.
#[test]
fn scroll_up_and_down_between_the_margins() {
    let size = Size { width: 4, height: 2 };
    assert_same(
        size,
        "aaaa\r\nbbbb\x1b[?69h\x1b[2;3s\x1b[S",
        "abba\r\nb\x1b[4Gb\x1b[?69h\x1b[2;3s",
    );
    assert_same(
        size,
        "aaaa\r\nbbbb\x1b[?69h\x1b[2;3s\x1b[T",
        "a\x1b[4Ga\r\nbaab\x1b[?69h\x1b[2;3s",
    );
}

// And IL and DL, which also move the cursor to the left margin.
#[test]
fn insert_and_delete_lines_between_the_margins() {
    let size = Size { width: 6, height: 4 };
    let setup = "aaaaaa\r\nbbbbbb\r\ncccccc\r\ndddddd\x1b[2;3r\x1b[?69h\x1b[2;4s\x1b[2;3H";
    let margins = "\x1b[2;3r\x1b[?69h\x1b[2;4s\x1b[2;2H";
    assert_same(
        size,
        &format!("{setup}\x1b[L"),
        &format!("aaaaaa\r\nb\x1b[5Gbb\r\ncbbbcc\r\ndddddd{margins}"),
    );
    assert_same(
        size,
        &format!("{setup}\x1b[M"),
        &format!("aaaaaa\r\nbcccbb\r\nc\x1b[5Gcc\r\ndddddd{margins}"),
    );
}

// Outside of the margins, IL does nothing at all, not even move the cursor.
#[test]
fn insert_lines_outside_of_the_margins() {
    let setup = "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[2;4s\x1b[2;6H";
    assert_same(Size { width: 6, height: 3 }, &format!("{setup}\x1b[L"), setup);
}

// Scrolling between the margins blanks out the wide chars that straddle one,
// since only half of each would move.
#[test]
fn scrolling_between_the_margins_splits_wide_chars() {
    assert_same(
        Size { width: 6, height: 2 },
        "a中bcd\r\nefghij\x1b[?69h\x1b[3;6s\x1b[S",
        "a\x1b[3Gghij\r\nef\x1b[?69h\x1b[3;6s",
    );
}

// ICH and DCH only shift the cells up to the right margin.
#[test]
fn insert_and_delete_chars_before_the_right_margin() {
    let size = Size { width: 6, height: 1 };
    let margins = "\x1b[?69h\x1b[2;4s\x1b[1;2H";
    assert_same(size, &format!("abcdef{margins}\x1b[@"), &format!("a\x1b[3Gbcef{margins}"));
    assert_same(size, &format!("abcdef{margins}\x1b[P"), &format!("acd\x1b[5Gef{margins}"));
}

// And neither does insert mode.
#[test]
fn insert_mode_before_the_right_margin() {
    assert_same(
        Size { width: 6, height: 1 },
        "abcdef\x1b[?69h\x1b[2;4s\x1b[4h\x1b[1;2Hx",
        "axbcef\x1b[?69h\x1b[2;4s\x1b[4h\x1b[1;3H",
    );
}

// CR goes to the left margin, unless the cursor is left of it.
#[test]
fn carriage_return_goes_to_the_left_margin() {
    assert_same(
        Size { width: 6, height: 2 },
        "\x1b[?69h\x1b[3;5s\x1b[1;4H\rx\x1b[2;2H\ry",
        "\x1b[1;3Hx\x1b[2;1Hy\x1b[?69h\x1b[3;5s\x1b[2;2H",
    );
}

// Moving sideways, the cursor stops at the margins, unless it starts out
// beyond them.
#[test]
fn cursor_movement_stops_at_the_margins() {
    assert_same(
        Size { width: 8, height: 1 },
        "\x1b[?69h\x1b[3;5s\x1b[1;4H\x1b[9Cx\x1b[9Dy\x1b[1;7H\x1b[9Cz\x1b[1;2H\x1b[9Dw",
        "w\x1b[1;3Hy\x1b[1;5Hx\x1b[1;8Hz\x1b[?69h\x1b[3;5s\x1b[1;2H",
    );
}

// A tab stops in front of the right margin.
#[test]
fn tab_stops_at_the_right_margin() {
    assert_same(
        Size { width: 20, height: 1 },
        "\x1b[?69h\x1b[1;5s\tx",
        "\x1b[?69h\x1b[1;5s\x1b[1;5Hx",
    );
}

// RI on the top row only scrolls what is between the margins, and outside of
// them the cursor has nowhere to go.
#[test]
fn reverse_index_between_the_margins() {
    let size = Size { width: 6, height: 3 };
    let setup = "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[1;3s";
    assert_same(size, &format!("{setup}\x1bM"), "\x1b[4Gaaa\r\naaabbb\r\nbbbccc\x1b[?69h\x1b[1;3s");
    assert_same(size, &format!("{setup}\x1b[1;5H\x1bM"), &format!("{setup}\x1b[1;5H"));
}

// In origin mode, cursor positions are relative to the margins, and the
// cursor cannot leave them.
#[test]
fn origin_mode_is_relative_to_the_margins() {
    let size = Size { width: 8, height: 4 };
    let margins = "\x1b[2;3r\x1b[?69h\x1b[3;6s\x1b[?6h";
    assert_same(
        size,
        &format!("{margins}\x1b[1;1Hx\x1b[9;9Hy"),
        &format!("\x1b[2;3Hx{margins}\x1b[2;4Hy"),
    );
    assert_same(
        size,
        &format!("{margins}\x1b[2Gx\x1b[2dy"),
        &format!("\x1b[2;4Hx\x1b[3;5Hy{margins}\x1b[2;4H"),
    );
}

// Turning left/right margin mode off drops the margins.
#[test]
fn margin_mode_off_drops_the_margins() {
    assert_same(
        Size { width: 6, height: 3 },
        "aaaaaa\r\nbbbbbb\r\ncccccc\x1b[?69h\x1b[1;3s\x1b[?69l\x1b[?69h\x1b[3;1H\n",
        "aaaaaa\r\nbbbbbb\r\ncccccc\r\n\x1b[?69h",
    );
}

// tmux drops the margins it set with a bare `CSI s`, which does not save the
// cursor while left/right margin mode is on.
#[test]
fn bare_csi_s_drops_the_margins() {
    assert_same(Size { width: 6, height: 3 }, "\x1b[?69h\x1b[2;4s\x1b[2;3H\x1b[s", "\x1b[?69h");
}

// Like the scroll region, the margins do not survive a resize, but left/right
// margin mode stays on.
#[test]
fn resize_drops_the_margins() {
    let (size, wider) = (Size { width: 6, height: 3 }, Size { width: 8, height: 3 });
    let mut got = term_after(size, "\x1b[?69h\x1b[2;4s");
    got.resize(wider);
    let mut want = term_after(size, "\x1b[?69h");
    want.resize(wider);
    assert_eq!(dump(&got), dump(&want));
    assert!(dump(&got).contains("\x1b[?69h"));
}

// DECSTR turns left/right margin mode off, so `CSI s` saves the cursor again.
#[test]
fn soft_reset_turns_margin_mode_off() {
    assert_same(
        Size { width: 6, height: 3 },
        "\x1b[?69h\x1b[2;4s\x1b[!p\x1b[2;3H\x1b[2;4s\x1b[H",
        "\x1b[!p\x1b[2;3H\x1b[s\x1b[H",
    );
}
