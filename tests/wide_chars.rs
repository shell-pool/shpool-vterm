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

//! Tests for wide chars (most emoji and CJK), which take up two columns.
//!
//! A wide char that only has one of its halves overwritten or erased has to
//! go away completely, like it does in a real terminal. If the other half
//! stuck around, every char after it on the line would get restored one
//! column off.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion, Size, Term};

frag! {
    overwrite_right_half_of_wide_char { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("😊b"),
       term::ControlCodes::cursor_position(1, 2),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from(" Xb"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    overwrite_left_half_of_wide_char { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("😊b"),
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("X b"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    wide_char_over_halves_of_two_wide_chars { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊😊"),
       term::ControlCodes::cursor_position(1, 3),
       term::Raw::from("😃")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a 😃"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    erase_to_end_of_line_from_right_half { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊b"),
       term::ControlCodes::cursor_position(1, 3),
       term::control_codes().erase_to_end_of_line
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    erase_to_start_of_line_through_left_half { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊b"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().erase_to_start_of_line
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("   b"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    erase_character_on_right_half { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊b"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a  b"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    delete_character_on_right_half { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊b"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a b"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

// Pushing a wide char halfway off the right edge removes it.
frag! {
    insert_character_pushes_wide_char_off_the_edge { scrollback_lines: 100, width: 4, height: 2 }
    <= term::Raw::from("ab😊"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from(" ab"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    insert_character_in_the_middle_of_a_wide_char { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("a😊b"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a   b"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

// The padding cell to the right of a wide char has the same attrs as the
// char itself, so an attr run does not get interrupted by it.
frag! {
    attrs_run_through_a_wide_char { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().bold,
       term::Raw::from("😊a"),
       term::control_codes().clear_attrs
    => ContentRegion::All =>
            reset_codes,
            term::control_codes().bold,
            term::Raw::from("😊a"),
            term::control_codes().reset_font_weight,
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

// The alt screen just cuts lines off when it gets narrower, and a wide char
// that would end up sticking out past the edge has to go.
#[test]
fn alt_screen_narrowing_resize_cuts_wide_char() {
    let mut term = Term::new(100, Size { width: 4, height: 2 });
    term.process(b"\x1b[?1049h");
    term.process("ab😊".as_bytes());
    term.resize(Size { width: 3, height: 2 });

    // The cursor was waiting to wrap at the right edge, and still is. What
    // is left of the wide char is a blank, which is what gets printed in the
    // last column to get the wrap back.
    use shpool_vterm::term::AsTermInput;
    let mut want = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut want);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut want);
    term::control_codes().enable_alt_screen.term_input_into(&mut want);
    term::Raw::from("ab").term_input_into(&mut want);
    term::Crlf::default().term_input_into(&mut want);
    term::ControlCodes::cursor_position(1, 3).term_input_into(&mut want);
    term::Raw::from(" ").term_input_into(&mut want);
    term::control_codes().clear_attrs.term_input_into(&mut want);
    assert_eq!(
        String::from_utf8_lossy(term.contents(ContentRegion::All).as_slice()),
        String::from_utf8_lossy(&want),
    );
}
