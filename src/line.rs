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
        let end = self.painted_width();
        for (col, cell) in self.cells[..end].iter().enumerate() {
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

    /// A line full of `fill`, the blank that erasing and scrolling leave
    /// behind (see `Cell::blank`).
    ///
    /// Without a background color to paint, a blank looks just like a cell
    /// nothing has been written to, so there is no need to store any.
    pub fn blank(width: usize, fill: &Cell) -> Self {
        let mut line = Line::new();
        line.fill_to(width, fill);
        line
    }

    /// Pad the line out to `width` with `fill` if `fill` has a background
    /// color to show. Otherwise the implicit blank cells past the end of the
    /// line already look right.
    fn fill_to(&mut self, width: usize, fill: &Cell) {
        if fill.attrs().has_attrs() {
            self.cells.resize(std::cmp::max(width, self.cells.len()), fill.clone());
        }
    }

    /// Store the implicit blank cells past the end of the line out to `len`,
    /// for an edit that moves cells around in front of there.
    fn pad_to(&mut self, len: usize) {
        if self.cells.len() < len {
            self.cells.resize(len, Cell::empty());
        }
    }

    /// How many columns painting the line covers.
    ///
    /// Blanks at the end of the line look just like the cells past the end
    /// of it that nothing has been written to, so there is no need to paint
    /// them.
    fn painted_width(&self) -> usize {
        self.cells.iter().rposition(|cell| !cell.looks_unused()).map_or(0, |i| i + 1)
    }

    /// Whether painting `next` right after this line wraps onto it, the same
    /// way that printing it did in the first place.
    ///
    /// That takes a line that wrapped, and a char at the start of `next`
    /// that makes the terminal wrap: any char at all once the line reaches
    /// the last column, or a wide char that does not fit in the columns the
    /// line leaves free.
    fn wraps_onto(&self, next: &Line, width: usize) -> bool {
        if !self.is_wrapped || next.painted_width() == 0 {
            return false;
        }
        let first_width = next.cells.first().map_or(1, |cell| std::cmp::max(cell.width(), 1));
        self.painted_width() + first_width as usize > width
    }

    /// Emit the codes that wrap the line above this one onto it, before it
    /// gets painted.
    ///
    /// Printing the first char of the line is what wraps. If that scrolls
    /// the screen, the terminal fills the new row with the background color
    /// the char gets printed with, which the row the app printed the line
    /// on did not necessarily get filled with. So a char with a background
    /// color gets printed without one first, and then the cursor goes back
    /// to the start of the row for the painting to print it again.
    fn dump_wrap_into(&self, buf: &mut Vec<u8>) {
        let Some(first) = self.cells.first() else {
            return;
        };
        if matches!(first.attrs().bgcolor, term::Color::Default) {
            return;
        }
        if first.is_wide_padding() {
            buf.push(b' ');
        } else {
            first.term_input_into(buf);
        }
        buf.push(b'\r');
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
    /// or by overwriting with `fill`, the blank erasing leaves behind.
    pub fn erase(&mut self, width: usize, section: Section, fill: &Cell) {
        match section {
            Section::StartTo(col) => {
                let end = std::cmp::min(col + 1, width);
                self.split_wide_char_at(end);
                if fill.attrs().has_attrs() {
                    while self.cells.len() < end {
                        self.cells.push(Cell::empty());
                    }
                }
                for i in 0..std::cmp::min(end, self.cells.len()) {
                    self.cells[i] = fill.clone();
                }
            }
            Section::ToEnd(col) => {
                self.truncate(col);
                self.is_wrapped = false;
                if fill.attrs().has_attrs() {
                    while self.cells.len() < col {
                        self.cells.push(Cell::empty());
                    }
                    self.fill_to(width, fill);
                }
            }
            Section::Whole => {
                self.truncate(0);
                self.is_wrapped = false;
                self.fill_to(width, fill);
            }
        }
    }

    /// Insert n new blank cells at the current position, dropping
    /// any cells which spill over width.
    pub fn insert_character(&mut self, width: usize, col: usize, n: usize, fill: &Cell) {
        if col >= width {
            return;
        }
        while self.cells.len() < col {
            self.cells.push(Cell::empty());
        }
        self.split_wide_char_at(col);
        let blanks = vec![fill.clone(); std::cmp::min(n, width - col)];
        self.cells.splice(col..col, blanks);
        self.truncate(width);
    }

    /// Like `insert_character`, but with the right margin at `right`. Only
    /// the cells up to the margin shift over, and the ones that get pushed
    /// past it are lost. Nothing happens at or past the margin.
    pub fn insert_character_in(
        &mut self,
        width: usize,
        col: usize,
        right: usize,
        n: usize,
        fill: &Cell,
    ) {
        if right >= width {
            self.insert_character(width, col, n, fill);
            return;
        }
        if col >= right {
            return;
        }

        let n = std::cmp::min(n, right - col);
        self.pad_to(right);
        self.split_wide_char_at(col);
        self.split_wide_char_at(right - n);
        self.split_wide_char_at(right);
        self.cells.drain(right - n..right);
        self.cells.splice(col..col, std::iter::repeat(fill.clone()).take(n));
    }

    /// Delete n cells at the current position, sucking cells to the
    /// right towards the cursor, and backfilling their old position
    /// with `fill`, the blank erasing leaves behind.
    ///
    /// This implements DCH (Delete Character).
    pub fn delete_character(&mut self, width: usize, col: usize, fill: &Cell, n: usize) {
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
        // for.
        while self.cells.len() < width {
            self.cells.push(fill.clone());
        }
    }

    /// Like `delete_character`, but with the right margin at `right`. Only
    /// the cells up to the margin get pulled in, and the backfill goes in
    /// front of it. Nothing happens at or past the margin.
    pub fn delete_character_in(
        &mut self,
        width: usize,
        col: usize,
        right: usize,
        fill: &Cell,
        n: usize,
    ) {
        if right >= width {
            self.delete_character(width, col, fill, n);
            return;
        }
        if col >= right {
            return;
        }

        let n = std::cmp::min(n, right - col);
        self.pad_to(right);
        self.split_wide_char_at(col);
        self.split_wide_char_at(col + n);
        self.split_wide_char_at(right);
        self.cells.drain(col..col + n);
        self.cells.splice(right - n..right - n, std::iter::repeat(fill.clone()).take(n));
    }

    /// Blank out n characters to the right of the current cursor. Unlike
    /// delete, this leaves the cells in place, just clobbers their contents.
    ///
    /// This implements ECH (Erase Character)
    pub fn erase_character(&mut self, width: usize, col: usize, fill: &Cell, n: usize) {
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
            self.cells[i] = fill.clone();
        }
        while self.cells.len() < erase_to {
            self.cells.push(fill.clone());
        }
    }

    /// Put `cells` in place of the ones starting at `col`, handing back the
    /// cells that were there. A wide char that straddles either end gets
    /// blanked out first, since only one half of it would go.
    pub fn swap_cells(&mut self, col: usize, cells: Vec<Cell>) -> Vec<Cell> {
        let end = col + cells.len();
        self.pad_to(end);
        self.split_wide_char_at(col);
        self.split_wide_char_at(end);
        self.cells.splice(col..end, cells).collect()
    }
}

