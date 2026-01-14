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
    line,
    scrollback::Scrollback,
    term::{self, AsTermInput, Pos},
};

use tracing::warn;

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
    // The slot where cursor position info is saved by the SCP/RCP
    // and ESC 7 / ESC 8 commands.
    pub saved_cursor: SavedCursor,
}

impl Screen {
    /// Create a new scrollback mode screen (a regular terminal screen).
    pub fn scrollback(mut scrollback_lines: usize, size: crate::Size) -> Self {
        if scrollback_lines < size.height {
            scrollback_lines = size.height;
        }

        Screen {
            grid: Grid::Scrollback(Scrollback::new(scrollback_lines)),
            size: size,
            cursor: Pos { row: 0, col: 0 },
            saved_cursor: SavedCursor::new(Pos { row: 0, col: 0 }),
        }
    }

    /// Create a new alt screen mode screen (used by ncurses apps like vim).
    pub fn alt(size: crate::Size) -> Self {
        Screen {
            grid: Grid::AltScreen(AltScreen::new(size)),
            size: size,
            cursor: Pos { row: 0, col: 0 },
            saved_cursor: SavedCursor::new(Pos { row: 0, col: 0 }),
        }
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
            warn!("attempt to set scrollback lines on non-scrollback screen");
        }
    }

    pub fn resize(&mut self, new_size: crate::Size) {
        match &mut self.grid {
            Grid::Scrollback(scrollback) => scrollback.reflow(new_size.width),
            Grid::AltScreen(altscreen) => altscreen.resize(new_size),
        }
        self.size = new_size;

        self.cursor.clamp_to(self.size);
        self.saved_cursor.pos.clamp_to(self.size);
    }

    pub fn clamp(&mut self) {
        self.cursor.clamp_to(self.size);
    }

    pub fn write_at_cursor(&mut self, cell: Cell) -> anyhow::Result<()> {
        self.cursor = match &mut self.grid {
            Grid::Scrollback(scrollback) => {
                scrollback.write_at_cursor(self.size, self.cursor, cell)?
            }
            Grid::AltScreen(altscreen) => {
                altscreen.write_at_cursor(self.size, self.cursor, cell)?
            }
        };

        Ok(())
    }

    /// Erase whichever screen is currently active from the cursor
    /// position to the bottom. Used to implement 'CSI 0 J'
    pub fn erase_to_end(&mut self) {
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase_to_end(self.size, self.cursor),
            Grid::AltScreen(alt) => alt.erase_to_end(self.cursor),
        }
    }

    /// Erase whichever screen is currently active from the top to the
    /// cursor position. Used to implement 'CSI 1 J'
    pub fn erase_from_start(&mut self) {
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase_from_start(self.size, self.cursor),
            Grid::AltScreen(alt) => alt.erase_from_start(self.cursor),
        }
    }

    /// Erase whichever screen is currently active, not including scrollback.
    /// Used to implement 'CSI 2 J' and 'CSI 3 J' (which includes the
    /// scrollback).
    pub fn erase(&mut self, include_scrollback: bool) {
        match &mut self.grid {
            Grid::Scrollback(s) => s.erase(self.size, include_scrollback),
            Grid::AltScreen(alt) => alt.erase(),
        }
    }

    pub fn erase_to_end_of_line(&mut self) {
        match &mut self.grid {
            Grid::Scrollback(s) => {
                if let Some(l) = s.get_line_mut(self.size, self.cursor.row) {
                    l.erase(line::Section::ToEnd(self.cursor.col));
                }
            }
            Grid::AltScreen(alt) => {
                alt.get_line_mut(self.cursor.row).erase(line::Section::ToEnd(self.cursor.col))
            }
        }
    }

    pub fn erase_to_start_of_line(&mut self) {
        match &mut self.grid {
            Grid::Scrollback(s) => {
                if let Some(l) = s.get_line_mut(self.size, self.cursor.row) {
                    l.erase(line::Section::StartTo(self.cursor.col));
                }
            }
            Grid::AltScreen(alt) => {
                alt.get_line_mut(self.cursor.row).erase(line::Section::StartTo(self.cursor.col))
            }
        }
    }

    pub fn erase_line(&mut self) {
        match &mut self.grid {
            Grid::Scrollback(s) => {
                if let Some(l) = s.get_line_mut(self.size, self.cursor.row) {
                    l.erase(line::Section::Whole);
                }
            }
            Grid::AltScreen(alt) => alt.get_line_mut(self.cursor.row).erase(line::Section::Whole),
        }
    }
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

impl AsTermInput for Screen {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        match &self.grid {
            Grid::Scrollback(scrollback) => scrollback.term_input_into(buf),
            Grid::AltScreen(altscreen) => altscreen.term_input_into(buf),
        }

        term::ControlCodes::cursor_position(
            (self.cursor.row + 1) as u16,
            (self.cursor.col + 1) as u16,
        )
        .term_input_into(buf);
    }
}

/// A position that the terminal was writing at. Includes attributes that
/// have been previously set via control codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SavedCursor {
    pub pos: Pos,
    pub attrs: term::Attrs,
}

impl SavedCursor {
    pub fn new(pos: Pos) -> Self {
        SavedCursor { pos, attrs: term::Attrs::default() }
    }
}

#[derive(Debug)]
enum Grid {
    Scrollback(Scrollback),
    AltScreen(AltScreen),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Attrs;
    use crate::Size;

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
                alt.buf[0].set_cell(10, 9, crate::cell::Cell::new('a', Attrs::default())).unwrap();
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

        screen.write_at_cursor(c.clone())?;

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
        screen.write_at_cursor(Cell::new('1', term::Attrs::default()))?;
        screen.write_at_cursor(Cell::new('2', term::Attrs::default()))?;

        // This should wrap to next line
        screen.write_at_cursor(Cell::new('3', term::Attrs::default()))?;

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
            screen.write_at_cursor(Cell::new('X', term::Attrs::default()))?;
        }

        let c_top = Cell::new('T', term::Attrs::default());
        let c_mid = Cell::new('M', term::Attrs::default());
        let c_bot = Cell::new('B', term::Attrs::default());

        for _ in 0..10 {
            screen.write_at_cursor(c_top.clone())?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_mid.clone())?;
        }
        for _ in 0..10 {
            screen.write_at_cursor(c_bot.clone())?;
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
            screen.write_at_cursor(Cell::new(
                char::from_digit(i, 10).unwrap(),
                term::Attrs::default(),
            ))?;
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
            screen.write_at_cursor(Cell::new(
                char::from_digit(i, 10).unwrap(),
                term::Attrs::default(),
            ))?;
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
            (10, 20), // Start wide, go narrow
            (5, 10),  // Start narrow, go wide
            (10, 10), // No change
        ];

        for (start_w, end_w) in shapes {
            let start_size = Size { width: start_w, height: 10 };
            let mut screen = Screen::scrollback(100, start_size);

            // Fill with deterministic data
            let count = 30;
            for i in 0..count {
                screen.write_at_cursor(Cell::new(
                    char::from_u32(65 + i % 26).unwrap(),
                    term::Attrs::default(),
                ))?;
            }

            // Resize
            screen.resize(Size { width: end_w, height: 10 });

            // Resize back
            screen.resize(start_size);

            // Verify content is identical to if we just pushed it
            let mut expected_screen = Screen::scrollback(100, start_size);
            for i in 0..count {
                expected_screen.write_at_cursor(Cell::new(
                    char::from_u32(65 + i % 26).unwrap(),
                    term::Attrs::default(),
                ))?;
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
