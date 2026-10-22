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
    cell::{self, Cell, Fill},
    term::{self, AsTermInput},
};

use anyhow::anyhow;
use std::borrow::Cow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Line {
    /// The cells stored in this line.
    pub cells: Vec<Cell>,
    /// The background color of every cell past the end of `cells`.
    ///
    /// Erasing to the end of a line, or opening up a new one, while a
    /// background color is set paints the rest of the line with it (see
    /// `Cell::blank`). Keeping that here means a painted line costs no more
    /// to store than a plain one.
    pub blank_bgcolor: term::Color,
    /// The foreground color of every cell past the end of `cells`, which
    /// the blanks keep along with the background color, like in xterm.
    pub blank_fgcolor: term::Color,
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
            // The attrs of a blank that don't show up on a blank, like the
            // fg color it got erased with, are not worth painting.
            let cell = if cell.looks_unused() { cell::empty() } else { cell };
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
        Line::blank(Fill::default())
    }

    /// A line full of the blanks that erasing and scrolling leave behind
    /// while `fill` holds the current colors (see `Cell::blank`). They are
    /// all past the end of the line, so none of them have to be stored.
    pub fn blank(fill: Fill) -> Self {
        Line { cells: vec![], blank_bgcolor: fill.bg, blank_fgcolor: fill.fg, is_wrapped: false }
    }

    /// The colors of the cells past the end of the line.
    pub fn blank_fill(&self) -> Fill {
        Fill { fg: self.blank_fgcolor, bg: self.blank_bgcolor }
    }

    /// Set the colors of the cells past the end of the line.
    pub fn set_blank_fill(&mut self, fill: Fill) {
        self.blank_fgcolor = fill.fg;
        self.blank_bgcolor = fill.bg;
    }

    /// Store copies of the blanks past the end of the line out to `len`, for
    /// an edit that changes cells in front of there.
    fn pad_to(&mut self, len: usize) {
        if self.cells.len() < len {
            self.cells.resize(len, Cell::blank(self.blank_fill()));
        }
    }

    /// Drop the cells at the end of the line that are the same as the
    /// blanks past the end of it, since those look like that anyway.
    ///
    /// A wrapped line keeps them. They are in the middle of its logical
    /// line, and only stored cells count as part of that when it gets
    /// reflowed.
    fn trim_blanks(&mut self) {
        if self.is_wrapped {
            return;
        }
        let blank = Cell::blank(self.blank_fill());
        while self.cells.last() == Some(&blank) {
            self.cells.pop();
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

    /// Get the cell at the given grid position. Past the end of the stored
    /// cells, that is a blank with the line's `blank_fill`.
    pub fn get_cell(&self, width: usize, col: usize) -> Option<Cow<'_, Cell>> {
        if col >= width {
            return None;
        }

        if col >= self.cells.len() {
            let fill = self.blank_fill();
            return Some(if fill == Fill::default() {
                Cow::Borrowed(cell::empty())
            } else {
                Cow::Owned(Cell::blank(fill))
            });
        }

        return Some(Cow::Borrowed(&self.cells[col]));
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

    /// Set the given column to the given cell, followed by the padding cells
    /// that keep the columns lined up if it is a wide char.
    ///
    /// A wide char that this only covers one half of gets blanked out
    /// entirely, like real terminals do. Leaving the other half behind would
    /// throw off the columns of everything after it when we dump the line.
    ///
    /// Setting the last cell to the same blank as the ones past the end of
    /// the line makes it part of them, so it doesn't get stored.
    pub fn set_cell(&mut self, width: usize, col: usize, cell: Cell) -> anyhow::Result<()> {
        let end = col + std::cmp::max(cell.width() as usize, 1);
        if end > width {
            return Err(anyhow!("{}..{} out of bounds (width={})", col, end, width));
        }

        self.pad_to(end);
        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(end);

        for pad_col in col + 1..end {
            self.cells[pad_col] = Cell::wide_pad(cell.attrs().clone());
        }
        self.cells[col] = cell;
        if end == self.cells.len() {
            self.trim_blanks();
        }

        Ok(())
    }

    /// Blank out the wide char that straddles the boundary between `col - 1`
    /// and `col`, if there is one.
    ///
    /// This has to happen before any edit that touches the cells on only one
    /// side of the boundary, since the edit would otherwise leave half of the
    /// wide char behind.
    fn blank_out_wide_char_at(&mut self, col: usize) {
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

    /// The line the way a screen `width` columns wide shows it.
    ///
    /// Reflowing onto a one column screen keeps each wide char on a row of
    /// its own, padding and all, so that it is still there once the screen
    /// gets wider again. Until then it shows up as a blank, and the cells
    /// that stick out past the edge of the screen don't show up at all.
    ///
    /// The cells past the end of the line show up out to the edge if they
    /// look any different from cells nothing has been written to, so that
    /// painting the line paints them too.
    pub fn shown_at(&self, width: usize) -> Cow<'_, Line> {
        let too_wide = |cell: &Cell| cell.width() as usize > width;
        let paint_blanks = self.cells.len() < width && self.blank_bgcolor != term::Color::Default;
        // A zero width screen keeps its lines the way they were laid out
        // before, since there is nothing to lay them out on.
        if width == 0
            || (self.cells.len() <= width && !self.cells.iter().any(too_wide) && !paint_blanks)
        {
            return Cow::Borrowed(self);
        }

        let mut shown = self.clone();
        for col in 0..shown.cells.len() {
            let cell = &shown.cells[col];
            if too_wide(cell) && !cell.is_wide_padding() {
                let attrs = cell.attrs().clone();
                let end = std::cmp::min(col + cell.width() as usize, shown.cells.len());
                for cell in shown.cells[col..end].iter_mut() {
                    *cell = Cell::empty_with_attrs(attrs.clone());
                }
            }
        }
        shown.cells.truncate(width);
        if paint_blanks {
            shown.pad_to(width);
        }
        Cow::Owned(shown)
    }

    /// Trim the line to the new width, dropping any cells too far to the right.
    pub fn truncate(&mut self, width: usize) {
        // A wide char that would stick out past the new edge goes too.
        self.blank_out_wide_char_at(width);
        self.cells.truncate(width);
    }

    /// Clobber the given section with the blanks that erasing leaves behind
    /// while `fill` holds the current colors. Erasing out to the end of the
    /// line makes `fill` the line's `blank_fill` rather than storing any of
    /// them.
    pub fn erase(&mut self, width: usize, section: Section, fill: Fill) {
        match section {
            Section::StartTo(col) => {
                let end = std::cmp::min(col + 1, width);
                self.blank_out_wide_char_at(end);
                // The cells past the end of the line only look like the
                // blanks already if they have the same colors.
                if fill != self.blank_fill() {
                    self.pad_to(end);
                }
                let blank = Cell::blank(fill);
                for i in 0..std::cmp::min(end, self.cells.len()) {
                    self.cells[i] = blank.clone();
                }
                self.trim_blanks();
            }
            Section::ToEnd(col) => {
                self.truncate(col);
                self.is_wrapped = false;
                // The cells in front of `col` keep their old colors.
                if fill != self.blank_fill() {
                    self.pad_to(col);
                    self.set_blank_fill(fill);
                }
                self.trim_blanks();
            }
            Section::Whole => {
                self.truncate(0);
                self.is_wrapped = false;
                self.set_blank_fill(fill);
            }
        }
    }

    /// Insert n new blank cells with the colors in `fill` at the current
    /// position, dropping any cells which spill over width.
    pub fn insert_character(&mut self, width: usize, col: usize, n: usize, fill: Fill) {
        if col >= width {
            return;
        }
        // Past the end of the line, inserting cells that look just like the
        // ones already there changes nothing.
        if col >= self.cells.len() && fill == self.blank_fill() {
            return;
        }
        self.pad_to(col);
        self.blank_out_wide_char_at(col);
        let blanks = vec![Cell::blank(fill); std::cmp::min(n, width - col)];
        self.cells.splice(col..col, blanks);
        self.truncate(width);
        self.trim_blanks();
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
        fill: Fill,
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
        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(right - n);
        self.blank_out_wide_char_at(right);
        self.cells.drain(right - n..right);
        self.cells.splice(col..col, std::iter::repeat(Cell::blank(fill)).take(n));
        self.trim_blanks();
    }

    /// Delete n cells at the current position, sucking cells to the
    /// right towards the cursor, and backfilling their old position
    /// with blanks with the colors in `fill`.
    ///
    /// This implements DCH (Delete Character).
    pub fn delete_character(&mut self, width: usize, col: usize, fill: Fill, n: usize) {
        if col >= width {
            return;
        }
        let n = std::cmp::min(n, width - col);

        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(col + n);

        // The backfill goes in at the right edge. If it has the line's
        // `blank_fill`, the cells past the end of the line that get pulled
        // in look just like it, so only the stored cells have to move.
        // Otherwise the whole line has to be stored.
        let backfill = fill != self.blank_fill();
        if backfill {
            self.pad_to(width);
        }

        // Everything past the end of the cells buffer is implicitly blank, so
        // when the cursor sits out there we have no physical cells to remove
        // and the drain range collapses to an empty one at the end.
        let delete_from = std::cmp::min(col, self.cells.len());
        let delete_to = std::cmp::min(self.cells.len(), col + n);
        self.cells.drain(delete_from..delete_to);

        if backfill {
            self.cells.resize(std::cmp::max(width, self.cells.len()), Cell::blank(fill));
        }
        self.trim_blanks();
    }

    /// Like `delete_character`, but with the right margin at `right`. Only
    /// the cells up to the margin get pulled in, and the backfill goes in
    /// front of it. Nothing happens at or past the margin.
    pub fn delete_character_in(
        &mut self,
        width: usize,
        col: usize,
        right: usize,
        fill: Fill,
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
        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(col + n);
        self.blank_out_wide_char_at(right);
        self.cells.drain(col..col + n);
        self.cells.splice(right - n..right - n, std::iter::repeat(Cell::blank(fill)).take(n));
        self.trim_blanks();
    }

    /// Blank out n characters to the right of the current cursor. Unlike
    /// delete, this leaves the cells in place, just clobbers their contents.
    ///
    /// This implements ECH (Erase Character)
    pub fn erase_character(&mut self, width: usize, col: usize, fill: Fill, n: usize) {
        if col >= width {
            return;
        }

        let erase_to = std::cmp::min(width, col.saturating_add(n));
        // The cells past the end of the line only look like the blanks
        // already if they have the same colors.
        if fill != self.blank_fill() {
            self.pad_to(erase_to);
        }
        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(col.saturating_add(n));

        let blank = Cell::blank(fill);
        for i in col..std::cmp::min(self.cells.len(), erase_to) {
            self.cells[i] = blank.clone();
        }
        self.trim_blanks();
    }

    /// Put `cells` in place of the ones starting at `col`, handing back the
    /// cells that were there. A wide char that straddles either end gets
    /// blanked out first, since only one half of it would go.
    pub fn swap_cells(&mut self, col: usize, cells: Vec<Cell>) -> Vec<Cell> {
        let end = col + cells.len();
        self.pad_to(end);
        self.blank_out_wide_char_at(col);
        self.blank_out_wide_char_at(end);
        let swapped_out = self.cells.splice(col..end, cells).collect();
        self.trim_blanks();
        swapped_out
    }
}

/// Emit the codes that paint `lines`, from the top down, each on the row
/// below the one before, the way a screen `width` columns wide shows them
/// (see `Line::shown_at`).
///
/// Instead of breaking a line that wrapped onto the next one off from it
/// with a CRLF, we let the terminal wrap it again where we can, so that it
/// knows that the two belong together. That way it can still copy them as
/// one line, and reflow them when its window gets resized.
pub fn dump_lines_into<'a>(buf: &mut Vec<u8>, width: usize, lines: impl Iterator<Item = &'a Line>) {
    let mut lines = lines.map(|line| line.shown_at(width)).peekable();
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
    fn set() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let c1 = Cell::new('a', term::Attrs::default());
        let c2 = Cell::new('b', term::Attrs::default());

        // Set within current length (needs push first to not be out of bounds
        // of vector if we treated it strictly, but set() handles
        // extension)

        // set at 0
        line.set_cell(width, 0, c1.clone())?;
        assert_eq!(line.get_cell(width, 0).as_deref(), Some(&c1));

        // set at 2 (should pad with empty)
        line.set_cell(width, 2, c2.clone())?;
        assert_eq!(line.get_cell(width, 0).as_deref(), Some(&c1));
        assert!(line.get_cell(width, 1).unwrap().is_empty());
        assert_eq!(line.get_cell(width, 2).as_deref(), Some(&c2));

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

        // A wide char has to fit entirely.
        match line.set_cell(width, 4, Cell::new('😊', term::Attrs::default())) {
            Err(e) => assert!(format!("{e:?}").contains("out of bounds")),
            _ => assert!(false, "expected out of bounds error"),
        }

        Ok(())
    }

    #[test]
    fn set_wide() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let mut attrs = term::Attrs::default();
        attrs.italic = true;

        line.set_cell(width, 1, Cell::new('😊', attrs.clone()))?;
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
        line.set_cell(width, 0, wide.clone())?;
        line.set_cell(width, 1, narrow.clone())?;
        assert_eq!(format!("{line}"), "*x\n");

        // Left half.
        let mut line = Line::new();
        line.set_cell(width, 0, wide.clone())?;
        line.set_cell(width, 0, narrow.clone())?;
        assert_eq!(format!("{line}"), "x*\n");

        // A wide char over the halves of two others.
        let mut line = Line::new();
        line.set_cell(width, 0, wide.clone())?;
        line.set_cell(width, 2, wide.clone())?;
        line.set_cell(width, 1, wide.clone())?;
        assert_eq!(format!("{line}"), "*😊-*\n");

        Ok(())
    }

    #[test]
    fn truncate_through_wide() -> anyhow::Result<()> {
        let width = 5;
        let mut line = Line::new();
        line.set_cell(width, 0, Cell::new('a', term::Attrs::default()))?;
        line.set_cell(width, 1, Cell::new('😊', term::Attrs::default()))?;
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

    const BLUE: Fill = Fill { fg: term::Color::Default, bg: term::Color::Idx(4) };

    #[test]
    fn set_blank_at_end_is_not_stored() -> anyhow::Result<()> {
        let width = 5;
        let mut line = Line::blank(BLUE);
        line.set_cell(width, 0, Cell::new('a', term::Attrs::default()))?;
        line.set_cell(width, 2, Cell::new('b', term::Attrs::default()))?;
        assert_eq!(line.cells.len(), 3);

        // The cell in front of the "b" is a stored copy of the blanks past the
        // end of the line, and goes along with it.
        line.set_cell(width, 2, Cell::blank(BLUE))?;
        assert_eq!(line.cells.len(), 1);
        assert_eq!(line.get_cell(width, 3).as_deref(), Some(&Cell::blank(BLUE)));

        Ok(())
    }

    #[test]
    fn erase_to_end_only_changes_the_blank_fill() -> anyhow::Result<()> {
        let width = 5;
        let mut line = Line::new();
        for (col, c) in "abc".chars().enumerate() {
            line.set_cell(width, col, Cell::new(c, term::Attrs::default()))?;
        }

        line.erase(width, Section::ToEnd(1), BLUE);
        assert_eq!(format!("{line}"), "a\n");
        assert_eq!(line.get_cell(width, 4).as_deref(), Some(&Cell::blank(BLUE)));

        // Past the end of the cells, the ones in front of the erase keep
        // their old colors.
        line.erase(width, Section::ToEnd(3), Fill::default());
        assert_eq!(
            line.cells,
            vec![Cell::new('a', term::Attrs::default()), Cell::blank(BLUE), Cell::blank(BLUE)]
        );
        assert_eq!(line.blank_fill(), Fill::default());

        line.erase(width, Section::Whole, BLUE);
        assert!(line.cells.is_empty());
        assert_eq!(line.blank_fill(), BLUE);

        Ok(())
    }

    // Blanks that look just like the cells past the end of the line don't need
    // storing there.
    #[test]
    fn edits_with_the_blank_fill_store_nothing_past_the_end() -> anyhow::Result<()> {
        let width = 10;
        let mut line = Line::blank(BLUE);
        line.set_cell(width, 0, Cell::new('a', term::Attrs::default()))?;

        line.erase_character(width, 3, BLUE, 2);
        line.insert_character(width, 3, 2, BLUE);
        line.delete_character(width, 3, BLUE, 2);
        assert_eq!(line.cells.len(), 1);

        // Blanks with different colors do have to be stored.
        line.erase_character(width, 3, Fill::default(), 2);
        assert_eq!(line.cells.len(), 5);

        Ok(())
    }

    // Blanks keep the fg color they got erased with, but it doesn't show on a
    // blank, so it doesn't get painted.
    #[test]
    fn erase_keeps_the_fgcolor_without_painting_it() -> anyhow::Result<()> {
        let width = 5;
        let red = Fill { fg: term::Color::Idx(1), bg: term::Color::Default };
        let mut line = Line::new();
        for (col, c) in "abc".chars().enumerate() {
            line.set_cell(width, col, Cell::new(c, term::Attrs::default()))?;
        }

        line.erase(width, Section::ToEnd(1), red);
        line.erase_character(width, 0, red, 1);
        line.set_cell(width, 1, Cell::new('b', term::Attrs::default()))?;
        assert_eq!(line.blank_fill(), red);
        assert_eq!(line.get_cell(width, 0).as_deref(), Some(&Cell::blank(red)));
        assert_eq!(line.get_cell(width, 4).as_deref(), Some(&Cell::blank(red)));

        assert!(matches!(line.shown_at(width), Cow::Borrowed(_)));
        let mut buf = vec![];
        line.term_input_into(&mut buf);
        assert_eq!(buf, b" b");

        Ok(())
    }

    #[test]
    fn delete_character_backfills_past_the_end() {
        let width = 5;
        let mut line = Line::new();
        line.delete_character(width, 1, BLUE, 2);
        assert_eq!(line.cells[..3], vec![Cell::empty(); 3]);
        assert_eq!(line.cells[3..], vec![Cell::blank(BLUE); 2]);
    }

    // The blanks at the end of a wrapped line are in the middle of its logical
    // line, so they stay.
    #[test]
    fn wrapped_line_keeps_its_blanks() -> anyhow::Result<()> {
        let width = 5;
        let mut line = Line::new();
        for (col, c) in "abcde".chars().enumerate() {
            line.set_cell(width, col, Cell::new(c, term::Attrs::default()))?;
        }
        line.is_wrapped = true;

        line.erase_character(width, 3, Fill::default(), 2);
        assert_eq!(format!("{line}"), "abc**⏎\n");

        Ok(())
    }

    #[test]
    fn shown_at_paints_the_blanks_to_the_edge() -> anyhow::Result<()> {
        let width = 4;
        let mut line = Line::blank(BLUE);
        line.set_cell(width, 0, Cell::new('a', term::Attrs::default()))?;
        let shown = line.shown_at(width);
        assert_eq!(shown.cells.len(), width);
        assert_eq!(shown.cells[1..], vec![Cell::blank(BLUE); 3]);

        // Plain blanks look like nothing was written there, so they don't
        // need painting.
        let mut line = Line::new();
        line.set_cell(width, 0, Cell::new('a', term::Attrs::default()))?;
        assert!(matches!(line.shown_at(width), Cow::Borrowed(_)));

        Ok(())
    }

    #[test]
    fn insert_and_delete_before_right_margin() -> anyhow::Result<()> {
        let width = 6;
        let mut line = Line::new();
        for (col, c) in "abcdef".chars().enumerate() {
            line.set_cell(width, col, Cell::new(c, term::Attrs::default()))?;
        }

        // The margin is in front of the "e".
        let mut inserted = line.clone();
        inserted.insert_character_in(width, 1, 4, 1, Fill::default());
        assert_eq!(format!("{inserted}"), "a*bcef\n");

        let mut deleted = line.clone();
        deleted.delete_character_in(width, 1, 4, Fill::default(), 1);
        assert_eq!(format!("{deleted}"), "acd*ef\n");

        Ok(())
    }

    #[test]
    fn swap_cells_through_wide() -> anyhow::Result<()> {
        let width = 6;
        let wide = Cell::new('😊', term::Attrs::default());
        let x = Cell::new('x', term::Attrs::default());
        let mut line = Line::new();
        line.set_cell(width, 0, wide.clone())?;
        line.set_cell(width, 2, x.clone())?;
        line.set_cell(width, 3, wide.clone())?;

        // The wide chars straddle the ends of the swapped cells, so they go
        // entirely rather than leaving half of themselves behind. What is
        // left of the one at the end is just part of the blank space past
        // the end of the line now.
        let y = Cell::new('y', term::Attrs::default());
        let swapped_out = line.swap_cells(1, vec![y; 3]);
        assert_eq!(swapped_out, vec![Cell::empty(), x, Cell::empty()]);
        assert_eq!(format!("{line}"), "*yyy\n");

        Ok(())
    }
}
