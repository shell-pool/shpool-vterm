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

#[macro_use]
#[path = "support/input.rs"]
mod input;

use shpool_vterm::{term, ContentRegion, Size, Term};

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
    assert_restores(
        Size { width: 10, height: 3 },
        &input![
            term::Raw::from("$ ls"),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(3, 1),
            term::Raw::from("12:00"),
        ],
        &input![term::control_codes().restore_cursor, term::Raw::from(" -l")],
    );
}

// The attrs, charsets and origin mode get saved along with the position.
#[test]
fn saved_cursor_state() {
    assert_restores(
        Size { width: 10, height: 5 },
        &input![
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().bold,
            term::ControlCodes::designate_charset(0, b'0'),
            term::control_codes().save_cursor,
            term::control_codes().disable_scroll_region_origin_mode,
            term::control_codes().clear_attrs,
            term::control_codes().designate_g0_us_ascii,
            term::ControlCodes::cursor_position(5, 1),
            term::Raw::from("x"),
        ],
        &input![
            term::control_codes().restore_cursor,
            term::Raw::from("q"),
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("q"),
        ],
    );
}

// A cursor saved in origin mode can be outside of the scroll region by the
// time of the restore, since the region might have changed since.
#[test]
fn cursor_saved_outside_of_the_scroll_region() {
    assert_restores(
        Size { width: 5, height: 5 },
        &input![
            term::ControlCodes::set_scroll_region(2, 5),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(4, 1),
            term::control_codes().save_cursor,
            term::ControlCodes::set_scroll_region(2, 3),
        ],
        &input![
            term::ControlCodes::set_scroll_region(1, 5),
            term::control_codes().restore_cursor,
            term::Raw::from("X"),
        ],
    );
}

// A cursor saved with a wrap pending wraps once it gets restored.
#[test]
fn saved_cursor_with_a_pending_wrap() {
    assert_restores(
        Size { width: 3, height: 3 },
        &input![
            term::Raw::from("abc"),
            term::control_codes().save_cursor,
            term::Raw::from("\r\nd"),
        ],
        &input![term::control_codes().restore_cursor, term::Raw::from("e")],
    );
}

// Leaving the alt screen restores the cursor that entering it saved.
#[test]
fn cursor_saved_by_the_alt_screen() {
    assert_restores(
        Size { width: 10, height: 3 },
        &input![
            term::control_codes().bold,
            term::Raw::from("$ vi"),
            term::control_codes().enable_alt_screen,
            term::control_codes().clear_attrs,
            term::Raw::from("~\r\n~"),
        ],
        &input![term::control_codes().disable_alt_screen, term::Raw::from("x")],
    );
}

// The switch to the alt screen erases it with the background color of the
// saved cursor, but that color does not belong on the alt screen.
#[test]
fn cursor_saved_by_the_alt_screen_with_a_background_color() {
    assert_restores(
        Size { width: 5, height: 2 },
        &input![
            term::ControlCodes::bgcolor_idx(4),
            term::control_codes().enable_alt_screen,
            term::control_codes().clear_attrs,
            term::control_codes().erase_screen,
            term::Raw::from("x"),
        ],
        &input![term::control_codes().disable_alt_screen, term::Raw::from("y")],
    );
}

// The alt screen has its own saved cursor.
#[test]
fn saved_cursor_on_the_alt_screen() {
    assert_restores(
        Size { width: 10, height: 3 },
        &input![
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().underline,
            term::control_codes().save_cursor,
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("status"),
        ],
        &input![term::control_codes().restore_cursor, term::Raw::from("x")],
    );
}

// Restoring the cursor when the app never saved one homes it, whatever the
// terminal had saved before the restore.
#[test]
fn unsaved_cursor() {
    assert_restores_into(
        &input![
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().bold,
            term::control_codes().save_cursor,
        ],
        Size { width: 10, height: 3 },
        b"$ ls",
        &input![term::control_codes().restore_cursor, term::Raw::from("x")],
    );
}
