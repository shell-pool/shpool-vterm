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

mod altscreen;
mod cell;
mod line;
mod screen;
mod scrollback;

#[cfg(not(feature = "internal-test"))]
mod term;

#[cfg(feature = "internal-test")]
pub mod term;

use crate::{
    cell::Cell,
    screen::Screen,
    term::{AsTermInput, BlinkStyle, FontWeight, FrameStyle, UnderlineStyle},
};

use tracing::warn;

use crate::screen::SavedCursor;

/// A representation of a terminal.
pub struct Term {
    parser: vte::Parser,
    state: State,
}

impl Term {
    /// Create a new terminal with the given width and height.
    ///
    /// Note that width will only be used when generated output
    /// to determine where wrapping should be place.
    ///
    /// scrollback_lines must be at least size.height. If it is
    /// less than size.height, it will be automatically adjusted
    /// to be equal to size.height.
    pub fn new(scrollback_lines: usize, size: Size) -> Self {
        Term { parser: vte::Parser::new(), state: State::new(scrollback_lines, size) }
    }

    /// Get the current terminal size.
    pub fn size(&self) -> Size {
        self.state.screen().size
    }

    /// Set the terminal size.
    ///
    /// This will implicitly size up the scrollback_lines if
    /// it is currently less than size.height.
    pub fn resize(&mut self, size: Size) {
        if size.height > self.scrollback_lines() {
            self.set_scrollback_lines(size.height);
        }

        self.state.scrollback.resize(size);
        self.state.altscreen.resize(size);
    }

    /// Get the current number of lines of stored scrollback.
    pub fn scrollback_lines(&self) -> usize {
        self.state.scrollback.scrollback_lines().expect("scrollback screen to have lines")
    }

    /// Set the number of lines of scrollback to store. This will drop
    /// data when resizing down. When resizing up, no new memory is allocated,
    /// capacity is simply expanded.
    ///
    /// If the given value is less than size().height, it will be overridden
    /// to match the current height. You cannot store less scrollback than
    /// there are lines in the visible screen region.
    pub fn set_scrollback_lines(&mut self, scrollback_lines: usize) {
        self.state.scrollback.set_scrollback_lines(scrollback_lines);
    }

    /// Process the given chunk of input. This should be the data read off
    /// a pty running a shell.
    pub fn process(&mut self, buf: &[u8]) {
        self.parser.advance(&mut self.state, buf);
    }

    /// Get the current contents of the terminal encoded via terminal
    /// escape sequences. The contents buffer will be prefixed with
    /// a reset code, so inputing this to any terminal emulator will
    /// reset the emulator to the contents of this Term instance.
    pub fn contents(&self) -> Vec<u8> {
        let mut buf = vec![];
        term::control_codes().clear_attrs.term_input_into(&mut buf);
        term::ControlCodes::cursor_position(1, 1).term_input_into(&mut buf);
        term::control_codes().clear_screen.term_input_into(&mut buf);
        self.state.term_input_into(&mut buf);

        buf
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.fmt(f)
    }
}

/// The size of the terminal.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// The complete terminal state. An internal implementation detail.
struct State {
    /// The state for the normal terminal screen.
    scrollback: Screen,
    /// The state for the alternate screen.
    altscreen: Screen,
    /// The currently active screen mode.
    screen_mode: ScreenMode,
    /// The current cursor attrs. These are shared between the scrollback
    /// and alt screens, which is why they are stored here rather than
    /// with the curors themsevles.
    cursor_attrs: term::Attrs,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.screen_mode {
            ScreenMode::Scrollback => {
                writeln!(f, "Screen Mode: Scrollback")?;
                write!(f, "{}", self.scrollback)?;
            }
            ScreenMode::Alt => {
                writeln!(f, "Screen Mode: AltScreen")?;
                write!(f, "{}", self.altscreen)?;
            }
        }

        Ok(())
    }
}

impl State {
    fn new(scrollback_lines: usize, size: Size) -> Self {
        State {
            scrollback: Screen::scrollback(scrollback_lines, size),
            altscreen: Screen::alt(size),
            screen_mode: ScreenMode::Scrollback,
            cursor_attrs: term::Attrs::default(),
        }
    }

