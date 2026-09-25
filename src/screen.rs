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

//! The screen module defines a trait that encapsulates the functionality
//! which is shared between the normal scrollback screen and the altscreen.

use crate::{
    altscreen::AltScreen,
    cell::Cell,
    charset::Charsets,
    line::Line,
    log,
    scrollback::Scrollback,
    term::{self, AsTermInput, OriginMode, Pos, Region, ScrollRegion},
};

use anyhow::{anyhow, Context};

/// A screen containts some kind of grid of cells, plus top
/// level fields that are common to all screen variants.
#[derive(Debug)]
pub struct Screen {
    // The actual storage for lines of cells. This will take
    // different forms depending on which type of screen this
    // is, and determins which type of screen it is.
    grid: Grid,
    // The size of the visible window.
    pub size: crate::Size,
    /// The current position of the cursor within the in-view window described
    /// by `size`. (0,0) is the upper left.
    pub cursor: Pos,
    /// Set when a char has just been written into the last column.
    ///
    /// The cursor stays on the last column rather than moving down to the
    /// next line right away, and the wrap only happens if another char gets
    /// printed. This is what lets a program write into the bottom right cell
    /// without scrolling, or end a full width line with CRLF without leaving
    /// a blank line behind. Anything that explicitly moves the cursor cancels
    /// the pending wrap. DEC calls this the "last column flag".
    pub pending_wrap: bool,
    // The slot where cursor position info is saved by the SCP/RCP
    // and ESC 7 / ESC 8 commands.
    pub saved_cursor: SavedCursor,
    logger: log::Context,
}

impl Screen {
    /// Create a new scrollback mode screen (a regular terminal screen).
    pub fn scrollback(mut scrollback_lines: usize, size: crate::Size) -> Self {
        if scrollback_lines < size.height {
            scrollback_lines = size.height;
        }

        Screen {
            grid: Grid::Scrollback(Scrollback::new(scrollback_lines)),
            size,
            cursor: Pos { row: 0, col: 0 },
            pending_wrap: false,
            saved_cursor: SavedCursor::new(Pos { row: 0, col: 0 }),
            logger: log::Context::None,
        }
    }

    /// Create a new alt screen mode screen (used by ncurses apps like vim).
    pub fn alt(size: crate::Size) -> Self {
        Screen {
            grid: Grid::AltScreen(AltScreen::new(size)),
            size,
            cursor: Pos { row: 0, col: 0 },
            pending_wrap: false,
            saved_cursor: SavedCursor::new(Pos { row: 0, col: 0 }),
            logger: log::Context::None,
        }
    }

    pub fn set_logger(&mut self, logger: log::Context) {
        self.grid.set_logger(logger.clone());
        self.logger = logger;
    }

    /// Return the number of scrollback lines iff this is a scrollback screen.
    pub fn scrollback_lines(&self) -> Option<usize> {
        if let Grid::Scrollback(scrollback) = &self.grid {
            Some(scrollback.scrollback_lines())
        } else {
            None
        }
    }

