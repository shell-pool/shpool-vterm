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

//! Tests for charset switching.
//!
//! Curses apps draw lines and boxes by designating the DEC special graphics
//! charset and printing plain letters, which the terminal displays as line
//! drawing chars. The screen has to hold what got displayed, and the charset
//! state itself has to survive a reattach, or everything the app draws from
//! then on comes out as letters.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion, Size, Term};

frag! {
    box_drawing { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::Raw::from("lqk"),
       term::control_codes().designate_g0_us_ascii,
       term::Raw::from("lqk")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("┌─┐lqk"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
}

frag! {
    designation_is_restored { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::Raw::from("x")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("│"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(0, b'0')
}

frag! {
    shift_out_and_back_in { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(1, b'0'),
       term::Raw::from("q\x0eq\x0fq")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("q─q"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(1, b'0')
}

frag! {
    shift_out_is_restored { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(1, b'0'),
       term::Raw::from("\x0eq")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("─"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(1, b'0'),
            term::Raw::from("\x0e")
}

frag! {
    locking_shift_2_is_restored { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(2, b'0'),
       term::control_codes().locking_shift_2,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("─"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(2, b'0'),
            term::control_codes().locking_shift_2
}

frag! {
    single_shift_lasts_for_one_char { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(3, b'0'),
       term::control_codes().single_shift_3,
       term::Raw::from("qq")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("─q"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(3, b'0')
}

frag! {
    uk_pound_sign { scrollback_lines: 100, width: 10, height: 3 }
    <= term::control_codes().designate_g0_uk_ascii,
       term::Raw::from("#1")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("£1"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().designate_g0_uk_ascii
}

frag! {
    only_ascii_gets_translated { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::Raw::from("é中q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("é中─"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(0, b'0')
}

frag! {
    repeat_repeats_the_displayed_char { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::Raw::from("q"),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("────"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(0, b'0')
}

frag! {
    unsupported_charset_is_ascii { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::ControlCodes::designate_charset(0, b'K'),
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("q"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    restore_cursor_restores_charsets { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::control_codes().save_cursor,
       term::control_codes().designate_g0_us_ascii,
       term::control_codes().restore_cursor,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("─"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(0, b'0')
}

// Like in xterm, DECSC and DECRC leave a pending single shift alone. It still
// applies to the next char printed.
frag! {
    restore_cursor_keeps_a_pending_single_shift { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(2, b'0'),
       term::control_codes().save_cursor,
       term::control_codes().single_shift_2,
       term::control_codes().restore_cursor,
       term::Raw::from("qq")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("─q"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(2, b'0')
}

// And a single shift that has been used up does not come back.
frag! {
    restore_cursor_does_not_bring_back_a_used_single_shift { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(2, b'0'),
       term::control_codes().single_shift_2,
       term::control_codes().save_cursor,
       term::Raw::from("q"),
       term::control_codes().restore_cursor,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("q"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::designate_charset(2, b'0')
}

frag! {
    soft_reset_resets_charsets { scrollback_lines: 100, width: 10, height: 3 }
    <= term::ControlCodes::designate_charset(0, b'0'),
       term::control_codes().soft_reset,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("q"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// An app that switched charsets before the user detached still expects to be
// in them after the reattach.
#[test]
fn charsets_survive_a_restore() {
    let size = Size { width: 10, height: 3 };
    let mut session = Term::new(100, size);
    session.process(b"\x1b)0\x1b*A\x0e");

    let mut client = Term::new(100, size);
    client.process(&session.contents(ContentRegion::All));

    for term in [&mut session, &mut client] {
        term.process(b"lqk\x0f\x1bN#q");
    }
    let contents = client.contents(ContentRegion::All);
    assert_eq!(contents, session.contents(ContentRegion::All));
    assert!(String::from_utf8_lossy(&contents).contains("┌─┐£q"));
}
