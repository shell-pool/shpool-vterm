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

//! The grid module defines a view of the terminal where every cell is
//! assigned to a definite row and column. It is used at dump time to
//! dump the precise information requested and is also used to display
//! some commands.

use crate::{
    cell::{self, Cell},
    term::{self, BufWrite},
};
use std::collections::VecDeque;

use anyhow::anyhow;
use tracing::warn;

// A grid stores all the termianal state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Grid {
    /// The entire scrollback buffer for the terminal.
    ///
    /// The bottom of the terminal is stored at the front of the deque
    /// and the top is stored at the back of the deque.
    scrollback: VecDeque<Line>,
    /// The number of lines of scrollback to store, independent of the
    /// size of the grid that is in view.
    scrollback_lines: usize,
    /// The number of lines at the bottom of the scrollback (front of the deque)
    /// which are logically in view. This is the height of the terminal that the user
    /// has configured or resized to.
    size: crate::Size,
    /// The current position of the cursor within the in-view window described
    /// by `size`. (0,0) is the upper left.
    cursor: crate::Pos,
    /// The attributes that the new cells will get created with when
    /// the cursor writes.
    current_attrs: term::Attrs,
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.scrollback.iter().rev() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl BufWrite for Grid {
    fn write_buf(&self, buf: &mut Vec<u8>) {
        for (i, line) in self.scrollback.iter().enumerate().rev() {
            line.write_buf(buf);
            if i != 0 {
                term::Crlf::default().write_buf(buf);
            }
        }
    }
}

impl Grid {
    /// Create a new grid with the given number of lines of scrollback
    /// storage, and the given size window in view.
    pub fn new(scrollback_lines: usize, size: crate::Size) -> Self {
        Grid {
            scrollback: VecDeque::new(),
            scrollback_lines,
            size,
            cursor: crate::Pos { row: 0, col: 0 },
            current_attrs: term::Attrs::default(),
        }
    }

    /// Get the size of the grid.
    pub fn size(&self) -> crate::Size {
        self.size
    }

    /// Resize the grid to the new size, reflowing all lines to match the new
    /// width.
    pub fn resize(&mut self, size: crate::Size) {
        self.reflow(size.width);
        self.size = size;
    }

    /// Get the max number of scrollback lines this grid
    /// can store.
    pub fn scrollback_lines(&self) -> usize {
        self.scrollback_lines
    }

