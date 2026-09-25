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

//! The line module defines the representation of the fixed width lines
//! of cells. Lines are used both in the "scrollback" main screen and in the
//! alt screen.

use crate::{
    cell::{self, Cell},
    term::{self, AsTermInput},
};

use anyhow::anyhow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Line {
    /// The cells stored in this line.
    pub cells: Vec<Cell>,
    /// If true, indicates that this line was automatically wrapped due to
    /// the terminal width. The following line is part of the same logical
    /// line and should be reflowed together with this line on terminal resize.
    pub is_wrapped: bool,
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cell in &self.cells {
            write!(f, "{}", cell)?;
        }
        if self.is_wrapped {
            writeln!(f, "⏎")?;
        } else {
            writeln!(f)?;
        }
        Ok(())
    }
}

impl AsTermInput for Line {
    // We start every line with blank attrs, and it is our responsibility
    // to reset the attrs at the end of each line. We could produce more
    // optimal output if we fused attr runs across lines, but it is probably
    // fine to just do it like this and it is better to keep things simple
    // unless we need to fuse.
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        let blank_attrs = term::Attrs::default();
        let mut current_attrs = &blank_attrs;
        // The column up to which the chars we have emitted so far reach.
        let mut covered_until = 0;

        for (col, cell) in self.cells.iter().enumerate() {
            if cell.attrs() != current_attrs {
                for code in current_attrs.transition_to(cell.attrs()) {
                    code.term_input_into(buf);
                }
                current_attrs = cell.attrs();
            }

            if cell.is_wide_padding() {
                // Padding normally sits under the right half of the wide char
                // in front of it and has nothing to emit. If that wide char
                // is gone, emit a blank so the columns after it still line up.
                if col >= covered_until {
                    buf.push(b' ');
                }
                continue;
            }
            cell.term_input_into(buf);
            covered_until = col + std::cmp::max(cell.width() as usize, 1);
        }

        if current_attrs != &blank_attrs {
            for code in current_attrs.transition_to(&blank_attrs) {
                code.term_input_into(buf);
            }
        }
    }
}

/// A line contains a list of cells.
///
/// Note that a line can't really be used on its own because the grid
/// width is not stored within the line. For this reason, a line is really
/// an internal implementation detail of a grid, since most operations need
/// to have the grid width passed down by the grid implementation.
impl Line {
    pub fn new() -> Self {
        Line { cells: vec![], is_wrapped: false }
    }

    /// Get the cell at the given grid position.
    pub fn get_cell(&self, width: usize, col: usize) -> Option<&Cell> {
        if col >= width {
            return None;
        }

        if col >= self.cells.len() {
            return Some(cell::empty());
        }

        return Some(&self.cells[col]);
    }

    /// Get a mutable reference to the cell at the given grid position.
    ///
    /// Unlike `get_cell` this does not conjure up a blank cell for positions
    /// past the end of the stored cells, since there is nothing there to
    /// modify.
    pub fn get_cell_mut(&mut self, width: usize, col: usize) -> Option<&mut Cell> {
        if col >= width || col >= self.cells.len() {
            return None;
        }

        Some(&mut self.cells[col])
    }

    /// Write `cell` at `col`, followed by the padding cells that keep the
    /// columns lined up if it is a wide char.
    ///
    /// A wide char that the write only covers one half of gets blanked out
    /// entirely, like real terminals do. Leaving the other half behind would
    /// throw off the columns of everything after it when we dump the line.
    pub fn write_cell(&mut self, width: usize, col: usize, cell: Cell) -> anyhow::Result<()> {
        let end = col + std::cmp::max(cell.width() as usize, 1);
        if end > width {
            return Err(anyhow!("{}..{} out of bounds (width={})", col, end, width));
        }

        while self.cells.len() < end {
            self.cells.push(Cell::empty());
        }
        self.split_wide_char_at(col);
        self.split_wide_char_at(end);

        for pad_col in col + 1..end {
            self.cells[pad_col] = Cell::wide_pad(cell.attrs().clone());
        }
        self.cells[col] = cell;

        Ok(())
    }

