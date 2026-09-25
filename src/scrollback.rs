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
    /// Set if the cursor is waiting to wrap, in which case `col` is just past
    /// the last column. That is the same spot in the logical line as the
    /// start of the row below, but the cursor has not moved down to it yet.
    pub pending_wrap: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AnchorRow {
    /// On a stored line, counted up from the bottom of the buffer.
    Line(usize),
    /// This many rows below the last stored line, where a program has moved
    /// the cursor down without writing anything.
    BelowContent(usize),
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
    ///
    /// A cursor with a wrap pending should be passed in just past the last
    /// column.
    pub fn anchor_cursor(
        &self,
        size: crate::Size,
        cursor: Pos,
        pending_wrap: bool,
    ) -> CursorAnchor {
        let row = match self.idx_from_bottom(size, cursor.row) {
            Some(idx) => AnchorRow::Line(idx),
            // A program can walk the cursor below the last line we have data
            // for by emitting newlines without writing anything, so remember
            // how big the gap was.
            None => AnchorRow::BelowContent(cursor.row - self.lines_below_grid_start(size)),
        };

        CursorAnchor { row, col: cursor.col, pending_wrap }
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
        if new_width == 0 {
            // Nothing can be laid out on a zero width screen. Leave the lines
            // alone so that they can still be reflowed properly once the
            // screen gets a real size again.
            return;
        }

        let old_len = self.buf.len();
        // The anchored lines, counted from the top so that we can spot them
        // as we go through the old buffer. Their index from the bottom is no
        // use here: splitting a line below them renumbers the buffer.
        let anchor_lines: Vec<Option<usize>> = anchors
            .iter()
            .map(|anchor| match anchor.row {
                AnchorRow::Line(idx) => old_len.checked_sub(idx + 1),
                // Nothing is stored at the cursor, so there is no cell to
                // follow and the gap carries over untouched.
                AnchorRow::BelowContent(_) => None,
            })
            .collect();
        // How far into the current logical line each anchored cell sits.
        let mut offsets: Vec<Option<usize>> = vec![None; anchors.len()];
        // The new line each anchor came out on, counted from the top, and
        // its column on that line.
        let mut new_positions: Vec<Option<(usize, usize)>> = vec![None; anchors.len()];

        // The new lines, top to bottom.
        let mut new_lines: Vec<Line> = Vec::with_capacity(old_len);
        let mut logical_line: Vec<Cell> = vec![];
        let old_buf = std::mem::take(&mut self.buf);
        for (i, line) in old_buf.into_iter().rev().enumerate() {
            for ((offset, anchor_line), anchor) in
                offsets.iter_mut().zip(anchor_lines.iter()).zip(anchors.iter())
            {
                if *anchor_line == Some(i) {
                    *offset = Some(logical_line.len() + anchor.col);
                }
            }

            logical_line.extend(line.cells);
            // The last line has nothing to continue onto, even if it claims
            // to wrap.
            if line.is_wrapped && i + 1 < old_len {
                continue;
            }

            let mut rows = rewrap(&mut logical_line, new_width);
            for (offset, new_pos) in offsets.iter_mut().zip(new_positions.iter_mut()) {
                let Some(offset) = offset.take() else {
                    continue;
                };

                // The anchored cell is on the last row that starts at or
                // before it.
                let mut nth = rows.iter().rposition(|(start, _)| *start <= offset).unwrap_or(0);
                let mut col = offset - rows[nth].0;
                // A cursor out in the blank space past the end of the content
                // can end up beyond the end of the last row. Give it rows to
                // sit on rather than moving it back, so that it keeps its
                // place in the logical line.
                //
                // Only stored cells count towards the offsets into a logical
                // line, so any row that the cursor sits at the end of or past
                // gets padded out to the full width. Otherwise the text that
                // gets written there would be pulled back into the blank
                // space by the next reflow.
                while col > new_width {
                    pad_line(&mut rows[nth].1, new_width);
                    rows[nth].1.is_wrapped = true;
                    let start = rows[nth].0 + new_width;
                    rows.push((start, Line::new()));
                    nth += 1;
                    col -= new_width;
                }
                if col == new_width {
                    pad_line(&mut rows[nth].1, new_width);
                }
                *new_pos = Some((new_lines.len() + nth, col));
            }
            new_lines.extend(rows.into_iter().map(|(_, row)| row));
            logical_line.clear();
        }

        for (anchor, new_pos) in anchors.iter_mut().zip(new_positions) {
            if let Some((nth, col)) = new_pos {
                anchor.row = AnchorRow::Line(new_lines.len() - 1 - nth);
                anchor.col = col;
                // Just past the end of a row is where the cursor would be
                // waiting to wrap if the line had been written at this width,
                // so that is what it does now, whether or not it was before.
                anchor.pending_wrap = col == new_width;
            }
        }

        // The bottom line goes at the front of the buffer.
        self.buf = new_lines.into_iter().rev().collect();
        // Narrowing makes for more lines, and the ones that no longer fit in
        // the scrollback fall off the top. An anchor on one of them resolves
        // to the top row.
        while self.buf.len() > self.lines {
            self.buf.pop_back();
        }
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

    /// The line at the given screen row, ready to be written to.
    ///
    /// We only store the rows that have been written to, so this has to fill
    /// in the blank rows between the last stored line and the one we want.
    /// Returns None if the row is not on the screen.
    pub fn materialize_line(&mut self, size: crate::Size, row: usize) -> Option<&mut Line> {
        if row >= size.height {
            return None;
        }

        while self.buf.len() < row + 1 {
            self.add_line(Line::new());
        }
        self.get_line_mut(size, row)
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

/// Chop a logical line up into rows of at most `width` cells, returning each
/// row along with the offset into the logical line that it starts at.
///
/// `width` must not be zero.
fn rewrap(cells: &mut Vec<Cell>, width: usize) -> Vec<(usize, Line)> {
    // Blank cells at the end of the line are just the part of it that
    // nothing has been written to (DCH, for one, pads lines out to the full
    // width). They should not spill over onto rows of their own.
    while cells.last().is_some_and(looks_unused) {
        cells.pop();
    }

    // A wide char that is wider than a whole row can't be shown at all. Blank
    // it out rather than drop it, since the offsets into the logical line
    // have to stay put.
    for col in 0..cells.len() {
        let cell_width = cells[col].width() as usize;
        if cell_width > width && !cells[col].is_wide_padding() {
            let attrs = cells[col].attrs().clone();
            let end = std::cmp::min(col + cell_width, cells.len());
            for cell in cells[col..end].iter_mut() {
                *cell = Cell::empty_with_attrs(attrs.clone());
            }
        }
    }

    let mut rows = vec![];
    let mut start = 0;
    while start < cells.len() {
        let mut end = std::cmp::min(start + width, cells.len());
        // Never split a wide char from its padding. If it does not fit, it
        // starts the next row and the columns it would have used stay blank,
        // just like when a wide char gets written at the end of a line.
        let mut owner = end;
        while owner > start && owner < cells.len() && cells[owner].is_wide_padding() {
            owner -= 1;
        }
        if owner > start {
            end = owner;
        }

        rows.push((
            start,
            Line { cells: cells[start..end].to_vec(), is_wrapped: end < cells.len() },
        ));
        start = end;
    }

    // A blank line still takes up a row.
    if rows.is_empty() {
        rows.push((0, Line::new()));
    }

    rows
}

/// True for a cell that looks just like one nothing has been written to: a
/// blank without any attrs that would show up on a blank.
fn looks_unused(cell: &Cell) -> bool {
    let attrs = cell.attrs();
    cell.is_empty()
        && !cell.is_wide_padding()
        && matches!(attrs.bgcolor, term::Color::Default)
        && !attrs.inverse
        && attrs.underline.is_none()
        && !attrs.strikethrough
        && !attrs.overline
        && attrs.framed.is_none()
}

/// Pad `line` out to `width` cells with blanks.
fn pad_line(line: &mut Line, width: usize) {
    while line.cells.len() < width {
        line.cells.push(Cell::empty());
    }
}