    /// Set a new max number of scrollback lines this grid can
    /// store. If this is less than the current number, trailing
    /// data will be dropped.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        while self.scrollback.len() > scrollback_lines {
            self.scrollback.pop_back();
        }
        self.scrollback_lines = scrollback_lines;
    }

    /// Get the cell at the given grid coordinates.
    pub fn get(&self, pos: crate::Pos) -> Option<&Cell> {
        if let Some(line) = self.get_line(pos.row) {
            return line.get_cell(self.size.width, pos.col);
        }
        None
    }

    /// Set the cell at the given grid coordinates.
    pub fn set(&mut self, pos: crate::Pos, cell: Cell) -> anyhow::Result<()> {
        let width = self.size.width;
        if let Some(line) = self.get_line_mut(pos.row) {
            return line.set_cell(width, pos.col, cell);
        }

        Ok(())
    }

    /// Write the given cell at the current cursor, advancing
    /// the cursor.
    pub fn write_at_cursor(&mut self, cell: Cell) -> anyhow::Result<()> {
        if self.size.width < 1 {
            return Err(anyhow!("cannot write to zero width terminal grid"));
        }

        while self.scrollback.len() < self.cursor.row + 1 {
            // TODO: these lines will all count as having
            // not been wrapped and will be retained on reflow.
            // Is that actually what we want?
            self.add_line(Line::new());
        }

        if self.cursor.col >= self.size.width {
            if let Some(line) = self.get_line_mut(self.cursor.row) {
                line.is_wrapped = true;
            } else {
                return Err(anyhow!(
                    "unexpectedly missing line when setting wrap marker"
                ));
            }

            self.cursor.col = 0;
            self.cursor.row += 1;

            if self.scrollback.len() < self.cursor.row + 1 {
                self.add_line(Line::new())
            }
        }

        self.set(self.cursor, cell)?;
        self.cursor.col += 1;

        Ok(())
    }

    fn add_line(&mut self, line: Line) {
        self.scrollback.push_front(line);
        while self.scrollback.len() > self.scrollback_lines {
            self.scrollback.pop_back();
        }
    }

    fn reflow(&mut self, new_width: usize) {
        let mut new_scrollback = VecDeque::with_capacity(self.scrollback.len());
        let mut logical_line = VecDeque::new();
        while let Some(grid_line) = self.scrollback.pop_back() {
            let is_wrapped = grid_line.is_wrapped;
            logical_line.push_back(grid_line);

            if !is_wrapped {
                // We've gotten to the end of the logical line. We now
                // need to chop it up into grid lines by the new width.
                let mut line = Line::new();
                while let Some(chunk) = logical_line.pop_front() {
                    let remainder = new_width - line.cells.len();
                    if chunk.cells.len() < remainder {
                        line.cells.extend_from_slice(chunk.cells.as_slice());

                        if line.cells.len() == new_width {
                            new_scrollback.push_front(line);
                            line = Line::new();
                        }
                    } else {
                        // Complete the partial line.
                        line.cells.extend_from_slice(&chunk.cells[..remainder]);
                        line.is_wrapped = chunk.cells.len() > remainder || !logical_line.is_empty();
                        new_scrollback.push_front(line);
                        line = Line::new();

                        let remaining_chunks: Vec<_> =
                            chunk.cells[remainder..].chunks(new_width).collect();
                        for (i, c) in remaining_chunks.iter().enumerate() {
                            line.cells.extend_from_slice(c);
                            if i < remaining_chunks.len() - 1 {
                                line.is_wrapped = true;
                            } else {
                                line.is_wrapped = !logical_line.is_empty();
                            }

                            if line.cells.len() == new_width {
                                new_scrollback.push_front(line);
                                line = Line::new();
                            }
                        }
                    }
                }

                if line.cells.len() != 0 {
                    new_scrollback.push_front(line);
                    line = Line::new();
                }
            }
        }

        self.scrollback = new_scrollback;
    }

    fn get_line(&self, row: usize) -> Option<&Line> {
        let in_view_scrollback_len = if self.scrollback.len() < self.size.height {
            self.scrollback.len()
        } else {
            self.size.height
        };

        if row >= in_view_scrollback_len {
            return None;
        }

        let idx_from_bottom = (in_view_scrollback_len - 1) - row;
        Some(&self.scrollback[idx_from_bottom])
    }

    fn get_line_mut(&mut self, row: usize) -> Option<&mut Line> {
        let in_view_scrollback_len = if self.scrollback.len() < self.size.height {
            self.scrollback.len()
        } else {
            self.size.height
        };

        if row >= in_view_scrollback_len {
            return None;
        }

        let idx_from_bottom = (in_view_scrollback_len - 1) - row;
        Some(&mut self.scrollback[idx_from_bottom])
    }
}

impl vte::Perform for Grid {
    fn print(&mut self, c: char) {
        let attrs = self.current_attrs.clone();
        if let Err(e) = self.write_at_cursor(Cell::new(c, attrs)) {
            warn!("writing char at cursor: {e:?}");
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor.row += 1;
            }
            b'\r' => {
                self.cursor.col = 0;
            }
            _ => {
                warn!("execute: unhandled byte {}", byte);
            }
        }
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