    /// Blank out the wide char that straddles the boundary between `col - 1`
    /// and `col`, if there is one.
    ///
    /// This has to happen before any edit that touches the cells on only one
    /// side of the boundary, since the edit would otherwise leave half of the
    /// wide char behind.
    fn split_wide_char_at(&mut self, col: usize) {
        if col >= self.cells.len() || !self.cells[col].is_wide_padding() {
            return;
        }

        let mut start = col;
        while start > 0 && self.cells[start].is_wide_padding() {
            start -= 1;
        }
        // Only blank the char we walked back to if it is really the wide
        // char that owns this padding.
        let owner = &self.cells[start];
        if !owner.is_wide_padding() && start + (owner.width() as usize) <= col {
            start += 1;
        }
        let mut end = col + 1;
        while end < self.cells.len() && self.cells[end].is_wide_padding() {
            end += 1;
        }

        let attrs = self.cells[start].attrs().clone();
        for cell in self.cells[start..end].iter_mut() {
            *cell = Cell::empty_with_attrs(attrs.clone());
        }
    }

    /// Trim the line to the new width, dropping any cells too far to the right.
    pub fn truncate(&mut self, width: usize) {
        // A wide char that would stick out past the new edge goes too.
        self.split_wide_char_at(width);
        self.cells.truncate(width);
    }

    /// Clobber the given section, either by trimming the underlying storage
    /// or by overwriting with empty cells.
    pub fn erase(&mut self, section: Section) {
        match section {
            Section::StartTo(col) => {
                self.split_wide_char_at(col + 1);
                for i in 0..std::cmp::min(col + 1, self.cells.len()) {
                    self.cells[i] = Cell::empty();
                }
            }
            Section::ToEnd(col) => {
                self.truncate(col);
                self.is_wrapped = false;
            }
            Section::Whole => {
                self.truncate(0);
                self.is_wrapped = false;
            }
        }
    }

    /// Insert n new blank cells at the current position, dropping
    /// any cells which spill over width.
    pub fn insert_character(&mut self, width: usize, col: usize, n: usize) {
        if col >= width {
            return;
        }
        while self.cells.len() < col {
            self.cells.push(Cell::empty());
        }
        self.split_wide_char_at(col);
        let empties = vec![Cell::empty(); std::cmp::min(n, width - col)];
        self.cells.splice(col..col, empties);
        self.truncate(width);
    }

    /// Delete n cells at the current position, sucking cells to the
    /// right towards the cursor, and backfilling their old position
    /// with empty cells that have the current background attributes
    /// set.
    ///
    /// This implements DCH (Delete Character).
    pub fn delete_character(&mut self, width: usize, col: usize, attrs: &term::Attrs, n: usize) {
        if col >= width {
            return;
        }

        self.split_wide_char_at(col);
        self.split_wide_char_at(col.saturating_add(n));

        // Everything past the end of the cells buffer is implicitly blank, so
        // when the cursor sits out there we have no physical cells to remove
        // and the drain range collapses to an empty one at the end.
        let delete_from = std::cmp::min(col, self.cells.len());
        let delete_to = std::cmp::min(self.cells.len(), col.saturating_add(n));
        let num_to_delete = delete_to - delete_from;

        self.cells.drain(delete_from..delete_to);

        // Inject the empty cells that were logically already present
        // when the cells buffer was short.
        while self.cells.len() < width - num_to_delete {
            self.cells.push(Cell::empty());
        }

        // Inject the "backfill" cells that the semantics of DCH call
        // for. These are empty cells with the current attributes set.
        while self.cells.len() < width {
            self.cells.push(Cell::empty_with_attrs(attrs.clone()));
        }
    }

    /// Blank out n characters to the right of the current cursor. Unlike
    /// delete, this leaves the cells in place, just clobbers their contents.
    ///
    /// This implements ECH (Erase Character)
    pub fn erase_character(&mut self, width: usize, col: usize, attrs: &term::Attrs, n: usize) {
        if col >= width {
            return;
        }

        while self.cells.len() < col {
            self.cells.push(Cell::empty());
        }
        self.split_wide_char_at(col);
        self.split_wide_char_at(col.saturating_add(n));

        let erase_to = std::cmp::min(width, col.saturating_add(n));
        for i in col..std::cmp::min(self.cells.len(), erase_to) {
            self.cells[i] = Cell::empty_with_attrs(attrs.clone());
        }
        while self.cells.len() < erase_to {
            self.cells.push(Cell::empty_with_attrs(attrs.clone()));
        }
    }
}

