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
