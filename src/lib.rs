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

use std::collections::BTreeMap;

use crate::{
    cell::Cell,
    screen::{SavedCursor, Screen},
    term::{
        AsTermInput, BlinkStyle, ControlCodes, FontWeight, FrameStyle, OriginMode, UnderlineStyle, LinkTarget,
    },
};

use smallvec::SmallVec;
use tracing::{debug, warn};

mod altscreen;
mod cell;
mod line;
mod screen;
mod scrollback;

#[cfg(not(feature = "internal-test"))]
mod term;

#[cfg(feature = "internal-test")]
pub mod term;

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
    pub fn contents(&self, dump_region: ContentRegion) -> Vec<u8> {
        let mut buf = vec![];
        term::control_codes().clear_attrs.term_input_into(&mut buf);
        term::ControlCodes::cursor_position(1, 1).term_input_into(&mut buf);
        term::control_codes().clear_screen.term_input_into(&mut buf);
        self.state.dump_contents_into(&mut buf, dump_region);

        buf
    }
}

/// A section of the screen to dump.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ContentRegion {
    /// The whole terminal state, including all scrollback data.
    All,
    /// Only the visible lines.
    Screen,
    /// The bottom N lines, including (N - height) lines of scrollback.
    BottomLines(usize),
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
    /// The terminal title, as set by `OSC 0` and `OSC 2`.
    title: Option<SmallVec<[u8; 8]>>,
    /// The terminal icon name, as set by `OSC 0` and `OSC 1`.
    icon_name: Option<SmallVec<[u8; 8]>>,
    /// The terminal working directory (some terminal emulators use this
    /// to know what directory to start new shells in).
    working_dir: Option<WorkingDir>,
    /// A table mapping color index to a particular color spec.
    /// This is set by OSC 4. We use a tree for deterministic output
    /// to make testing easier. A hash would work just as well.
    palette_overrides: BTreeMap<usize, Vec<u8>>,
}

struct WorkingDir {
    host: SmallVec<[u8; 8]>,
    dir: SmallVec<[u8; 8]>,
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
            title: None,
            icon_name: None,
            working_dir: None,
            palette_overrides: BTreeMap::new(),
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

