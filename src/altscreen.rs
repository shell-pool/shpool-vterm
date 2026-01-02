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

use crate::{line::Line, term::Cursor, Pos};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AltScreen {
    /// The entire grid the altscreen. size.height should always equal
    /// buf.len().
    ///
    /// buf[0] is at the top of the screen and buf[buf.len()-1] is at the bottom.
    buf: Vec<Line>,
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

impl AltScreen {
    fn new(size: crate::Size) -> Self {
        let mut buf = Vec::new();
        for _ in 0..size.height {
            buf.push(Line::new());
        }
        AltScreen {
            buf,
            size,
            cursor: Cursor::new(Pos { row: 0, col: 0 }),
            saved_cursor: Cursor::new(Pos { row: 0, col: 0 }),
        }
    }

    /// Resize the alt screen. This does not perform any reflow logic,
    /// instead just trimming any cells that are no longer within the
    /// screen.
    fn resize(&mut self, new_size: crate::Size) {
        assert_eq!(self.buf.len(), self.size.height);

        if new_size.width < self.size.width {
            for line in self.buf.iter_mut() {
                line.truncate(new_size.width);
            }
        }

        if new_size.height > self.size.height {
            for _ in 0..(new_size.height - self.size.height) {
                self.buf.push(Line::new());
            }
        } else if new_size.height < self.size.height {
            for _ in 0..(self.size.height - new_size.height) {
                self.buf.pop();
            }
        }
        // no-op if they have the same height

        self.size = new_size;

        self.cursor.clamp_to(self.size);
        self.saved_cursor.clamp_to(self.size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Attrs;
    use crate::{Pos, Size};

    #[test]
    fn resize_grow_height() {
        let mut screen = AltScreen::new(Size {
            width: 10,
            height: 5,
        });
        screen.resize(Size {
            width: 10,
            height: 10,
        });
        assert_eq!(screen.buf.len(), 10);
        assert_eq!(screen.size.height, 10);
    }

    #[test]
    fn resize_shrink_height() {
        let mut screen = AltScreen::new(Size {
            width: 10,
            height: 10,
        });
        screen.resize(Size {
            width: 10,
            height: 5,
        });
        assert_eq!(screen.buf.len(), 5);
        assert_eq!(screen.size.height, 5);
    }

    #[test]
    fn resize_shrink_width() {
        let mut screen = AltScreen::new(Size {
            width: 10,
            height: 5,
        });
        // Add a cell at col 9
        screen.buf[0]
            .set_cell(10, 9, crate::cell::Cell::new('a', Attrs::default()))
            .unwrap();
        assert_eq!(screen.buf[0].cells.len(), 10);

        screen.resize(Size {
            width: 5,
            height: 5,
        });
        assert_eq!(screen.size.width, 5);
        // Line should be truncated
        assert_eq!(screen.buf[0].cells.len(), 5);
    }

    #[test]
    fn cursor_clamping() {
        let mut screen = AltScreen::new(Size {
            width: 10,
            height: 10,
        });
        screen.cursor.pos = Pos { row: 9, col: 9 };
        screen.saved_cursor.pos = Pos { row: 8, col: 8 };

        screen.resize(Size {
            width: 5,
            height: 5,
        });

        assert_eq!(screen.cursor.pos.row, 4);
        assert_eq!(screen.cursor.pos.col, 4);
        assert_eq!(screen.saved_cursor.pos.row, 4);
        assert_eq!(screen.saved_cursor.pos.col, 4);
    }
}
