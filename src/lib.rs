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
    charset::{Charset, Charsets},
    screen::{SavedCursor, Screen},
    term::{
        AsTermInput, BlinkStyle, ControlCodes, FontWeight, FrameStyle, LinkTarget, OriginMode,
        Region, UnderlineStyle,
    },
};

use bitvec::{bitvec, vec::BitVec};
use smallvec::SmallVec;
use unicode_width::UnicodeWidthChar;

#[macro_use]
mod visibility;

#[macro_use]
mod log;

mod altscreen;
mod cell;
mod charset;
mod line;
mod screen;
mod scrollback;

#[cfg(not(feature = "unstable-internal-test"))]
mod term;

#[cfg(feature = "unstable-internal-test")]
pub mod term;

const MAX_TITLE_STACK_DEPTH: usize = 64;

/// The most bytes of free form OSC text, like a title or the working dir,
/// that we hold on to. vte buffers whole OSC strings however long they get,
/// and every one we store gets replayed on each reattach.
const MAX_OSC_TEXT_LEN: usize = 8192;

/// Limits on hyperlinks (OSC 8), which VTE uses too. Every cell of a link
/// carries its own copy of the target, so these add up.
const MAX_LINK_URL_LEN: usize = 2083;
const MAX_LINK_PARAMS_LEN: usize = 250;

/// The number of colors that OSC 4 can change: the 256 color palette, then
/// xterm's special colors (bold, underline, blink, reverse and italic),
/// which it numbers from 256 up.
const NUM_PALETTE_COLORS: usize = 261;

/// The longest color spec (like `rgb:ff/80/00`) we hold on to.
const MAX_COLOR_SPEC_LEN: usize = 128;

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
        let controls = term::control_codes();

        // Reset alone does not terminate active links, so before
        // we issue a reset, we'll issue an end link to fully
        // reset the link.
        controls.end_link.term_input_into(&mut buf);

        // We cannot know what state the terminal we are restoring into is
        // in. Often it is whatever state the last session left it in when
        // the connection dropped, so every mode the restore might replay
        // has to be switched off here, or it will outlive the app that set
        // it.
        //
        // Leaving the alt screen goes first. xterm restores the saved
        // cursor, along with its attrs, origin mode and charsets, when it
        // leaves the alt screen, which would undo any reset sent before.
        // Painting onto a stranded alt screen would also throw away every
        // line that scrolls off the top of it.
        controls.disable_alt_screen.term_input_into(&mut buf);
        controls.clear_attrs.term_input_into(&mut buf);

        // A leftover scroll region or origin mode would scroll the
        // contents we are about to paint. Clear them before homing the
        // cursor, since origin mode moves where home is.
        controls.unset_scroll_region.term_input_into(&mut buf);
        controls.disable_scroll_region_origin_mode.term_input_into(&mut buf);

        // Insert mode would shove the contents we paint to the right,
        // without auto-wrap long lines would pile up in the last column,
        // and a line drawing charset would turn letters into box parts.
        controls.disable_insert_mode.term_input_into(&mut buf);
        controls.enable_autowrap.term_input_into(&mut buf);
        controls.designate_g0_us_ascii.term_input_into(&mut buf);
        controls.designate_g1_us_ascii.term_input_into(&mut buf);
        controls.designate_g2_us_ascii.term_input_into(&mut buf);
        controls.designate_g3_us_ascii.term_input_into(&mut buf);
        buf.push(term::SHIFT_IN);

        // An app that hid the cursor and never got the chance to show it
        // again would leave it hidden for good.
        controls.show_cursor.term_input_into(&mut buf);

        // These change what the terminal sends rather than what it shows,
        // so leftovers turn keypresses, mouse movement, focus changes and
        // pastes into garbage input for whatever is running now.
        controls.disable_application_cursor_keys.term_input_into(&mut buf);
        controls.disable_application_keypad_mode.term_input_into(&mut buf);
        ControlCodes::dec_private_modes_reset(&MOUSE_MODES).term_input_into(&mut buf);
        controls.disable_report_focus.term_input_into(&mut buf);
        controls.disable_paste_mode.term_input_into(&mut buf);

        term::ControlCodes::cursor_position(1, 1).term_input_into(&mut buf);
        controls.clear_screen.term_input_into(&mut buf);
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

