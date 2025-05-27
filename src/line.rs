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

use tracing::{error, warn};
use unicode_width::UnicodeWidthChar;

use crate::cell::Cell;
use crate::term::BufWrite;

/// A logical line. May be arbitrarily long and will only get wrapped
/// when transforming the term into a grid view.
#[derive(Debug, Clone)]
pub struct Logical {
    contents: Vec<Cell>,
}

impl Logical {
    pub fn new() -> Self {
        Logical { contents: vec![] }
    }

    /// Insert the given char into the line.
    pub fn print(&mut self, c: char) {
        match UnicodeWidthChar::width(c) {
            Some(0) => {
                if let Some(last_cell) = self.contents.last_mut() {
                    last_cell.add_char(c);
                } else {
                    warn!("zero-width char written to empty line, dropping");
                }
            }
            Some(_) => self.contents.push(Cell::new(c)),
            None => error!("control char passed to line::Logical.push"),
        }
    }
}

impl BufWrite for Logical {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        for c in self.contents.iter() {
            c.write_buf(buf);
        }
    }
}