    /// Set the number of scrollback lines. Only works if this is a scrollback
    /// screen.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        if let Grid::Scrollback(scrollback) = &mut self.grid {
            scrollback.set_scrollback_lines(self.size, scrollback_lines);
        } else {
            warn!(self.logger, "attempt to set scrollback lines on non-scrollback screen");
        }
    }

    pub fn set_scroll_region(&mut self, scroll_region: ScrollRegion) {
        self.store_scroll_region(clamp_scroll_region(scroll_region, self.size));
    }

    fn store_scroll_region(&mut self, scroll_region: ScrollRegion) {
        match &mut self.grid {
            Grid::Scrollback(scrollback) => scrollback.scroll_region = scroll_region,
            Grid::AltScreen(altscreen) => altscreen.scroll_region = scroll_region,
        }
    }

    pub fn set_origin_mode(&mut self, origin_mode: OriginMode) {
        match &mut self.grid {
            Grid::Scrollback(s) => s.origin_mode = origin_mode,
            Grid::AltScreen(alt) => alt.origin_mode = origin_mode,
        }
    }

    /// Given a 1-indexed position as the user would directly provide in
    /// a CUP command, update the cursor position, taking the current origin
    /// mode and scroll region into account.
    pub fn set_cursor(&mut self, pos: Pos) {
        match self.grid.origin_mode() {
            OriginMode::Term => {
                self.cursor.row = pos.row.saturating_sub(1);
                self.cursor.col = pos.col.saturating_sub(1);
            }
            OriginMode::ScrollRegion => match self.grid.scroll_region() {
                ScrollRegion::TrackSize => {
                    self.cursor.row = pos.row.saturating_sub(1);
                    self.cursor.col = pos.col.saturating_sub(1);
                }
                ScrollRegion::Window { top, .. } => {
                    self.cursor.row = pos.row.saturating_sub(1) + top;
                    self.cursor.col = pos.col.saturating_sub(1);
                }
            },
        }
    }

    pub fn dump_contents_into(&self, buf: &mut Vec<u8>, dump_region: crate::ContentRegion) {
        match &self.grid {
            Grid::Scrollback(scrollback) => {
                scrollback.dump_contents_into(buf, self.size, dump_region)
            }
            Grid::AltScreen(altscreen) => altscreen.term_input_into(buf),
        }

        // Origin mode has to be restored before the cursor, since enabling
        // it homes the cursor.
        let origin_mode = self.grid.origin_mode();
        if matches!(origin_mode, OriginMode::ScrollRegion) {
            term::control_codes().enable_scroll_region_origin_mode.term_input_into(buf);
        }

        // Rows are relative to the top of the scroll region once origin
        // mode is on, mirroring `set_cursor`.
        let row = match (origin_mode, self.grid.scroll_region()) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { top, .. }) => {
                self.cursor.row.saturating_sub(*top)
            }
            _ => self.cursor.row,
        };

        if self.pending_wrap {
            self.dump_pending_wrap_into(buf, row);
        } else {
            term::ControlCodes::cursor_position((row + 1) as u16, (self.cursor.col + 1) as u16)
                .term_input_into(buf);
        }
    }

    /// Leave the cursor on the last column with a wrap pending.
    ///
    /// There is no control code that sets the pending wrap flag, so we get
    /// the terminal into that state the same way the application did, by
    /// printing the char that sits at the end of the line. It is already on
    /// the screen, so printing it again over itself changes nothing else.
    /// `row` is the cursor row as `dump_contents_into` addresses it.
    fn dump_pending_wrap_into(&self, buf: &mut Vec<u8>, row: usize) {
        let width = self.size.width;
        let line = self.grid.get_line(self.size, self.cursor.row);
        let cell_at = |col: usize| line.and_then(|l| l.get_cell(width, col));

        // The last column might be the padding half of a wide char, in which
        // case the char to print starts one column further left.
        let mut col = width.saturating_sub(1);
        while col > 0 && cell_at(col).is_some_and(|c| c.is_wide_padding()) {
            col -= 1;
        }
        let cell = match cell_at(col) {
            Some(cell)
                if !cell.is_wide_padding()
                    && col + std::cmp::max(cell.width() as usize, 1) == width =>
            {
                cell.clone()
            }
            // The last column is not covered by anything we can print, which
            // happens when a wide char has been partly overwritten. A blank
            // looks the same.
            _ => {
                col = width.saturating_sub(1);
                Cell::empty()
            }
        };

        term::ControlCodes::cursor_position((row + 1) as u16, (col + 1) as u16)
            .term_input_into(buf);
        let blank_attrs = term::Attrs::default();
        for code in blank_attrs.transition_to(cell.attrs()) {
            code.term_input_into(buf);
        }
        cell.term_input_into(buf);
        for code in cell.attrs().transition_to(&blank_attrs) {
            code.term_input_into(buf);
        }
    }

    /// Emit the codes needed to undo the global terminal state that
    /// `dump_contents_into` leaves set.
    ///
    /// We track the scroll region and origin mode per screen, but a real
    /// terminal only has one of each, so a dump that restores more than one
    /// screen has to clean up after the earlier screens. We only emit the
    /// codes we actually need because restore buffers get written to the
    /// wire on every reattach.
    ///
    /// Both codes home the cursor, so this returns whether it emitted any.
    pub fn dump_global_state_reset_into(&self, buf: &mut Vec<u8>) -> bool {
        let mut homed = false;
        if matches!(self.grid.scroll_region(), ScrollRegion::Window { .. }) {
            term::control_codes().unset_scroll_region.term_input_into(buf);
            homed = true;
        }

        if matches!(self.grid.origin_mode(), OriginMode::ScrollRegion) {
            term::control_codes().disable_scroll_region_origin_mode.term_input_into(buf);
            homed = true;
        }

        homed
    }

    /// Whether `dump_contents_into` leaves the cursor in the top left corner.
    pub fn dump_leaves_cursor_home(&self) -> bool {
        self.cursor == Pos { row: 0, col: 0 } && !self.pending_wrap
    }

    pub fn resize(&mut self, new_size: crate::Size) {
        let old_size = self.size;
        // A cursor with a wrap pending is logically just past the char in
        // the last column, and that is the spot it should keep following.
        let cursor = past_pending_wrap(self.cursor, self.pending_wrap);
        let saved_cursor = past_pending_wrap(self.saved_cursor.pos, self.saved_cursor.pending_wrap);
        let ((cursor, pending_wrap), (saved_cursor, saved_pending_wrap)) = match &mut self.grid {
            Grid::Scrollback(scrollback) => {
                // A row is derived from the buffer length and the height, both
                // of which this changes, so the cursor has to be re-derived
                // rather than just clamped. The saved cursor is a row too, and
                // a resize between DECSC and DECRC moves it just the same.
                let mut anchors = [
                    scrollback.anchor_cursor(old_size, cursor, self.pending_wrap),
                    scrollback.anchor_cursor(
                        old_size,
                        saved_cursor,
                        self.saved_cursor.pending_wrap,
                    ),
                ];
                // Only the width decides where lines wrap.
                if new_size.width != old_size.width {
                    scrollback.reflow(old_size.width, new_size.width, &mut anchors);
                }
                (
                    (scrollback.resolve_cursor(new_size, anchors[0]), anchors[0].pending_wrap),
                    (scrollback.resolve_cursor(new_size, anchors[1]), anchors[1].pending_wrap),
                )
            }
            Grid::AltScreen(altscreen) => {
                altscreen.resize(new_size);
                ((cursor, self.pending_wrap), (saved_cursor, self.saved_cursor.pending_wrap))
            }
        };
        self.size = new_size;

        // Like in other terminals, a resize drops the scroll region. The rows
        // it covered have moved or might not even be there anymore, and apps
        // set up a new one when they redraw for the new size.
        self.store_scroll_region(ScrollRegion::TrackSize);

        (self.cursor, self.pending_wrap) = settle_cursor(cursor, pending_wrap, self.size);
        (self.saved_cursor.pos, self.saved_cursor.pending_wrap) =
            settle_cursor(saved_cursor, saved_pending_wrap, self.size);
    }

    /// Bring the cursor back within the region it may occupy after it has
    /// been explicitly moved.
    ///
    /// Every explicit cursor movement ends up here, and moving the cursor
    /// cancels any pending wrap, so this clears it as well.
    pub fn clamp(&mut self) {
        self.pending_wrap = false;
        match &self.grid {
            Grid::Scrollback(scrollback) => {
                scrollback.clamp_to_scroll_region(&mut self.cursor, &self.size)
            }
            Grid::AltScreen(altscreen) => {
                altscreen.clamp_to_scroll_region(&mut self.cursor, &self.size)
            }
        }
    }

    //
    // Control Code Handlers
    //

    /// Write a cell at the cursor and advance the cursor past it, wrapping
    /// onto the next line first if a wrap is pending or the cell does not
    /// fit in what is left of the current line.
    ///
    /// In insert mode (IRM) the rest of the line gets shifted right to make
    /// room for the cell rather than being overwritten. With autowrap
    /// (DECAWM) off the cursor never leaves the line, and chars that run
    /// into the right edge overwrite the last column instead.
    pub fn write_at_cursor(
        &mut self,
        cell: Cell,
        insert_mode: bool,
        autowrap: bool,
    ) -> anyhow::Result<()> {
        let width = self.size.width;
        if width == 0 || self.size.height == 0 {
            return Err(anyhow!("cannot write to a zero sized screen"));
        }
        let cell_width = cell.width() as usize;
        if cell_width > width {
            return Err(anyhow!("{} column wide char does not fit on the screen", cell_width));
        }
        self.cursor.clamp_to(self.size);

        // The blank that a scroll opens up if the cell has to wrap onto a new
        // line at the bottom of the scroll region.
        let fill = Cell::blank(cell.attrs());
        if self.pending_wrap {
            if autowrap {
                self.wrap(&fill);
            } else {
                // Autowrap was turned off while the wrap was pending.
                self.pending_wrap = false;
            }
        }
        // A wide char never gets split across lines. If it does not fit, it
        // goes on the next line and the columns it did not use stay blank.
        // Without autowrap it gets squeezed in at the end of this one.
        if self.cursor.col + cell_width > width {
            if autowrap {
                self.wrap(&fill);
            } else {
                self.cursor.col = width - cell_width;
            }
        }

        let col = self.cursor.col;
        let Some(line) = self.grid.materialize_line(self.size, self.cursor.row) else {
            return Err(anyhow!("no line for cursor row {}", self.cursor.row));
        };
        if insert_mode {
            // The blanks this opens up get written over right away.
            line.insert_character(width, col, cell_width, &Cell::empty());
        }
        line.write_cell(width, col, cell).context("writing cell")?;

        if col + cell_width < width {
            self.cursor.col = col + cell_width;
        } else {
            self.cursor.col = width - 1;
            self.pending_wrap = autowrap;
        }

        Ok(())
    }

    /// Move to the start of the next line because the current one is full.
    fn wrap(&mut self, fill: &Cell) {
        // The line only continues onto the next one if the cursor is really
        // going to get there. Below the scroll region on the last row it has
        // nowhere to go, so the next char just overwrites this line.
        let (_, bottom) = self.grid.scroll_region().as_region(&self.size).row_bounds();
        if self.cursor.row + 1 == bottom || self.cursor.row + 1 < self.size.height {
            if let Some(line) = self.grid.materialize_line(self.size, self.cursor.row) {
                line.is_wrapped = true;
            }
        }

        self.linefeed(fill);
        self.cursor.col = 0;
    }

    /// Move the cursor down a row, scrolling the content of the scroll region
    /// up instead if the cursor is on its bottom row. This implements LF and
    /// is also how the cursor gets to the next line when wrapping.
    ///
    /// The cursor can sit outside of the scroll region, in which case it
    /// just moves down until it reaches the bottom of the screen.
    ///
    /// `fill` is the blank that scrolling opens up the new row with, which
    /// is painted with the current background color (see `Cell::blank`).
    pub fn linefeed(&mut self, fill: &Cell) {
        self.pending_wrap = false;
        let (_, bottom) = self.grid.scroll_region().as_region(&self.size).row_bounds();
        if self.cursor.row + 1 == bottom {
            self.scroll_up(1, fill);
        } else if self.cursor.row + 1 < self.size.height {
            self.cursor.row += 1;
        }
    }

    /// Erase whichever screen is currently active from the cursor
    /// position to the bottom, leaving `fill` behind. Used to implement
    /// 'CSI 0 J'
    pub fn erase_to_end(&mut self, fill: &Cell) {
        self.pending_wrap = false;
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase_to_end(self.size, self.cursor, fill),
            Grid::AltScreen(alt) => alt.erase_to_end(width, self.cursor, fill),
        }
    }

    /// Erase whichever screen is currently active from the top to the
    /// cursor position, leaving `fill` behind. Used to implement 'CSI 1 J'
    pub fn erase_from_start(&mut self, fill: &Cell) {
        self.pending_wrap = false;
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase_from_start(self.size, self.cursor, fill),
            Grid::AltScreen(alt) => alt.erase_from_start(width, self.cursor, fill),
        }
    }

    /// Erase whichever screen is currently active, not including scrollback,
    /// leaving `fill` behind. Used to implement 'CSI 2 J'.
    pub fn erase(&mut self, fill: &Cell) {
        self.pending_wrap = false;
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase(self.size, fill),
            Grid::AltScreen(alt) => alt.erase(width, fill),
        }
    }

    /// Drop the scrollback, leaving what is on the screen alone. Used to
    /// implement 'CSI 3 J'. The alt screen has no scrollback to drop.
    pub fn erase_scrollback(&mut self) {
        if let Grid::Scrollback(s) = &mut self.grid {
            s.erase_scrollback(self.size);
        }
    }

    /// Gets the current line. If the cursor is not currently over an actual
    /// line, this returns nothing.
    pub fn get_line_mut(&mut self) -> Option<&mut Line> {
        match &mut self.grid {
            Grid::Scrollback(s) => s.get_line_mut(self.size, self.cursor.row),
            Grid::AltScreen(alt) => alt.get_line_mut(self.cursor.row),
        }
    }

    /// Gets the current line for an edit that leaves `fill` behind.
    ///
    /// A row that nothing is stored for is blank already, so an edit that
    /// only leaves plain blanks behind has nothing to do there and this
    /// returns nothing, but blanks painted with a background color have to
    /// be stored.
    pub fn line_to_erase(&mut self, fill: &Cell) -> Option<&mut Line> {
        if fill.attrs().has_attrs() {
            self.grid.materialize_line(self.size, self.cursor.row)
        } else {
            self.get_line_mut()
        }
    }

    /// SU (CSI S). Move the content of the scroll region up by `n` rows,
    /// opening rows of `fill` at the bottom. The cursor does not move.
    pub fn scroll_up(&mut self, n: usize, fill: &Cell) {
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.scroll_up(&self.size, n, fill),
            Grid::AltScreen(alt) => alt.scroll_up(width, n, fill),
        }
    }

    /// SD (CSI T). Move the content of the scroll region down by `n` rows,
    /// opening rows of `fill` at the top. The cursor does not move.
    pub fn scroll_down(&mut self, n: usize, fill: &Cell) {
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.scroll_down(&self.size, n, fill),
            Grid::AltScreen(alt) => alt.scroll_down(width, n, fill),
        }
    }

    pub fn scroll_region(&self, by_origin_mode: bool) -> ScrollRegion {
        if by_origin_mode {
            match self.grid.origin_mode() {
                OriginMode::Term => ScrollRegion::TrackSize,
                OriginMode::ScrollRegion => self.grid.scroll_region().clone(),
            }
        } else {
            self.grid.scroll_region().clone()
        }
    }

    /// Handler for the Insert Line command (CSI n L).
    ///
    /// n lines of `fill` are inserted above the current line, dropping any
    /// lines that get pushed out of the current scroll region.
    pub fn insert_lines(&mut self, n: usize, fill: &Cell) {
        self.pending_wrap = false;
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.insert_lines(&self.cursor, &self.size, n, fill),
            Grid::AltScreen(alt) => alt.insert_lines(width, &self.cursor, n, fill),
        }
    }

    /// Handler for the Delete Line command (CSI n M).
    ///
    /// n lines below the current line are deleted (including the current line),
    /// sucking any lines below the current line up. New lines of `fill` are
    /// inserted at the bottom of the scroll region.
    pub fn delete_lines(&mut self, n: usize, fill: &Cell) {
        self.pending_wrap = false;
        let width = self.size.width;
        match &mut self.grid {
            Grid::Scrollback(s) => s.delete_lines(&self.cursor, &self.size, n, fill),
            Grid::AltScreen(alt) => alt.delete_lines(width, &self.cursor, n, fill),
        }
    }
}

