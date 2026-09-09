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
        AsTermInput, BlinkStyle, ControlCodes, FontWeight, FrameStyle, LinkTarget, OriginMode,
        Region, UnderlineStyle,
    },
};

use bitvec::{bitvec, vec::BitVec};
use smallvec::SmallVec;

#[macro_use]
mod visibility;

#[macro_use]
mod log;

mod altscreen;
mod cell;
mod line;
mod screen;
mod scrollback;

#[cfg(not(feature = "unstable-internal-test"))]
mod term;

#[cfg(feature = "unstable-internal-test")]
pub mod term;

const MAX_TITLE_STACK_DEPTH: usize = 64;

/// A representation of a terminal.
pub struct Term {
    parser: vte::Parser,
    state: State,
    logger: log::Context,
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
        Term {
            parser: vte::Parser::new(),
            state: State::new(scrollback_lines, size),
            logger: log::Context::None,
        }
    }

    /// Attach a tag to this term to help uniquely identify it
    /// in log and error messages. This is useful for applications
    /// which juggle multiple vterm instances at once.
    pub fn tag(&mut self, tag: String) {
        let logger = log::Context::Tag(tag);
        self.logger = logger.clone();
        self.state.set_logger(logger);
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

        self.state.resize(size);
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

        // Reset alone does not terminate active links, so before
        // we issue a reset, we'll issue an end link to fully
        // reset the link.
        term::control_codes().end_link.term_input_into(&mut buf);

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
    /// The last graphic char that was printed. This is used by REP
    /// (CSI Pn b).
    last_print_char: Option<char>,
    /// The current cursor attrs. These are shared between the scrollback
    /// and alt screens, which is why they are stored here rather than
    /// with the curors themsevles. If we think of the cursor as a paintbrush,
    /// these attrs are the color paint that it is currently holding.
    cursor_attrs: term::Attrs,
    /// The style for the cursor itself, not for the characters that
    /// the cursor is emitting.
    cursor_style: term::CursorStyle,
    /// The terminal title, as set by `OSC 0` and `OSC 2`.
    title_stack: Vec<SmallVec<[u8; 8]>>,
    /// The terminal icon name, as set by `OSC 0` and `OSC 1`.
    icon_name_stack: Vec<SmallVec<[u8; 8]>>,
    /// The terminal working directory (some terminal emulators use this
    /// to know what directory to start new shells in).
    working_dir: Option<WorkingDir>,
    /// A table mapping color index to a particular color spec.
    /// This is set by OSC 4. We use a tree for deterministic output
    /// to make testing easier. A hash would work just as well.
    palette_overrides: BTreeMap<usize, Vec<u8>>,
    /// Color overrides for things like foreground and background.
    /// These slots extend from OSC 10 to OSC 19.
    functional_colors: [Option<Vec<u8>>; 10],
    /// Tracks if the cursor is currently hidden. Controlled
    /// via the `CSI ? 25 {h,l}` codes.
    cursor_hidden: bool,
    /// Tracks cursor blinking mode. Controlled via `CSI ? 12 {h,l}`.
    cursor_blinking: Option<bool>,
    /// Tracks application keypad mode state. Controlled via
    /// `CSI ? 1 {h,l}`.
    application_keypad_mode_enabled: bool,
    /// When set, the underlying terminal is supposed to emit
    /// `\x1b[I` sentinals when the window gains focus. For our
    /// purposes we just need to know how to track and restore
    /// the state.
    ///
    /// Controlled via `CSI ? 1004 {h,l}`.
    report_focus: bool,
    /// Tracks paste mode. Controlled via `CSI ? 2004 {h,l}`.
    in_paste_mode: bool,
    /// Tracks insertion / replacement mode (IRM). Controlled via `CSI 4 {h,l}`.
    insert_mode: bool,
    /// Tab stop columns. By default, these are spaced 8 cols apart
    /// starting at col 9, but they can be directly manipulated by certain
    /// control codes as well.
    tabstops: BitVec,
    logger: log::Context,
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
        let mut st = State {
            scrollback: Screen::scrollback(scrollback_lines, size),
            altscreen: Screen::alt(size),
            screen_mode: ScreenMode::Scrollback,
            cursor_attrs: term::Attrs::default(),
            cursor_style: term::CursorStyle::Default,
            title_stack: vec![],
            icon_name_stack: vec![],
            working_dir: None,
            palette_overrides: BTreeMap::new(),
            functional_colors: [NONE_VEC; 10],
            cursor_hidden: false,
            cursor_blinking: None,
            application_keypad_mode_enabled: false,
            report_focus: false,
            in_paste_mode: false,
            insert_mode: false,
            tabstops: bitvec![0; size.width],
            last_print_char: None,
            logger: log::Context::None,
        };
        st.fill_tabstops(0, size.width);
        st
    }

    fn set_logger(&mut self, logger: log::Context) {
        self.scrollback.set_logger(logger.clone());
        self.altscreen.set_logger(logger.clone());
        self.logger = logger;
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

    fn resize(&mut self, size: Size) {
        let orig_len = self.tabstops.len();
        self.tabstops.resize(size.width, false);
        if size.width > orig_len {
            self.fill_tabstops(orig_len, size.width);
        }

        self.scrollback.resize(size);
        self.altscreen.resize(size);
    }

    /// Fill in the default tabstops within the given range.
    fn fill_tabstops(&mut self, start: usize, end: usize) {
        assert!(end <= self.tabstops.len());

        for i in start..end {
            if i > 0 && i % 8 == 0 {
                self.tabstops.set(i, true);
            }
        }
    }

    /// Dump the current tabstop state into the given control code
    /// vector. This is assumed to be right after a reset, so it will
    /// elide setting tabstops in the default position. The cursor
    /// MUST be in position (1, 1) when this routine is called.
    fn dump_tabstops(&self, buf: &mut Vec<u8>) {
        let controls = term::control_codes();
        if self.tabstops.len() > 8 && self.tabstops.not_any() {
            // If there are no tabstops, we just clobber them all as
            // a special case to help speed things up a bit.
            ControlCodes::tab_clear(Some(3)).term_input_into(buf);
            return;
        }

        let mut codes = vec![];
        for i in 0..self.tabstops.len() {
            let bit = self.tabstops.get(i).is_some_and(|b| *b);
            let i: u16 = match i.try_into() {
                Ok(i) => i,
                Err(e) => {
                    warn!(self.logger, "generating tabstop codes: index out of bounds: {:?}", e);
                    return;
                }
            };
            if i > 0 && i % 8 == 0 {
                // this is set by default
                if !bit {
                    codes.push(ControlCodes::cursor_position(1, i + 1));
                    codes.push(ControlCodes::tab_clear(None));
                }
            } else {
                // this is unset by default
                if bit {
                    codes.push(ControlCodes::cursor_position(1, i + 1));
                    codes.push(controls.horizontal_tab_set.clone());
                }
            }
        }

        if !codes.is_empty() {
            for code in codes.into_iter() {
                code.term_input_into(buf);
            }
            ControlCodes::cursor_position(1, 1).term_input_into(buf);
        }
    }

    fn dump_contents_into(&self, buf: &mut Vec<u8>, dump_region: ContentRegion) {
        self.dump_tabstops(buf);

        match self.screen_mode {
            ScreenMode::Scrollback => self.scrollback.dump_contents_into(buf, dump_region),
            ScreenMode::Alt => self.altscreen.dump_contents_into(buf, dump_region),
        }

        let controls = term::control_codes();

        // restore cursor attributes (the screen will have already restored our
        // position).
        controls.clear_attrs.term_input_into(buf);
        let mut cursor_attrs = self.cursor_attrs.clone();
        // Avoid starting a link even if there is one active in the
        // terminal state because the reconnecting terminal almost
        // certainly has forgotten it was in the middle of drawing
        // a link and will wind up creating a massive link if we
        // fully faithfully restore the cursor attr state..
        cursor_attrs.link_target = None;
        let codes = term::Attrs::default().transition_to(&cursor_attrs);
        for c in codes.into_iter() {
            c.term_input_into(buf);
        }
        if self.cursor_style != term::CursorStyle::Default {
            self.cursor_style.term_input_into(buf);
        }

        // Restore the title / icon name. Most terminals treat theses as the
        // same thing these days, but we'll go the extra mile and differentiate
        // rather than just always sending `OSC 0 ; <title> ST` in case there is
        // a terminal that actually makes a distinction.
        match (self.title_stack.last(), self.icon_name_stack.last()) {
            (Some(title), Some(icon_name)) if !title.is_empty() && title == icon_name => {
                ControlCodes::set_title_and_icon_name(title.clone()).term_input_into(buf)
            }
            (Some(title), Some(icon_name)) => {
                if !title.is_empty() {
                    ControlCodes::set_title(title.clone()).term_input_into(buf);
                }
                if !icon_name.is_empty() {
                    ControlCodes::set_icon_name(icon_name.clone()).term_input_into(buf);
                }
            }
            (Some(title), None) => {
                if !title.is_empty() {
                    ControlCodes::set_title(title.clone()).term_input_into(buf);
                }
            }
            (None, Some(icon_name)) => {
                if !icon_name.is_empty() {
                    ControlCodes::set_icon_name(icon_name.clone()).term_input_into(buf);
                }
            }
            (None, None) => {}
        }

        if let Some(working_dir) = &self.working_dir {
            ControlCodes::set_working_dir(working_dir.host.clone(), working_dir.dir.clone())
                .term_input_into(buf);
        }

        if !self.palette_overrides.is_empty() {
            ControlCodes::set_color_indices(
                self.palette_overrides
                    .iter()
                    .map(|(idx, color_spec)| (*idx, SmallVec::from(color_spec.as_slice()))),
            )
            .term_input_into(buf);
        }

        if self.cursor_hidden {
            controls.hide_cursor.term_input_into(buf);
        }
        if let Some(blinking) = self.cursor_blinking {
            if blinking {
                controls.enable_cursor_blink.term_input_into(buf);
            } else {
                controls.disable_cursor_blink.term_input_into(buf);
            }
        }
        if self.application_keypad_mode_enabled {
            controls.enable_application_keypad_mode.term_input_into(buf);
        }
        if self.report_focus {
            controls.enable_report_focus.term_input_into(buf);
        }
        if self.in_paste_mode {
            controls.enable_paste_mode.term_input_into(buf);
        }
        if self.insert_mode {
            controls.enable_insert_mode.term_input_into(buf);
        }

        // Generate fused functional color commands from any runs in the
        // functional colors table.
        let mut functional_color_idx = 0;
        while functional_color_idx < self.functional_colors.len() {
            if let Some(color_spec) = &self.functional_colors[functional_color_idx] {
                let start_idx = functional_color_idx;
                let mut color_specs = vec![color_spec.as_slice()];

                functional_color_idx += 1;
                while functional_color_idx < self.functional_colors.len() {
                    if let Some(s) = &self.functional_colors[functional_color_idx] {
                        color_specs.push(s.as_slice());
                    } else {
                        break;
                    }
                    functional_color_idx += 1;
                }

                ControlCodes::set_functional_color(start_idx, color_specs).term_input_into(buf);
            }

            functional_color_idx += 1;
        }
    }

    /// Set a run within the functional colors table starting at the given
    /// index. This implements OSC 10 through OSC 19.
    fn set_functional_color<'a, I>(&mut self, mut idx: usize, mut params_iter: I)
    where
        I: Iterator<Item = &'a &'a [u8]>,
    {
        while let Some(color_spec) = params_iter.next() {
            if idx >= self.functional_colors.len() {
                return;
            }

            if *color_spec != [b'?'] {
                self.functional_colors[idx] = Some(Vec::from(*color_spec));
            }

            idx += 1;
        }
    }

    fn set_title(&mut self, title: SmallVec<[u8; 8]>) {
        if let Some(top) = self.title_stack.last_mut() {
            *top = title;
        } else {
            self.title_stack.push(title);
        }
    }

    fn set_icon_name(&mut self, icon_name: SmallVec<[u8; 8]>) {
        if let Some(top) = self.icon_name_stack.last_mut() {
            *top = icon_name;
        } else {
            self.icon_name_stack.push(icon_name);
        }
    }

    fn write_char_at_cursor(&mut self, cell: Cell) {
        let insert_mode = self.insert_mode;
        let screen = self.screen_mut();
        screen.snap_to_bottom();

        // In insert mode (ECMA-48 IRM), incoming characters do not overwrite
        // existing text under the cursor. Instead, existing characters are
        // shifted to the right, dropping any characters that spill past the
        // terminal width.
        //
        // `Line::insert_character` does not write `cell` itself; it inserts
        // blank cells to make room for `cell.width()`. The subsequent
        // call to `screen.write_at_cursor(cell)` then writes the actual
        // character into the newly opened space at the cursor position
        // and advances the cursor.
        if insert_mode {
            let width = screen.size.width;
            let col = screen.cursor.col;
            if col < width {
                if let Some(l) = screen.get_line_mut() {
                    l.insert_character(width, col, cell.width() as usize);
                }
            }
        }

        if let Err(e) = screen.write_at_cursor(cell) {
            warn!(self.logger, "writing char at cursor: {:?}", e);
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
        trace!(self.logger, "print: {}", c);
        self.last_print_char = Some(c);
        let attrs = self.cursor_attrs.clone();
        self.write_char_at_cursor(Cell::new(c, attrs));
    }

    fn execute(&mut self, byte: u8) {
        self.last_print_char = None;
        trace!(self.logger, "execute: byte {}", byte);
        match byte {
            b'\n' => {
                let screen = self.screen_mut();
                let (scroll_top, scroll_bottom) =
                    screen.scroll_region(false).as_region(&screen.size).row_bounds();
                let within_scroll =
                    scroll_top <= screen.cursor.row && screen.cursor.row < scroll_bottom;
                screen.cursor.row += 1;
                if within_scroll {
                    if screen.cursor.row >= scroll_bottom {
                        screen.scroll_down(1);
                        screen.cursor.row -= 1;
                    }
                } else {
                    screen.clamp();
                }
            }
            b'\r' => self.screen_mut().cursor.col = 0,
            b'\t' => {
                let mut col = self.screen().cursor.col;
                col += 1;
                while col < self.tabstops.len() && !self.tabstops.get(col).is_some_and(|b| *b) {
                    col += 1;
                }

                let screen = self.screen_mut();
                screen.cursor.col = col;
                screen.clamp();
            }
            b'\x08' => {
                // backspace
                let screen = self.screen_mut();
                screen.cursor.col = screen.cursor.col.saturating_sub(1);
            }
            // bell, ignore
            b'\x07' => {}
            _ => {
                warn!(self.logger, "execute: unhandled byte {}", byte);
            }
        }
    }

    fn hook(&mut self, _params: &vte::Params, intermediates: &[u8], ignore: bool, action: char) {
        self.last_print_char = None;
        debug!(
            self.logger,
            "unhandled hook{}: {:?} {}",
            if ignore { " (ignored)" } else { "" },
            intermediates,
            action
        );
    }

    fn put(&mut self, byte: u8) {
        trace!(self.logger, "unhandled put: {}", byte);
        self.last_print_char = None;
    }

    fn unhook(&mut self) {
        debug!(self.logger, "unhandled unhook");
        self.last_print_char = None;
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
        trace!(self.logger, "osc_dispatch: {:?}", params);
        self.last_print_char = None;

        let mut params_iter = params.iter();
        match params_iter.next() {
            // Title manipulation
            Some([b'0']) => if let Some(title) = params_iter.next() {
                let title: SmallVec<[u8; 8]> = title.to_vec().into();
                self.set_title(title.clone());
                self.set_icon_name(title);
            } else {
                warn!(self.logger, "OSC 0 with no title param");
            },
            Some([b'1']) => if let Some(icon_name) = params_iter.next() {
                let icon_name: SmallVec<[u8; 8]> = icon_name.to_vec().into();
                self.set_icon_name(icon_name);
            } else {
                warn!(self.logger, "OSC 1 with no icon_name param");
            },
            Some([b'2']) => if let Some(title) = params_iter.next() {
                let title: SmallVec<[u8; 8]> = title.to_vec().into();
                self.set_title(title);
            } else {
                warn!(self.logger, "OSC 2 with no title param");
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
                        Err(e) => warn!(self.logger, "OSC 4: idx is an invalid number '{}': {}", s, e),
                    },
                    Err(e) => warn!(self.logger, "OSC 4: invalid idx '{:?}': {}", idx, e),
                }
            },
            Some([b'1', b'0', b'4']) => while let Some(idx) = params_iter.next() {
                match std::str::from_utf8(idx) {
                    Ok(s) => match s.parse::<usize>() {
                        Ok(i) => {
                            self.palette_overrides.remove(&i);
                        },
                        Err(e) => warn!(self.logger, "OSC 104: idx is an invalid number '{}': {}", s, e),
                    },
                    Err(e) => warn!(self.logger, "OSC 104: invalid idx '{:?}': {}", idx, e),
                }
            },

            // Working dir
            Some([b'7']) => if let (Some(host), Some(dir)) = (params_iter.next(), params_iter.next()) {
                self.working_dir = Some(WorkingDir {
                    host: host.to_vec().into(),
                    dir: dir.to_vec().into(),
                });
            } else {
                warn!(self.logger, "OSC 7 with fewer than 2 params");
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
            },

            // Functional colors (foreground, background and whatnot).
            Some([b'1', x]) if b'0' <= *x && *x <= b'9' =>
                self.set_functional_color((*x - b'0') as usize, params_iter),

            Some([b'5', b'2']) => debug!(self.logger, "ignoring OSC 52 (clipboard)"),
            Some([b'9']) => debug!(self.logger, "ignoring OSC 9 (desktop notification)"),
            Some([b'7', b'7', b'7']) => debug!(self.logger, "ignoring OSC 777"),
            Some([b'1', b'3', b'3']) => debug!(self.logger, "ignoring OSC 133 (iterm2 marks)"),
            Some([b'3', b'0', b'0', b'8']) => debug!(self.logger, "ignoring OSC 3008 (systemd context signaling)"),

            _ => warn!(self.logger, "unhandled 'OSC {:?} {}'", params, if bell_terminated {
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
            warn!(self.logger, "malformed CSI seq");
            return;
        }
        if tracing::enabled!(tracing::Level::TRACE) {
            trace!(self.logger, "csi_dispatch: intermediates={:?} params={:?} {}",
                intermediates, params.iter().collect::<Vec<_>>(), action);
        }

        let mut params_iter = params.iter();

        if action != 'b' || !intermediates.is_empty() {
            self.last_print_char = None;
        }

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
            // HPA (Horizontal Position Absolute, CSI n `)
            // CHA (Cursor Horizontal Absolute, CSI n G)
            '`' | 'G' => {
                let n = param_or(&mut params_iter, 1) as usize;
                let n = n.saturating_sub(1); // translate to 0 indexing

                let screen = self.screen_mut();
                screen.cursor.col = n;
                screen.clamp();
            }
            // HVP (Horizontal and Vertical Position)
            // CUP (Cursor Set Position)
            'f' | 'H' => {
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
                    _ => warn!(self.logger, "unhandled 'CSI {:?} J'", code),
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
                    _ => warn!(self.logger, "unhandled 'CSI {:?} K'", code),
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
            // CTC (Cusor Tabulation Control)
            'W' => {
                let code = param_or(&mut params_iter, 0) as usize;
                match code {
                    0 => {
                        let col = self.screen().cursor.col;
                        self.tabstops.set(col, true);
                    },
                    2 => {
                        let col = self.screen().cursor.col;
                        self.tabstops.set(col, false);
                    }
                    5 => {
                        self.tabstops.fill(false);
                    }
                    _ => warn!(self.logger, "unhandled 'CSI {:?} W'", code),
                }
            }
            // CBT (Cursor Backward Tabulation)
            'Z' if intermediates.is_empty() => {
                let n = param_or(&mut params_iter, 1) as usize;
                let mut col = self.screen().cursor.col;
                for _ in 0..n {
                    if col == 0 {
                        break;
                    }
                    col -= 1;
                    while col > 0 && !self.tabstops.get(col).is_some_and(|b| *b) {
                        col -= 1;
                    }
                }

                let screen = self.screen_mut();
                screen.cursor.col = col;
                screen.clamp();
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
            // ECH (Erase Character)
            'X' => {
                let n = param_or(&mut params_iter, 1) as usize;

                let attrs = self.cursor_attrs.clone();

                let screen = self.screen_mut();
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.get_line_mut() {
                    l.erase_character(width, col, &attrs, n);
                }
            }
            // REP (Repeat Preceding Character)
            'b' if intermediates.is_empty() => if let Some(c) = self.last_print_char {
                let n = param_or(&mut params_iter, 1) as usize;

                let cell = Cell::new(c, self.cursor_attrs.clone());
                for _ in 0..n {
                    self.write_char_at_cursor(cell.clone());
                }
            }
            'c' => debug!(self.logger, "CSI ... c - device attribute query"),
            // VPA (Vertical Line Position Absolute)
            'd' => {
                let row = param_or(&mut params_iter, 1) as usize;
                let col = self.screen().cursor.col + 1;
                let screen = self.screen_mut();
                screen.set_cursor(term::Pos { row, col });
                screen.clamp();
            }

            // SCP (Save Cursor Position)
            's' => {
                let screen = self.screen_mut();
                let cursor = screen.cursor.clone();
                screen.saved_cursor.pos = cursor;
            }
            // Window Title Operations
            't' => while let Some(code) = params_iter.next() {
                match code {
                    [14] => debug!(self.logger, "CSI 14 t - pixel size query"),
                    [16] => debug!(self.logger, "CSI 16 t - cell size query"),
                    [18] => debug!(self.logger, "CSI 18 t - term size query"),
                    [19] => debug!(self.logger, "CSI 19 t - display size query"),
                    [22] => {
                        let code = param_or(&mut params_iter, 0) as usize;
                        if (code == 0 || code == 1) && self.icon_name_stack.len() < MAX_TITLE_STACK_DEPTH {
                            if let Some(icon_name) = self.icon_name_stack.last().cloned() {
                                self.icon_name_stack.push(icon_name);
                            } else {
                                self.icon_name_stack.push(SmallVec::new());
                            }
                        }

                        if (code == 0 || code == 2) && self.title_stack.len() < MAX_TITLE_STACK_DEPTH {
                            if let Some(title) = self.title_stack.last().cloned() {
                                self.title_stack.push(title);
                            } else {
                                self.title_stack.push(SmallVec::new());
                            }
                        }
                    }
                    [23] => {
                        let code = param_or(&mut params_iter, 0) as usize;
                        if code == 0 || code == 1 {
                            self.icon_name_stack.pop();
                        }

                        if code == 0 || code == 2 {
                            self.title_stack.pop();
                        }
                    }
                    _ => warn!(self.logger, "unhandled CSI ... {:?} t", code),
                }
            }
            // RCP (Restore Cursor Position)
            'u' => {
                let screen = self.screen_mut();
                screen.cursor = screen.saved_cursor.pos;
                screen.clamp();
            }

            // TBC (Tabulation Clear, CSI 3 g, CSI 0 g, CSI g)
            'g' => {
                let code = param_or(&mut params_iter, 0) as usize;
                match code {
                    0 => {
                        let col = self.screen().cursor.col;
                        self.tabstops.set(col, false);
                    },
                    3 => {
                        self.tabstops.fill(false);
                    }
                    _ => warn!(self.logger, "unhandled 'CSI {:?} g'", code),
                }
            }

            'h' => match intermediates {
                [] => while let Some(code) = params_iter.next() {
                    match code {
                        [4] => self.insert_mode = true,
                        _ => {
                            warn!(
                                self.logger,
                                "Unhandled CSI h command: CSI {:?} {:?} h",
                                intermediates,
                                params.iter().collect::<Vec<&[u16]>>()
                            );
                            return;
                        }
                    }
                }
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1] => self.application_keypad_mode_enabled = true,
                        [4] => {},
                        [6] => self.screen_mut().set_origin_mode(OriginMode::ScrollRegion),
                        [12] => self.cursor_blinking = Some(true),
                        [25] => self.cursor_hidden = false,
                        [1004] => self.report_focus = true,
                        // enable alt screen
                        [1049] => {
                            // The alt-screen gets reset upon entry, so we need to
                            // clobber it here.
                            self.altscreen = Screen::alt(self.altscreen.size);
                            self.screen_mode = ScreenMode::Alt;
                        }
                        [2004] => self.in_paste_mode = true,
                        // Means "pause visual rendering." We are not rendering
                        // anything visually so we don't care.
                        [2026] => {},

                        _ => {
                            warn!(
                                self.logger,
                                "Unhandled CSI h command: CSI {:?} {:?} h",
                                intermediates,
                                params.iter().collect::<Vec<&[u16]>>()
                            );
                            return;
                        }
                    }
                }
                _ => warn!(
                    self.logger,
                    "Unhandled CSI h command: CSI {:?} {:?} h",
                    intermediates,
                    params.iter().collect::<Vec<&[u16]>>()
                ),
            }
            'l' => match intermediates {
                [] => while let Some(code) = params_iter.next() {
                    match code {
                        [4] => self.insert_mode = false,
                        _ => {
                            warn!(
                                self.logger,
                                "Unhandled CSI l command: CSI {:?} {:?} l",
                                intermediates,
                                params.iter().collect::<Vec<&[u16]>>()
                            );
                            return;
                        }
                    }
                }
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1] => self.application_keypad_mode_enabled = false,
                        [4] => {},
                        [6] => self.screen_mut().set_origin_mode(OriginMode::Term),
                        [12] => self.cursor_blinking = Some(false),
                        [25] => self.cursor_hidden = true,
                        [1004] => self.report_focus = false,
                        [1049] => self.screen_mode = ScreenMode::Scrollback,
                        [2004] => self.in_paste_mode = false,
                        // Means "resume & flush visual rendering." We are
                        // not rendering anything visually so we don't care.
                        [2026] => {},
                        _ => {
                            warn!(
                                self.logger,
                                "Unhandled CSI l command: CSI {:?} {:?} l",
                                intermediates,
                                params.iter().collect::<Vec<&[u16]>>()
                            );
                            return;
                        }
                    }
                }
                _ => warn!(
                    self.logger,
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
                    [6] => debug!(self.logger, "ignoring DSR (CSI 6 n), that's the real terminal's job"),
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

                    // Underline Color Handling.
                    [59] => self.cursor_attrs.underline_color = term::Color::Default,
                    param if !param.is_empty() && param[0] == 58 => {
                        match parse_extended_color(param, &mut params_iter) {
                            Some(color) => self.cursor_attrs.underline_color = color,
                            None => warn!(self.logger, "unhandled incomplete 'CSI 58 ... m'"),
                        }
                    }

                    // Background Color Handling.
                    [49] => self.cursor_attrs.bgcolor = term::Color::Default,
                    [n] if 40 <= *n && *n < 48 => match (*n - 40).try_into() {
                        Ok(i) => self.cursor_attrs.bgcolor = term::Color::Idx(i),
                        Err(e) => warn!(self.logger, "out of bounds bgcolor idx (1): {:?}", e),
                    }
                    [n] if 100 <= *n && *n < 108 => match (*n - 92).try_into() {
                        Ok(i) => self.cursor_attrs.bgcolor = term::Color::Idx(i),
                        Err(e) => warn!(self.logger, "out of bounds bgcolor idx (2): {:?}", e),
                    }
                    param if !param.is_empty() && param[0] == 48 => {
                        match parse_extended_color(param, &mut params_iter) {
                            Some(color) => self.cursor_attrs.bgcolor = color,
                            None => warn!(self.logger, "unhandled incomplete 'CSI 48 ... m'"),
                        }
                    }

                    // Foreground Color Handling.
                    [39] => self.cursor_attrs.fgcolor = term::Color::Default,
                    [n] if 30 <= *n && *n < 38 => match (*n - 30).try_into() {
                        Ok(i) => self.cursor_attrs.fgcolor = term::Color::Idx(i),
                        Err(e) => warn!(self.logger, "out of bounds fgcolor idx (1): {:?}", e),
                    }
                    [n] if 90 <= *n && *n < 98 => match (*n - 82).try_into() {
                        Ok(i) => self.cursor_attrs.fgcolor = term::Color::Idx(i),
                        Err(e) => warn!(self.logger, "out of bounds fgcolor idx (2): {:?}", e),
                    }
                    param if !param.is_empty() && param[0] == 38 => {
                        match parse_extended_color(param, &mut params_iter) {
                            Some(color) => self.cursor_attrs.fgcolor = color,
                            None => warn!(self.logger, "unhandled incomplete 'CSI 38 ... m'"),
                        }
                    }

                    _ => warn!(self.logger, "unhandled 'CSI {:?} m'", param),
                }
            }
            'p' => match intermediates {
                // DECSTR (DEC Soft Terminal Reset)
                [b'!'] => {
                    self.tabstops.fill(false);
                    let width = self.screen().size.width;
                    self.fill_tabstops(0, width);
                    self.cursor_style = term::CursorStyle::Default;
                    self.cursor_attrs = term::Attrs::default();
                    self.cursor_blinking = None;
                    self.insert_mode = false;

                    warn!(self.logger, "DECSTR only partially handled");
                }
                // DECRQM (DEC Request Mode Private)
                [b'?', b'$'] => {
                    // TODO(#4): actuate query state machine.
                    //
                    // In the future, we'll want to expose an API that
                    // allows the embedding application to stream the
                    // response of the underlying terminal so that we
                    // can sniff its response and figure out what capabilities
                    // it supports. This is the key to handling kitty's
                    // im-such-a-special-boy escape sequences for example
                    // (half the reason to write this crate), but for the
                    // moment we just suppress the warning log and convert
                    // to a debug log.
                    debug!(self.logger, "ignoring DECRQM query: params={:?}", params.iter().collect::<Vec<_>>());
                }
                _ => warn!(
                    self.logger,
                    "Unhandled CSI p command: CSI {:?} {:?} p",
                    intermediates,
                    params.iter().collect::<Vec<&[u16]>>()
                ),
            },
            // DECSCUSR (Set Cursor Style / Shape)
            'q' if intermediates == [b' '] => {
                let code = param_or(&mut params_iter, 0) as usize;
                match term::CursorStyle::try_from(code) {
                    Ok(style) => self.cursor_style = style,
                    Err(e) => warn!(self.logger, "parsing cursor style: {:?}", e),
                }
            },
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
                warn!(self.logger, "unhandled action {}", action);
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            warn!(self.logger, "malformed ESC seq");
            return;
        }
        trace!(self.logger, "esc_dispatch: {}", byte);
        self.last_print_char = None;

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
            // HTS (Horizontal Tabluation Set, ESC H)
            ([], b'H') => {
                let col = self.screen().cursor.col;
                self.tabstops.set(col, true);
            }
            // RI (Reverse Index)
            ([], b'M') => {
                let screen = self.screen_mut();
                let (scroll_top, _) =
                    screen.scroll_region(false).as_region(&screen.size).row_bounds();

                if screen.cursor.row == scroll_top {
                    screen.insert_lines(1);
                } else if screen.cursor.row > 0 {
                    screen.cursor.row -= 1;
                }
            }
            // RIS (Reset to Initial State)
            ([], b'c') => {
                self.tabstops.fill(false);
                let width = self.screen().size.width;
                self.fill_tabstops(0, width);
                self.cursor_style = term::CursorStyle::Default;
                self.cursor_attrs = term::Attrs::default();
                self.cursor_blinking = None;
                self.insert_mode = false;

                warn!(self.logger, "RIS only partially handled");
            }

            ([], b'=') => self.application_keypad_mode_enabled = true,
            ([], b'>') => self.application_keypad_mode_enabled = false,

            // Designates US-ASCII or UK-ASCII as a G0-G3 character set. We handle
            // utf-8, which is a superset of ascii, so this is a no-op.
            ([b'(' | b')' | b'*' | b'+'], b'B' | b'A') => {}

            // OSC terminators that get sent to the esc handler as well,
            // we can ignore them.
            ([], 92) => {}

            _ => warn!(self.logger, "unhandled ESC seq ({:?}, {})", intermediates, byte),
        }
    }

    fn terminated(&self) -> bool {
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

fn parse_extended_color<'params>(
    first_param: &[u16],
    params_iter: &mut vte::ParamsIter<'params>,
) -> Option<term::Color> {
    if first_param.len() > 1 {
        // Colon-delimited subparameters: e.g. [58, 2, r, g, b] or [58, 2,
        // space_id, r, g, b]
        match first_param[1] {
            5 => {
                if first_param.len() >= 3 {
                    let idx = first_param[2].try_into().ok()?;
                    Some(term::Color::Idx(idx))
                } else {
                    None
                }
            }
            2 => {
                if first_param.len() == 5 {
                    let r = first_param[2].try_into().ok()?;
                    let g = first_param[3].try_into().ok()?;
                    let b = first_param[4].try_into().ok()?;
                    Some(term::Color::Rgb(r, g, b))
                } else if first_param.len() >= 6 {
                    // Includes color space ID (e.g. 58:2:0:r:g:b or
                    // 58:2::r:g:b)
                    let r = first_param[3].try_into().ok()?;
                    let g = first_param[4].try_into().ok()?;
                    let b = first_param[5].try_into().ok()?;
                    Some(term::Color::Rgb(r, g, b))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        // Semicolon-delimited parameters: e.g. [58], [2], [r], [g], [b]
        match params_iter.next() {
            Some([5]) => {
                let n = param_or(params_iter, 0);
                let idx = n.try_into().ok()?;
                Some(term::Color::Idx(idx))
            }
            Some([2]) => {
                let r = param_or(params_iter, 0);
                let g = param_or(params_iter, 0);
                let b = param_or(params_iter, 0);
                let r = r.try_into().ok()?;
                let g = g.try_into().ok()?;
                let b = b.try_into().ok()?;
                Some(term::Color::Rgb(r, g, b))
            }
            _ => None,
        }
    }
}

const NONE_VEC: Option<Vec<u8>> = None;