    // Handle escape codes beginning with the CSI indicator ('\x1b[').
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            // CUU (Cursor Up)
            'A' => {
                let n = p1(params, 1) as usize;
                if n > self.cursor.row {
                    self.cursor.row = 0;
                } else {
                    self.cursor.row -= n;
                }
            }
            // CUD (Cursor Down)
            'B' => {
                let n = p1(params, 1) as usize;
                self.cursor.row = std::cmp::min(
                    self.size.height - 1, self.cursor.row + n);
            }
            // CUF (Cursor Forward)
            'C' => {
                let n = p1(params, 1) as usize;
                self.cursor.col = std::cmp::min(
                    self.size.width - 1, self.cursor.col + n);
            }
            // CUF (Cursor Backwards)
            'D' => {
                let n = p1(params, 1) as usize;
                if n > self.cursor.col {
                    self.cursor.col = 0;
                } else {
                    self.cursor.col -= n;
                }
            }
            // CNL (Cursor Next Line)
            'E' => {
                let n = p1(params, 1) as usize;
                self.cursor.row = std::cmp::min(
                    self.size.height - 1, self.cursor.row + n);
                self.cursor.col = 0;
            }
            // CPL (Cursor Prev Line)
            'F' => {
                let n = p1(params, 1) as usize;
                if n > self.cursor.row {
                    self.cursor.row = 0;
                } else {
                    self.cursor.row -= n;
                }
                self.cursor.col = 0;
            }

            // cell attribute manipulation
            'm' => {
                let mut param_iter = params.iter();
                while let Some(param) = param_iter.next() {
                    if param.len() < 1 {
                        warn!("m action with no params. Not sure what to do.");
                        continue;
                    }

                    match param[0] {
                        0 => {
                            self.current_attrs = term::Attrs::default();
                        }

                        // Underline Handling
                        // TODO: there are lots of other underline styles. To fix,
                        // we need to update attrs.
                        //
                        // Kitty extensions:
                        //      CSI 4 : 3 m => curly
                        //      CSI 4 : 2 m => double
                        //
                        // Other:
                        //      CSI 21 m => double
                        //      CSI 58 ; 2 ; r ; g ; b m => RGB colored underline
                        4 => self.current_attrs.underline = true,
                        // TODO: should really be a double underline.
                        21 => self.current_attrs.underline = true,
                        24 => self.current_attrs.underline = false,

                        // Bold Handling.
                        1 => self.current_attrs.bold = true,
                        22 => self.current_attrs.bold = false,

                        // Italic Handling.
                        3 => self.current_attrs.italic = true,
                        23 => self.current_attrs.italic = false,

                        // Inverse Handling.
                        7 => self.current_attrs.inverse = true,
                        27 => self.current_attrs.inverse = false,

                        _ => {
                            warn!("unhandled m action: {:?}", params);
                        }
                    }
                }
            }
            _ => {
                warn!("unhandled action {}", action);
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // TODO: stub
    }

    fn terminated(&self) -> bool {
        // TODO: stub
        false
    }
}

