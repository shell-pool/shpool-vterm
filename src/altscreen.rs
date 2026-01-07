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

//! The altscreen module defines the representation of the alt screen.

use std::collections::VecDeque;

use crate::{
    cell::Cell,
    line::{self, Line},
    term::{self, AsTermInput, Pos},
};

use anyhow::{anyhow, Context};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AltScreen {
    /// The entire grid the altscreen. size.height should always equal
    /// buf.len().
    ///
    /// buf[0] is at the top of the screen and buf[buf.len()-1] is at the bottom.
    pub buf: VecDeque<Line>,
}

impl AltScreen {
    pub fn new(size: crate::Size) -> Self {
        let mut buf = VecDeque::new();
        for _ in 0..size.height {
            buf.push_back(Line::new());
        }
        AltScreen { buf }
    }

    /// Write the given cell to the given cursor position, returning the next
    /// cursor position.
    pub fn write_at_cursor(
        &mut self,
        size: crate::Size,
        mut cursor: Pos,
        cell: Cell,
    ) -> anyhow::Result<Pos> {
        if size.width < 1 {
            return Err(anyhow!("cannot write to zero width terminal grid"));
        }

        let cell_width = cell.width() as usize;
        self.buf[cursor.row]
            .set_cell(size.width, cursor.col, cell)
            .context("setting cell in alt screen")?;

        cursor.col += cell_width;
        if cursor.col >= size.width {
            cursor.row += 1;
            cursor.col = 0;

            // If we are the very end, scroll by a line.
            // TODO: if `CSI ? 7 1` has been sent by the application
            // to disable scrolling, we should instead leave the cursor
            // where it is in this case.
            if cursor.row >= size.height {
                self.buf.pop_front();
                self.buf.push_back(Line::new());
            }
        }
        cursor.clamp_to(size);

        Ok(cursor)
    }

    /// Resize the alt screen. This does not perform any reflow logic,
    /// instead just trimming any cells that are no longer within the
    /// screen.
    pub fn resize(&mut self, new_size: crate::Size) {
        for line in self.buf.iter_mut() {
            line.truncate(new_size.width);
        }

        let old_height = self.buf.len();
        if new_size.height > old_height {
            for _ in 0..(new_size.height - old_height) {
                self.buf.push_back(Line::new());
            }
        } else if new_size.height < old_height {
            for _ in 0..(old_height - new_size.height) {
                self.buf.pop_back();
            }
        }
        // no-op if they have the same height
    }

    pub fn get_line_mut(&mut self, row: usize) -> &mut Line {
        assert!(row <= self.buf.len());
        &mut self.buf[row]
    }

    pub fn erase_to_end(&mut self, cursor: Pos) {
        self.buf[cursor.row].truncate(cursor.col);

        for i in (cursor.row + 1)..self.buf.len() {
            self.buf[i].truncate(0);
        }
    }

    pub fn erase_from_start(&mut self, cursor: Pos) {
        for i in 0..cursor.row {
            self.buf[i].truncate(0);
        }
        self.buf[cursor.row].erase(line::Section::StartTo(cursor.col));
    }

    pub fn erase(&mut self) {
        for line in self.buf.iter_mut() {
            line.truncate(0);
        }
    }
}

impl std::fmt::Display for AltScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.buf.iter() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl AsTermInput for AltScreen {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        for (i, line) in self.buf.iter().enumerate() {
            line.term_input_into(buf);
            if i != self.buf.len() - 1 {
                term::Crlf::default().term_input_into(buf);
            }
        }
    }
}
