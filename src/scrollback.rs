// Copyright 2025 Google LLC
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

use std::collections::VecDeque;

use crate::line;

/// The scrollback buffer of logical lines.
///
/// We only really have this for ownership reasons. Otherwise
/// it would all just be inlined in the Term struct.
pub struct Buffer {
    size: crate::Size,
    pub lines: VecDeque<line::Logical>,
}

impl Buffer {
    /// Create a new scrollback buffer with the given width and height.
    ///
    /// Note that width will only be used when generated output
    /// to determine where wrapping should be place.
    pub fn new(size: crate::Size) -> Self {
        let mut b = Buffer {
            size,
            lines: VecDeque::with_capacity(size.height),
        };
        b.push_line(line::Logical::new());
        b
    }

    /// Get the current terminal size.
    pub fn size(&self) -> crate::Size {
        self.size
    }

    /// Set the terminal size.
    pub fn set_size(&mut self, size: crate::Size) {
        self.size = size;
    }

    //
    // Internal helpers
    //

    /// Push a new line, making sure old lines are discared.
    fn push_line(&mut self, line: line::Logical) {
        // TODO: if lines maintain a mapping to their grid position,
        // this would invalidate that mapping. It should either be
        // recomputed or if we are lazy about it, invalidated.
        self.lines.truncate(self.size.height - 1);
        self.lines.push_front(line);
    }
}

impl vte::Perform for Buffer {
    fn print(&mut self, c: char) {
        self.lines[0].print(c);
    }

    fn execute(&mut self, _byte: u8) {
        // TODO: stub
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // TODO: stub
    }

    fn put(&mut self, _byte: u8) {
        // TODO: stub
    }

    fn unhook(&mut self) {
        // TODO: stub
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // TODO: stub
    }

    fn csi_dispatch(
        &mut self,
        _params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
        // TODO: stub
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // TODO: stub
    }

    fn terminated(&self) -> bool {
        // TODO: stub
        false
    }
}
