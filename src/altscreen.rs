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

//! The altscreen module defines the representation of the alt screen.

use std::collections::VecDeque;

use crate::{
    line::{self, Line},
    log,
    term::{self, AsTermInput, OriginMode, Pos, ScrollRegion},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct AltScreen {
    /// The entire grid the altscreen. size.height should always equal
    /// buf.len().
    ///
    /// buf[0] is at the top of the screen and buf[buf.len()-1] is at the
    /// bottom.
    pub buf: VecDeque<Line>,
    /// The region of the screen in which scrolling happens.
    /// This is set by DECSTBM (CSI n ; n r).
    pub scroll_region: ScrollRegion,
    pub origin_mode: OriginMode,
    logger: log::Context,
}

impl AltScreen {
    pub fn new(size: crate::Size) -> Self {
        let mut buf = VecDeque::new();
        for _ in 0..size.height {
            buf.push_back(Line::new());
        }
        AltScreen {
            buf,
            scroll_region: ScrollRegion::default(),
            origin_mode: OriginMode::default(),
            logger: log::Context::None,
        }
    }

    pub fn set_logger(&mut self, logger: log::Context) {
        self.logger = logger;
    }

    /// The half open range of rows that scrolling operates on.
    ///
    /// The scroll region is set by the application and is not required to
    /// describe rows that exist, so it gets clipped to the grid here. An
    /// empty range means there is nothing to scroll.
    fn scroll_region_rows(&self) -> (usize, usize) {
        match self.scroll_region {
            ScrollRegion::TrackSize => (0, self.buf.len()),
            ScrollRegion::Window { top, bottom } => {
                (std::cmp::min(top, self.buf.len()), std::cmp::min(bottom, self.buf.len()))
            }
        }
    }

    /// SU (CSI S). Move the content of the scroll region up by `rows`,
    /// opening blank rows at the bottom. Also what a linefeed at the bottom
    /// of the scroll region does.
    pub fn scroll_up(&mut self, rows: usize) {
        if let ScrollRegion::TrackSize = self.scroll_region {
            for _ in 0..rows {
                self.buf.pop_front();
                self.buf.push_back(Line::new());
            }
            return;
        }

        let (top, bottom) = self.scroll_region_rows();
        if top >= bottom {
            return;
        }

        if rows > bottom - top {
            // If we have to scroll past the whole scroll region, just
            // clobber everything.
            for i in top..bottom {
                self.buf[i] = Line::new();
            }
            return;
        }

        let to_shuffle = (bottom - top) - rows;
        for i in 0..to_shuffle {
            self.buf[top + i] = self.buf[top + rows + i].clone();
        }
        for i in 0..rows {
            self.buf[top + to_shuffle + i] = Line::new();
        }
    }

    /// SD (CSI T). Move the content of the scroll region down by `rows`,
    /// opening blank rows at the top. Rows pushed past the bottom of the
    /// region are lost. Also what a reverse index at the top of the scroll
    /// region does.
    pub fn scroll_down(&mut self, rows: usize) {
        let (top, _) = self.scroll_region_rows();
        self.insert_lines(&Pos { row: top, col: 0 }, rows);
    }

    pub fn clamp_to_scroll_region(&self, cursor: &mut Pos, size: &crate::Size) {
        match self.origin_mode {
            OriginMode::Term => cursor.clamp_to(size),
            OriginMode::ScrollRegion => cursor.clamp_to(self.scroll_region.as_region(size)),
        }
    }

    /// Resize the alt screen. This does not perform any reflow logic,
    /// instead just trimming any cells that are no longer within the
    /// screen.
    pub fn resize(&mut self, new_size: crate::Size) {
        for line in self.buf.iter_mut() {
            line.truncate(new_size.width);
        }

        let old_height = self.buf.len();
        if new_size.height > old_height {
            for _ in 0..(new_size.height - old_height) {
                self.buf.push_back(Line::new());
            }
        } else if new_size.height < old_height {
            for _ in 0..(old_height - new_size.height) {
                self.buf.pop_back();
            }
        }
        // no-op if they have the same height
    }

    /// The line at the given row, or None if that row is not on the screen.
    pub fn get_line_mut(&mut self, row: usize) -> Option<&mut Line> {
        self.buf.get_mut(row)
    }

    //
    // Command handlers
    //

    pub fn erase_to_end(&mut self, cursor: Pos) {
        if let Some(line) = self.buf.get_mut(cursor.row) {
            line.truncate(cursor.col);
        }

        let end = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { bottom, .. }) => *bottom,
            _ => self.buf.len(),
        };

        for i in (cursor.row + 1)..std::cmp::min(end, self.buf.len()) {
            self.buf[i].truncate(0);
        }
    }

    pub fn erase_from_start(&mut self, cursor: Pos) {
        let start = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { top, .. }) => *top,
            _ => 0,
        };

        for i in start..std::cmp::min(cursor.row, self.buf.len()) {
            self.buf[i].truncate(0);
        }
        if let Some(line) = self.buf.get_mut(cursor.row) {
            line.erase(line::Section::StartTo(cursor.col));
        }
    }

    pub fn erase(&mut self) {
        let (start, end) = match (self.origin_mode, &self.scroll_region) {
            (OriginMode::ScrollRegion, ScrollRegion::Window { top, bottom }) => (*top, *bottom),
            _ => (0, self.buf.len()),
        };

        for i in start..std::cmp::min(end, self.buf.len()) {
            self.buf[i].truncate(0);
        }
    }

    pub fn insert_lines(&mut self, cursor: &Pos, n: usize) {
        let (top, bottom) = self.scroll_region_rows();
        if cursor.row < top || bottom <= cursor.row {
            // Insert Line does nothing when the cursor is outside
            // the scroll region.
            return;
        }

        // We want to solve for `shuffle_lines` in:
        //
        // ```
        // (cursor.row - top) + min(n, bottom - cursor.row) + shuffle_lines =
        //    (bottom - top)
        // ```
        let lines_to_insert = std::cmp::min(n, bottom - cursor.row);
        let shuffle_lines = (bottom - top) - lines_to_insert - (cursor.row - top);
        for i in 0..shuffle_lines {
            // By using std::mem::replace rather than cloning we can avoid a
            // little work on the second pass.
            let bottom_offset = bottom - 1 - i;
            self.buf[bottom_offset] =
                std::mem::replace(&mut self.buf[bottom_offset - lines_to_insert], Line::new());
        }

        // clober any lines that are not handled by the initial pass.
        for i in shuffle_lines..lines_to_insert {
            self.buf[cursor.row + i] = Line::new();
        }
    }

    pub fn delete_lines(&mut self, cursor: &Pos, n: usize) {
        let (top, bottom) = self.scroll_region_rows();
        if cursor.row < top || bottom <= cursor.row {
            // Delete Line does nothing when the cursor is outside
            // the scroll region.
            return;
        }

        let lines_to_delete = std::cmp::min(n, bottom - cursor.row);
        let shuffle_lines = (bottom - top) - lines_to_delete - (cursor.row - top);

        for i in 0..shuffle_lines {
            self.buf[cursor.row + i] =
                std::mem::replace(&mut self.buf[cursor.row + lines_to_delete + i], Line::new());
        }
        for i in shuffle_lines..lines_to_delete {
            self.buf[cursor.row + i] = Line::new();
        }
    }
}

