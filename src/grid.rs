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
    // The number of lines at the bottom of the scrollback (front of the deque)
    // which are logically in view. This is the height of the terminal that the user
    // has configured or resized to.
    size: crate::Size,
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
            return line.get(self.size.width, pos.col);
        }
        None
    }

    /// Set the cell at the given grid coordinates.
    pub fn set(&mut self, pos: crate::Pos, cell: Cell) {
        let width = self.size.width;
        if let Some(line) = self.get_line_mut(pos.row) {
            return line.set(width, pos.col, cell);
        }
    }

    /// Push the given cell to the grid.
    pub fn push(&mut self, cell: Cell) {
        if self.scrollback.is_empty() {
            self.scrollback.push_front(Line::new());
        }

        let mut bottom_line = &mut self.scrollback[0];
        if bottom_line.cells.len() >= self.size.width {
            bottom_line.is_wrapped = true;
            self.push_line(Line::new());
            bottom_line = &mut self.scrollback[0];
        }
        bottom_line.push(self.size.width, cell);
    }

    /// Enter a carrage return, causing the terminal to begin placing text on
    /// a new line.
    pub fn push_newline(&mut self) {
        self.push_line(Line::new());
    }

    fn push_line(&mut self, line: Line) {
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
        if row >= self.size.height {
            return None;
        }
        let idx_from_bottom = (self.size.height - 1) - row;
        Some(&self.scrollback[idx_from_bottom])
    }

    fn get_line_mut(&mut self, row: usize) -> Option<&mut Line> {
        if row >= self.size.height {
            return None;
        }
        let idx_from_bottom = (self.size.height - 1) - row;
        Some(&mut self.scrollback[idx_from_bottom])
    }
}