/// The mouse reporting modes we track, in the order they get replayed into
/// a restore buffer.
///
/// 1000, 1002 and 1003 select how much the client reports (press only, press
/// plus drag, or all motion). 1005, 1006, 1015 and 1016 select how those
/// reports are encoded. All of them change what the client writes to the pty,
/// so dropping them on reattach leaves the client and the application
/// disagreeing about the wire format.
const MOUSE_MODES: [u16; 7] = [1000, 1002, 1003, 1005, 1006, 1015, 1016];

/// The index into `State::mouse_modes` for a DEC private mode parameter.
fn mouse_mode_idx(param: &[u16]) -> Option<usize> {
    match param {
        [mode] => MOUSE_MODES.iter().position(|m| m == mode),
        _ => None,
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
    /// to know what directory to start new shells in), as the `file://` URL
    /// that `OSC 7` sets it to.
    working_dir: Option<SmallVec<[u8; 8]>>,
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
    /// Tracks application cursor keys mode (DECCKM), which changes what the
    /// arrow keys send. Controlled via `CSI ? 1 {h,l}`.
    application_cursor_keys_enabled: bool,
    /// Tracks application keypad mode (DECKPAM), which changes what the
    /// numeric keypad sends. Controlled via `ESC =` and `ESC >`.
    ///
    /// This is a different mode to DECCKM above, covering a different group
    /// of keys, so the two cannot share a flag.
    application_keypad_mode_enabled: bool,
    /// Tracks the mouse reporting modes listed in `MOUSE_MODES`, indexed
    /// in parallel with it.
    ///
    /// The tracking modes and the encoding modes are recorded independently
    /// and replayed exactly as the application set them, rather than being
    /// collapsed into a single effective mode, since the application will
    /// expect everything it set to still be in force after a reattach.
    mouse_modes: [bool; MOUSE_MODES.len()],
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
    /// Tracks auto-wrap mode (DECAWM). When it is off, chars written at the
    /// right edge of the screen overwrite the last column instead of wrapping
    /// onto the next line. Controlled via `CSI ? 7 {h,l}`.
    autowrap: bool,
    /// The charsets printed chars get translated through. Apps switch to
    /// the DEC special graphics set to draw lines and boxes.
    charsets: Charsets,
    /// Tab stop columns. By default, these are spaced 8 cols apart
    /// starting at col 9, but they can be directly manipulated by certain
    /// control codes as well.
    tabstops: BitVec,
    logger: log::Context,
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
            application_cursor_keys_enabled: false,
            application_keypad_mode_enabled: false,
            mouse_modes: [false; MOUSE_MODES.len()],
            report_focus: false,
            in_paste_mode: false,
            insert_mode: false,
            autowrap: true,
            charsets: Charsets::default(),
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
            ScreenMode::Alt => {
                // Restore the regular scrollback first so that after the user
                // exits their curses app, they can still see shell history.
                self.scrollback.dump_contents_into(buf, dump_region.clone());

                // Re-enable alt screen, then dump the contents. This is
                // not actually super important in practice because basically
                // every curses app respects SIGWINCH. We may even want to
                // consider exposing a knob to disable alt-screen dumping
                // since it might make things less flickery. Not worth doing
                // for now though.
                term::control_codes().enable_alt_screen.term_input_into(buf);

                // Switching screens does not clear the scroll region or
                // origin mode the scrollback restore just set, and neither
                // is per-screen in a real terminal, so we have to clear them
                // ourselves. This has to happen before the contents get
                // painted, since it is the paint that a stranded scroll
                // region corrupts.
                let homed = self.scrollback.dump_global_state_reset_into(buf);

                // Switching screens doesn't move the cursor either, but the
                // alt screen gets painted from the top left corner down.
                if !homed && !self.scrollback.dump_leaves_cursor_home() {
                    ControlCodes::cursor_position(1, 1).term_input_into(buf);
                }

                self.altscreen.dump_contents_into(buf, dump_region)
            }
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
            ControlCodes::set_working_dir(working_dir.clone()).term_input_into(buf);
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
        if self.application_cursor_keys_enabled {
            controls.enable_application_cursor_keys.term_input_into(buf);
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
        // This has to come after the screen contents, since restoring a
        // pending wrap relies on the terminal wrapping.
        if !self.autowrap {
            controls.disable_autowrap.term_input_into(buf);
        }
        for (idx, mode) in MOUSE_MODES.iter().enumerate() {
            if self.mouse_modes[idx] {
                ControlCodes::dec_private_modes_set(&[*mode]).term_input_into(buf);
            }
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

        // The screen holds chars that have already been through the
        // charsets, so painting it has to happen with plain ascii in place.
        // Switch the charsets over last, once there is nothing left to paint.
        self.charsets.dump_into(buf);
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

            if *color_spec != [b'?'] && color_spec.len() <= MAX_COLOR_SPEC_LEN {
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

    /// DECOM. Switching origin mode either way homes the cursor, which is the
    /// top of the scroll region once origin mode is on.
    fn set_origin_mode(&mut self, origin_mode: OriginMode) {
        let screen = self.screen_mut();
        screen.set_origin_mode(origin_mode);
        screen.set_cursor(term::Pos { row: 1, col: 1 });
        screen.clamp();
    }

    fn write_char_at_cursor(&mut self, cell: Cell) {
        let (insert_mode, autowrap) = (self.insert_mode, self.autowrap);
        if let Err(e) = self.screen_mut().write_at_cursor(cell, insert_mode, autowrap) {
            warn!(self.logger, "writing char at cursor: {:?}", e);
        }
    }

    /// Attach a zero width char to the grapheme cluster it modifies.
    ///
    /// A zero width char describes the glyph to its left, which lives in the
    /// cell the cursor most recently moved past. Wide chars leave padding
    /// cells behind them, so we skip back over those to reach the cell that
    /// actually owns the glyph. If there is no glyph to the left of the
    /// cursor there is nothing to modify and we drop the char, which is what
    /// xterm does.
    fn add_modifier_char(&mut self, c: char) {
        let screen = self.screen_mut();
        let width = screen.size.width;
        // With a wrap pending, the cursor stays on top of the cell it just
        // moved past rather than to the right of it.
        let col = if screen.pending_wrap {
            Some(screen.cursor.col)
        } else {
            screen.cursor.col.checked_sub(1)
        };
        let Some(mut col) = col else {
            return;
        };

        let Some(line) = screen.get_line_mut() else {
            return;
        };

        while line.get_cell(width, col).is_some_and(|cell| cell.is_wide_padding()) {
            match col.checked_sub(1) {
                Some(prev) => col = prev,
                None => return,
            }
        }

        match line.get_cell_mut(width, col) {
            // An empty cell renders as a space and has no cluster to extend.
            Some(cell) if !cell.is_empty() => cell.add_char(c),
            _ => {}
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
        // Store the char that gets displayed rather than the one that was
        // sent, so the dump doesn't depend on the charsets.
        let c = self.charsets.translate(c);

        match UnicodeWidthChar::width(c) {
            // Control chars have no printable form. vte routes the C0 set to
            // `execute`, but anything else that lands here would corrupt the
            // restore buffer if we stored it in a cell.
            None => {
                warn!(self.logger, "print: dropping control char {:?}", c);
                return;
            }
            // Combining marks, variation selectors and ZWJ modify the cluster
            // to their left instead of occupying a column of their own.
            Some(0) => {
                self.add_modifier_char(c);
                return;
            }
            Some(_) => {}
        }

        self.last_print_char = Some(c);
        let attrs = self.cursor_attrs.clone();
        self.write_char_at_cursor(Cell::new(c, attrs));
    }

    fn execute(&mut self, byte: u8) {
        self.last_print_char = None;
        trace!(self.logger, "execute: byte {}", byte);
        match byte {
            b'\n' => {
                let fill = Cell::blank(&self.cursor_attrs);
                self.screen_mut().linefeed(&fill);
            }
            b'\r' => {
                let screen = self.screen_mut();
                screen.cursor.col = 0;
                screen.pending_wrap = false;
            }
            b'\t' => {
                let mut col = self.screen().cursor.col;
                col += 1;
                while col < self.tabstops.len() && !self.tabstops.get(col).is_some_and(|b| *b) {
                    col += 1;
                }

                // A tab stops at the last column. If the cursor is already
                // there it does not move at all, and a pending wrap stays
                // pending.
                let screen = self.screen_mut();
                let col = std::cmp::min(col, screen.size.width.saturating_sub(1));
                if col != screen.cursor.col {
                    screen.cursor.col = col;
                    screen.clamp();
                }
            }
            b'\x08' => {
                // backspace
                let screen = self.screen_mut();
                screen.cursor.col = screen.cursor.col.saturating_sub(1);
                screen.pending_wrap = false;
            }
            // bell, ignore
            b'\x07' => {}
            // SO (Shift Out) and SI (Shift In) switch printed chars over to
            // the G1 charset and back to G0.
            term::SHIFT_OUT => self.charsets.lock_shift(1),
            term::SHIFT_IN => self.charsets.lock_shift(0),
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

        // vte splits the whole string on ';', but free form text like titles
        // and URLs can contain ';' too, so commands that end in such text
        // glue the rest of their params back together.
        let rest = params.get(1..).unwrap_or(&[]);
        let mut params_iter = rest.iter();
        match params.first() {
            // Title manipulation
            Some([b'0']) => if rest.is_empty() {
                warn!(self.logger, "OSC 0 with no title param");
            } else {
                let title = osc_text(rest);
                self.set_title(title.clone());
                self.set_icon_name(title);
            },
            Some([b'1']) => if rest.is_empty() {
                warn!(self.logger, "OSC 1 with no icon_name param");
            } else {
                self.set_icon_name(osc_text(rest));
            },
            Some([b'2']) => if rest.is_empty() {
                warn!(self.logger, "OSC 2 with no title param");
            } else {
                self.set_title(osc_text(rest));
            },

            // Color Palette
            Some([b'4']) => while let (Some(idx), Some(color_spec)) = (params_iter.next(), params_iter.next()) {
                if *color_spec == [b'?'] {
                    // If the program is querying for a color, we just ignore
                    // that control code. The real terminal is responsible for
                    // responding.
                    continue;
                }

                match palette_idx(idx) {
                    Some(i) if color_spec.len() <= MAX_COLOR_SPEC_LEN => {
                        self.palette_overrides.insert(i, color_spec.to_vec());
                    },
                    _ => warn!(self.logger, "OSC 4: ignoring color {:?} = {:?}", idx, color_spec),
                }
            },
            // With no indices, OSC 104 resets the whole palette.
            Some([b'1', b'0', b'4']) if rest.iter().all(|idx| idx.is_empty()) =>
                self.palette_overrides.clear(),
            Some([b'1', b'0', b'4']) => for idx in rest.iter().filter(|idx| !idx.is_empty()) {
                match palette_idx(idx) {
                    Some(i) => {
                        self.palette_overrides.remove(&i);
                    },
                    None => warn!(self.logger, "OSC 104: invalid idx '{:?}'", idx),
                }
            },

            // Working dir, which shells send as a `file://host/path` URL.
            // An empty one unsets it.
            Some([b'7']) => {
                let url = rest.join(&b';');
                self.working_dir = if url.is_empty() {
                    None
                } else if url.len() > MAX_OSC_TEXT_LEN {
                    warn!(self.logger, "OSC 7: ignoring over-long working dir");
                    None
                } else {
                    Some(url.into())
                };
            },

            // Links. Depending on params, OSC 8 both starts and ends links:
            // an empty URL ends the current link, whatever the params say.
            Some([b'8']) => {
                let url = rest.get(1..).unwrap_or(&[]).join(&b';');
                self.cursor_attrs.link_target = if url.is_empty() {
                    None
                } else if url.len() > MAX_LINK_URL_LEN {
                    warn!(self.logger, "OSC 8: ignoring over-long link");
                    None
                } else {
                    let params: &[u8] = match rest.first() {
                        Some(&params) if params.len() <= MAX_LINK_PARAMS_LEN => params,
                        _ => &[],
                    };
                    Some(LinkTarget { params: SmallVec::from_slice(params), url: url.into() })
                };
            },

            // Functional colors (foreground, background and whatnot).
            Some([b'1', x]) if x.is_ascii_digit() =>
                self.set_functional_color((*x - b'0') as usize, params_iter),
            // OSC 110 through OSC 119 reset them one at a time.
            Some([b'1', b'1', x]) if x.is_ascii_digit() =>
                self.functional_colors[(*x - b'0') as usize] = None,

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

        // Private markers (`?`, `>`, `<`, `=`) and intermediate bytes
        // (`SP`, `$`, `!`, ...) turn a final byte into a completely different
        // command, e.g. `CSI > 4 ; 2 m` sets key modifier options rather than
        // underline + faint and `CSI ? 1 ; 1 ; 0 S` is a graphics query rather
        // than a scroll. Every arm must check that the intermediates are the
        // ones it expects, or we will mangle the screen when an application
        // sends a sequence we don't otherwise know about.
        let plain = intermediates.is_empty();

        match action {
            // CUU (Cursor Up)
            'A' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
                screen.clamp();
            }
            // CUD (Cursor Down)
            'B' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.clamp();
            }
            // CUF (Cursor Forward)
            'C' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col += n;
                screen.clamp();
            }
            // CUF (Cursor Backwards)
            'D' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.col = screen.cursor.col.saturating_sub(n);
                screen.clamp();
            }
            // CNL (Cursor Next Line)
            'E' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row += n;
                screen.cursor.col = 0;
                screen.clamp();
            }
            // CPL (Cursor Prev Line)
            'F' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.cursor.row = screen.cursor.row.saturating_sub(n);
                screen.cursor.col = 0;
                screen.clamp();
            }
            // HPA (Horizontal Position Absolute, CSI n `)
            // CHA (Cursor Horizontal Absolute, CSI n G)
            '`' | 'G' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let n = n.saturating_sub(1); // translate to 0 indexing

                let screen = self.screen_mut();
                screen.cursor.col = n;
                screen.clamp();
            }
            // HVP (Horizontal and Vertical Position)
            // CUP (Cursor Set Position)
            'f' | 'H' if plain => {
                // parse the params and adjust 1 indexing to 0 indexing
                let row = param_or(&mut params_iter, 1) as usize;
                let col = param_or(&mut params_iter, 1) as usize;
                let screen = self.screen_mut();
                screen.set_cursor(term::Pos { row, col });
                screen.clamp();
            }
            // ED (Erase in Display)
            // DECSED (Selective Erase in Display, CSI ? n J). We don't track
            // the protected attribute (DECSCA), so this is the same as ED.
            'J' if plain || intermediates == [b'?'] => while let Some(code) = params_iter.next() {
                let fill = Cell::blank(&self.cursor_attrs);
                match code {
                    [] | [0] => self.screen_mut().erase_to_end(&fill),
                    [1] => self.screen_mut().erase_from_start(&fill),
                    [2] => self.screen_mut().erase(&fill),
                    // Only the main screen has scrollback, but it goes
                    // even if the alt screen is up.
                    [3] => self.scrollback.erase_scrollback(),
                    _ => warn!(self.logger, "unhandled 'CSI {:?} J'", code),
                }
            }
            // EL (Erase in Line)
            // DECSEL (Selective Erase in Line, CSI ? n K), see DECSED above.
            'K' if plain || intermediates == [b'?'] => while let Some(code) = params_iter.next() {
                let col = self.screen().cursor.col;
                let section = match code {
                    [] | [0] => line::Section::ToEnd(col),
                    [1] => line::Section::StartTo(col),
                    [2] => line::Section::Whole,
                    _ => {
                        warn!(self.logger, "unhandled 'CSI {:?} K'", code);
                        continue;
                    }
                };

                let fill = Cell::blank(&self.cursor_attrs);
                let screen = self.screen_mut();
                screen.pending_wrap = false;
                let width = screen.size.width;
                if let Some(l) = screen.line_to_erase(&fill) {
                    l.erase(width, section, &fill);
                }
            }
            // IL (Insert Line)
            'L' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let fill = Cell::blank(&self.cursor_attrs);
                self.screen_mut().insert_lines(n, &fill);
            }
            // DL (Delete Line)
            'M' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let fill = Cell::blank(&self.cursor_attrs);
                self.screen_mut().delete_lines(n, &fill);
            }
            // SU (Scroll Up)
            'S' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;
                let fill = Cell::blank(&self.cursor_attrs);
                self.screen_mut().scroll_up(n as usize, &fill);
            }
            // CTC (Cusor Tabulation Control)
            'W' if plain => {
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
            // DECST8C (Set Tab at Every 8 Columns, CSI ? 5 W)
            'W' if intermediates == [b'?'] => match param_or(&mut params_iter, 0) {
                5 => {
                    self.tabstops.fill(false);
                    let width = self.tabstops.len();
                    self.fill_tabstops(0, width);
                }
                code => warn!(self.logger, "unhandled 'CSI ? {:?} W'", code),
            }
            // CBT (Cursor Backward Tabulation)
            'Z' if plain => {
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
            //
            // xterm also has a five param form of `CSI T` that starts mouse
            // highlight tracking, which has nothing to do with scrolling.
            'T' if plain && params.len() <= 1 => {
                let n = param_or(&mut params_iter, 1) as usize;
                let fill = Cell::blank(&self.cursor_attrs);
                self.screen_mut().scroll_down(n as usize, &fill);
            }

            // ICH (Insert Character)
            '@' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;

                let fill = Cell::blank(&self.cursor_attrs);
                let screen = self.screen_mut();
                screen.pending_wrap = false;
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.line_to_erase(&fill) {
                    l.insert_character(width, col, n, &fill);
                }
            }
            // DCH (Delete Character)
            'P' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;

                let fill = Cell::blank(&self.cursor_attrs);
                let screen = self.screen_mut();
                screen.pending_wrap = false;
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.line_to_erase(&fill) {
                    l.delete_character(width, col, &fill, n);
                }
            }
            // ECH (Erase Character)
            'X' if plain => {
                let n = param_or(&mut params_iter, 1) as usize;

                let fill = Cell::blank(&self.cursor_attrs);
                let screen = self.screen_mut();
                screen.pending_wrap = false;
                let width = screen.size.width;
                let col = screen.cursor.col;
                if let Some(l) = screen.line_to_erase(&fill) {
                    l.erase_character(width, col, &fill, n);
                }
            }
            // REP (Repeat Preceding Character)
            'b' if plain => if let Some(c) = self.last_print_char {
                let n = param_or(&mut params_iter, 1) as usize;

                let cell = Cell::new(c, self.cursor_attrs.clone());
                for _ in 0..n {
                    self.write_char_at_cursor(cell.clone());
                }
            }
            'c' => debug!(self.logger, "CSI ... c - device attribute query"),
            // VPA (Vertical Line Position Absolute)
            'd' if plain => {
                let row = param_or(&mut params_iter, 1) as usize;
                let col = self.screen().cursor.col + 1;
                let screen = self.screen_mut();
                screen.set_cursor(term::Pos { row, col });
                screen.clamp();
            }

            // SCP (Save Cursor Position)
            's' if plain => {
                let screen = self.screen_mut();
                let cursor = screen.cursor.clone();
                screen.saved_cursor.pos = cursor;
            }
            // Window Title Operations
            't' if plain => while let Some(code) = params_iter.next() {
                match code {
                    [] | [0] => debug!(self.logger, "CSI 0 t - ignoring"),
                    [14, ..] => debug!(self.logger, "CSI 14 t - pixel size query"),
                    [16, ..] => debug!(self.logger, "CSI 16 t - cell size query"),
                    [18, ..] => debug!(self.logger, "CSI 18 t - term size query"),
                    [19, ..] => debug!(self.logger, "CSI 19 t - display size query"),
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
            'u' if plain => {
                let screen = self.screen_mut();
                screen.cursor = screen.saved_cursor.pos;
                screen.clamp();
            }

            // TBC (Tabulation Clear, CSI 3 g, CSI 0 g, CSI g)
            'g' if plain => {
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
                        }
                    }
                }
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1] => self.application_cursor_keys_enabled = true,
                        // 132 Column Mode (DECCOLM). Terminal dimensions are controlled
                        // by the client window/multiplexer, not child process escape sequences.
                        [3] => {},
                        // Smooth Scroll Mode (DECSCLM). Visual display scrolling timing
                        // is irrelevant in a headless virtual terminal.
                        [4] => {},
                        [6] => self.set_origin_mode(OriginMode::ScrollRegion),
                        [7] => self.autowrap = true,
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
                            if let Some(idx) = mouse_mode_idx(code) {
                                self.mouse_modes[idx] = true;
                            } else {
                                warn!(
                                    self.logger,
                                    "Unhandled CSI h command: CSI {:?} {:?} h",
                                    intermediates,
                                    params.iter().collect::<Vec<&[u16]>>()
                                );
                            }
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
                        }
                    }
                }
                [b'?'] => while let Some(code) = params_iter.next() {
                    match code {
                        [1] => self.application_cursor_keys_enabled = false,
                        // 80 Column Mode (DECCOLM). Terminal dimensions are controlled
                        // by the client window/multiplexer. Standard terminfo `is2` sends
                        // `\E[?3;4l` on startup; resetting column width or clearing the screen
                        // here would break sessions wider than 80 columns.
                        [3] => {},
                        // Jump Scroll Mode (DECSCLM). Visual display scrolling timing
                        // is irrelevant in a headless virtual terminal.
                        [4] => {},
                        [6] => self.set_origin_mode(OriginMode::Term),
                        [7] => self.autowrap = false,
                        [12] => self.cursor_blinking = Some(false),
                        [25] => self.cursor_hidden = true,
                        [1004] => self.report_focus = false,
                        [1049] => self.screen_mode = ScreenMode::Scrollback,
                        [2004] => self.in_paste_mode = false,
                        // Means "resume & flush visual rendering." We are
                        // not rendering anything visually so we don't care.
                        [2026] => {},
                        _ => {
                            if let Some(idx) = mouse_mode_idx(code) {
                                self.mouse_modes[idx] = false;
                            } else {
                                warn!(
                                    self.logger,
                                    "Unhandled CSI l command: CSI {:?} {:?} l",
                                    intermediates,
                                    params.iter().collect::<Vec<&[u16]>>()
                                );
                            }
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
            'm' if plain => while let Some(param) = params_iter.next() {
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
                    self.autowrap = true;
                    self.charsets = Charsets::default();

                    warn!(self.logger, "DECSTR only partially handled");
                }
                // DECRQM (DEC Request Mode, both the private and ANSI forms)
                [b'?', b'$'] | [b'$'] => {
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
            'r' if plain => {
                let height = self.screen().size.height;
                // Like in xterm, a bottom that is missing or off the screen
                // means the last row.
                let top = param_or(&mut params_iter, 1) as usize;
                let bottom = match maybe_param(&mut params_iter) {
                    Some(b) if (b as usize) <= height => b as usize,
                    _ => height,
                };

                // A region has to be at least two rows high, and anything
                // else gets ignored.
                if top < bottom {
                    let screen = self.screen_mut();
                    screen.set_scroll_region(if top == 1 && bottom == height {
                        term::ScrollRegion::TrackSize
                    } else {
                        term::ScrollRegion::Window { top: top - 1, bottom }
                    });
                    // Setting the region homes the cursor, which is the top
                    // of the region itself in origin mode.
                    screen.set_cursor(term::Pos { row: 1, col: 1 });
                    screen.clamp();
                }
            }

            _ => {
                warn!(
                    self.logger,
                    "unhandled CSI command: CSI {:?} {:?} {}",
                    intermediates,
                    params.iter().collect::<Vec<&[u16]>>(),
                    action
                );
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
                let charsets = self.charsets.clone();
                let screen = self.screen_mut();
                let pos = screen.cursor.clone();
                let pending_wrap = screen.pending_wrap;
                screen.saved_cursor = SavedCursor { pos, attrs, pending_wrap, charsets };
            }
            // restore cursor (ESC 8)
            ([], b'8') => {
                let screen = self.screen_mut();
                screen.cursor = screen.saved_cursor.pos;
                screen.pending_wrap = screen.saved_cursor.pending_wrap;
                let SavedCursor { attrs, charsets, .. } = screen.saved_cursor.clone();
                self.cursor_attrs = attrs;
                self.charsets = charsets;
            }
            // HTS (Horizontal Tabluation Set, ESC H)
            ([], b'H') => {
                let col = self.screen().cursor.col;
                self.tabstops.set(col, true);
            }
            // RI (Reverse Index)
            ([], b'M') => {
                let fill = Cell::blank(&self.cursor_attrs);
                let screen = self.screen_mut();
                screen.pending_wrap = false;
                let (scroll_top, _) =
                    screen.scroll_region(false).as_region(&screen.size).row_bounds();

                if screen.cursor.row == scroll_top {
                    screen.insert_lines(1, &fill);
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
                self.autowrap = true;
                self.charsets = Charsets::default();

                warn!(self.logger, "RIS only partially handled");
            }

            // DECKPAM / DECKPNM (application and numeric keypad mode)
            ([], b'=') => self.application_keypad_mode_enabled = true,
            ([], b'>') => self.application_keypad_mode_enabled = false,

            // SCS (Select Character Set) designates a set of 94 chars into
            // one of the G0-G3 slots.
            ([b'('], designator) => {
                self.charsets.designate(0, Charset::from_designator(designator))
            }
            ([b')'], designator) => {
                self.charsets.designate(1, Charset::from_designator(designator))
            }
            ([b'*'], designator) => {
                self.charsets.designate(2, Charset::from_designator(designator))
            }
            ([b'+'], designator) => {
                self.charsets.designate(3, Charset::from_designator(designator))
            }
            // Sets of 96 chars only make sense for the upper half of an 8 bit
            // charset, which utf-8 leaves no room for.
            ([b'-' | b'.' | b'/'], _) => debug!(self.logger, "ignoring 96 char set designation"),
            // LS2 / LS3 (Locking Shift 2 / 3)
            ([], b'n') => self.charsets.lock_shift(2),
            ([], b'o') => self.charsets.lock_shift(3),
            // SS2 / SS3 (Single Shift 2 / 3)
            ([], b'N') => self.charsets.single_shift(2),
            ([], b'O') => self.charsets.single_shift(3),
            // Select utf-8 (ESC % G) or the terminal's default encoding
            // (ESC % @). We only speak utf-8.
            ([b'%'], b'G' | b'@') => {}

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

/// Glues free form OSC text that vte split on ';' back together, cutting it
/// off (on a char boundary) if it gets unreasonably long.
fn osc_text(params: &[&[u8]]) -> SmallVec<[u8; 8]> {
    let mut text = params.join(&b';');
    if text.len() > MAX_OSC_TEXT_LEN {
        let mut end = MAX_OSC_TEXT_LEN;
        // Don't leave part of a UTF-8 sequence dangling off the end.
        while end > MAX_OSC_TEXT_LEN - 3 && (text[end] & 0xc0) == 0x80 {
            end -= 1;
        }
        text.truncate(end);
    }
    text.into()
}

/// Parses the index of a palette color, as used by OSC 4 and OSC 104.
fn palette_idx(idx: &[u8]) -> Option<usize> {
    let idx = std::str::from_utf8(idx).ok()?.parse::<usize>().ok()?;
    if idx < NUM_PALETTE_COLORS {
        Some(idx)
    } else {
        None
    }
}

const NONE_VEC: Option<Vec<u8>> = None;
