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

mod attrs;
mod cell;
mod grid;
mod term;

use term::BufWrite;

/// A representation of a terminal.
///
/// The terminal state is represented as a dequeue of lines which
/// might be arbitrarily long. If a new line is added to the terminal
/// when the queue is full, the oldest line will be discarded. It is
/// importaint to understand that lines in the terminal state do not
/// corrispond to physical lines as they would appear on the screen.
/// A single logical line could wrap zero or more times, causing it
/// to take up one or more physical lines when displayed.
///
/// TODO: Will I need to have a "grid view" layer that allows indexing
/// into the logical data as if it were laid out in a grid? I think this
/// may be required to handle certain control sequences. Might need to
/// use some sort of weird tree structure that notes the grid line that
/// each logical line starts on and then allows you to find the logical
/// line for each grid line.
pub struct Term {
    parser: vte::Parser,
    grid: grid::Grid,
}

impl Term {
    /// Create a new terminal with the given width and height.
    ///
    /// Note that width will only be used when generated output
    /// to determine where wrapping should be place.
    pub fn new(scrollback_lines: usize, size: Size) -> Self {
        Term {
            parser: vte::Parser::new(),
            grid: grid::Grid::new(scrollback_lines, size),
        }
    }

    /// Get the current terminal size.
    pub fn size(&self) -> Size {
        self.grid.size()
    }

    /// Set the terminal size.
    pub fn resize(&mut self, size: Size) {
        self.grid.resize(size);
    }

    /// Get the current number of lines of stored scrollback.
    pub fn scrollback_lines(&self) -> usize {
        self.grid.scrollback_lines()
    }

    /// Set the number of lines of scrollback to store. This will drop
    /// data when resizing down. When resizing up, no new memory is allocated,
    /// capacity is simply expanded.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        self.grid.set_scrollback_lines(scrollback_lines);
    }

    /// Process the given chunk of input. This should be the data read off
    /// a pty running a shell.
    pub fn process(&mut self, buf: &[u8]) {
        self.parser.advance(&mut self.grid, buf);
    }

    /// Get the current contents of the terminal encoded via terminal
    /// escape sequences. The contents buffer will be prefixed with
    /// a reset code, so inputing this to any terminal emulator will
    /// reset the emulator to the contents of this Term instance.
    pub fn contents(&self) -> Vec<u8> {
        let mut buf = vec![];
        term::ClearAttrs::default().write_buf(&mut buf);
        term::ClearScreen::default().write_buf(&mut buf);
        self.grid.write_buf(&mut buf);

        buf
    }
}

/// The size of the terminal.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// A position within the terminal. Generally, this refers to a grid
/// mode view of the terminal, not the underlying logical lines mode
/// that we actually store the data in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

// TODO: handle clear

#[cfg(test)]
mod test {
    use crate::term;
    use crate::term::BufWrite;

    macro_rules! frag {
        {
            $test_name:ident
            {
                scrollback_lines: $scrollback_lines:expr ,
                width: $width:expr ,
                height: $height:expr
            }
            <= $( $input_expr:expr ),*
            => $( $output_expr:expr ),*
        } => {
            #[test]
            fn $test_name() {
                let mut input: Vec<u8> = vec![];
                $(
                    $input_expr.write_buf(&mut input);
                )*
                let mut output: Vec<u8> = vec![];
                $(
                    $output_expr.write_buf(&mut output);
                )*
                round_trip_frag(input.as_slice(), output.as_slice(),
                                $scrollback_lines,
                                crate::Size{width: $width, height: $height});
            }
        }
    }

    frag! {
        simple_str { scrollback_lines: 100, width: 100, height: 100 }
        <= term::Raw::from("foobar")
        => term::ClearAttrs::default(),
            term::ClearScreen::default(),
            term::Raw::from("foobar")
    }

    frag! {
        newline2line { scrollback_lines: 100, width: 100, height: 100 }
        <= term::Raw::from("foo\r\nbar")
        => term::ClearAttrs::default(),
            term::ClearScreen::default(),
            term::Raw::from("foo"),
            term::Crlf::default(),
            term::Raw::from("bar")
    }

    frag! {
        underline_rt { scrollback_lines: 100, width: 100, height: 100 }
        <= term::Raw::from("a"),
           term::control_codes().underline,
           term::Raw::from("b"),
           term::control_codes().undo_underline,
           term::Raw::from("a")
        => term::ClearAttrs::default(),
           term::ClearScreen::default(),
           term::Raw::from("a"),
           term::control_codes().underline,
           term::Raw::from("b"),
           term::control_codes().undo_underline,
           term::Raw::from("a")
    }

    frag! {
        bold_rt { scrollback_lines: 100, width: 100, height: 100 }
        <= term::Raw::from("a"),
           term::control_codes().bold,
           term::Raw::from("b"),
           term::control_codes().undo_bold,
           term::Raw::from("a")
        => term::ClearAttrs::default(),
           term::ClearScreen::default(),
           term::Raw::from("a"),
           term::control_codes().bold,
           term::Raw::from("b"),
           term::control_codes().undo_bold,
           term::Raw::from("a")
    }

    frag! {
        italic_rt { scrollback_lines: 100, width: 100, height: 100 }
        <= term::Raw::from("a"),
           term::control_codes().italic,
           term::Raw::from("b"),
           term::control_codes().undo_italic,
           term::Raw::from("a")
        => term::ClearAttrs::default(),
           term::ClearScreen::default(),
           term::Raw::from("a"),
           term::control_codes().italic,
           term::Raw::from("b"),
           term::control_codes().undo_italic,
           term::Raw::from("a")
    }

    fn round_trip_frag(
        input: &[u8],
        want_output: &[u8],
        scrollback_lines: usize,
        size: crate::Size,
    ) {
        let mut term = crate::Term::new(scrollback_lines, size);
        term.process(input);
        assert_eq!(term.contents().as_slice(), want_output);
    }
}
