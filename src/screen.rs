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

//! The screen module defines the state of the terminal screen: the main
//! screen and the alt screen, along with the cursor and the rest of the
//! state that a real terminal only has one of and shares between them.

use crate::{
    altscreen::AltScreen,
    cell::Cell,
    charset::Charsets,
    line::Line,
    log,
    scrollback::Scrollback,
    term::{self, AsTermInput, OriginMode, Pos, Region, ScrollRegion},
    Modes,
};

use anyhow::{anyhow, Context};

/// The main screen and the alt screen, along with the state they share.
///
/// Switching screens leaves the cursor, the scroll region and origin mode as
/// they were, since a real terminal only has one of each.
#[derive(Debug)]
pub struct Screen {
    // The storage for the lines of both screens.
    grids: Grids,
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
    /// The region of the screen in which scrolling happens.
    /// This is set by DECSTBM (CSI n ; n r).
    scroll_region: ScrollRegion,
    origin_mode: OriginMode,
    logger: log::Context,
}

impl Screen {
    /// Create a new screen, with the main screen active.
    pub fn new(mut scrollback_lines: usize, size: crate::Size) -> Self {
        if scrollback_lines < size.height {
            scrollback_lines = size.height;
        }

        Screen {
            grids: Grids {
                scrollback: Scrollback::new(scrollback_lines),
                scrollback_saved_cursor: None,
                altscreen: AltScreen::new(size),
                altscreen_saved_cursor: None,
                alt_active: false,
            },
            size,
            cursor: Pos { row: 0, col: 0 },
            pending_wrap: false,
            scroll_region: ScrollRegion::default(),
            origin_mode: OriginMode::default(),
            logger: log::Context::None,
        }
    }

    pub fn set_logger(&mut self, logger: log::Context) {
        self.grids.scrollback.set_logger(logger.clone());
        self.grids.altscreen.set_logger(logger.clone());
        self.logger = logger;
    }

    /// Return the number of lines of scrollback the main screen stores.
    pub fn scrollback_lines(&self) -> usize {
        self.grids.scrollback.scrollback_lines()
    }

