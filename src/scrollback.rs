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

//! The scrollback module defines the representation of the main terminal
//! screen. This gets stiched together with the alt screen module to form
//! a complete terminal representation in lib.rs.

use crate::{
    cell::Cell,
    line::Line,
    term::{self, AsTermInput, Cursor},
};
use std::collections::VecDeque;

use anyhow::{anyhow, Context};
use tracing::warn;

// A scrollback stores the termianal state for the main screen.
// Alt screen state is stored seperately.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scrollback {
    /// The entire scrollback buffer for the terminal.
    ///
    /// The bottom of the terminal is stored at the front of the deque
    /// and the top is stored at the back of the deque.
    buf: VecDeque<Line>,
    /// The number of lines of scrollback to store, independent of the
    /// size of the grid that is in view.
    lines: usize,
    /// The number of lines at the bottom of the scrollback (front of the deque)
    /// which are logically in view. This is the height of the terminal that the user
    /// has configured or resized to.
    size: crate::Size,
    /// The current position of the cursor within the in-view window described
    /// by `size`. (0,0) is the upper left.
    cursor: Cursor,
    // The slot where cursor position info is saved by the SCP/RCP
    // and ESC 7 / ESC 8 commands.
    saved_cursor: Cursor,
}

// TODO: to support the alt screen, I need to refactor the grid structures
// into a Scrollback struct, then make an AltScreen struct that actually
// has a fixed grid of cells plus its own dedicated cursor/saved_cursor.

impl std::fmt::Display for Scrollback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.buf.iter().rev() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl AsTermInput for Scrollback {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        for (i, line) in self.buf.iter().enumerate().rev() {
            line.term_input_into(buf);
            if i != 0 {
                term::Crlf::default().term_input_into(buf);
            }
        }
    }
}

impl Scrollback {
    /// Create a new grid with the given number of lines of scrollback
    /// storage, and the given size window in view.
    pub fn new(scrollback_lines: usize, size: crate::Size) -> Self {
        Scrollback {
            buf: VecDeque::new(),
            lines: scrollback_lines,
            size,
            cursor: Cursor {
                pos: crate::Pos { row: 0, col: 0 },
                attrs: term::Attrs::default(),
            },
            saved_cursor: Cursor {
                pos: crate::Pos { row: 0, col: 0 },
                attrs: term::Attrs::default(),
            },
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
        self.lines
    }

    /// Set a new max number of scrollback lines this grid can
    /// store. If this is less than the current number, trailing
    /// data will be dropped.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        while self.buf.len() > scrollback_lines {
            self.buf.pop_back();
        }
        self.lines = scrollback_lines;
    }

    /// Get the cell at the given grid coordinates.
    #[allow(dead_code)]
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

        while self.buf.len() < self.cursor.pos.row + 1 {
            // TODO: these lines will all count as having
            // not been wrapped and will be retained on reflow.
            // Is that actually what we want?
            self.add_line(Line::new());
        }

        if self.cursor.pos.col + cell.width() as usize >= self.size.width + 1 {
            if let Some(line) = self.get_line_mut(self.cursor.pos.row) {
                line.is_wrapped = true;
            } else {
                return Err(anyhow!(
                    "unexpectedly missing line when setting wrap marker"
                ));
            }

            self.cursor.pos.col = 0;
            self.cursor.pos.row += 1;

            if self.buf.len() < self.cursor.pos.row + 1 {
                self.add_line(Line::new())
            }
        }

        let mut npad = if cell.width() > 1 {
            cell.width() - 1
        } else {
            0
        };
        self.set(self.cursor.pos, cell)
            .context("setting main cell")?;
        self.cursor.pos.col += 1;
        while npad > 0 {
            assert!(self.cursor.pos.col < self.size.width);

            self.set(self.cursor.pos, Cell::wide_pad())
                .context("padding after wide char")?;
            self.cursor.pos.col += 1;
            npad -= 1;
        }

        Ok(())
    }

    fn add_line(&mut self, line: Line) {
        self.buf.push_front(line);
        while self.buf.len() > self.lines {
            self.buf.pop_back();
        }
    }

    fn reflow(&mut self, new_width: usize) {
        // TODO: this needs to move the cursor and saved cursor to have them
        // point to the same cell that they did at the start of the reflow
        // process. We currently don't do that, so reflow is broken.

        let mut new_scrollback = VecDeque::with_capacity(self.buf.len());
        let mut logical_line = VecDeque::new();
        while let Some(grid_line) = self.buf.pop_back() {
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

        self.buf = new_scrollback;
    }

    fn get_line(&self, row: usize) -> Option<&Line> {
        let in_view_scrollback_len = if self.buf.len() < self.size.height {
            self.buf.len()
        } else {
            self.size.height
        };

        if row >= in_view_scrollback_len {
            return None;
        }

        let idx_from_bottom = (in_view_scrollback_len - 1) - row;
        Some(&self.buf[idx_from_bottom])
    }

    fn get_line_mut(&mut self, row: usize) -> Option<&mut Line> {
        let in_view_scrollback_len = if self.buf.len() < self.size.height {
            self.buf.len()
        } else {
            self.size.height
        };

        if row >= in_view_scrollback_len {
            return None;
        }

        let idx_from_bottom = (in_view_scrollback_len - 1) - row;
        Some(&mut self.buf[idx_from_bottom])
    }
}

