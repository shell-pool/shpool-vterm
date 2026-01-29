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

use smallvec::{smallvec, SmallVec};
use std::sync::OnceLock;
use unicode_width::UnicodeWidthChar;

use crate::term::{self, AsTermInput};

static EMPTY_CELL: OnceLock<Cell> = OnceLock::new();

// A shared empty cell const. Should be used to generate empty cell
// references when needed to avoid duplicating empty cells to reference
// everywhere.
pub fn empty() -> &'static Cell {
    EMPTY_CELL.get_or_init(|| Cell::empty())
}

/// A cell in a terminal.
#[derive(Clone, Eq, PartialEq)]
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
    empty: bool,
    /// Indicates a special type of empty cell inserted after wide chars
    /// to keep the grid correctly aligned.
    wide_padding: bool,
    /// The attributes of the cell.
    attrs: term::Attrs,
}

// Prove that a `SmallVec<[char, 2]>` is just as memory efficient as a
// `SmallVec<[char, 1]>`. This is because a SmallVec is a discrimnated
// union between the array and the dynamic version of the vector, so there
// is a minimum size to the array.
static_assertions::const_assert!(
    (std::mem::size_of::<SmallVec<[char; 2]>>() == std::mem::size_of::<SmallVec<[char; 1]>>())
        || std::mem::size_of::<usize>() != 8
);

#[allow(dead_code)]
impl Cell {
    /// Create a new cell wrapping the given char.
    pub fn new(c: char, attrs: term::Attrs) -> Self {
        let width = match UnicodeWidthChar::width(c) {
            None => panic!("control chars cannot create cells"),
            Some(0) => panic!("zero width chars cannot create cells"),
            Some(w) => w,
        };

        Cell {
            grapheme_cluster: smallvec![c],
            width: width as u8,
            empty: false,
            wide_padding: false,
            attrs,
        }
    }

    pub fn empty() -> Self {
        Cell {
            grapheme_cluster: smallvec![],
            width: 0,
            empty: true,
            wide_padding: false,
            attrs: term::Attrs::default(),
        }
    }

    pub fn empty_with_attrs(attrs: term::Attrs) -> Self {
        Cell { grapheme_cluster: smallvec![], width: 0, empty: true, wide_padding: false, attrs }
    }

    pub fn wide_pad() -> Self {
        Cell {
            grapheme_cluster: smallvec![],
            width: 0,
            empty: true,
            wide_padding: true,
            attrs: term::Attrs::default(),
        }
    }

    /// Append a modifier char to the grapheme_cluster.
    pub fn add_char(&mut self, c: char) {
        assert!(UnicodeWidthChar::width(c).unwrap_or(0) > 0, "non-zero width char added to cell");

        self.grapheme_cluster.push(c);
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn attrs(&self) -> &term::Attrs {
        &self.attrs
    }
}

impl AsTermInput for Cell {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        // N.B. while cells store attributes, they are not responsible
        // for generating control codes to display them. Instead, lines
        // produce attribute control codes at cell boundaries.
        let mut utf8_buf = [0u8; 4];
        for c in self.grapheme_cluster.iter() {
            let utf8_slice = c.encode_utf8(&mut utf8_buf);
            buf.extend(utf8_slice.as_bytes());
        }

        if self.empty && !self.wide_padding {
            assert!(self.grapheme_cluster.is_empty());
            buf.push(b' ');
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.grapheme_cluster {
            write!(f, "{}", c)?;
        }

        if self.wide_padding {
            write!(f, "-")?;
        } else if self.empty {
            write!(f, "*")?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<")?;
        for c in &self.grapheme_cluster {
            write!(f, "{}", c)?;
        }

        if self.wide_padding {
            write!(f, "-")?;
        } else if self.empty {
            write!(f, "☐")?;
        }

        if self.attrs.has_attrs() {
            write!(f, "/{}", self.attrs)?;
        }

        write!(f, ">")?;
        Ok(())
    }
}