    /// Set the number of lines of scrollback the main screen stores.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        self.grids.scrollback.set_scrollback_lines(self.size, scrollback_lines);
    }

    /// Whether the alt screen is the active one.
    pub fn alt_active(&self) -> bool {
        self.grids.alt_active
    }

    /// Switch to the alt screen.
    pub fn enter_alt_screen(&mut self) {
        self.grids.alt_active = true;
    }

    /// Switch back to the main screen.
    pub fn exit_alt_screen(&mut self) {
        self.grids.alt_active = false;
    }

    /// The slot where the SCP/RCP and ESC 7 / ESC 8 commands save the
    /// cursor. Like in xterm, the main screen and the alt screen each have
    /// their own, and this is the one of the active screen. It stays empty
    /// until something gets saved.
    pub fn saved_cursor(&self) -> Option<&SavedCursor> {
        self.grids.saved_cursor().as_ref()
    }

    /// `saved_cursor`, for changing it.
    pub fn saved_cursor_mut(&mut self) -> &mut Option<SavedCursor> {
        self.grids.saved_cursor_mut()
    }

    pub fn set_scroll_region(&mut self, scroll_region: ScrollRegion) {
        self.scroll_region = clamp_scroll_region(scroll_region, self.size);
    }

    pub fn origin_mode(&self) -> OriginMode {
        self.origin_mode
    }

    pub fn set_origin_mode(&mut self, origin_mode: OriginMode) {
        self.origin_mode = origin_mode;
    }

    /// Given a 1-indexed position as the user would directly provide in
    /// a CUP command, update the cursor position, taking the current origin
    /// mode and scroll region into account.
    pub fn set_cursor(&mut self, pos: Pos) {
        match self.origin_mode {
            OriginMode::Term => {
                self.cursor.row = pos.row.saturating_sub(1);
                self.cursor.col = pos.col.saturating_sub(1);
            }
            OriginMode::ScrollRegion => match self.scroll_region {
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

    /// Emit the codes that paint the screens and put the cursor back.
    ///
    /// The saved cursors get put back too, since an app that saved the
    /// cursor before a reattach can restore it after.
    pub fn dump_contents_into(&self, buf: &mut Vec<u8>, dump_region: crate::ContentRegion) {
        // Restore the main screen first so that after the user exits their
        // curses app, they can still see shell history.
        self.grids.scrollback.dump_contents_into(buf, self.size, dump_region);
        if self.grids.alt_active {
            self.dump_switch_into(buf);
            self.grids.altscreen.dump_contents_into(buf, self.size.width);
        }

        // `Term::contents` leaves the terminal with the cursor saved at home
        // and everything else at its default, which restores just like an
        // empty slot, so there is only something to do if the app saved a
        // different one.
        match self.saved_cursor() {
            Some(saved) if *saved != SavedCursor::new(Pos { row: 0, col: 0 }) => {
                self.dump_saved_cursor_into(buf, self.grids.active(), saved);
                term::control_codes().save_cursor.term_input_into(buf);
                saved.dump_reset_into(buf);
            }
            _ => {}
        }

        // The scroll region comes after the saved cursor, which might be
        // outside of it.
        self.scroll_region.term_input_into(buf);
        let (top, _) = self.scroll_region.as_region(&self.size).row_bounds();
        let grid = self.grids.active();
        self.dump_cursor_into(buf, grid, self.origin_mode, top, self.cursor, self.pending_wrap);
    }

    /// Emit the codes that switch from the main screen to the alt screen,
    /// leaving the cursor in the top left corner to paint it from.
    fn dump_switch_into(&self, buf: &mut Vec<u8>) {
        // Like DECSC, the switch saves the cursor along with its attrs,
        // charsets and origin mode, which leaving the alt screen puts back,
        // so all of that has to be the way the main screen has it saved.
        let saved = SavedCursor::or_home(self.grids.scrollback_saved_cursor.as_ref());
        self.dump_saved_cursor_into(buf, Grid::Scrollback(&self.grids.scrollback), &saved);

        // Re-enable alt screen, then dump the contents. This is
        // not actually super important in practice because basically
        // every curses app respects SIGWINCH. We may even want to
        // consider exposing a knob to disable alt-screen dumping
        // since it might make things less flickery. Not worth doing
        // for now though.
        term::control_codes().enable_alt_screen.term_input_into(buf);

        let homed = saved.dump_reset_into(buf);
        // The switch erased the alt screen with the saved background color,
        // and painting the alt screen only covers the cells it has stored.
        if !matches!(saved.dump_attrs().bgcolor, term::Color::Default) {
            term::control_codes().erase_screen.term_input_into(buf);
        }
        // Switching screens doesn't move the cursor, but the alt screen gets
        // painted from the top left corner down.
        if !homed && (saved.pos != Pos { row: 0, col: 0 } || saved.pending_wrap) {
            term::ControlCodes::cursor_position(1, 1).term_input_into(buf);
        }
    }

    /// Put the terminal in the state that `saved` describes, so that saving
    /// the cursor there saves the same thing the app saved. `grid` is the
    /// grid of the screen that `saved` belongs to.
    ///
    /// The terminal must not have a scroll region in place, since a cursor
    /// saved in origin mode could be outside of it.
    fn dump_saved_cursor_into(&self, buf: &mut Vec<u8>, grid: Grid<'_>, saved: &SavedCursor) {
        self.dump_cursor_into(buf, grid, saved.origin_mode, 0, saved.pos, saved.pending_wrap);
        for code in term::Attrs::default().transition_to(&saved.dump_attrs()) {
            code.term_input_into(buf);
        }
        // Restoring a pending wrap prints a char that has already been
        // through the charsets, so they have to come last.
        saved.charsets.dump_into(buf);
    }

    /// Put the cursor at `pos` on `grid` in the given origin mode, with a
    /// wrap pending if `pending_wrap` is set. `top` is the top row of the
    /// scroll region that the terminal has in place.
    fn dump_cursor_into(
        &self,
        buf: &mut Vec<u8>,
        grid: Grid<'_>,
        origin_mode: OriginMode,
        top: usize,
        pos: Pos,
        pending_wrap: bool,
    ) {
        // Origin mode has to be restored before the cursor, since enabling
        // it homes the cursor.
        let mut row = pos.row;
        if matches!(origin_mode, OriginMode::ScrollRegion) {
            term::control_codes().enable_scroll_region_origin_mode.term_input_into(buf);
            // Rows are relative to the top of the scroll region once origin
            // mode is on, mirroring `set_cursor`.
            row = row.saturating_sub(top);
        }

        if pending_wrap {
            self.dump_pending_wrap_into(buf, grid, pos.row, row);
        } else {
            term::ControlCodes::cursor_position((row + 1) as u16, (pos.col + 1) as u16)
                .term_input_into(buf);
        }
    }

    /// Leave the cursor on the last column of row `line_row` of `grid` with a
    /// wrap pending.
    ///
    /// There is no control code that sets the pending wrap flag, so we get
    /// the terminal into that state the same way the application did, by
    /// printing the char that sits at the end of the line. It is already on
    /// the screen, so printing it again over itself changes nothing else.
    /// `row` is the row the terminal addresses `line_row` as.
    fn dump_pending_wrap_into(
        &self,
        buf: &mut Vec<u8>,
        grid: Grid<'_>,
        line_row: usize,
        row: usize,
    ) {
        let width = self.size.width;
        let line = grid.get_line(self.size, line_row);
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
                cell.into_owned()
            }
            // The last column is not covered by anything we can print, which
            // happens when a wide char has been partly overwritten. A blank
            // looks the same.
            _ => {
                col = width.saturating_sub(1);
                Cell::empty()
            }
        };
        // Neither do the attrs of a blank that don't show up on a blank.
        let cell = if cell.looks_unused() { Cell::empty() } else { cell };

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

    pub fn resize(&mut self, new_size: crate::Size) {
        // shpool resizes the terminal on every reattach, usually to the size
        // it already is. The kernel only sends SIGWINCH when the size changes,
        // so the app won't redraw, and nothing about the screen, like the
        // scroll region, should change either.
        if new_size == self.size {
            return;
        }
        let old_size = self.size;
        // A cursor with a wrap pending is logically just past the char in
        // the last column, and that is the spot it should keep following.
        let cursor = logical_unwrapped_pos(self.cursor, self.pending_wrap);
        // An empty slot has nothing to follow, so the top left corner, which
        // gets tracked below anyway, stands in for it.
        let saved = SavedCursor::or_home(self.grids.scrollback_saved_cursor.as_ref());
        let saved_cursor = logical_unwrapped_pos(saved.pos, saved.pending_wrap);

        // A row of the main screen is derived from the buffer length and the
        // height, both of which this changes, so the cursor has to be
        // re-derived rather than just clamped. The saved cursor is a row too,
        // and a resize between DECSC and DECRC moves it just the same, even
        // while the alt screen is up.
        let scrollback = &mut self.grids.scrollback;
        let mut anchors = [
            scrollback.anchor_cursor(old_size, cursor, self.pending_wrap),
            scrollback.anchor_cursor(old_size, saved_cursor, saved.pending_wrap),
            // Where the top of the screen ends up says how many rows the old
            // screen contents take up after the resize.
            scrollback.anchor_cursor(old_size, Pos { row: 0, col: 0 }, false),
        ];
        // While the alt screen is up, the cursor is on it instead, so the
        // anchors on the main screen start after it.
        let first = if self.grids.alt_active { 1 } else { 0 };
        // Only the width decides where lines wrap.
        if new_size.width != old_size.width {
            scrollback.reflow(new_size.width, &mut anchors[first..]);
        }
        let [cursor_anchor, saved_anchor, top_anchor] = anchors;
        let mut anchors = [cursor_anchor, saved_anchor];
        scrollback.trim_blank_rows(new_size, top_anchor, &mut anchors[first..]);
        let [cursor_anchor, saved_anchor] = anchors;
        let saved_cursor =
            (scrollback.resolve_cursor(new_size, saved_anchor), saved_anchor.pending_wrap);
        let (cursor, pending_wrap) = if self.grids.alt_active {
            (cursor, self.pending_wrap)
        } else {
            (scrollback.resolve_cursor(new_size, cursor_anchor), cursor_anchor.pending_wrap)
        };
        self.grids.altscreen.resize(new_size);
        self.size = new_size;

        // Like in other terminals, a resize drops the scroll region. The rows
        // it covered have moved or might not even be there anymore, and apps
        // set up a new one when they redraw for the new size.
        self.scroll_region = ScrollRegion::TrackSize;

        (self.cursor, self.pending_wrap) = settle_cursor(cursor, pending_wrap, self.size);
        if let Some(saved) = &mut self.grids.scrollback_saved_cursor {
            (saved.pos, saved.pending_wrap) =
                settle_cursor(saved_cursor.0, saved_cursor.1, self.size);
        }
        if let Some(saved) = &mut self.grids.altscreen_saved_cursor {
            let pos = logical_unwrapped_pos(saved.pos, saved.pending_wrap);
            (saved.pos, saved.pending_wrap) = settle_cursor(pos, saved.pending_wrap, self.size);
        }
    }

    /// Move the cursor up `n` rows. It stops at the top of the scroll region,
    /// unless it started out above it. This implements CUU.
    pub fn cursor_up(&mut self, n: usize) {
        let (top, _) = self.scroll_region.as_region(&self.size).row_bounds();
        let min = if self.cursor.row < top { 0 } else { top };
        self.cursor.row = std::cmp::max(self.cursor.row.saturating_sub(n), min);
        self.clamp();
    }

    /// Move the cursor down `n` rows. It stops at the bottom of the scroll
    /// region, unless it started out below it. This implements CUD.
    pub fn cursor_down(&mut self, n: usize) {
        let (_, bottom) = self.scroll_region.as_region(&self.size).row_bounds();
        let end = if self.cursor.row < bottom { bottom } else { self.size.height };
        self.cursor.row = std::cmp::min(self.cursor.row.saturating_add(n), end.saturating_sub(1));
        self.clamp();
    }

    /// Bring the cursor back within the region it may occupy after it has
    /// been explicitly moved.
    ///
    /// Every explicit cursor movement ends up here, and moving the cursor
    /// cancels any pending wrap, so this clears it as well.
    pub fn clamp(&mut self) {
        self.pending_wrap = false;
        match self.origin_mode {
            OriginMode::Term => self.cursor.clamp_to(self.size),
            OriginMode::ScrollRegion => {
                self.cursor.clamp_to(self.scroll_region.as_region(&self.size))
            }
        }
    }

    //
    // Control Code Handlers
    //

    /// The modes that `write_at_cursor` looks at. Callers have to mask out
    /// all the others.
    pub const WRITE_MODES: Modes = Modes::INSERT.union(Modes::AUTOWRAP_DISABLED);

    /// Write a cell at the cursor and advance the cursor past it, wrapping
    /// onto the next line first if a wrap is pending or the cell does not
    /// fit in what is left of the current line.
    ///
    /// `modes` may only contain `WRITE_MODES`. In insert mode (IRM) the rest
    /// of the line gets shifted right to make room for the cell rather than
    /// being overwritten. With autowrap (DECAWM) off the cursor never leaves
    /// the line, and chars that run into the right edge overwrite the last
    /// column instead.
    pub fn write_at_cursor(&mut self, cell: Cell, modes: Modes) -> anyhow::Result<()> {
        debug_assert!(
            Self::WRITE_MODES.contains(modes),
            "write_at_cursor only takes {:?}, got {:?}",
            Self::WRITE_MODES,
            modes
        );
        let insert_mode = modes.contains(Modes::INSERT);
        let autowrap = !modes.contains(Modes::AUTOWRAP_DISABLED);
        let width = self.size.width;
        if width == 0 || self.size.height == 0 {
            return Err(anyhow!("cannot write to a zero sized screen"));
        }
        let cell_width = cell.width() as usize;
        if cell_width > width {
            return Err(anyhow!("{} column wide char does not fit on the screen", cell_width));
        }
        self.cursor.clamp_to(self.size);

        // The background color that a scroll paints the new line with if the
        // cell has to wrap onto it at the bottom of the scroll region.
        let fill = cell.attrs().bgcolor;
        if self.pending_wrap {
            if autowrap {
                self.wrap(fill);
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
                self.wrap(fill);
            } else {
                self.cursor.col = width - cell_width;
            }
        }

        let col = self.cursor.col;
        let Some(line) = self.grids.active_mut().ensure_line(self.size, self.cursor.row) else {
            return Err(anyhow!("no line for cursor row {}", self.cursor.row));
        };
        if insert_mode {
            // The blanks this opens up get written over right away.
            line.insert_character(width, col, cell_width, term::Color::Default);
        }
        line.set_cell(width, col, cell).context("setting cell")?;

        if col + cell_width < width {
            self.cursor.col = col + cell_width;
        } else {
            self.cursor.col = width - 1;
            self.pending_wrap = autowrap;
        }

        Ok(())
    }

    /// Move to the start of the next line because the current one is full.
    fn wrap(&mut self, fill: term::Color) {
        // The line only continues onto the next one if the cursor is really
        // going to get there. Below the scroll region on the last row it has
        // nowhere to go, so the next char just overwrites this line.
        let (_, bottom) = self.scroll_region.as_region(&self.size).row_bounds();
        if self.cursor.row + 1 == bottom || self.cursor.row + 1 < self.size.height {
            if let Some(line) = self.grids.active_mut().ensure_line(self.size, self.cursor.row) {
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
    /// `fill` is the background color that scrolling paints the new row with
    /// (see `Cell::blank`).
    pub fn linefeed(&mut self, fill: term::Color) {
        self.pending_wrap = false;
        let (_, bottom) = self.scroll_region.as_region(&self.size).row_bounds();
        if self.cursor.row + 1 == bottom {
            self.scroll_up(1, fill);
        } else if self.cursor.row + 1 < self.size.height {
            self.cursor.row += 1;
        }
    }

    /// Erase whichever screen is currently active from the cursor
    /// position to the bottom, leaving blanks painted with `fill` behind. Used
    /// to implement 'CSI 0 J'
    pub fn erase_to_end(&mut self, fill: term::Color) {
        self.pending_wrap = false;
        let width = self.size.width;
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.erase_to_end(self.size, self.cursor, fill),
            GridMut::AltScreen(alt) => alt.erase_to_end(width, self.cursor, fill),
        }
    }

    /// Erase whichever screen is currently active from the top to the
    /// cursor position, leaving blanks painted with `fill` behind. Used to
    /// implement 'CSI 1 J'
    pub fn erase_from_start(&mut self, fill: term::Color) {
        self.pending_wrap = false;
        let width = self.size.width;
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.erase_from_start(self.size, self.cursor, fill),
            GridMut::AltScreen(alt) => alt.erase_from_start(width, self.cursor, fill),
        }
    }

    /// Erase whichever screen is currently active, not including scrollback,
    /// leaving blanks painted with `fill` behind. Used to implement 'CSI 2 J'.
    pub fn erase(&mut self, fill: term::Color) {
        self.pending_wrap = false;
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.erase(self.size, fill),
            GridMut::AltScreen(alt) => alt.erase(fill),
        }
    }

    /// Drop the scrollback of the main screen, leaving what is on the screen
    /// alone, even while the alt screen is up. Used to implement 'CSI 3 J'.
    pub fn erase_scrollback(&mut self) {
        self.grids.scrollback.erase_scrollback(self.size);
    }

    /// Gets the current line. If the cursor is not currently over an actual
    /// line, this returns nothing.
    pub fn get_line_mut(&mut self) -> Option<&mut Line> {
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.get_line_mut(self.size, self.cursor.row),
            GridMut::AltScreen(alt) => alt.get_line_mut(self.cursor.row),
        }
    }

    /// Gets the current line for an edit that leaves blanks painted with `fill`
    /// behind.
    ///
    /// A row that nothing is stored for is blank already, so an edit that
    /// only leaves plain blanks behind has nothing to do there and this
    /// returns nothing, but blanks painted with a background color have to
    /// be stored.
    pub fn line_to_erase(&mut self, fill: term::Color) -> Option<&mut Line> {
        if fill != term::Color::Default {
            self.grids.active_mut().ensure_line(self.size, self.cursor.row)
        } else {
            self.get_line_mut()
        }
    }

    /// SU (CSI S). Move the content of the scroll region up by `n` rows,
    /// opening rows of `fill` at the bottom. The cursor does not move.
    pub fn scroll_up(&mut self, n: usize, fill: term::Color) {
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.scroll_up(&self.size, &self.scroll_region, n, fill),
            GridMut::AltScreen(alt) => alt.scroll_up(&self.scroll_region, n, fill),
        }
    }

    /// SD (CSI T). Move the content of the scroll region down by `n` rows,
    /// opening rows of `fill` at the top. The cursor does not move.
    pub fn scroll_down(&mut self, n: usize, fill: term::Color) {
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.scroll_down(&self.size, &self.scroll_region, n, fill),
            GridMut::AltScreen(alt) => alt.scroll_down(&self.scroll_region, n, fill),
        }
    }

    /// Whether the cursor is on one of the rows of the scroll region.
    fn cursor_in_scroll_region(&self) -> bool {
        let (top, bottom) = self.scroll_region.as_region(&self.size).row_bounds();
        top <= self.cursor.row && self.cursor.row < bottom
    }

    pub fn scroll_region(&self, by_origin_mode: bool) -> ScrollRegion {
        if by_origin_mode {
            match self.origin_mode {
                OriginMode::Term => ScrollRegion::TrackSize,
                OriginMode::ScrollRegion => self.scroll_region.clone(),
            }
        } else {
            self.scroll_region.clone()
        }
    }

    /// Handler for the Insert Line command (CSI n L).
    ///
    /// n lines of `fill` are inserted above the current line, dropping any
    /// lines that get pushed out of the current scroll region. The cursor
    /// goes back to the start of the line. Outside of the scroll region,
    /// nothing happens at all.
    pub fn insert_lines(&mut self, n: usize, fill: term::Color) {
        if !self.cursor_in_scroll_region() {
            return;
        }
        self.pending_wrap = false;
        self.cursor.col = 0;
        let region = &self.scroll_region;
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.insert_lines(&self.cursor, &self.size, region, n, fill),
            GridMut::AltScreen(alt) => alt.insert_lines(&self.cursor, region, n, fill),
        }
    }

    /// Handler for the Delete Line command (CSI n M).
    ///
    /// n lines below the current line are deleted (including the current line),
    /// sucking any lines below the current line up. New lines of `fill` are
    /// inserted at the bottom of the scroll region. Like with IL, the cursor
    /// goes back to the start of the line, and nothing happens outside of the
    /// scroll region.
    pub fn delete_lines(&mut self, n: usize, fill: term::Color) {
        if !self.cursor_in_scroll_region() {
            return;
        }
        self.pending_wrap = false;
        self.cursor.col = 0;
        let region = &self.scroll_region;
        match self.grids.active_mut() {
            GridMut::Scrollback(s) => s.delete_lines(&self.cursor, &self.size, region, n, fill),
            GridMut::AltScreen(alt) => alt.delete_lines(&self.cursor, region, n, fill),
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
fn logical_unwrapped_pos(pos: Pos, pending_wrap: bool) -> Pos {
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

        match self.grids.active() {
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
///
/// Restoring the cursor when nothing has been saved restores `new` at the
/// top left corner, which homes the cursor and puts the attrs, charsets and
/// origin mode back to their defaults, like in xterm and kitty.
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
    /// Whether origin mode (DECOM) was on. DECRC turns it back on or off,
    /// and a position saved in origin mode stays inside the scroll region.
    pub origin_mode: OriginMode,
}

impl SavedCursor {
    pub fn new(pos: Pos) -> Self {
        SavedCursor {
            pos,
            attrs: term::Attrs::default(),
            pending_wrap: false,
            charsets: Charsets::default(),
            origin_mode: OriginMode::Term,
        }
    }

    /// What restoring the cursor from `slot` restores.
    pub fn or_home(slot: Option<&SavedCursor>) -> SavedCursor {
        slot.cloned().unwrap_or_else(|| SavedCursor::new(Pos { row: 0, col: 0 }))
    }

    /// The attrs to restore the saved cursor with. Like with the live
    /// cursor attrs, a link gets left out, since the terminal we restore
    /// into has no idea that it is in the middle of one.
    fn dump_attrs(&self) -> term::Attrs {
        term::Attrs { link_target: None, ..self.attrs.clone() }
    }

    /// Undo the attrs, charsets and origin mode that restoring this saved
    /// cursor into a terminal set. Returns whether this homed the cursor.
    fn dump_reset_into(&self, buf: &mut Vec<u8>) -> bool {
        for code in self.dump_attrs().transition_to(&term::Attrs::default()) {
            code.term_input_into(buf);
        }
        self.charsets.dump_reset_into(buf);
        if matches!(self.origin_mode, OriginMode::ScrollRegion) {
            term::control_codes().disable_scroll_region_origin_mode.term_input_into(buf);
            return true;
        }
        false
    }
}

/// The grids of the main screen and the alt screen, and which one is active.
///
/// Each screen also has its own slot for DECSC to save the cursor into, the
/// way xterm does it.
#[derive(Debug)]
struct Grids {
    scrollback: Scrollback,
    scrollback_saved_cursor: Option<SavedCursor>,
    altscreen: AltScreen,
    altscreen_saved_cursor: Option<SavedCursor>,
    alt_active: bool,
}

impl Grids {
    /// The grid of the active screen.
    fn active(&self) -> Grid<'_> {
        if self.alt_active {
            Grid::AltScreen(&self.altscreen)
        } else {
            Grid::Scrollback(&self.scrollback)
        }
    }

    /// `active`, for changing it.
    fn active_mut(&mut self) -> GridMut<'_> {
        if self.alt_active {
            GridMut::AltScreen(&mut self.altscreen)
        } else {
            GridMut::Scrollback(&mut self.scrollback)
        }
    }

    /// The saved cursor slot of the active screen.
    fn saved_cursor(&self) -> &Option<SavedCursor> {
        if self.alt_active {
            &self.altscreen_saved_cursor
        } else {
            &self.scrollback_saved_cursor
        }
    }

    /// `saved_cursor`, for changing it.
    fn saved_cursor_mut(&mut self) -> &mut Option<SavedCursor> {
        if self.alt_active {
            &mut self.altscreen_saved_cursor
        } else {
            &mut self.scrollback_saved_cursor
        }
    }
}

/// The grid of one of the screens, for the logic that differs between the
/// two kinds.
#[derive(Clone, Copy)]
enum Grid<'a> {
    Scrollback(&'a Scrollback),
    AltScreen(&'a AltScreen),
}

impl<'a> Grid<'a> {
    /// The line at the given screen row, if there is one stored.
    fn get_line(self, size: crate::Size, row: usize) -> Option<&'a Line> {
        match self {
            Grid::Scrollback(s) => s.get_line(size, row),
            Grid::AltScreen(alt) => alt.buf.get(row),
        }
    }
}

/// `Grid`, for changing the grid.
enum GridMut<'a> {
    Scrollback(&'a mut Scrollback),
    AltScreen(&'a mut AltScreen),
}

impl<'a> GridMut<'a> {
    /// The line at the given screen row, ready to be written to. Returns
    /// None if the row is not on the screen.
    fn ensure_line(self, size: crate::Size, row: usize) -> Option<&'a mut Line> {
        match self {
            GridMut::Scrollback(s) => s.ensure_line(size, row),
            GridMut::AltScreen(alt) => alt.get_line_mut(row),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Attrs;
    use crate::Size;

    /// A screen of the given size with the alt screen active.
    fn alt_screen(size: Size) -> Screen {
        let mut screen = Screen::new(size.height, size);
        screen.enter_alt_screen();
        screen
    }

    #[test]
    fn altscreen_get_line_mut_at_end_of_buf() {
        let mut screen = alt_screen(Size { width: 5, height: 3 });
        screen.cursor = Pos { row: 3, col: 0 };

        assert!(screen.get_line_mut().is_none());
    }

    #[test]
    fn altscreen_get_line_mut_past_end_of_buf() {
        let mut screen = alt_screen(Size { width: 5, height: 3 });
        screen.cursor = Pos { row: 7, col: 0 };

        assert!(screen.get_line_mut().is_none());
    }

    #[test]
    fn altscreen_resize_grow_height() {
        let mut screen = alt_screen(Size { width: 10, height: 5 });
        screen.resize(Size { width: 10, height: 10 });

        assert_eq!(screen.grids.altscreen.buf.len(), 10);
        assert_eq!(screen.size.height, 10);
    }

    #[test]
    fn altscreen_resize_shrink_height() {
        let mut screen = alt_screen(Size { width: 10, height: 10 });
        screen.resize(Size { width: 10, height: 5 });
        assert_eq!(screen.grids.altscreen.buf.len(), 5);
        assert_eq!(screen.size.height, 5);
    }

    #[test]
    fn altscreen_resize_shrink_width() {
        let mut screen = alt_screen(Size { width: 10, height: 5 });

        let alt = &mut screen.grids.altscreen;
        alt.buf[0].set_cell(10, 9, crate::cell::Cell::new('a', Attrs::default())).unwrap();
        assert_eq!(alt.buf[0].cells.len(), 10);

        screen.resize(Size { width: 5, height: 5 });
        assert_eq!(screen.size.width, 5);

        // Line should be truncated
        assert_eq!(screen.grids.altscreen.buf[0].cells.len(), 5);
    }

    #[test]
    fn altscreen_cursor_clamping() {
        let mut screen = alt_screen(Size { width: 10, height: 10 });
        screen.cursor = Pos { row: 9, col: 9 };
        *screen.saved_cursor_mut() = Some(SavedCursor::new(Pos { row: 8, col: 8 }));

        screen.resize(Size { width: 5, height: 5 });

        assert_eq!(screen.cursor.row, 4);
        assert_eq!(screen.cursor.col, 4);
        let saved = screen.saved_cursor().expect("saved cursor");
        assert_eq!(saved.pos.row, 4);
        assert_eq!(saved.pos.col, 4);
    }

    fn get_screen_cell(screen: &Screen, row: usize, col: usize) -> Option<Cell> {
        screen
            .grids
            .scrollback
            .get_line(screen.size, row)
            .and_then(|l| l.get_cell(screen.size.width, col))
            .map(|cell| cell.into_owned())
    }

    #[test]
    fn scrollback_grid_new() {
        let size = Size { width: 10, height: 5 };
        let screen = Screen::new(5, size);
        assert_eq!(screen.size, size);
        assert!(screen.grids.scrollback.buf.is_empty());
    }

    #[test]
    fn scrollback_push_simple() -> anyhow::Result<()> {
        let size = Size { width: 5, height: 2 };
        let mut screen = Screen::new(5, size);
        let c = Cell::new('x', term::Attrs::default());

        screen.write_at_cursor(c.clone(), Modes::empty())?;

        let pos = Pos { row: 0, col: 0 };
        assert_eq!(
            get_screen_cell(&screen, pos.row, pos.col),
            Some(c),
            "Scrollback:\n{:?}",
            screen.grids.scrollback
        );

        Ok(())
    }

    #[test]
    fn scrollback_push_wrapping() -> anyhow::Result<()> {
        let size = Size { width: 2, height: 5 };
        let mut screen = Screen::new(5, size);

        // Fill first line
        screen.write_at_cursor(Cell::new('1', term::Attrs::default()), Modes::empty())?;
        screen.write_at_cursor(Cell::new('2', term::Attrs::default()), Modes::empty())?;

        // This should wrap to next line
        screen.write_at_cursor(Cell::new('3', term::Attrs::default()), Modes::empty())?;

        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('1', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback,
        );
        assert_eq!(
            get_screen_cell(&screen, 0, 1),
            Some(Cell::new('2', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback,
        );
        assert_eq!(
            get_screen_cell(&screen, 1, 0),
            Some(Cell::new('3', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback,
        );

        Ok(())
    }

    #[test]
    fn scrollback_indexing() -> anyhow::Result<()> {
        let size = Size { width: 10, height: 3 };
        let mut screen = Screen::new(3, size);

        // Populate an initial line that will get pushed off
        for _ in 0..10 {
            screen.write_at_cursor(Cell::new('X', term::Attrs::default()), Modes::empty())?;
        }

        let c_top = Cell::new('T', term::Attrs::default());
        let c_mid = Cell::new('M', term::Attrs::default());
        let c_bot = Cell::new('B', term::Attrs::default());

        for _ in 0..10 {
            screen.write_at_cursor(c_top.clone(), Modes::empty())?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_mid.clone(), Modes::empty())?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_bot.clone(), Modes::empty())?;
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
        let mut screen = Screen::new(20, size);

        // Create a line: "0123456789"
        for i in 0..10 {
            screen.write_at_cursor(
                Cell::new(char::from_digit(i, 10).unwrap(), term::Attrs::default()),
                Modes::empty(),
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
            screen.grids.scrollback
        );
        // Row 0, col 0 -> '0'
        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('0', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback
        );

        Ok(())
    }

    #[test]
    fn scrollback_resize_wider() -> anyhow::Result<()> {
        let size = Size { width: 5, height: 5 };
        let mut screen = Screen::new(30, size);

        // Create two wrapped lines: "01234" (wrapped) -> "56789"
        for i in 0..10 {
            screen.write_at_cursor(
                Cell::new(char::from_digit(i, 10).unwrap(), term::Attrs::default()),
                Modes::empty(),
            )?;
        }

        // Verify initial state
        assert_eq!(
            get_screen_cell(&screen, 1, 0),
            Some(Cell::new('5', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback
        );

        // Resize to width 10. Should merge back to "0123456789"
        let new_size = Size { width: 10, height: 5 };
        screen.resize(new_size);

        // Should all be on top line (Row 0)
        assert_eq!(
            get_screen_cell(&screen, 0, 0),
            Some(Cell::new('0', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback
        );
        assert_eq!(
            get_screen_cell(&screen, 0, 9),
            Some(Cell::new('9', term::Attrs::default())),
            "Scrollback:\n{:?}",
            screen.grids.scrollback
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
            let mut screen = Screen::new(100, start_size);

            // Fill with deterministic data
            let count = 30;
            for i in 0..count {
                screen.write_at_cursor(
                    Cell::new(char::from_u32(65 + i % 26).unwrap(), term::Attrs::default()),
                    Modes::empty(),
                )?;
            }

            // Resize
            screen.resize(Size { width: end_w, height: 10 });

            // Resize back
            screen.resize(start_size);

            // Verify content is identical to if we just pushed it
            let mut expected_screen = Screen::new(100, start_size);
            for i in 0..count {
                expected_screen.write_at_cursor(
                    Cell::new(char::from_u32(65 + i % 26).unwrap(), term::Attrs::default()),
                    Modes::empty(),
                )?;
            }

            assert_eq!(
                screen.grids.scrollback, expected_screen.grids.scrollback,
                "Scrollback state mismatch after roundtrip resize {} -> {} -> {}",
                start_w, end_w, start_w
            );
        }

        Ok(())
    }

    // Full screen programs paint their background by clearing the screen
    // with a background color set, and scroll more painted rows in as they
    // go. None of that takes storing any cells.
    #[test]
    fn painting_the_screen_stores_no_cells() {
        let fill = term::Color::Idx(4);

        let mut screen = Screen::new(100, Size { width: 80, height: 24 });
        screen.erase(fill);
        screen.scroll_up(10, fill);
        screen.insert_lines(3, fill);
        screen.erase_to_end(fill);

        let sb = &screen.grids.scrollback;
        assert!(!sb.buf.is_empty());
        for line in sb.buf.iter() {
            assert!(line.cells.is_empty());
            assert_eq!(line.blank_bgcolor, fill);
        }
    }

    // Rows painted with a background color are not blank, even though they
    // store no cells, so a shorter screen keeps them.
    #[test]
    fn painted_rows_survive_the_screen_getting_shorter() {
        let fill = term::Color::Idx(4);

        let mut screen = Screen::new(100, Size { width: 5, height: 4 });
        screen.erase(fill);
        screen.resize(Size { width: 5, height: 2 });

        assert_eq!(screen.grids.scrollback.buf.len(), 4);
    }
}