impl vte::Perform for Scrollback {
    fn print(&mut self, c: char) {
        let attrs = self.cursor.attrs.clone();
        if let Err(e) = self.write_at_cursor(Cell::new(c, attrs)) {
            warn!("writing char at cursor: {e:?}");
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor.pos.row += 1;
            }
            b'\r' => {
                self.cursor.pos.col = 0;
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
                let n = p1_or(params, 1) as usize;
                self.cursor.pos.row = self.cursor.pos.row.saturating_sub(n)
            }
            // CUD (Cursor Down)
            'B' => {
                let n = p1_or(params, 1) as usize;
                self.cursor.pos.row = self.cursor.pos.row + n;
                self.cursor.clamp_to(self.size);
            }
            // CUF (Cursor Forward)
            'C' => {
                let n = p1_or(params, 1) as usize;
                self.cursor.pos.col = self.cursor.pos.col + n;
                self.cursor.clamp_to(self.size);
            }
            // CUF (Cursor Backwards)
            'D' => {
                let n = p1_or(params, 1) as usize;
                self.cursor.pos.col = self.cursor.pos.col.saturating_sub(n);
            }
            // CNL (Cursor Next Line)
            'E' => {
                let n = p1_or(params, 1) as usize;
                self.cursor.pos.row = self.cursor.pos.row + n;
                self.cursor.pos.col = 0;
                self.cursor.clamp_to(self.size);
            }
            // CPL (Cursor Prev Line)
            'F' => {
                let n = p1_or(params, 1) as usize;
                if n > self.cursor.pos.row {
                    self.cursor.pos.row = 0;
                } else {
                    self.cursor.pos.row -= n;
                }
                self.cursor.pos.col = 0;
            }
            // CHA (Cursor Horizontal Absolute)
            'G' => {
                let n = p1_or(params, 1) as usize;
                let n = n - 1; // translate to 0 indexing
                self.cursor.pos.col = n;
                self.cursor.clamp_to(self.size);
            }
            // CUP (Cursor Set Position)
            'H' => {
                if let Some((row, col)) = p2(params) {
                    // adjust 1 indexing to 0 indexing
                    let (row, col) = ((row - 1) as usize, (col - 1) as usize);
                    self.cursor.pos.row = row;
                    self.cursor.pos.col = col;
                    self.cursor.clamp_to(self.size);
                }
            }

            // SCP (Save Cursor Position)
            's' => self.saved_cursor.pos = self.cursor.pos,
            // RCP (Restore Cursor Position)
            'u' => self.cursor.pos = self.saved_cursor.pos,

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
                            self.cursor.attrs = term::Attrs::default();
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
                        4 => self.cursor.attrs.underline = true,
                        // TODO: should really be a double underline.
                        21 => self.cursor.attrs.underline = true,
                        24 => self.cursor.attrs.underline = false,

                        // Bold Handling.
                        1 => self.cursor.attrs.bold = true,
                        22 => self.cursor.attrs.bold = false,

                        // Italic Handling.
                        3 => self.cursor.attrs.italic = true,
                        23 => self.cursor.attrs.italic = false,

                        // Inverse Handling.
                        7 => self.cursor.attrs.inverse = true,
                        27 => self.cursor.attrs.inverse = false,

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

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            warn!("malformed ESC seq");
            return;
        }

