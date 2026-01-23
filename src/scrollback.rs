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
    line::{self, Line},
    term::{self, AsTermInput, OriginMode, Pos, ScrollRegion},
    ContentRegion,
};
use std::collections::VecDeque;

use anyhow::{anyhow, Context};

// A scrollback stores the termianal state for the main screen.
// Alt screen state is stored seperately.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Scrollback {
    /// The entire scrollback buffer for the terminal.
    ///
    /// The bottom of the terminal is stored at the front of the deque
    /// and the top is stored at the back of the deque.
    pub buf: VecDeque<Line>,
    /// How far above the bottom of the buffer the visible window
    /// is.
    scroll_offset: usize,
    /// The number of lines of scrollback to store, independent of the
    /// size of the grid that is in view.
    lines: usize,
    /// The region of the screen in which scrolling happens.
    /// This is set by DECSTBM (CSI n ; n r).
    pub scroll_region: ScrollRegion,
    origin_mode: OriginMode,
}

impl std::fmt::Display for Scrollback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.buf.iter().rev() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl Scrollback {
    /// Create a new grid with the given number of lines of scrollback
    /// storage, and the given size window in view.
    pub fn new(scrollback_lines: usize) -> Self {
        Scrollback {
            buf: VecDeque::new(),
            scroll_offset: 0,
            lines: scrollback_lines,
            scroll_region: ScrollRegion::default(),
            origin_mode: OriginMode::default(),
        }
    }

    /// Get the max number of scrollback lines this grid
    /// can store.
    pub fn scrollback_lines(&self) -> usize {
        self.lines
    }

    /// Set a new max number of scrollback lines this grid can
    /// store. If this is less than the current number, trailing
    /// data will be dropped.
    pub fn set_scrollback_lines(&mut self, size: crate::Size, mut scrollback_lines: usize) {
        if scrollback_lines < size.height {
            scrollback_lines = size.height;
        }

        while self.buf.len() > scrollback_lines {
            self.buf.pop_back();
        }
        self.lines = scrollback_lines;
    }

    /// Set the cell at the given grid coordinates.
    pub fn set(&mut self, size: crate::Size, pos: Pos, cell: Cell) -> anyhow::Result<()> {
        if let Some(line) = self.get_line_mut(size, pos.row) {
            return line.set_cell(size.width, pos.col, cell);
        }

        Ok(())
    }

    fn add_line(&mut self, line: Line) {
        self.buf.push_front(line);
        while self.buf.len() > self.lines {
            self.buf.pop_back();
        }
    }

    pub fn snap_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn clamp_to_scroll_region(&self, cursor: &mut Pos, size: &crate::Size) {
        match self.origin_mode {
            OriginMode::Term => cursor.clamp_to(*size),
            OriginMode::ScrollRegion => cursor.clamp_to(self.scroll_region.as_region(size)),
        }
    }

    pub fn dump_contents_into(
        &self,
        buf: &mut Vec<u8>,
        size: crate::Size,
        dump_region: ContentRegion,
    ) {
        let lines_iter: Box<dyn Iterator<Item = (usize, &Line)>> = match dump_region {
            ContentRegion::All => Box::new(self.buf.iter().enumerate().rev()),
            ContentRegion::Screen => Box::new(
                self.buf.iter().skip(self.scroll_offset).take(size.height).enumerate().rev(),
            ),
            ContentRegion::BottomLines(nlines) => {
                Box::new(self.buf.iter().take(nlines).enumerate().rev())
            }
        };

        for (i, line) in lines_iter {
            line.term_input_into(buf);
            if i != 0 {
                term::Crlf::default().term_input_into(buf);
            }
        }

        if let ScrollRegion::Window { top, bottom } = self.scroll_region {
            // We have a zero index [) (clopen) range and we need a 1 indexed
            // [] (fully closed) range, so we need to shift top up, but bottom
            // is already right.
            term::ControlCodes::set_scroll_region((top + 1) as u16, bottom as u16)
                .term_input_into(buf);
        }

        let generate_scroll = self.scroll_offset > 0
            && matches!(self.scroll_region, ScrollRegion::TrackSize)
            && match dump_region {
                ContentRegion::All => true,
                ContentRegion::Screen => false,
                ContentRegion::BottomLines(n) => n >= size.height + self.scroll_offset,
            };
        if generate_scroll {
            term::ControlCodes::scroll_up(self.scroll_offset as u16).term_input_into(buf);
        }
    }