/// Bring a scroll region back onto the grid.
///
/// Everything downstream assumes `bottom` is a real row: the scrolling code
/// indexes the grid with it, and LF walks the cursor off the screen chasing a
/// bottom it can never reach. A region with no rows left in it is dropped.
fn clamp_scroll_region(scroll_region: ScrollRegion, size: crate::Size) -> ScrollRegion {
    let ScrollRegion::Window { top, bottom } = scroll_region else {
        return ScrollRegion::TrackSize;
    };

    let bottom = std::cmp::min(bottom, size.height);
    if top >= bottom {
        return ScrollRegion::TrackSize;
    }

    ScrollRegion::Window { top, bottom }
}

/// Where a cursor logically is. A pending wrap means it is just past the char
/// in the last column, even though it is displayed on top of it.
fn past_pending_wrap(pos: Pos, pending_wrap: bool) -> Pos {
    Pos { row: pos.row, col: pos.col + pending_wrap as usize }
}

/// Bring a cursor that has been through a resize back onto the screen.
///
/// A cursor that is waiting to wrap can come out of a resize just past the
/// end of a line, and then it should still be waiting to wrap. Anywhere else
/// on the line there is no wrap left to do.
fn settle_cursor(mut pos: Pos, pending_wrap: bool, size: crate::Size) -> (Pos, bool) {
    let pending_wrap = pending_wrap && pos.col >= size.width;
    pos.clamp_to(size);
    (pos, pending_wrap)
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.size.width {
            write!(f, "-")?;
        }
        writeln!(f, "")?;

        match &self.grid {
            Grid::Scrollback(s) => write!(f, "{}", s)?,
            Grid::AltScreen(alt) => write!(f, "{}", alt)?,
        }

        for _ in 0..self.size.width {
            write!(f, "-")?;
        }

        Ok(())
    }
}