        match (intermediates, byte) {
            // save cursor (ESC 7)
            ([], b'7') => self.saved_cursor = self.cursor.clone(),
            // restore cursor (ESC 8)
            ([], b'8') => self.cursor = self.saved_cursor.clone(),

            _ => warn!("unhandled ESC seq ({intermediates:?}, {byte})"),
        }
    }

    fn terminated(&self) -> bool {
        // TODO: stub
        false
    }
}

fn p1_or(params: &vte::Params, default: u16) -> u16 {
    let n = params.iter().flatten().next().map(|x| *x).unwrap_or(0);
    if n == 0 {
        default
    } else {
        n
    }
}

fn p2(params: &vte::Params) -> Option<(u16, u16)> {
    let mut i = params.iter();
    if let Some(arg) = i.next() {
        let a1 = if arg.len() == 1 {
            arg[0]
        } else {
            return None;
        };
        if let Some(arg) = i.next() {
            if arg.len() == 1 {
                return Some((a1, arg[0]));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pos, Size};

    #[test]
    fn test_grid_new() {
        let size = Size {
            width: 10,
            height: 5,
        };
        let grid = Scrollback::new(5, size);
        assert_eq!(grid.size, size);
        assert!(grid.buf.is_empty());
    }

    #[test]
    fn test_grid_push_simple() -> anyhow::Result<()> {
        let size = Size {
            width: 5,
            height: 2,
        };
        let mut grid = Scrollback::new(5, size);
        let c = Cell::new('x', term::Attrs::default());

        grid.write_at_cursor(c.clone())?;

        let pos = Pos { row: 0, col: 0 };
        assert_eq!(grid.get(pos), Some(&c), "Scrollback:\n{grid}");

        Ok(())
    }

    #[test]
    fn test_grid_push_wrapping() -> anyhow::Result<()> {
        let size = Size {
            width: 2,
            height: 5,
        };
        let mut grid = Scrollback::new(5, size);

        // Fill first line
        grid.write_at_cursor(Cell::new('1', term::Attrs::default()))?;
        grid.write_at_cursor(Cell::new('2', term::Attrs::default()))?;

        // This should wrap to next line
        grid.write_at_cursor(Cell::new('3', term::Attrs::default()))?;

        assert_eq!(
            grid.get(Pos { row: 0, col: 0 }),
            Some(&Cell::new('1', term::Attrs::default())),
            "Scrollback:\n{grid}",
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 1 }),
            Some(&Cell::new('2', term::Attrs::default())),
            "Scrollback:\n{grid}",
        );
        assert_eq!(
            grid.get(Pos { row: 1, col: 0 }),
            Some(&Cell::new('3', term::Attrs::default())),
            "Scrollback:\n{grid}",
        );

        Ok(())
    }

    #[test]
    fn test_grid_indexing() -> anyhow::Result<()> {
        let size = Size {
            width: 10,
            height: 3,
        };
        let mut grid = Scrollback::new(3, size);

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
            "Failed to get top row. Scrollback:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(middle),
            Some(&c_mid),
            "Failed to get middle row. Scrollback:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(bottom),
            Some(&c_bot),
            "Failed to get bottom row. Scrollback:\n{:?}",
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
        let mut grid = Scrollback::new(20, size);

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
            "Scrollback:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 0 }),
            Some(&Cell::new('0', term::Attrs::default())),
            "Scrollback:\n{:?}",
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
        let mut grid = Scrollback::new(30, size);

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
            "Scrollback:\n{:?}",
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
            "Scrollback:\n{:?}",
            grid
        );
        assert_eq!(
            grid.get(Pos { row: 0, col: 9 }),
            Some(&Cell::new('9', term::Attrs::default())),
            "Scrollback:\n{:?}",
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
            let mut grid = Scrollback::new(100, start_size);

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
            let mut expected_grid = Scrollback::new(100, start_size);
            for i in 0..count {
                expected_grid.write_at_cursor(Cell::new(
                    char::from_u32(65 + i % 26).unwrap(),
                    term::Attrs::default(),
                ))?;
            }

            assert_eq!(
                grid, expected_grid,
                "Scrollback state mismatch after roundtrip resize {} -> {} -> {}",
                start_w, end_w, start_w
            );
        }

        Ok(())
    }
}
