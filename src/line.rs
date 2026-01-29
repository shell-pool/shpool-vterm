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
pub struct Line {
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

        for cell in self.cells.iter() {
            if cell.attrs() != current_attrs {
                for code in current_attrs.transition_to(cell.attrs()) {
                    code.term_input_into(buf);
                }
                current_attrs = cell.attrs();
            }
            cell.term_input_into(buf);
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
    #[allow(dead_code)]
    pub fn get_cell(&self, width: usize, col: usize) -> Option<&Cell> {
        if col >= width {
            return None;
        }

        if col >= self.cells.len() {
            return Some(cell::empty());
        }

        return Some(&self.cells[col]);
    }

    /// Set the given column to the given cell.
    pub fn set_cell(&mut self, width: usize, col: usize, cell: Cell) -> anyhow::Result<()> {
        if col >= width {
            return Err(anyhow!("{} out of bounds (width={})", col, width));
        }

        if col >= self.cells.len() {
            while self.cells.len() < col {
                self.cells.push(Cell::empty())
            }
            self.cells.push(cell);
            return Ok(());
        }

        self.cells[col] = cell;
        Ok(())
    }

    /// Trim the line to the new width, dropping any cells too far to the right.
    pub fn truncate(&mut self, width: usize) {
        self.cells.truncate(width);
    }

    /// Clobber the given section, either by trimming the underlying storage
    /// or by overwriting with empty cells.
    pub fn erase(&mut self, section: Section) {
        match section {
            Section::StartTo(col) => {
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
        let empties = vec![Cell::empty(); n];
        self.cells.splice(col..col, empties);
        self.cells.truncate(width);
    }

    /// Delete n cells at the current position, sucking cells to the
    /// right towards the cursor, and backfilling their old position
    /// with empty cells that have the current background attributes
    /// set.
    ///
    /// This implements DCH (Delete Character).
    pub fn delete_character(&mut self, width: usize, col: usize, attrs: &term::Attrs, n: usize) {
        let delete_to = std::cmp::min(self.cells.len(), col + n);
        let num_to_delete = delete_to - col;

        self.cells.drain(col..delete_to);

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
}

/// Specify a region of the line.
pub enum Section {
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
    fn set() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let c1 = Cell::new('a', term::Attrs::default());
        let c2 = Cell::new('b', term::Attrs::default());

        // Set within current length (needs push first to not be out of bounds of vector
        // if we treated it strictly, but set() handles extension)

        // set at 0
        line.set_cell(width, 0, c1.clone())?;
        assert_eq!(line.get_cell(width, 0), Some(&c1));

        // set at 2 (should pad with empty)
        line.set_cell(width, 2, c2.clone())?;
        assert_eq!(line.get_cell(width, 0), Some(&c1));
        assert!(line.get_cell(width, 1).unwrap().is_empty());
        assert_eq!(line.get_cell(width, 2), Some(&c2));

        Ok(())
    }

    #[test]
    fn set_oob() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        match line.set_cell(width, 5, Cell::new('a', term::Attrs::default())) {
            Err(e) => assert!(format!("{e:?}").contains("out of bounds")),
            _ => assert!(false, "expected out of bounds error"),
        }

        Ok(())
    }
}