/// Specify a region of the line.
pub(crate) enum Section {
    StartTo(usize),
    ToEnd(usize),
    Whole,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let line = Line::new();
        assert!(line.cells.is_empty());
        assert!(!line.is_wrapped);
    }

    #[test]
    fn write() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let c1 = Cell::new('a', term::Attrs::default());
        let c2 = Cell::new('b', term::Attrs::default());

        // Set within current length (needs push first to not be out of bounds
        // of vector if we treated it strictly, but write_cell() handles
        // extension)

        // set at 0
        line.write_cell(width, 0, c1.clone())?;
        assert_eq!(line.get_cell(width, 0), Some(&c1));

        // set at 2 (should pad with empty)
        line.write_cell(width, 2, c2.clone())?;
        assert_eq!(line.get_cell(width, 0), Some(&c1));
        assert!(line.get_cell(width, 1).unwrap().is_empty());
        assert_eq!(line.get_cell(width, 2), Some(&c2));

        Ok(())
    }

    #[test]
    fn write_oob() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        match line.write_cell(width, 5, Cell::new('a', term::Attrs::default())) {
            Err(e) => assert!(format!("{e:?}").contains("out of bounds")),
            _ => assert!(false, "expected out of bounds error"),
        }

        // A wide char has to fit entirely.
        match line.write_cell(width, 4, Cell::new('😊', term::Attrs::default())) {
            Err(e) => assert!(format!("{e:?}").contains("out of bounds")),
            _ => assert!(false, "expected out of bounds error"),
        }

        Ok(())
    }

    #[test]
    fn write_wide() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let mut attrs = term::Attrs::default();
        attrs.italic = true;

        line.write_cell(width, 1, Cell::new('😊', attrs.clone()))?;
        assert_eq!(line.cells.len(), 3);
        assert!(line.cells[0].is_empty());
        assert_eq!(line.cells[1].width(), 2);
        assert!(line.cells[2].is_wide_padding());
        assert_eq!(line.cells[2].attrs(), &attrs);

        Ok(())
    }

    #[test]
    fn overwrite_half_of_wide() -> anyhow::Result<()> {
        let width = 5;
        let wide = Cell::new('😊', term::Attrs::default());
        let narrow = Cell::new('x', term::Attrs::default());

        // Right half.
        let mut line = Line::new();
        line.write_cell(width, 0, wide.clone())?;
        line.write_cell(width, 1, narrow.clone())?;
        assert_eq!(format!("{line}"), "*x\n");

        // Left half.
        let mut line = Line::new();
        line.write_cell(width, 0, wide.clone())?;
        line.write_cell(width, 0, narrow.clone())?;
        assert_eq!(format!("{line}"), "x*\n");

        // A wide char over the halves of two others.
        let mut line = Line::new();
        line.write_cell(width, 0, wide.clone())?;
        line.write_cell(width, 2, wide.clone())?;
        line.write_cell(width, 1, wide.clone())?;
        assert_eq!(format!("{line}"), "*😊-*\n");

        Ok(())
    }

    #[test]
    fn truncate_through_wide() -> anyhow::Result<()> {
        let width = 5;
        let mut line = Line::new();
        line.write_cell(width, 0, Cell::new('a', term::Attrs::default()))?;
        line.write_cell(width, 1, Cell::new('😊', term::Attrs::default()))?;
        line.truncate(2);
        assert_eq!(format!("{line}"), "a*\n");

        Ok(())
    }

    #[test]
    fn orphaned_padding_dumps_as_blank() {
        // Nothing should be able to produce a line like this, but if something
        // does we still want the columns after it to line up.
        let mut line = Line::new();
        line.cells.push(Cell::new('a', term::Attrs::default()));
        line.cells.push(Cell::wide_pad(term::Attrs::default()));
        line.cells.push(Cell::new('b', term::Attrs::default()));

        let mut buf = vec![];
        line.term_input_into(&mut buf);
        assert_eq!(buf, b"a b");
    }
}
