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
    log,
    term::{self, AsTermInput, OriginMode, Pos, ScrollRegion},
    ContentRegion,
};
use std::collections::VecDeque;

use anyhow::{anyhow, Context};

// A scrollback stores the termianal state for the main screen.
// Alt screen state is stored seperately.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Scrollback {
    /// The entire scrollback buffer for the terminal.
    ///
    /// The bottom of the terminal is stored at the front of the deque
    /// and the top is stored at the back of the deque.
    pub buf: VecDeque<Line>,
    /// The number of lines of scrollback to store, independent of the
    /// size of the grid that is in view.
    lines: usize,
    /// The region of the screen in which scrolling happens.
    /// This is set by DECSTBM (CSI n ; n r).
    pub scroll_region: ScrollRegion,
    pub origin_mode: OriginMode,
    logger: log::Context,
}

/// Where the cursor sits relative to the buffer contents.
///
/// A screen row is derived from the buffer length and the height, so it does
/// not survive a resize. A position relative to the stored lines does.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct CursorAnchor {
    pub row: AnchorRow,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AnchorRow {
    /// On a stored line, counted up from the bottom of the buffer.
    Line(usize),
    /// This many rows below the last stored line, where a program has moved
    /// the cursor down without writing anything.
    BelowContent(usize),
}