/// Emit the codes that paint `lines`, from the top down, each on the row
/// below the one before.
///
/// Instead of breaking a line that wrapped onto the next one off from it
/// with a CRLF, we let the terminal wrap it again where we can, so that it
/// knows that the two belong together. That way it can still copy them as
/// one line, and reflow them when its window gets resized.
pub fn dump_lines_into<'a>(buf: &mut Vec<u8>, width: usize, lines: impl Iterator<Item = &'a Line>) {
    let mut lines = lines.peekable();
    let mut wrapping = false;
    while let Some(line) = lines.next() {
        if wrapping {
            line.dump_wrap_into(buf);
        }
        line.term_input_into(buf);

        let Some(next) = lines.peek() else {
            break;
        };
        wrapping = line.wraps_onto(next, width);
        if !wrapping {
            term::Crlf::default().term_input_into(buf);
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

    #[test]
    fn insert_and_delete_before_right_margin() -> anyhow::Result<()> {
        let width = 6;
        let mut line = Line::new();
        for (col, c) in "abcdef".chars().enumerate() {
            line.write_cell(width, col, Cell::new(c, term::Attrs::default()))?;
        }

        // The margin is in front of the "e".
        let mut inserted = line.clone();
        inserted.insert_character_in(width, 1, 4, 1, &Cell::empty());
        assert_eq!(format!("{inserted}"), "a*bcef\n");

        let mut deleted = line.clone();
        deleted.delete_character_in(width, 1, 4, &Cell::empty(), 1);
        assert_eq!(format!("{deleted}"), "acd*ef\n");

        Ok(())
    }

    #[test]
    fn swap_cells_through_wide() -> anyhow::Result<()> {
        let width = 6;
        let wide = Cell::new('😊', term::Attrs::default());
        let x = Cell::new('x', term::Attrs::default());
        let mut line = Line::new();
        line.write_cell(width, 0, wide.clone())?;
        line.write_cell(width, 2, x.clone())?;
        line.write_cell(width, 3, wide.clone())?;

        // The wide chars straddle the ends of the swapped cells, so they go
        // entirely rather than leaving half of themselves behind.
        let y = Cell::new('y', term::Attrs::default());
        let swapped_out = line.swap_cells(1, vec![y; 3]);
        assert_eq!(swapped_out, vec![Cell::empty(), x, Cell::empty()]);
        assert_eq!(format!("{line}"), "*yyy*\n");

        Ok(())
    }
}
