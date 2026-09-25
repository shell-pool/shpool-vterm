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

//! Tests for RIS (ESC c), which resets the whole terminal, and DECSTR
//! (CSI ! p), which only resets the modes that apps tend to leave behind.

#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

// Like in xterm and kitty, RIS clears the scrollback along with the screen.
frag! {
    ris_clears_the_screen_and_scrollback { scrollback_lines: 100, width: 10, height: 2 }
    <= term::Raw::from("11\r\n22\r\n33"),
       term::control_codes().hard_reset,
       term::Raw::from("x")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    ris_leaves_the_alt_screen { scrollback_lines: 100, width: 10, height: 3 }
    <= term::Raw::from("main\x1b[?1049halt"),
       term::control_codes().hard_reset,
       term::Raw::from("x")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("x"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_modes { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\x1b[?1h\x1b=\x1b[?1000h\x1b[?1006h\x1b[?1004h\x1b[?2004h"),
       term::Raw::from("\x1b[?25l\x1b[?12h\x1b[4h\x1b[?7l\x1b[5 q\x1b(0"),
       term::Raw::from("\x1b[2;4r\x1b[?6h\x1b]4;1;rgb:ff/00/00\x07\x1b[1;31m"),
       term::control_codes().hard_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_the_saved_cursor { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\r\n  abc\x1b7"),
       term::control_codes().hard_reset,
       term::Raw::from("x\x1b8y")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("y"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// The title describes the session rather than the state of the terminal, and
// xterm keeps it too.
frag! {
    ris_keeps_the_title { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\x1b]2;hi\x07"),
       term::control_codes().hard_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'h', b'i'])
}

frag! {
    decstr_keeps_the_screen_and_the_cursor { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("ab\r\ncd"),
       term::control_codes().soft_reset,
       term::Raw::from("e")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ab"),
            term::Crlf::default(),
            term::Raw::from("cde"),
            term::ControlCodes::cursor_position(2, 4),
            term::control_codes().clear_attrs
}

frag! {
    decstr_resets_modes { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\x1b[?1h\x1b=\x1b[?25l\x1b[?12h\x1b[4h\x1b[?7l\x1b[5 q\x1b(0"),
       term::Raw::from("\x1b]4;1;rgb:ff/00/00\x07\x1b[1;31m"),
       term::control_codes().soft_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// Unlike DECSTBM and DECOM, DECSTR drops the scroll region and origin mode
// without homing the cursor.
frag! {
    decstr_resets_the_scroll_region_and_origin_mode { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\x1b[2;4r\x1b[?6h\x1b[2;2Hx"),
       term::control_codes().soft_reset,
       term::Raw::from("y")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" xy"),
            term::ControlCodes::cursor_position(3, 4),
            term::control_codes().clear_attrs
}

frag! {
    decstr_resets_the_saved_cursor { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("abc\x1b7"),
       term::control_codes().soft_reset,
       term::Raw::from("\x1b8x")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("xbc"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// Like in xterm, the modes that change what the terminal reports only get
// reset by RIS.
frag! {
    decstr_keeps_reporting_modes { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("\x1b[?1000h\x1b[?1004h\x1b[?2004h"),
       term::control_codes().soft_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_report_focus,
            term::control_codes().enable_paste_mode,
            term::ControlCodes::dec_private_modes_set(&[1000])
}
