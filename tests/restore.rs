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

//! Tests for carrying on with a session after restoring it into a terminal.
//!
//! These restore a session into a terminal and then feed the same input to
//! both, which should leave them in the same state. That catches state that
//! an app set up before the restore and is still counting on after it, but
//! that the restore did not carry over.

use shpool_vterm::{ContentRegion, Size, Term};

/// Check that restoring a session that has been through `setup` into a
/// terminal that has been through `client_setup` leaves the terminal in the
/// same state as the session, and that it stays that way once both have been
/// through `then`.
fn assert_restores_into(client_setup: &[u8], size: Size, setup: &[u8], then: &[u8]) {
    let mut session = Term::new(100, size);
    session.process(setup);
    let mut client = Term::new(100, size);
    client.process(client_setup);
    client.process(&session.contents(ContentRegion::All));
    assert_same(&client, &session, "after the restore");

    session.process(then);
    client.process(then);
    assert_same(&client, &session, "after carrying on");
}

/// `assert_restores_into` for a fresh terminal.
fn assert_restores(size: Size, setup: &[u8], then: &[u8]) {
    assert_restores_into(b"", size, setup, then);
}

fn assert_same(client: &Term, session: &Term, when: &str) {
    assert_eq!(
        String::from_utf8_lossy(&client.contents(ContentRegion::All)),
        String::from_utf8_lossy(&session.contents(ContentRegion::All)),
        "{when}"
    );
}

// A prompt that saves the cursor, draws something somewhere else and then
// restores the cursor to carry on where it was.
#[test]
fn saved_cursor() {
    assert_restores(Size { width: 10, height: 3 }, b"$ ls\x1b7\x1b[3;1H12:00", b"\x1b8 -l");
}

// The attrs, charsets and origin mode get saved along with the position.
#[test]
fn saved_cursor_state() {
    assert_restores(
        Size { width: 10, height: 5 },
        b"\x1b[2;4r\x1b[?6h\x1b[2;2H\x1b[1m\x1b(0\x1b7\x1b[?6l\x1b[m\x1b(B\x1b[5;1Hx",
        b"\x1b8q\x1b[Hq",
    );
}

// A cursor saved in origin mode can be outside of the scroll region by the
// time of the restore, since the region might have changed since.
#[test]
fn cursor_saved_outside_of_the_scroll_region() {
    assert_restores(
        Size { width: 5, height: 5 },
        b"\x1b[2;5r\x1b[?6h\x1b[4;1H\x1b7\x1b[2;3r",
        b"\x1b[1;5r\x1b8X",
    );
}

// A cursor saved with a wrap pending wraps once it gets restored.
#[test]
fn saved_cursor_with_a_pending_wrap() {
    assert_restores(Size { width: 3, height: 3 }, b"abc\x1b7\r\nd", b"\x1b8e");
}

// Leaving the alt screen restores the cursor that entering it saved.
#[test]
fn cursor_saved_by_the_alt_screen() {
    assert_restores(
        Size { width: 10, height: 3 },
        b"\x1b[1m$ vi\x1b[?1049h\x1b[m~\r\n~",
        b"\x1b[?1049lx",
    );
}

// The switch to the alt screen erases it with the background color of the
// saved cursor, but that color does not belong on the alt screen.
#[test]
fn cursor_saved_by_the_alt_screen_with_a_background_color() {
    assert_restores(
        Size { width: 5, height: 2 },
        b"\x1b[44m\x1b[?1049h\x1b[m\x1b[2Jx",
        b"\x1b[?1049ly",
    );
}

// The alt screen has its own saved cursor.
#[test]
fn saved_cursor_on_the_alt_screen() {
    assert_restores(
        Size { width: 10, height: 3 },
        b"\x1b[?1049h\x1b[2;3H\x1b[4m\x1b7\x1b[m\x1b[Hstatus",
        b"\x1b8x",
    );
}

// Restoring the cursor when the app never saved one homes it, whatever the
// terminal had saved before the restore.
#[test]
fn unsaved_cursor() {
    assert_restores_into(
        b"\x1b[3;3H\x1b[1m\x1b7",
        Size { width: 10, height: 3 },
        b"$ ls",
        b"\x1b8x",
    );
}

// A line that wrapped still wraps after the restore, so it gets reflowed
// along with the rest of it when the window gets resized.
#[test]
fn wrapped_lines_reflow_after_a_restore() {
    let size = Size { width: 4, height: 8 };
    let mut session = Term::new(100, size);
    session.process(b"$ echo abcdef\r\nabcdef\r\n$ ");
    let mut client = Term::new(100, size);
    client.process(&session.contents(ContentRegion::All));

    let size = Size { width: 20, height: 8 };
    session.resize(size);
    client.resize(size);
    assert_same(&client, &session, "after the resize");
}

// Wrapping onto a new row at the bottom of the screen scrolls, which paints
// the new row with the background color. The row that the app wrapped onto
// was not at the bottom, so it did not get painted.
#[test]
fn wrap_that_scrolls_on_restore() {
    assert_restores(
        Size { width: 3, height: 3 },
        b"1\r\n2\r\n3\r\n4\r\n5\x1b[Habc\x1b[44md\x1b[m",
        b"",
    );
}

// Themes can change a lot of the palette, and all of it has to come back.
#[test]
fn lots_of_palette_colors() {
    let mut setup = vec![];
    for idx in 0..20 {
        setup.extend_from_slice(format!("\x1b]4;{idx};rgb:{idx:02x}/00/00\x1b\\").as_bytes());
    }
    assert_restores(Size { width: 10, height: 3 }, &setup, b"");
}