fn p1(params: &vte::Params, default: u16) -> u16 {
    let n = params.iter().flatten().next().map(|x| *x).unwrap_or(0);
    if n == 0 {
        default
    } else {
        n
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Line {
    /// The cells stored in this line.
    cells: Vec<Cell>,
    /// If true, indicates that this line was automatically wrapped due to
    /// the terminal width. The following line is part of the same logical
    /// line and should be reflowed together with this line on terminal resize.
    is_wrapped: bool,
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cell in &self.cells {
            write!(f, "{}", cell)?;
        }
        if !self.is_wrapped {
            writeln!(f, "⏎")?;
        } else {
            writeln!(f)?;
        }
        Ok(())
    }
}

impl BufWrite for Line {
    // We start every line with blank attrs, and it is our responsibility
    // to reset the attrs at the end of each line. We could produce more
    // optimal output if we fused attr runs across lines, but it is probably
    // fine to just do it like this and it is better to keep things simple
    // unless we need to fuse.
    fn write_buf(&self, buf: &mut Vec<u8>) {
        let blank_attrs = term::Attrs::default();
        let mut current_attrs = &blank_attrs;

        for cell in self.cells.iter() {
            if cell.attrs() != current_attrs {
                for code in current_attrs.transition_to(cell.attrs()) {
                    code.write_buf(buf);
                }
                current_attrs = cell.attrs();
            }
            cell.write_buf(buf);
        }

        if current_attrs != &blank_attrs {
            for code in current_attrs.transition_to(&blank_attrs) {
                code.write_buf(buf);
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
    fn new() -> Self {
        Line {
            cells: vec![],
            is_wrapped: false,
        }
    }

    /// Get the cell at the given grid position.
    fn get_cell(&self, width: usize, col: usize) -> Option<&Cell> {
        if col >= width {
            return None;
        }

        if col >= self.cells.len() {
            return Some(cell::empty());
        }

        return Some(&self.cells[col]);
    }

    // Set the given column to the given cell.
    //
    // Panics: if this is out of bounds.
    fn set_cell(&mut self, width: usize, col: usize, cell: Cell) -> anyhow::Result<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pos, Size};

    #[test]
    fn test_line_new() {
        let line = Line::new();
        assert!(line.cells.is_empty());
        assert!(!line.is_wrapped);
    }

    #[test]
    fn test_line_set() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        let c1 = Cell::new('a', term::Attrs::default());
        let c2 = Cell::new('b', term::Attrs::default());

        // Set within current length (needs push first to not be out of bounds of vector if we treated it strictly,
        // but set() handles extension)

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
    fn test_line_set_oob() -> anyhow::Result<()> {
        let mut line = Line::new();
        let width = 5;
        match line.set_cell(width, 5, Cell::new('a', term::Attrs::default())) {
            Err(e) => assert!(format!("{e:?}").contains("out of bounds")),
            _ => assert!(false, "expected out of bounds error"),
        }

        Ok(())
    }

    #[test]
    fn test_grid_new() {
        let size = Size {
            width: 10,
            height: 5,
        };
        let grid = Grid::new(5, size);
        assert_eq!(grid.size, size);
        assert!(grid.scrollback.is_empty());
    }

    #[test]
    fn test_grid_push_simple() -> anyhow::Result<()> {
        let size = Size {
            width: 5,
            height: 2,
        };
        let mut grid = Grid::new(5, size);
        let c = Cell::new('x', term::Attrs::default());

        grid.write_at_cursor(c.clone())?;

        let pos = Pos { row: 0, col: 0 };
        assert_eq!(grid.get(pos), Some(&c), "Grid:\n{grid}");

        Ok(())
    }

    #[test]
    fn test_grid_push_wrapping() -> anyhow::Result<()> {
        let size = Size {
            width: 2,
            height: 5,
        };
        let mut grid = Grid::new(5, size);

        // Fill first line
        grid.write_at_cursor(Cell::new('1', term::Attrs::default()))?;
        grid.write_at_cursor(Cell::new('2', term::Attrs::default()))?;

        // This should wrap to next line
        grid.write_at_cursor(Cell::new('3', term::Attrs::default()))?;

        assert_eq!(
            grid.get(Pos { row: 0, col: 0 }),
            Some(&Cell::new('1', term::Attrs::default())),
            "Grid:\n{grid}",
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 1 }),
            Some(&Cell::new('2', term::Attrs::default())),
            "Grid:\n{grid}",
        );
        assert_eq!(
            grid.get(Pos { row: 1, col: 0 }),
            Some(&Cell::new('3', term::Attrs::default())),
            "Grid:\n{grid}",
        );

        Ok(())
    }

    #[test]
    fn test_grid_indexing() -> anyhow::Result<()> {
        let size = Size {
            width: 10,
            height: 3,
        };
        let mut grid = Grid::new(3, size);

        // Populate 3 lines
        grid.add_line(Line::new());
        grid.add_line(Line::new());
        grid.add_line(Line::new());

        // grid.scrollback now has 3 empty lines.
        // Let's set some values explicitly to test indexing.

        let top = Pos { row: 0, col: 0 };
        let middle = Pos { row: 1, col: 0 };
        let bottom = Pos { row: 2, col: 0 };

        let c_top = Cell::new('T', term::Attrs::default());
        let c_mid = Cell::new('M', term::Attrs::default());
        let c_bot = Cell::new('B', term::Attrs::default());

        grid.set(top, c_top.clone())?;
        grid.set(middle, c_mid.clone())?;
        grid.set(bottom, c_bot.clone())?;

        assert_eq!(
            grid.get(top),
            Some(&c_top),
            "Failed to get top row. Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(middle),
            Some(&c_mid),
            "Failed to get middle row. Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(bottom),
            Some(&c_bot),
            "Failed to get bottom row. Grid:\n{:?}",
            grid
        );

        Ok(())
    }

    #[test]
    fn test_resize_narrower() -> anyhow::Result<()> {
        let size = Size {
            width: 10,
            height: 5,
        };
        let mut grid = Grid::new(20, size);

        // Create a line: "0123456789"
        for i in 0..10 {
            grid.write_at_cursor(Cell::new(
                char::from_digit(i, 10).unwrap(),
                term::Attrs::default(),
            ))?;
        }

        // Resize to width 5. Should split into "01234" and "56789"
        let new_size = Size {
            width: 5,
            height: 5,
        };
        grid.resize(new_size);

        // "56789" should be at bottom (row 1)
        // "01234" should be above (row 0)
        assert_eq!(
            grid.get(Pos { row: 1, col: 0 }),
            Some(&Cell::new('5', term::Attrs::default())),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 0 }),
            Some(&Cell::new('0', term::Attrs::default())),
            "Grid:\n{:?}",
            grid
        );

        Ok(())
    }

    #[test]
    fn test_resize_wider() -> anyhow::Result<()> {
        let size = Size {
            width: 5,
            height: 5,
        };
        let mut grid = Grid::new(30, size);

        // Create two wrapped lines: "01234" (wrapped) -> "56789"
        for i in 0..10 {
            grid.write_at_cursor(Cell::new(
                char::from_digit(i, 10).unwrap(),
                term::Attrs::default(),
            ))?;
        }

        // Verify initial state
        assert_eq!(
            grid.get(Pos { row: 1, col: 0 }),
            Some(&Cell::new('5', term::Attrs::default())),
            "Grid:\n{:?}",
            grid
        );

        // Resize to width 10. Should merge back to "0123456789"
        let new_size = Size {
            width: 10,
            height: 5,
        };
        grid.resize(new_size);

        // Should all be on top line
        assert_eq!(
            grid.get(Pos { row: 0, col: 0 }),
            Some(&Cell::new('0', term::Attrs::default())),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 9 }),
            Some(&Cell::new('9', term::Attrs::default())),
            "Grid:\n{:?}",
            grid
        );

        Ok(())
    }

    #[test]
    fn test_reflow_roundtrip() -> anyhow::Result<()> {
        // Parameterized-style test
        let shapes = vec![
            (10, 20), // Start wide, go narrow
            (5, 10),  // Start narrow, go wide
            (10, 10), // No change
        ];

        for (start_w, end_w) in shapes {
            let start_size = Size {
                width: start_w,
                height: 10,
            };
            let mut grid = Grid::new(100, start_size);

            // Fill with deterministic data
            let count = 30;
            for i in 0..count {
                grid.write_at_cursor(Cell::new(
                    char::from_u32(65 + i % 26).unwrap(),
                    term::Attrs::default(),
                ))?;
            }

            // Snapshot state (conceptually) - we can't easily clone the grid state to compare
            // directly if we don't have access to inner fields easily, but we can verify content.

            // Resize
            grid.resize(Size {
                width: end_w,
                height: 10,
            });

            // Resize back
            grid.resize(start_size);

            // Verify content is identical to if we just pushed it
            let mut expected_grid = Grid::new(100, start_size);
            for i in 0..count {
                expected_grid.write_at_cursor(Cell::new(
                    char::from_u32(65 + i % 26).unwrap(),
                    term::Attrs::default(),
                ))?;
            }

            assert_eq!(
                grid, expected_grid,
                "Grid state mismatch after roundtrip resize {} -> {} -> {}",
                start_w, end_w, start_w
            );
        }

        Ok(())
    }

    // TODO: write a test for a wide char
}
