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