/// A position that the terminal was writing at. Includes attributes that
/// have been previously set via control codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SavedCursor {
    pub pos: Pos,
    pub attrs: term::Attrs,
    /// Whether a wrap was pending. DECSC saves the flag and DECRC puts it
    /// back, so a program that saves the cursor right after filling the last
    /// column still wraps once it restores it and keeps printing.
    pub pending_wrap: bool,
    /// The charset designations and shifts. DECSC saves these too, so a
    /// program can draw a box somewhere else and then carry on printing
    /// with whatever charset it had before.
    pub charsets: Charsets,
}

impl SavedCursor {
    pub fn new(pos: Pos) -> Self {
        SavedCursor {
            pos,
            attrs: term::Attrs::default(),
            pending_wrap: false,
            charsets: Charsets::default(),
        }
    }
}

#[derive(Debug)]
enum Grid {
    Scrollback(Scrollback),
    AltScreen(AltScreen),
}

impl Grid {
    fn origin_mode(&self) -> OriginMode {
        match self {
            Grid::Scrollback(s) => s.origin_mode,
            Grid::AltScreen(alt) => alt.origin_mode,
        }
    }

    fn scroll_region(&self) -> &ScrollRegion {
        match self {
            Grid::Scrollback(s) => &s.scroll_region,
            Grid::AltScreen(alt) => &alt.scroll_region,
        }
    }