    fn screen_mut(&mut self) -> &mut Screen {
        match self.screen_mode {
            ScreenMode::Scrollback => &mut self.scrollback,
            ScreenMode::Alt => &mut self.altscreen,
        }
    }

    fn screen(&self) -> &Screen {
        match self.screen_mode {
            ScreenMode::Scrollback => &self.scrollback,
            ScreenMode::Alt => &self.altscreen,
        }
    }
}

impl AsTermInput for State {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        match self.screen_mode {
            ScreenMode::Scrollback => self.scrollback.term_input_into(buf),
            ScreenMode::Alt => self.altscreen.term_input_into(buf),
        }

        // restore cursor attributes (the screen will have already restored our
        // position).
        term::control_codes().clear_attrs.term_input_into(buf);
        let codes = term::Attrs::default().transition_to(&self.cursor_attrs);
        for c in codes.into_iter() {
            c.term_input_into(buf);
        }
    }
}

/// Indicates which screen mode is active.
enum ScreenMode {
    Scrollback,
    Alt,
}

impl vte::Perform for State {
    fn print(&mut self, c: char) {
        let attrs = self.cursor_attrs.clone();
        let screen = self.screen_mut();
        if let Err(e) = screen.write_at_cursor(Cell::new(c, attrs)) {
            warn!("writing char at cursor: {e:?}");
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.screen_mut().cursor.row += 1,
            b'\r' => self.screen_mut().cursor.col = 0,
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
    //
    // rustfmt has insane ideas about match arm formatting and there is
    // apparently no way to make it do the reasonable thing of preserving
    // horizontal whitespace by placing loops directly in match arm statement
    // position.
    #[rustfmt::skip]
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        if ignore {
            warn!("malformed CSI seq");
            return;
        }

        let mut params_iter = params.iter();

        match action {
            // CUU (Cursor Up)
            'A' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
            }
            // CUD (Cursor Down)
            'B' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.clamp();
            }
            // CUF (Cursor Forward)
            'C' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col += n;
                screen.clamp();
            }
            // CUF (Cursor Backwards)
            'D' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col = screen.cursor.col.saturating_sub(n);
            }
            // CNL (Cursor Next Line)
            'E' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.cursor.col = 0;
                screen.clamp();
            }
            // CPL (Cursor Prev Line)
            'F' => {
                let n = p1_or(params, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
                screen.cursor.col = 0;
            }
            // CHA (Cursor Horizontal Absolute)
            'G' => {
                let n = p1_or(params, 1) as usize;
                let n = n - 1; // translate to 0 indexing

                let screen = self.screen_mut();
                screen.cursor.col = n;
                screen.clamp();
            }
            // CUP (Cursor Set Position)
            'H' => {
                // parse the params and adjust 1 indexing to 0 indexing
                let default = [1];
                let row = *params_iter
                    .next()
                    .unwrap_or(&default)
                    .iter()
                    .next()
                    .unwrap_or(&default[0]);
                let col = *params_iter
                    .next()
                    .unwrap_or(&default)
                    .iter()
                    .next()
                    .unwrap_or(&default[0]);
                let row = row.saturating_sub(1) as usize;
                let col = col.saturating_sub(1) as usize;

                let screen = self.screen_mut();
                screen.cursor.row = row;
                screen.cursor.col = col;
                screen.clamp();
            }
            // ED (Erase in Display)
            'J' => while let Some(code) = params_iter.next() {
                match code {
                    [] | [0] => self.screen_mut().erase_to_end(),
                    [1] => self.screen_mut().erase_from_start(),
                    [2] => self.screen_mut().erase(false),
                    [3] => self.screen_mut().erase(true),
                    _ => warn!("unhandled 'CSI {code:?} J'"),
                }
            }
            // EL (Erase in Line)
            'K' => while let Some(code) = params_iter.next() {
                match code {
                    [] | [0] => self.screen_mut().erase_to_end_of_line(),
                    [1] => self.screen_mut().erase_to_start_of_line(),
                    [2] => self.screen_mut().erase_line(),
                    _ => warn!("unhandled 'CSI {code:?} K'"),
                }
            }

            // SCP (Save Cursor Position)
            's' => {
                let screen = self.screen_mut();
                let cursor = screen.cursor.clone();
                screen.saved_cursor.pos = cursor;
            }
            // RCP (Restore Cursor Position)
            'u' => {
                let screen = self.screen_mut();
                screen.cursor = screen.saved_cursor.pos;
            }

            'h' => {
                match intermediates {
                    [b'?'] => while let Some(code) = params_iter.next() {
                        match code {
                            [1049] => {
                                // The alt-screen gets reset upon entry, so we need to
                                // clobber it here.
                                self.altscreen = Screen::alt(self.altscreen.size);
                                self.screen_mode = ScreenMode::Alt;
                            }
                            _ => {
                                warn!(
                                    "Unhandled CSI l command: CSI {:?} {:?} l",
                                    intermediates,
                                    params.iter().collect::<Vec<&[u16]>>()
                                );
                                return;
                            }
                        }
                    }
                    _ => warn!(
                        "Unhandled CSI h command: CSI {:?} {:?} h",
                        intermediates,
                        params.iter().collect::<Vec<&[u16]>>()
                    ),
                }
            }
            'l' => match intermediates {
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1049] => self.screen_mode = ScreenMode::Scrollback,
                        _ => {
                            warn!(
                                "Unhandled CSI l command: CSI {:?} {:?} l",
                                intermediates,
                                params.iter().collect::<Vec<&[u16]>>()
                            );
                            return;
                        }
                    }
                }
                _ => warn!(
                    "Unhandled CSI l command: CSI {:?} {:?} l",
                    intermediates,
                    params.iter().collect::<Vec<&[u16]>>()
                ),
            },

            // cell attribute manipulation
            'm' => while let Some(param) = params_iter.next() {
                match param {
                    [] | [0] => self.cursor_attrs = term::Attrs::default(),

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
                    [4] => self.cursor_attrs.underline = Some(UnderlineStyle::Single),
                    [21] => self.cursor_attrs.underline = Some(UnderlineStyle::Double),
                    [24] => self.cursor_attrs.underline = None,

                    // Font Weight Handling.
                    [1] => self.cursor_attrs.font_weight = Some(FontWeight::Bold),
                    [2] => self.cursor_attrs.font_weight = Some(FontWeight::Faint),
                    [22] => self.cursor_attrs.font_weight = None,

                    // Italic Handling.
                    [3] => self.cursor_attrs.italic = true,
                    [23] => self.cursor_attrs.italic = false,

                    // Inverse Handling.
                    [7] => self.cursor_attrs.inverse = true,
                    [27] => self.cursor_attrs.inverse = false,

                    // Blink Handling
                    [5] => self.cursor_attrs.blink = Some(BlinkStyle::Slow),
                    [6] => self.cursor_attrs.blink = Some(BlinkStyle::Rapid),
                    [25] => self.cursor_attrs.blink = None,

                    // Conceal Handling
                    [8] => self.cursor_attrs.conceal = true,
                    [28] => self.cursor_attrs.conceal = false,

                    // Strikethrough Handling.
                    [9] => self.cursor_attrs.strikethrough = true,
                    [29] => self.cursor_attrs.strikethrough = false,

                    // Frame Handling.
                    [51] => self.cursor_attrs.framed = Some(FrameStyle::Frame),
                    [52] => self.cursor_attrs.framed = Some(FrameStyle::Circle),
                    [54] => self.cursor_attrs.framed = None,

                    // Overline Handling.
                    [53] => self.cursor_attrs.overline = true,
                    [55] => self.cursor_attrs.overline = false,

                    _ => warn!("unhandled 'CSI {param:?} m'"),
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
            ([], b'7') => {
                let attrs = self.cursor_attrs.clone();
                let screen = self.screen_mut();
                let pos = screen.cursor.clone();
                screen.saved_cursor = SavedCursor { pos, attrs };
            }
            // restore cursor (ESC 8)
            ([], b'8') => {
                let screen = self.screen_mut();
                screen.cursor = screen.saved_cursor.pos;
                self.cursor_attrs = screen.saved_cursor.attrs.clone();
            }

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
