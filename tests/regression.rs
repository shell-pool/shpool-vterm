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