    pub fn reflow(&mut self, new_width: usize) {
        // TODO: this needs to move the<Icursor and saved cursor to have them
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

    pub fn get_line_mut(&mut self, size: crate::Size, row: usize) -> Option<&mut Line> {
        let grid_start = self.lines_below_grid_start(size);
        if row >= grid_start {
            return None;
        }

        let idx_from_bottom = (grid_start - 1) - row;
        Some(&mut self.buf[idx_from_bottom])
    }

    #[allow(dead_code)]
    pub fn get_line(&self, size: crate::Size, row: usize) -> Option<&Line> {
        let grid_start = self.lines_below_grid_start(size);
        if row >= grid_start {
            return None;
        }

        let idx_from_bottom = (grid_start - 1) - row;
        Some(&self.buf[idx_from_bottom])
    }

    /// The number of lines at the front of the scrollback queue that
    /// are actually in view and are not just in the hidden scrollback
    /// region.
    pub fn lines_below_grid_start(&self, size: crate::Size) -> usize {
        let grid_start = size.height + self.scroll_offset;
        if self.buf.len() < grid_start {
            self.buf.len()
        } else {
            grid_start
        }
    }

    /// Write the given cell at the given cursor position, returning the next
    /// cursor position.
    pub fn write_at_cursor(
        &mut self,
        size: crate::Size,
        mut cursor: Pos,
        cell: Cell,
    ) -> anyhow::Result<Pos> {
        if size.width < 1 {
            return Err(anyhow!("cannot write to zero width terminal grid"));
        }

        // We do the wrapping before writing a cell rather than after
        // doing so to allow the user to avoid setting the wrap bit
        // by entering \r\n right after writing the very rightmost
        // cell.
        if cursor.col >= size.width {
            if let Some(line) = self.get_line_mut(size, cursor.row) {
                line.is_wrapped = true;
            } else {
                return Err(anyhow!("unexpectedly missing line when setting wrap marker"));
            }

            cursor.col = 0;
            cursor.row += 1;
        }

        // If we've run off the end, add a new line and clamp.
        if cursor.row >= size.height {
            self.add_line(Line::new());
            cursor.row -= 1;
        }

        assert!(self.lines >= size.height);
        while self.buf.len() < cursor.row + 1 {
            // TODO: these lines will all count as having
            // not been wrapped and will be retained on reflow.
            // Is that actually what we want?
            self.add_line(Line::new());
        }

        if cursor.col + cell.width() as usize >= size.width + 1 {
            if let Some(line) = self.get_line_mut(size, cursor.row) {
                line.is_wrapped = true;
            } else {
                return Err(anyhow!(
                    "unexpectedly missing line when setting wide char wrap marker"
                ));
            }

            cursor.col = 0;
            cursor.row += 1;

            if self.buf.len() < cursor.row + 1 {
                self.add_line(Line::new())
            }
        }

        let mut npad = cell.width().saturating_sub(1);
        self.set(size, cursor, cell).context("setting main cell")?;
        cursor.col += 1;
        while npad > 0 {
            assert!(cursor.col < size.width);

            self.set(size, cursor, Cell::wide_pad()).context("padding after wide char")?;
            cursor.col += 1;
            npad -= 1;
        }

        Ok(cursor)
    }

    pub fn erase_to_end(&mut self, size: crate::Size, cursor: Pos) {
        if let Some(snip_line) = self.get_line_mut(size, cursor.row) {
            snip_line.erase(line::Section::ToEnd(cursor.col));
        }

        for i in cursor.row + 1..size.height {
            if let Some(snip_line) = self.get_line_mut(size, i) {
                snip_line.erase(line::Section::Whole);
            }
        }
    }

    pub fn erase_from_start(&mut self, size: crate::Size, cursor: Pos) {
        for i in 0..cursor.row {
            if let Some(snip_line) = self.get_line_mut(size, i) {
                snip_line.erase(line::Section::Whole);
            }
        }
        if let Some(snip_line) = self.get_line_mut(size, cursor.row) {
            snip_line.erase(line::Section::StartTo(cursor.col));
        }
    }

    pub fn erase(&mut self, size: crate::Size, include_scrollback: bool) {
        if include_scrollback {
            self.buf.truncate(0);
            return;
        }

        for i in 0..size.height {
            if let Some(snip_line) = self.get_line_mut(size, i) {
                snip_line.erase(line::Section::Whole);
            }
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset += n;
        if self.scroll_offset > self.lines {
            self.scroll_offset = self.lines;
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }
}