impl std::fmt::Display for AltScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.buf.iter() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl AsTermInput for AltScreen {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        for (i, line) in self.buf.iter().enumerate() {
            line.term_input_into(buf);
            if i != self.buf.len() - 1 {
                term::Crlf::default().term_input_into(buf);
            }
        }

        self.scroll_region.term_input_into(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    const SIZE: crate::Size = crate::Size { width: 5, height: 3 };

    /// An alt screen with a distinct char in the first column of every row.
    fn alt_screen() -> AltScreen {
        let mut alt = AltScreen::new(SIZE);
        for (i, c) in ['a', 'b', 'c'].iter().enumerate() {
            alt.buf[i].set_cell(SIZE.width, 0, Cell::new(*c, term::Attrs::default())).unwrap();
        }
        alt
    }

    /// The first column of every row, with '.' for a blank row.
    fn first_col(alt: &AltScreen) -> String {
        alt.buf
            .iter()
            .map(|line| match line.cells.first() {
                Some(cell) if !cell.is_empty() => cell.to_string(),
                _ => String::from("."),
            })
            .collect()
    }

    // A scroll region is free to name rows the grid does not have.
    #[test]
    fn scroll_up_with_region_past_end_of_buf() {
        let mut alt = alt_screen();
        alt.scroll_region = ScrollRegion::Window { top: 0, bottom: SIZE.height + 6 };

        alt.scroll_up(1);

        assert_eq!(first_col(&alt), "bc.");
    }

    #[test]
    fn scroll_down_with_region_past_end_of_buf() {
        let mut alt = alt_screen();
        alt.scroll_region = ScrollRegion::Window { top: 0, bottom: SIZE.height + 6 };

        alt.scroll_down(1);

        assert_eq!(first_col(&alt), ".ab");
    }

    #[test]
    fn erase_with_region_past_end_of_buf() {
        let mut alt = alt_screen();
        alt.origin_mode = OriginMode::ScrollRegion;
        alt.scroll_region = ScrollRegion::Window { top: 0, bottom: SIZE.height + 6 };

        alt.erase();

        assert_eq!(first_col(&alt), "...");
    }

    #[test]
    fn insert_lines_past_last_row() {
        let mut alt = alt_screen();

        alt.insert_lines(&Pos { row: SIZE.height + 2, col: 0 }, 1);

        assert_eq!(first_col(&alt), "abc");
    }

    #[test]
    fn delete_lines_past_last_row() {
        let mut alt = alt_screen();

        alt.delete_lines(&Pos { row: SIZE.height + 2, col: 0 }, 1);

        assert_eq!(first_col(&alt), "abc");
    }

    // Nothing from the cursor down is on the screen, so there is nothing to
    // erase.
    #[test]
    fn erase_to_end_past_last_row() {
        let mut alt = alt_screen();

        alt.erase_to_end(Pos { row: SIZE.height + 2, col: 0 });

        assert_eq!(first_col(&alt), "abc");
    }

    // Everything on the screen is above the cursor, so all of it goes.
    #[test]
    fn erase_from_start_past_last_row() {
        let mut alt = alt_screen();

        alt.erase_from_start(Pos { row: SIZE.height + 2, col: 0 });

        assert_eq!(first_col(&alt), "...");
    }
}