    fn dump_contents_into(&self, buf: &mut Vec<u8>, dump_region: ContentRegion) {
        match self.screen_mode {
            ScreenMode::Scrollback => self.scrollback.dump_contents_into(buf, dump_region),
            ScreenMode::Alt => self.altscreen.dump_contents_into(buf, dump_region),
        }

        // restore cursor attributes (the screen will have already restored our
        // position).
        term::control_codes().clear_attrs.term_input_into(buf);
        let codes = term::Attrs::default().transition_to(&self.cursor_attrs);
        for c in codes.into_iter() {
            c.term_input_into(buf);
        }

        // Restore the title / icon name. Most terminals treat theses as the
        // same thing these days, but we'll go the extra mile and differentiate
        // rather than just always sending `OSC 0 ; <title> ST` in case there is
        // a terminal that actually makes a distinction.
        match (&self.title, &self.icon_name) {
            (Some(title), Some(icon_name)) if title == icon_name => {
                ControlCodes::set_title_and_icon_name(title.clone()).term_input_into(buf)
            }
            (Some(title), Some(icon_name)) => {
                ControlCodes::set_title(title.clone()).term_input_into(buf);
                ControlCodes::set_icon_name(icon_name.clone()).term_input_into(buf);
            }
            (Some(title), None) => {
                ControlCodes::set_title(title.clone()).term_input_into(buf);
            }
            (None, Some(icon_name)) => {
                ControlCodes::set_icon_name(icon_name.clone()).term_input_into(buf);
            }
            (None, None) => {}
        }

        if let Some(working_dir) = &self.working_dir {
            ControlCodes::set_working_dir(working_dir.host.clone(), working_dir.dir.clone())
                .term_input_into(buf);
        }

        if !self.palette_overrides.is_empty() {
            ControlCodes::set_color_indices(
                self.palette_overrides.iter().map(|(idx, color_spec)| {
                    (*idx, SmallVec::from(color_spec.as_slice()))
                })
            ).term_input_into(buf);
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
        screen.snap_to_bottom();
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

    // OSC commands are of the form
    // `OSC <p1> ; <p2> ... <pn> <terminator>` where
    // `OSC` is always `ESC]`, the params are byte sequences seperated by
    // semicolons, and the terminator is either `BEL` (0x7) or
    // `ST` (`ESC\`, 0x1b 0x5c). Modern applications use ST for the most
    // part, but some older applications will send BEL. We should be able
    // to just ignore the _bell_terminated flag and treat commands the
    // same regardless of the terminator they have.
    #[rustfmt::skip]
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let mut params_iter = params.iter();
        match params_iter.next() {
            // Title manipulation
            Some([b'0']) => if let Some(title) = params_iter.next() {
                self.title = Some(title.to_vec().into());
                self.icon_name = Some(title.to_vec().into());
            } else {
                warn!("OSC 0 with no title param");
            },
            Some([b'1']) => if let Some(icon_name) = params_iter.next() {
                self.icon_name = Some(icon_name.to_vec().into());
            } else {
                warn!("OSC 1 with no icon_name param");
            },
            Some([b'2']) => if let Some(title) = params_iter.next() {
                self.title = Some(title.to_vec().into());
            } else {
                warn!("OSC 2 with no title param");
            },

            // Color Palette
            Some([b'4']) => while let (Some(idx), Some(color_spec)) = (params_iter.next(), params_iter.next()) {
                if *color_spec == [b'?'] {
                    // If the program is querying for a color, we just ignore
                    // that control code. The real terminal is responsible for
                    // responding.
                    continue;
                }

                match std::str::from_utf8(idx) {
                    Ok(s) => match s.parse::<usize>() {
                        Ok(i) => {
                            self.palette_overrides.insert(i, color_spec.to_vec());
                        },
                        Err(e) => warn!("OSC 4: idx is an invalid number '{s}': {e}"),
                    },
                    Err(e) => warn!("OSC 4: invalid idx '{idx:?}': {e}"),
                }
            },
            Some([b'1', b'0', b'4']) => while let Some(idx) = params_iter.next() {
                match std::str::from_utf8(idx) {
                    Ok(s) => match s.parse::<usize>() {
                        Ok(i) => {
                            self.palette_overrides.remove(&i);
                        },
                        Err(e) => warn!("OSC 104: idx is an invalid number '{s}': {e}"),
                    },
                    Err(e) => warn!("OSC 104: invalid idx '{idx:?}': {e}"),
                }
            },

            // Working dir
            Some([b'7']) => if let (Some(host), Some(dir)) = (params_iter.next(), params_iter.next()) {
                self.working_dir = Some(WorkingDir {
                    host: host.to_vec().into(),
                    dir: dir.to_vec().into(),
                });
            } else {
                warn!("OSC 7 with fewer than 2 params");
            },

            // Links. Depending on params, OSC 8 both starts and ends links.
            Some([b'8']) => if let (Some(params), Some(url)) = (params_iter.next(), params_iter.next()) {
                if params.is_empty() && url.is_empty() {
                    self.cursor_attrs.link_target = None;
                } else {
                    self.cursor_attrs.link_target = Some(LinkTarget {
                        params: SmallVec::from_slice(params),
                        url: SmallVec::from_slice(url),
                    });
                }
            } else {
                self.cursor_attrs.link_target = None;
            }

            _ => warn!("unhandled 'OSC {:?} {}'", params, if bell_terminated {
                "BEL"
            } else {
                "ST"
            }),
        }
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
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
                screen.clamp();
            }
            // CUD (Cursor Down)
            'B' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.clamp();
            }
            // CUF (Cursor Forward)
            'C' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col += n;
                screen.clamp();
            }
            // CUF (Cursor Backwards)
            'D' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col = screen.cursor.col.saturating_sub(n);
                screen.clamp();
            }
            // CNL (Cursor Next Line)
            'E' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.cursor.col = 0;
                screen.clamp();
            }
            // CPL (Cursor Prev Line)
            'F' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
                screen.cursor.col = 0;
                screen.clamp();
            }
            // CHA (Cursor Horizontal Absolute)
            'G' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let n = n.saturating_sub(1); // translate to 0 indexing

                let screen = self.screen_mut();
                screen.cursor.col = n;
                screen.clamp();
            }
            // CUP (Cursor Set Position)
            'H' => {
                // parse the params and adjust 1 indexing to 0 indexing
                let row = param_or(&mut params_iter, 1) as usize;
                let col = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.set_cursor(term::Pos { row, col });
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
                    [] | [0] => {
                        let screen = self.screen_mut();
                        let col = screen.cursor.col;
                        if let Some(l) = screen.get_line_mut() {
                            l.erase(line::Section::ToEnd(col));
                        }
                    }
                    [1] => {
                        let screen = self.screen_mut();
                        let col = screen.cursor.col;
                        if let Some(l) = screen.get_line_mut() {
                            l.erase(line::Section::StartTo(col));
                        }
                    }
                    [2] => if let Some(l) = self.screen_mut().get_line_mut() {
                        l.erase(line::Section::Whole);
                    }
                    _ => warn!("unhandled 'CSI {code:?} K'"),
                }
            }
            // IL (Insert Line)
            'L' => {
                let n = param_or(&mut params_iter, 1) as usize;
                self.screen_mut().insert_lines(n);
            }
            // DL (Delete Line)
            'M' => {
                let n = param_or(&mut params_iter, 1) as usize;
                self.screen_mut().delete_lines(n);
            }
            // SU (Scroll Up)
            'S' => {
                let n = param_or(&mut params_iter, 1) as usize;
                self.screen_mut().scroll_up(n as usize);
            }
            // SD (Scroll Down)
            'T' => {
                let n = param_or(&mut params_iter, 1) as usize;
                self.screen_mut().scroll_down(n as usize);
            }

            // ICH (Insert Character)
            '@' => {
                let n = param_or(&mut params_iter, 1) as usize;

                let screen = self.screen_mut();
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.get_line_mut() {
                    l.insert_character(width, col, n);
                }
            }
            // DCH (Delete Character)
            'P' => {
                let n = param_or(&mut params_iter, 1) as usize;

                let attrs = self.cursor_attrs.clone();

                let screen = self.screen_mut();
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.get_line_mut() {
                    l.delete_character(width, col, &attrs, n);
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
                screen.clamp();
            }

            'h' => match intermediates {
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        // enable alt scree
                        [1049] => {
                            // The alt-screen gets reset upon entry, so we need to
                            // clobber it here.
                            self.altscreen = Screen::alt(self.altscreen.size);
                            self.screen_mode = ScreenMode::Alt;
                        }
                        // enable origin mode
                        [6] => self.screen_mut().set_origin_mode(OriginMode::ScrollRegion),
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
            'l' => match intermediates {
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1049] => self.screen_mode = ScreenMode::Scrollback,
                        // disable origin mode
                        [6] => self.screen_mut().set_origin_mode(OriginMode::Term),
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
            // DSR (Device Status Report)
            'n' => while let Some(param) = params_iter.next() {
                match param {
                    // TODO: We might want to store this to assert against the
                    // terminal output stream once we start scanning that.
                    // We'll need to implement terminal output stream scanning
                    // in order to properly handle kitty extensions at some
                    // point (since we need to know if the real terminal
                    // responded with a code indicating that it supported the
                    // extensions in order to determine how we should interpret
                    // control codes).
                    [6] => debug!("ignoring DSR (CSI 6 n), that's the real terminal's job"),
                    _ => {}
                }
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

                    // Background Color Handling.
                    [49] => self.cursor_attrs.bgcolor = term::Color::Default,
                    [n] if 40 <= *n && *n < 48 => match (*n - 40).try_into() {
                        Ok(i) => self.cursor_attrs.bgcolor = term::Color::Idx(i),
                        Err(e) => warn!("out of bounds bgcolor idx (1): {e:?}"),
                    }
                    [n] if 100 <= *n && *n < 108 => match (*n - 92).try_into() {
                        Ok(i) => self.cursor_attrs.bgcolor = term::Color::Idx(i),
                        Err(e) => warn!("out of bounds bgcolor idx (2): {e:?}"),
                    }
                    [48] => match params_iter.next() {
                        Some([5]) => {
                            let n = param_or(&mut params_iter, 0);
                            match n.try_into() {
                                Ok(i) => self.cursor_attrs.bgcolor = term::Color::Idx(i),
                                Err(e) => warn!("out of bounds bgcolor idx (3): {e:?}"),
                            }
                        },
                        Some([2]) => {
                            // N.B. apparently some very old termianls have a "space id"
                            // param before the three color params. It might make sense
                            // to fully slurp the params here and if there are 4 provided
                            // drop the first to avoid shifting the rgb. I'm guessing this
                            // is so rare as to not matter though.
                            let r = param_or(&mut params_iter, 0);
                            let g = param_or(&mut params_iter, 0);
                            let b = param_or(&mut params_iter, 0);
                            if let (Ok(r), Ok(g), Ok(b)) = (r.try_into(), g.try_into(), b.try_into()) {
                                self.cursor_attrs.bgcolor = term::Color::Rgb(r, g, b);
                            } else {
                                warn!("out of bounds color codes for CSI 48 2 ... m");
                            }
                        },
                        _ => warn!("unhandled incomplete 'CSI 48 ... m'"),
                    },

                    // Foreground Color Handling.
                    [39] => self.cursor_attrs.fgcolor = term::Color::Default,
                    [n] if 30 <= *n && *n < 38 => match (*n - 30).try_into() {
                        Ok(i) => self.cursor_attrs.fgcolor = term::Color::Idx(i),
                        Err(e) => warn!("out of bounds fgcolor idx (1): {e:?}"),
                    }
                    [n] if 90 <= *n && *n < 98 => match (*n - 82).try_into() {
                        Ok(i) => self.cursor_attrs.fgcolor = term::Color::Idx(i),
                        Err(e) => warn!("out of bounds fgcolor idx (2): {e:?}"),
                    }
                    [38] => match params_iter.next() {
                        Some([5]) => {

                            let n = param_or(&mut params_iter, 0);
                            match n.try_into() {
                                Ok(i) => self.cursor_attrs.fgcolor = term::Color::Idx(i),
                                Err(e) => warn!("out of bounds fgcolor idx (3): {e:?}"),
                            }
                        },
                        Some([2]) => {
                            // N.B. apparently some very old termianls have a "space id"
                            // param before the three color params. It might make sense
                            // to fully slurp the params here and if there are 4 provided
                            // drop the first to avoid shifting the rgb. I'm guessing this
                            // is so rare as to not matter though.
                            let r = param_or(&mut params_iter, 0);
                            let g = param_or(&mut params_iter, 0);
                            let b = param_or(&mut params_iter, 0);
                            if let (Ok(r), Ok(g), Ok(b)) = (r.try_into(), g.try_into(), b.try_into()) {
                                self.cursor_attrs.fgcolor = term::Color::Rgb(r, g, b);
                            } else {
                                warn!("out of bounds color codes for CSI 38 2 ... m");
                            }
                        },
                        _ => warn!("unhandled incomplete 'CSI 38 ... m'"),
                    },

                    _ => warn!("unhandled 'CSI {param:?} m'"),
                }
            }
            // DECSTBM (Set Scroll Region)
            'r' => {
                let top = maybe_param(&mut params_iter);
                let bottom = maybe_param(&mut params_iter);

                let screen = self.screen_mut();
                screen.set_scroll_region(match (top, bottom) {
                    (None, None) => term::ScrollRegion::TrackSize,
                    (Some(t), None) => term::ScrollRegion::Window {
                        top: t.saturating_sub(1) as usize,
                        bottom: screen.size.height,
                    },
                    (None, Some(b)) => term::ScrollRegion::Window {
                        top: 0,
                        bottom: b as usize,
                    },
                    (Some(t), Some(b)) => term::ScrollRegion::Window {
                        top: t.saturating_sub(1) as usize,
                        bottom: b as usize,
                    }
                });
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

fn param_or<'params>(params: &mut vte::ParamsIter<'params>, default: u16) -> u16 {
    maybe_param(params).unwrap_or(default)
}

fn maybe_param<'params>(params: &mut vte::ParamsIter<'params>) -> Option<u16> {
    match params.next() {
        Some([0]) => None,
        Some([p]) => Some(*p),
        _ => None,
    }
}
