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

//! Tests for the `ESC #` sequences: DECALN (ESC # 8), and the double
//! width/height line codes (ESC # 3 through 6), which we ignore.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

frag! {
    decaln_fills_screen { scrollback_lines: 10, width: 5, height: 3 }
    <= term::Raw::from("ab\r\ncd"),
       term::Raw::from("\x1b#8")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    decaln_resets_scroll_region_and_origin_mode { scrollback_lines: 10, width: 5, height: 3 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("\x1b#8")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    decaln_fills_with_plain_cells { scrollback_lines: 10, width: 5, height: 3 }
    <= term::ControlCodes::bgcolor_idx(4),
       term::Raw::from("\x1b#8")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            // The cursor attrs are left alone.
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    decaln_fills_alt_screen { scrollback_lines: 10, width: 5, height: 3 }
    <= term::Raw::from("ab"),
       term::control_codes().enable_alt_screen,
       term::Raw::from("\x1b#8")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::Crlf::default(),
            term::Raw::from("EEEEE"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    double_width_and_height_lines_ignored { scrollback_lines: 10, width: 10, height: 5 }
    <= term::Raw::from("\x1b#3a\r\n\x1b#4b\r\n\x1b#5c\r\n\x1b#6d")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::Crlf::default(),
            term::Raw::from("b"),
            term::Crlf::default(),
            term::Raw::from("c"),
            term::Crlf::default(),
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(4, 2),
            term::control_codes().clear_attrs
}