/// Follows a single anchor through a reflow.
struct AnchorTracker {
    /// The anchored line, counted from the top so that we can spot it as we
    /// drain the old buffer. Its index from the bottom is no use here:
    /// splitting a line below it renumbers the buffer underneath us.
    line: Option<usize>,
    /// How far into the logical line the anchored cell sits.
    offset: Option<usize>,
    /// Which of the new grid lines it came out on, counted from the top.
    grid_line: Option<usize>,
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
            lines: scrollback_lines,
            scroll_region: ScrollRegion::default(),
            origin_mode: OriginMode::default(),
            logger: log::Context::None,
        }
    }

    pub fn set_logger(&mut self, logger: log::Context) {
        self.logger = logger;
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

    pub fn clamp_to_scroll_region(&self, cursor: &mut Pos, size: &crate::Size) {
        match self.origin_mode {
            OriginMode::Term => cursor.clamp_to(*size),
            OriginMode::ScrollRegion => cursor.clamp_to(self.scroll_region.as_region(size)),
        }
    }

    /// Pin the cursor to the buffer contents rather than to a screen row.
    ///
    /// A row is derived from the buffer length and the height, so it does not
    /// mean the same thing before and after a resize.
    /// A position relative to the stored lines does, which is what lets the
    /// cursor stay on the line it was on.
    pub fn anchor_cursor(&self, size: crate::Size, cursor: Pos) -> CursorAnchor {
        let row = match self.idx_from_bottom(size, cursor.row) {
            Some(idx) => AnchorRow::Line(idx),
            // A program can walk the cursor below the last line we have data
            // for by emitting newlines without writing anything, so remember
            // how big the gap was.
            None => AnchorRow::BelowContent(cursor.row - self.lines_below_grid_start(size)),
        };

        CursorAnchor { row, col: cursor.col }
    }

    /// Turn an anchor back into a screen position against the current buffer.
    pub fn resolve_cursor(&self, size: crate::Size, anchor: CursorAnchor) -> Pos {
        let grid_start = self.lines_below_grid_start(size);
        let row = match anchor.row {
            // A shrinking resize can push the anchored line up off the top of
            // the screen, and there is no right answer once that happens. Park
            // the cursor on the nearest visible row.
            AnchorRow::Line(idx) => grid_start.saturating_sub(idx + 1),
            AnchorRow::BelowContent(gap) => grid_start + gap,
        };

        Pos { row, col: anchor.col }
    }

    pub fn dump_contents_into(
        &self,
        buf: &mut Vec<u8>,
        size: crate::Size,
        dump_region: ContentRegion,
    ) {
        let lines_iter: Box<dyn Iterator<Item = (usize, &Line)>> = match dump_region {
            ContentRegion::All => Box::new(self.buf.iter().enumerate().rev()),
            ContentRegion::Screen => Box::new(self.buf.iter().take(size.height).enumerate().rev()),
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

        self.scroll_region.term_input_into(buf);
    }

    /// Re-chop the buffer into grid lines of `new_width`, moving each of
    /// `anchors` along with the cell it names.
    pub fn reflow(&mut self, new_width: usize, anchors: &mut [CursorAnchor]) {
        let mut new_scrollback = VecDeque::with_capacity(self.buf.len());
        let mut logical_line = VecDeque::new();

        let mut trackers: Vec<AnchorTracker> = anchors
            .iter()
            .map(|anchor| AnchorTracker {
                line: match anchor.row {
                    AnchorRow::Line(idx) => Some(self.buf.len() - 1 - idx),
                    // Nothing is stored at the cursor, so there is no cell to
                    // follow and the gap carries over untouched.
                    AnchorRow::BelowContent(_) => None,
                },
                offset: None,
                grid_line: None,
            })
            .collect();
        let mut logical_len = 0;
        let mut drained = 0;

        while let Some(grid_line) = self.buf.pop_back() {
            for (tracker, anchor) in trackers.iter_mut().zip(anchors.iter()) {
                if tracker.line == Some(drained) {
                    tracker.offset = Some(logical_len + anchor.col);
                    tracker.line = None;
                }
            }
            drained += 1;
            logical_len += grid_line.cells.len();

            let is_wrapped = grid_line.is_wrapped;
            logical_line.push_back(grid_line);

            if !is_wrapped {
                // We've gotten to the end of the logical line. We now
                // need to chop it up into grid lines by the new width.
                let lines_before = new_scrollback.len();
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

                // A logical line that ended exactly on the width boundary has
                // already been fully flushed, so the leftover is not a row. A
                // logical line that flushed nothing at all was blank, and a
                // blank line still occupies a row.
                if !line.cells.is_empty() || new_scrollback.len() == lines_before {
                    new_scrollback.push_front(line);
                }

                for (tracker, anchor) in trackers.iter_mut().zip(anchors.iter_mut()) {
                    if let Some(offset) = tracker.offset.take() {
                        let produced = new_scrollback.len() - lines_before;
                        let nth = offset / new_width;
                        anchor.col = if nth < produced {
                            offset % new_width
                        } else {
                            // Parked past the end of its own logical line, so
                            // there is no cell to follow down. Settle on the
                            // last row the line produced and let the caller
                            // clamp the column.
                            new_width
                        };
                        tracker.grid_line =
                            Some(lines_before + nth.min(produced.saturating_sub(1)));
                    }
                }
                logical_len = 0;
            }
        }

        for (tracker, anchor) in trackers.iter().zip(anchors.iter_mut()) {
            if let Some(nth) = tracker.grid_line {
                anchor.row = AnchorRow::Line(new_scrollback.len() - 1 - nth);
            }
        }

        self.buf = new_scrollback;
    }

    // Resolve a logical offset in the visible grid to an actual Line.
    pub fn get_line_mut(&mut self, size: crate::Size, row: usize) -> Option<&mut Line> {
        if let Some(i) = self.idx_from_bottom(size, row) {
            Some(&mut self.buf[i])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn get_line(&self, size: crate::Size, row: usize) -> Option<&Line> {
        if let Some(i) = self.idx_from_bottom(size, row) {
            Some(&self.buf[i])
        } else {
            None
        }
    }

    /// The number of lines at the front of the scrollback queue that
    /// are actually in view and are not just in the hidden scrollback
    /// region.
    pub fn lines_below_grid_start(&self, size: crate::Size) -> usize {
        std::cmp::min(self.buf.len(), size.height)
    }

    /// Return the index from the bottom of the scrollback buffer (the
    /// front of self.buf) for the given logical row index. Returns None
    /// if there is currently no line for that index (row points below
    /// the portion of the screen for which we actually have data).
    fn idx_from_bottom(&self, size: crate::Size, row: usize) -> Option<usize> {
        let grid_start = self.lines_below_grid_start(size);
        if row >= grid_start {
            return None;
        }
        Some(grid_start - 1 - row)
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

    //
    // Command Handlers
    //

    pub fn erase_to_end(&mut self, size: crate::Size, cursor: Pos) {
        if let Some(snip_line) = self.get_line_mut(size, cursor.row) {
            snip_line.erase(line::Section::ToEnd(cursor.col));
        }

        let end = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { bottom, .. }) => *bottom,
            _ => size.height,
        };
        for i in cursor.row + 1..end {
            if let Some(snip_line) = self.get_line_mut(size, i) {
                snip_line.erase(line::Section::Whole);
            }
        }
    }

    pub fn erase_from_start(&mut self, size: crate::Size, cursor: Pos) {
        let start = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { top, .. }) => *top,
            _ => 0,
        };

        for i in start..cursor.row {
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

        let (start, end) = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { top, bottom }) => (*top, *bottom),
            _ => (0, size.height),
        };

        for i in start..end {
            if let Some(snip_line) = self.get_line_mut(size, i) {
                snip_line.erase(line::Section::Whole);
            }
        }
    }

    /// SU (CSI S). Move the content of the scroll region up by `n` rows,
    /// opening blank rows at the bottom. Also what a linefeed at the bottom
    /// of the scroll region does.
    ///
    /// With no scroll region the rows that leave the top of the screen stay
    /// in the buffer as scrollback, which is why this is not just
    /// `delete_lines` at the top of the screen.
    pub fn scroll_up(&mut self, size: &crate::Size, n: usize) {
        match self.scroll_region {
            ScrollRegion::TrackSize => {
                // The rows below the last stored line have no storage, but
                // they are still on the screen and the content has to move
                // up past them. Fill them in so that the new lines really
                // land at the bottom of the screen rather than right below
                // the content.
                while self.buf.len() < size.height {
                    self.add_line(Line::new());
                }

                // Scrolling by more than a screenful just blanks the screen,
                // so there is no point in pushing even more blank lines into
                // the scrollback.
                for _ in 0..std::cmp::min(n, size.height) {
                    self.add_line(Line::new());
                }
            }
            ScrollRegion::Window { top, bottom } => {
                if bottom - top < n {
                    // just clobber everything
                    for i in top..bottom {
                        if let Some(line) = self.get_line_mut(*size, i) {
                            line.erase(line::Section::Whole);
                        }
                    }
                } else {
                    let to_shuffle = (bottom - top) - n;
                    for i in 0..to_shuffle {
                        let from_line = self.get_line(*size, top + n + i).cloned();
                        if let Some(to_line) = self.get_line_mut(*size, top + i) {
                            if let Some(from_line) = from_line {
                                *to_line = from_line;
                            } else {
                                to_line.erase(line::Section::Whole);
                            }
                        } else {
                            warn!(self.logger, "scrollback::scroll_up: out of bounds shuffle");
                        }
                    }
                    for i in 0..n {
                        if let Some(line) = self.get_line_mut(*size, top + to_shuffle + i) {
                            line.erase(line::Section::Whole);
                        } else {
                            warn!(self.logger, "scrollback::scroll_up: out of bounds backfill");
                        }
                    }
                }
            }
        }
    }

    /// SD (CSI T). Move the content of the scroll region down by `n` rows,
    /// opening blank rows at the top. Rows pushed past the bottom of the
    /// region are lost. Also what a reverse index at the top of the scroll
    /// region does.
    pub fn scroll_down(&mut self, size: &crate::Size, n: usize) {
        let top = match self.scroll_region {
            ScrollRegion::TrackSize => 0,
            ScrollRegion::Window { top, .. } => top,
        };

        self.insert_lines(&Pos { row: top, col: 0 }, size, n);
    }

    pub fn insert_lines(&mut self, cursor: &Pos, size: &crate::Size, n: usize) {
        let bottom = match self.scroll_region {
            ScrollRegion::TrackSize => size.height,
            ScrollRegion::Window { top, bottom } => {
                if cursor.row < top || bottom <= cursor.row {
                    // Insert Line does nothing when the cursor is outside
                    // the scroll region.
                    return;
                }
                bottom
            }
        };

        let grid_start = self.lines_below_grid_start(*size);
        let row_idx = match self.idx_from_bottom(*size, cursor.row) {
            Some(r) => r,
            // If the cursor is pointing past the point where we have
            // data, inserting blanks below the current line is a no-op,
            // no matter how many we are inserting.
            None => return,
        };

        // The lines below the cursor. N.B. this is stored in
        // reverse order from how you normally visualize it.
        let mut lines_below_cursor = Vec::with_capacity(row_idx);
        for _ in 0..=row_idx {
            if let Some(l) = self.buf.pop_front() {
                lines_below_cursor.push(l);
            } else {
                error!(self.logger, "internal error: row idx computed incorrectly");
            }
        }

        let lines_to_insert = std::cmp::min(n, bottom - cursor.row);
        for _ in 0..lines_to_insert {
            self.buf.push_front(Line::new());
        }

        // Up until the bottom of the scroll region, backfill
        // from the end of the lines_below_cursor vec.
        let new_scroll_region_end = std::cmp::min(grid_start + lines_to_insert, bottom);
        let backfill_to_bottom = (new_scroll_region_end - cursor.row) - lines_to_insert;
        for i in 0..backfill_to_bottom {
            let take_idx = lines_below_cursor.len() - 1 - i;
            self.buf.push_front(std::mem::replace(&mut lines_below_cursor[take_idx], Line::new()));
        }

        // Past the scroll region, backfill from the start of the
        // lines_below_cursor vec.
        let backfill_past_scroll_region = grid_start.saturating_sub(bottom);
        for i in 0..backfill_past_scroll_region {
            let take_idx = backfill_past_scroll_region - 1 - i;
            self.buf.push_front(std::mem::replace(&mut lines_below_cursor[take_idx], Line::new()));
        }
    }

    pub fn delete_lines(&mut self, cursor: &Pos, size: &crate::Size, n: usize) {
        let bottom = match self.scroll_region {
            ScrollRegion::TrackSize => size.height,
            ScrollRegion::Window { top, bottom } => {
                if cursor.row < top || bottom <= cursor.row {
                    // Insert Line does nothing when the cursor is outside
                    // the scroll region.
                    return;
                }
                bottom
            }
        };

        let grid_start = self.lines_below_grid_start(*size);
        let row_idx = match self.idx_from_bottom(*size, cursor.row) {
            Some(r) => r,
            // If the cursor is pointing past the point where we have
            // data, inserting blanks below the current line is a no-op,
            // no matter how many we are inserting.
            None => return,
        };

        // The lines below the cursor. N.B. this is stored in
        // reverse order from how you normally visualize it.
        let mut lines_below_cursor = Vec::with_capacity(row_idx);
        for _ in 0..=row_idx {
            if let Some(l) = self.buf.pop_front() {
                lines_below_cursor.push(l);
            } else {
                error!(self.logger, "internal error: row idx computed incorrectly");
            }
        }

        let effective_bottom = std::cmp::min(grid_start, bottom);
        let lines_to_delete = std::cmp::min(n, effective_bottom - cursor.row);

        // Replace the undeleted lines from the scrollback region.
        let undeleted_lines_in_scrollback_buf = (effective_bottom - cursor.row) - lines_to_delete;
        for i in 0..undeleted_lines_in_scrollback_buf {
            let take_idx = lines_below_cursor.len() - lines_to_delete - 1 - i;
            self.buf.push_front(std::mem::replace(&mut lines_below_cursor[take_idx], Line::new()));
        }

        // Inject the blank lines we need to put in at the bottom of the
        // scrollback region.
        for _ in 0..lines_to_delete {
            self.buf.push_front(Line::new());
        }

        // Past the scroll region, backfill from the start of the
        // lines_below_cursor vec.
        let backfill_past_scroll_region = grid_start.saturating_sub(bottom);
        for i in 0..backfill_past_scroll_region {
            let take_idx = backfill_past_scroll_region - 1 - i;
            self.buf.push_front(std::mem::replace(&mut lines_below_cursor[take_idx], Line::new()));
        }
    }
}