impl vte::Perform for Grid {
    fn print(&mut self, c: char) {
        self.push(Cell::new(c));
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
    fn write_buf(&self, buf: &mut Vec<u8>) {
        for cell in self.cells.iter() {
            cell.write_buf(buf);
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
    fn get(&self, width: usize, col: usize) -> Option<&Cell> {
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
    fn set(&mut self, width: usize, col: usize, cell: Cell) {
        if col >= width {
            panic!("{} out of bounds (width={})", col, width);
        }

        if col >= self.cells.len() {
            while self.cells.len() < col {
                self.cells.push(Cell::empty())
            }
            self.cells.push(cell);
            return;
        }

        self.cells[col] = cell;
    }

    // Push the cell onto the end of this line.
    //
    // Panics: if this push would make line longer than width.
    fn push(&mut self, width: usize, cell: Cell) {
        if self.cells.len() >= width {
            panic!(
                "pushing cell to line of length {} would go over width {}",
                self.cells.len(),
                width
            );
        }

        self.cells.push(cell);
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
    fn test_line_push() {
        let mut line = Line::new();
        let width = 5;
        let c = Cell::new('a');

        line.push(width, c.clone());
        assert_eq!(line.cells.len(), 1);
        assert_eq!(line.get(width, 0), Some(&c));
    }

    #[test]
    #[should_panic(expected = "would go over width")]
    fn test_line_push_full() {
        let mut line = Line::new();
        let width = 1;
        line.push(width, Cell::new('a'));
        line.push(width, Cell::new('b')); // Should panic
    }

    #[test]
    fn test_line_set() {
        let mut line = Line::new();
        let width = 5;
        let c1 = Cell::new('a');
        let c2 = Cell::new('b');

        // Set within current length (needs push first to not be out of bounds of vector if we treated it strictly,
        // but set() handles extension)

        // set at 0
        line.set(width, 0, c1.clone());
        assert_eq!(line.get(width, 0), Some(&c1));

        // set at 2 (should pad with empty)
        line.set(width, 2, c2.clone());
        assert_eq!(line.get(width, 0), Some(&c1));
        assert!(line.get(width, 1).unwrap().is_empty());
        assert_eq!(line.get(width, 2), Some(&c2));
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_line_set_oob() {
        let mut line = Line::new();
        let width = 5;
        line.set(width, 5, Cell::new('a')); // Index 5 is OOB for width 5
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
    fn test_grid_push_simple() {
        let size = Size {
            width: 5,
            height: 2,
        };
        let mut grid = Grid::new(5, size);
        let c = Cell::new('x');

        grid.push(c.clone());

        // Should be at the bottom line
        let bottom_pos = Pos {
            row: size.height - 1,
            col: 0,
        };
        assert_eq!(grid.get(bottom_pos), Some(&c), "Grid:\n{:?}", grid);
    }

    #[test]
    fn test_grid_push_wrapping() {
        let size = Size {
            width: 2,
            height: 5,
        };
        let mut grid = Grid::new(5, size);

        // Fill first line
        grid.push(Cell::new('1'));
        grid.push(Cell::new('2'));

        // This should wrap to next line
        grid.push(Cell::new('3'));

        // Check internal structure for wrapping flag if possible, or just logical position
        // Row 4 is bottom (height=5, 0-indexed).
        // '1','2' should be at row 3 (second from bottom) if we pushed enough to wrap?
        // Wait, push adds to the *end* of the scrollback (bottom).
        // If we push '1', '2', they are on the bottom line.
        // '3' wraps, so '1','2' become the line *above* bottom.

        // Bottom is row=4. Line above is row=3.
        assert_eq!(
            grid.get(Pos { row: 3, col: 0 }),
            Some(&Cell::new('1')),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 3, col: 1 }),
            Some(&Cell::new('2')),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 4, col: 0 }),
            Some(&Cell::new('3')),
            "Grid:\n{:?}",
            grid
        );
    }

    #[test]
    fn test_grid_indexing() {
        let size = Size {
            width: 10,
            height: 3,
        };
        let mut grid = Grid::new(3, size);

        // Populate 3 lines
        // Line 0 (Top)
        grid.push_newline(); // Ensure we have lines
        grid.push_newline();
        grid.push_newline();

        // grid.scrollback now has 3 empty lines.
        // Let's set some values explicitly to test indexing.

        let top = Pos { row: 0, col: 0 };
        let middle = Pos { row: 1, col: 0 };
        let bottom = Pos { row: 2, col: 0 };

        let c_top = Cell::new('T');
        let c_mid = Cell::new('M');
        let c_bot = Cell::new('B');

        grid.set(top, c_top.clone());
        grid.set(middle, c_mid.clone());
        grid.set(bottom, c_bot.clone());

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
    }

    #[test]
    fn test_resize_narrower() {
        let size = Size {
            width: 10,
            height: 5,
        };
        let mut grid = Grid::new(20, size);

        // Create a line: "0123456789"
        for i in 0..10 {
            grid.push(Cell::new(char::from_digit(i, 10).unwrap()));
        }

        // Resize to width 5. Should split into "01234" and "56789"
        let new_size = Size {
            width: 5,
            height: 5,
        };
        grid.resize(new_size);

        // "56789" should be at bottom (row 4)
        // "01234" should be above (row 3)
        assert_eq!(
            grid.get(Pos { row: 4, col: 0 }),
            Some(&Cell::new('5')),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 3, col: 0 }),
            Some(&Cell::new('0')),
            "Grid:\n{:?}",
            grid
        );
    }

    #[test]
    fn test_resize_wider() {
        let size = Size {
            width: 5,
            height: 5,
        };
        let mut grid = Grid::new(30, size);

        // Create two wrapped lines: "01234" (wrapped) -> "56789"
        for i in 0..10 {
            grid.push(Cell::new(char::from_digit(i, 10).unwrap()));
        }

        // Verify initial state
        assert_eq!(
            grid.get(Pos { row: 4, col: 0 }),
            Some(&Cell::new('5')),
            "Grid:\n{:?}",
            grid
        );

        // Resize to width 10. Should merge back to "0123456789"
        let new_size = Size {
            width: 10,
            height: 5,
        };
        grid.resize(new_size);

        // Should all be on bottom line (row 4)
        assert_eq!(
            grid.get(Pos { row: 4, col: 0 }),
            Some(&Cell::new('0')),
            "Grid:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 4, col: 9 }),
            Some(&Cell::new('9')),
            "Grid:\n{:?}",
            grid
        );
    }

    #[test]
    fn test_reflow_roundtrip() {
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
                grid.push(Cell::new(char::from_u32(65 + i % 26).unwrap()));
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
                expected_grid.push(Cell::new(char::from_u32(65 + i % 26).unwrap()));
            }

            assert_eq!(
                grid, expected_grid,
                "Grid state mismatch after roundtrip resize {} -> {} -> {}",
                start_w, end_w, start_w
            );
        }
    }
}
