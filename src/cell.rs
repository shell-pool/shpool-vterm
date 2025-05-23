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

use unicode_width::UnicodeWidthChar;
use smallvec::{SmallVec, smallvec};

use crate::term::BufWrite;

/// A cell in a terminal.
#[derive(Debug, Clone)]
pub struct Cell {
    /// The contents of a cell. Usually just a single character,
    /// but in some cases there might be modifier codepoints like
    /// from diacritics or emoji modifiers.
    ///
    /// We use an inline length of 2 because on 64 bit systems
    /// it has the same memory footprint as an inline length of 1.
    /// (see below static assertion).
    grapheme_cluster: SmallVec<[char; 2]>,
    /// The width of the cell. It might be more than one if the grapheme cluster
    /// is wide. Some emojis and east asian characters have this property.
    width: u8,
    // TODO: add attrs like underline and whatnot
}

// Prove that a `SmallVec<[char, 2]>` is just as memory efficient as a
// `SmallVec<[char, 1]>`. This is because a SmallVec is a discrimnated
// union between the array and the dynamic version of the vector, so there
// is a minimum size to the array.
static_assertions::const_assert!(
    (std::mem::size_of::<SmallVec<[char; 2]>>() == std::mem::size_of::<SmallVec<[char; 1]>>())
    || std::mem::size_of::<usize>() != 8);


impl Cell {
    /// Create a new cell wrapping the given char.
    pub fn new(c: char) -> Self {
        let width = match UnicodeWidthChar::width(c) {
            None => panic!("control chars cannot create cells"),
            Some(0) => panic!("zero width chars cannot create cells"),
            Some(w) => w,
        };

        Cell {
            grapheme_cluster: smallvec![c],
            width: width as u8,
        }
    }

    /// Append a modifier char to the grapheme_cluster.
    pub fn add_char(&mut self, c: char) {
        assert!(UnicodeWidthChar::width(c).unwrap_or(0) > 0,
            "non-zero width char added to cell");

        self.grapheme_cluster.push(c);
    }
}

impl BufWrite for Cell {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        // TODO: apply attrs once we implement support for them by
        // transforming them into the appropriate escape sequences.
        
        let mut utf8_buf = [0u8; 4];
        for c in self.grapheme_cluster.iter() {
            let utf8_slice = c.encode_utf8(&mut utf8_buf);
            buf.extend(utf8_slice.as_bytes());
        }
    }
}