    fn set_logger(&mut self, logger: log::Context) {
        match self {
            Grid::Scrollback(s) => s.set_logger(logger),
            Grid::AltScreen(alt) => alt.set_logger(logger),
        }
    }

    /// The line at the given screen row, if there is one stored.
    fn get_line(&self, size: crate::Size, row: usize) -> Option<&Line> {
        match self {
            Grid::Scrollback(s) => s.get_line(size, row),
            Grid::AltScreen(alt) => alt.buf.get(row),
        }
    }

    /// The line at the given screen row, ready to be written to. Returns
    /// None if the row is not on the screen.
    fn materialize_line(&mut self, size: crate::Size, row: usize) -> Option<&mut Line> {
        match self {
            Grid::Scrollback(s) => s.materialize_line(size, row),
            Grid::AltScreen(alt) => alt.get_line_mut(row),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Attrs;
    use crate::Size;

    #[test]
    fn altscreen_get_line_mut_at_end_of_buf() {
        let mut screen = Screen::alt(Size { width: 5, height: 3 });
        screen.cursor = Pos { row: 3, col: 0 };

        assert!(screen.get_line_mut().is_none());
    }

    #[test]
    fn altscreen_get_line_mut_past_end_of_buf() {
        let mut screen = Screen::alt(Size { width: 5, height: 3 });
        screen.cursor = Pos { row: 7, col: 0 };

        assert!(screen.get_line_mut().is_none());
    }

    #[test]
    fn altscreen_resize_grow_height() {
        let mut screen = Screen::alt(Size { width: 10, height: 5 });
        screen.resize(Size { width: 10, height: 10 });

        match &screen.grid {
            Grid::AltScreen(alt) => {
                assert_eq!(alt.buf.len(), 10);
            }
            _ => panic!("wrong grid type"),
        }
        assert_eq!(screen.size.height, 10);
    }

    #[test]
    fn altscreen_resize_shrink_height() {
        let mut screen = Screen::alt(Size { width: 10, height: 10 });
        screen.resize(Size { width: 10, height: 5 });
        match &screen.grid {
            Grid::AltScreen(alt) => {
                assert_eq!(alt.buf.len(), 5);
            }
            _ => panic!("wrong grid type"),
        }
        assert_eq!(screen.size.height, 5);
    }

    #[test]
    fn altscreen_resize_shrink_width() {
        let mut screen = Screen::alt(Size { width: 10, height: 5 });

        match &mut screen.grid {
            Grid::AltScreen(alt) => {
                alt.buf[0]
                    .write_cell(10, 9, crate::cell::Cell::new('a', Attrs::default()))
                    .unwrap();
                assert_eq!(alt.buf[0].cells.len(), 10);
            }
            _ => panic!("wrong grid type"),
        }

        screen.resize(Size { width: 5, height: 5 });
        assert_eq!(screen.size.width, 5);

        // Line should be truncated
        match &screen.grid {
            Grid::AltScreen(alt) => {
                assert_eq!(alt.buf[0].cells.len(), 5);
            }
            _ => panic!("wrong grid type"),
        }
    }

    #[test]
    fn altscreen_cursor_clamping() {
        let mut screen = Screen::alt(Size { width: 10, height: 10 });
        screen.cursor = Pos { row: 9, col: 9 };
        screen.saved_cursor.pos = Pos { row: 8, col: 8 };

        screen.resize(Size { width: 5, height: 5 });

        assert_eq!(screen.cursor.row, 4);
        assert_eq!(screen.cursor.col, 4);
        assert_eq!(screen.saved_cursor.pos.row, 4);
        assert_eq!(screen.saved_cursor.pos.col, 4);
    }

    fn get_screen_cell(screen: &Screen, row: usize, col: usize) -> Option<Cell> {
        match &screen.grid {
            Grid::Scrollback(sb) => sb
                .get_line(screen.size, row)
                .and_then(|l| l.get_cell(screen.size.width, col))
                .cloned(),
            _ => None,
        }
    }

    #[test]
    fn scrollback_grid_new() {
        let size = Size { width: 10, height: 5 };
        let screen = Screen::scrollback(5, size);
        assert_eq!(screen.size, size);
        match &screen.grid {
            Grid::Scrollback(sb) => assert!(sb.buf.is_empty()),
            _ => panic!("wrong grid type"),
        }
    }

    #[test]
    fn scrollback_push_simple() -> anyhow::Result<()> {
        let size = Size { width: 5, height: 2 };
        let mut screen = Screen::scrollback(5, size);
        let c = Cell::new('x', term::Attrs::default());

        screen.write_at_cursor(c.clone(), false, true)?;

        let pos = Pos { row: 0, col: 0 };
        assert_eq!(
            get_screen_cell(&screen, pos.row, pos.col),
            Some(c),
            "Scrollback:\n{:?}",
            screen.grid
        );

        Ok(())
    }

    #[test]
    fn scrollback_push_wrapping() -> anyhow::Result<()> {
        let size = Size { width: 2, height: 5 };
        let mut screen = Screen::scrollback(5, size);

        // Fill first line
        screen.write_at_cursor(Cell::new('1', term::Attrs::default()), false, true)?;
        screen.write_at_cursor(Cell::new('2', term::Attrs::default()), false, true)?;

        // This should wrap to next line
        screen.write_at_cursor(Cell::new('3', term::Attrs::default()), false, true)?;

        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('1', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid,
        );
        assert_eq!(
            get_screen_cell(&screen, 0, 1),
            Some(Cell::new('2', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid,
        );
        assert_eq!(
            get_screen_cell(&screen, 1, 0),
            Some(Cell::new('3', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid,
        );

        Ok(())
    }

    #[test]
    fn scrollback_indexing() -> anyhow::Result<()> {
        let size = Size { width: 10, height: 3 };
        let mut screen = Screen::scrollback(3, size);

        // Populate an initial line that will get pushed off
        for _ in 0..10 {
            screen.write_at_cursor(Cell::new('X', term::Attrs::default()), false, true)?;
        }

        let c_top = Cell::new('T', term::Attrs::default());
        let c_mid = Cell::new('M', term::Attrs::default());
        let c_bot = Cell::new('B', term::Attrs::default());

        for _ in 0..10 {
            screen.write_at_cursor(c_top.clone(), false, true)?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_mid.clone(), false, true)?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_bot.clone(), false, true)?;
        }

        for r in 0..3 {
            for c in 0..10 {
                let expected = match r {
                    0 => &c_top,
                    1 => &c_mid,
                    2 => &c_bot,
                    _ => unreachable!(),
                };
                assert_eq!(get_screen_cell(&screen, r, c), Some(expected.clone()));
            }
        }

        Ok(())
    }

    #[test]
    fn scrollback_resize_narrower() -> anyhow::Result<()> {
        let size = Size { width: 10, height: 5 };
        let mut screen = Screen::scrollback(20, size);

        // Create a line: "0123456789"
        for i in 0..10 {
            screen.write_at_cursor(
                Cell::new(char::from_digit(i, 10).unwrap(), term::Attrs::default()),
                false,
                true,
            )?;
        }

        // Resize to width 5. Should split into "01234" and "56789"
        let new_size = Size { width: 5, height: 5 };
        screen.resize(new_size);

        // "56789" should be at row 1 (since it wrapped)
        // "01234" should be at row 0

        // Row 1, col 0 -> '5'
        assert_eq!(
            get_screen_cell(&screen, 1, 0),
            Some(Cell::new('5', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid
        );
        // Row 0, col 0 -> '0'
        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('0', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid
        );

        Ok(())
    }

    #[test]
    fn scrollback_resize_wider() -> anyhow::Result<()> {
        let size = Size { width: 5, height: 5 };
        let mut screen = Screen::scrollback(30, size);

        // Create two wrapped lines: "01234" (wrapped) -> "56789"
        for i in 0..10 {
            screen.write_at_cursor(
                Cell::new(char::from_digit(i, 10).unwrap(), term::Attrs::default()),
                false,
                true,
            )?;
        }

        // Verify initial state
        assert_eq!(
            get_screen_cell(&screen, 1, 0),
            Some(Cell::new('5', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid
        );

        // Resize to width 10. Should merge back to "0123456789"
        let new_size = Size { width: 10, height: 5 };
        screen.resize(new_size);

        // Should all be on top line (Row 0)
        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('0', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid
        );
        assert_eq!(
            get_screen_cell(&screen, 0, 9),
            Some(Cell::new('9', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grid
        );

        Ok(())
    }

    #[test]
    fn scrollback_reflow_roundtrip() -> anyhow::Result<()> {
        // Parameterized-style test
        let shapes = vec![
            (20, 10), // Start wide, go narrow
            (10, 5),
            (10, 20), // Start narrow, go wide
            (5, 10),
            (10, 10), // No change
        ];

        for (start_w, end_w) in shapes {
            let start_size = Size { width: start_w, height: 10 };
            let mut screen = Screen::scrollback(100, start_size);

            // Fill with deterministic data
            let count = 30;
            for i in 0..count {
                screen.write_at_cursor(
                    Cell::new(char::from_u32(65 + i % 26).unwrap(), term::Attrs::default()),
                    false,
                    true,
                )?;
            }

            // Resize
            screen.resize(Size { width: end_w, height: 10 });

            // Resize back
            screen.resize(start_size);

            // Verify content is identical to if we just pushed it
            let mut expected_screen = Screen::scrollback(100, start_size);
            for i in 0..count {
                expected_screen.write_at_cursor(
                    Cell::new(char::from_u32(65 + i % 26).unwrap(), term::Attrs::default()),
                    false,
                    true,
                )?;
            }

            match (&screen.grid, &expected_screen.grid) {
                (Grid::Scrollback(actual), Grid::Scrollback(expected)) => {
                    assert_eq!(
                        actual, expected,
                        "Scrollback state mismatch after roundtrip resize {} -> {} -> {}",
                        start_w, end_w, start_w
                    );
                }
                _ => panic!("wrong grid type"),
            }
        }

        Ok(())
    }
}
