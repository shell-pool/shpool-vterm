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

//! Tests for reflowing the main screen when its width changes.
//!
//! Most of these check that reflowing some output to a new width gives the
//! same result as writing that output at the new width in the first place.

use shpool_vterm::{ContentRegion, Size, Term};

fn dump(term: &Term) -> String {
    String::from_utf8_lossy(term.contents(ContentRegion::All).as_slice()).into_owned()
}

/// Check that writing `input` at size `from` and then resizing to `to` gives
/// the same terminal as writing `input` at size `to`.
fn assert_reflows_like_written(scrollback_lines: usize, input: &str, from: Size, to: Size) {
    let mut term = Term::new(scrollback_lines, from);
    term.process(input.as_bytes());
    term.resize(to);

    let mut want = Term::new(scrollback_lines, to);
    want.process(input.as_bytes());

    assert_eq!(dump(&term), dump(&want));
}

/// Check that resizing through each of `widths` and then back to the size we
/// started at leaves the terminal the way it was.
fn assert_round_trips(input: &str, size: Size, widths: &[usize]) {
    let mut term = Term::new(100, size);
    term.process(input.as_bytes());
    let want = dump(&term);

    for width in widths {
        term.resize(Size { width: *width, height: size.height });
    }
    term.resize(size);

    assert_eq!(dump(&term), want, "resizing through widths {:?}", widths);
}

#[test]
fn narrowing_moves_a_wide_char_to_the_next_row() {
    assert_reflows_like_written(
        100,
        "abcd😊",
        Size { width: 6, height: 3 },
        Size { width: 5, height: 3 },
    );
}

#[test]
fn widening_brings_a_wide_char_back_up() {
    assert_reflows_like_written(
        100,
        "abcd😊",
        Size { width: 5, height: 3 },
        Size { width: 6, height: 3 },
    );
}

#[test]
fn narrowing_never_splits_a_wide_char() {
    assert_reflows_like_written(
        100,
        "a😊😊😊😊",
        Size { width: 9, height: 4 },
        Size { width: 4, height: 4 },
    );
}

// DCH pads the line out to the full width with blanks, and those should not
// get wrapped onto a row of their own when the screen gets narrower.
#[test]
fn padding_from_delete_character_does_not_add_rows() {
    let mut term = Term::new(100, Size { width: 10, height: 3 });
    term.process(b"abc\r\x1b[P");
    term.resize(Size { width: 5, height: 3 });

    let mut want = Term::new(100, Size { width: 5, height: 3 });
    want.process(b"bc\r");
    assert_eq!(dump(&term), dump(&want));
}

// Inserting a line pushes the continuation of the wrapped line off the
// bottom of the screen, so the last line claims to continue onto a line that
// is not there anymore. It still has to survive the reflow.
#[test]
fn last_line_claiming_to_wrap_survives() {
    let mut term = Term::new(100, Size { width: 5, height: 2 });
    term.process(b"abcdef\x1b[H\x1b[L");
    term.resize(Size { width: 10, height: 2 });

    let mut want = Term::new(100, Size { width: 10, height: 2 });
    want.process(b"\r\nabcde\x1b[H");
    assert_eq!(dump(&term), dump(&want));
}

#[test]
fn narrowing_drops_lines_that_no_longer_fit_in_the_scrollback() {
    assert_reflows_like_written(
        3,
        "aaaa\r\nbbbb\r\ncc",
        Size { width: 4, height: 2 },
        Size { width: 2, height: 2 },
    );
}

#[test]
fn cursor_waiting_to_wrap_round_trips() {
    assert_round_trips(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZABCD",
        Size { width: 10, height: 10 },
        &[20, 7, 3, 30],
    );
}

#[test]
fn text_with_wide_chars_round_trips() {
    assert_round_trips(
        "ab😊cd😊😊ef\r\n😊😊😊\r\nabc",
        Size { width: 7, height: 10 },
        &[4, 11, 2, 5],
    );
}

// The cursor is out in the blank space past the end of the text. It should
// keep its place in the logical line rather than get pulled back onto the
// text, and the next char written should land where it would have.
#[test]
fn cursor_past_the_end_of_the_text_keeps_its_place() {
    assert_round_trips("abc\x1b[5C", Size { width: 10, height: 3 }, &[4]);

    let mut term = Term::new(100, Size { width: 10, height: 3 });
    term.process(b"abc\x1b[5C");
    term.resize(Size { width: 4, height: 3 });
    term.process(b"X");

    let mut want = Term::new(100, Size { width: 4, height: 3 });
    want.process(b"abc     X");
    assert_eq!(dump(&term), dump(&want));
}

// Right after the text fills a row at the new width is where the cursor
// would be waiting to wrap if the text had been written at that width.
#[test]
fn cursor_at_the_end_of_a_full_row_waits_to_wrap() {
    assert_reflows_like_written(
        100,
        "abcdef",
        Size { width: 10, height: 3 },
        Size { width: 3, height: 3 },
    );
}
