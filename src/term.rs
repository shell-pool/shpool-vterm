// The MIT License (MIT)
//
// Copyright (c) 2016 Jesse Luehrs
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::sync::OnceLock;

// TODO: read all of this from terminfo.
// https://github.com/meh/rust-terminfo/issues/41#issuecomment-3693863276
// might be a good place to start (look into the terminfo-lean crate for
// better licencing).

/// A position within the terminal. Generally, this refers to a grid
/// mode view of the terminal, not the underlying logical lines mode
/// that we actually store the data in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Pos {
    /// Ensure that the cursor is within the given region
    /// by moving to the closest edge if it is out of bounds.
    pub fn clamp_to<R>(&mut self, region: R)
    where
        R: Region,
    {
        let (low_row, high_row) = region.row_bounds();
        if self.row < low_row {
            self.row = low_row;
        }
        if self.row >= high_row {
            self.row = high_row - 1;
        }

        let (low_col, high_col) = region.col_bounds();
        if self.col < low_col {
            self.col = low_col;
        }
        if self.col >= high_col {
            self.col = high_col - 1;
        }
    }
}

pub trait Region {
    /// [low, high) bounds on valid rows for this region.
    fn row_bounds(&self) -> (usize, usize);
    /// [low, high) bounds on valid cols for this region.
    fn col_bounds(&self) -> (usize, usize);
}

impl Region for crate::Size {
    fn row_bounds(&self) -> (usize, usize) {
        (0, self.height)
    }
    fn col_bounds(&self) -> (usize, usize) {
        (0, self.width)
    }
}

pub trait AsTermInput {
    fn term_input_into(&self, buf: &mut Vec<u8>);
}

#[derive(Debug)]
#[must_use = "this struct does nothing unless you call term_input_into"]
pub struct Raw {
    inner: Vec<u8>,
}

#[allow(dead_code)]
impl Raw {
    pub fn new(inner: Vec<u8>) -> Self {
        Raw { inner }
    }
}

impl std::convert::From<&str> for Raw {
    fn from(value: &str) -> Self {
        Raw { inner: Vec::from(value.as_bytes()) }
    }
}

impl AsTermInput for Raw {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.inner.as_slice());
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[must_use = "this struct does nothing unless you call term_input_into"]
pub struct Attrs {
    pub fgcolor: Option<Color>,
    pub bgcolor: Option<Color>,
    pub font_weight: Option<FontWeight>,
    pub italic: bool,
    pub underline: Option<UnderlineStyle>,
    pub inverse: bool,
    pub blink: Option<BlinkStyle>,
    pub conceal: bool,
    pub strikethrough: bool,
    pub framed: Option<FrameStyle>,
    pub overline: bool,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UnderlineStyle {
    Single,
    Double,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FontWeight {
    Bold,
    Faint,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BlinkStyle {
    Slow,
    Rapid,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FrameStyle {
    Frame,
    Circle,
}

impl std::fmt::Display for Attrs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = &self.fgcolor {
            write!(f, "<FG {c:?}>")?;
        }

        if let Some(c) = &self.bgcolor {
            write!(f, "<BG {c:?}>")?;
        }

        match self.font_weight {
            Some(FontWeight::Bold) => write!(f, "b")?,
            Some(FontWeight::Faint) => write!(f, "f")?,
            _ => {}
        }
        if self.italic {
            write!(f, "i")?;
        }
        match self.underline {
            Some(UnderlineStyle::Single) => write!(f, "_")?,
            Some(UnderlineStyle::Double) => write!(f, "‗")?,
            _ => {}
        }
        if self.inverse {
            write!(f, "<")?;
        }
        match self.blink {
            Some(BlinkStyle::Slow) => write!(f, "*")?,
            Some(BlinkStyle::Rapid) => write!(f, "!")?,
            _ => {}
        }
        if self.conceal {
            write!(f, "?")?;
        }
        if self.strikethrough {
            write!(f, "-")?;
        }
        match self.framed {
            Some(FrameStyle::Frame) => write!(f, "□")?,
            Some(FrameStyle::Circle) => write!(f, "○")?,
            _ => {}
        }
        if self.overline {
            write!(f, "‾")?;
        }

        Ok(())
    }
}

impl Attrs {
    pub fn has_attrs(&self) -> bool {
        self.fgcolor.is_some()
            || self.bgcolor.is_some()
            || self.font_weight.is_some()
            || self.italic
            || self.underline.is_some()
            || self.inverse
            || self.blink.is_some()
            || self.conceal
            || self.strikethrough
            || self.framed.is_some()
            || self.overline
    }

    /// Given another set of attributes, generate the minimal control codes
    /// which will transition the terminal to the other set of attributes
    /// from this one.
    pub fn transition_to(&self, next: &Self) -> Vec<ControlCode> {
        let mut codes = vec![];

        let controls = control_codes();

        if self.fgcolor != next.fgcolor {
            if let Some(fgcolor) = next.fgcolor {
                codes.push(fgcolor.fgcode());
            } else {
                codes.push(controls.fgcolor_default.clone());
            }
        }

        if self.bgcolor != next.bgcolor {
            if let Some(bgcolor) = next.bgcolor {
                codes.push(bgcolor.bgcode());
            } else {
                codes.push(controls.bgcolor_default.clone());
            }
        }

        if self.italic && !next.italic {
            codes.push(controls.undo_italic.clone());
        } else if !self.italic && next.italic {
            codes.push(controls.italic.clone());
        }

        match (&self.underline, &next.underline) {
            (None, None) => {}
            (Some(_), None) => codes.push(controls.undo_underline.clone()),
            (None, Some(style)) => match style {
                UnderlineStyle::Single => codes.push(controls.underline.clone()),
                UnderlineStyle::Double => codes.push(controls.double_underline.clone()),
            },
            (Some(_), Some(style)) => {
                codes.push(controls.undo_underline.clone());
                match style {
                    UnderlineStyle::Single => codes.push(controls.underline.clone()),
                    UnderlineStyle::Double => codes.push(controls.double_underline.clone()),
                }
            }
        }

        if self.inverse && !next.inverse {
            codes.push(controls.undo_inverse.clone());
        } else if !self.inverse && next.inverse {
            codes.push(controls.inverse.clone());
        }

        match (&self.font_weight, &next.font_weight) {
            (None, None) => {}
            (Some(_), None) => codes.push(controls.reset_font_weight.clone()),
            (None, Some(style)) => match style {
                FontWeight::Bold => codes.push(controls.bold.clone()),
                FontWeight::Faint => codes.push(controls.faint.clone()),
            },
            (Some(_), Some(style)) => {
                codes.push(controls.reset_font_weight.clone());
                match style {
                    FontWeight::Bold => codes.push(controls.bold.clone()),
                    FontWeight::Faint => codes.push(controls.faint.clone()),
                }
            }
        }

        match (&self.blink, &next.blink) {
            (None, None) => {}
            (Some(_), None) => codes.push(controls.undo_blink.clone()),
            (None, Some(style)) => match style {
                BlinkStyle::Slow => codes.push(controls.slow_blink.clone()),
                BlinkStyle::Rapid => codes.push(controls.rapid_blink.clone()),
            },
            (Some(_), Some(style)) => {
                codes.push(controls.undo_blink.clone());
                match style {
                    BlinkStyle::Slow => codes.push(controls.slow_blink.clone()),
                    BlinkStyle::Rapid => codes.push(controls.rapid_blink.clone()),
                }
            }
        }

        if self.conceal && !next.conceal {
            codes.push(controls.undo_conceal.clone());
        } else if !self.conceal && next.conceal {
            codes.push(controls.conceal.clone());
        }

        if self.strikethrough && !next.strikethrough {
            codes.push(controls.undo_strikethrough.clone());
        } else if !self.strikethrough && next.strikethrough {
            codes.push(controls.strikethrough.clone());
        }

        match (&self.framed, &next.framed) {
            (None, None) => {}
            (Some(_), None) => codes.push(controls.undo_framed.clone()),
            (None, Some(style)) => match style {
                FrameStyle::Frame => codes.push(controls.framed.clone()),
                FrameStyle::Circle => codes.push(controls.encircled.clone()),
            },
            (Some(_), Some(style)) => {
                codes.push(controls.undo_framed.clone());
                match style {
                    FrameStyle::Frame => codes.push(controls.framed.clone()),
                    FrameStyle::Circle => codes.push(controls.encircled.clone()),
                }
            }
        }

        if self.overline && !next.overline {
            codes.push(controls.undo_overline.clone());
        } else if !self.overline && next.overline {
            codes.push(controls.overline.clone());
        }

        ControlCode::fuse_csi(codes)
    }
}

// A dictionary of standard control codes. Access codes via the
// control_codes() function. Most are constant struct members.
// Codes with dynamic params are generated on the fly via methods.
#[allow(dead_code)]
pub struct ControlCodes {
    pub clear_screen: ControlCode,
    pub clear_attrs: ControlCode,
    pub fgcolor_default: ControlCode,
    pub bgcolor_default: ControlCode,
    pub underline: ControlCode,
    pub double_underline: ControlCode,
    pub undo_underline: ControlCode,
    pub bold: ControlCode,
    pub faint: ControlCode,
    pub reset_font_weight: ControlCode,
    pub italic: ControlCode,
    pub undo_italic: ControlCode,
    pub inverse: ControlCode,
    pub undo_inverse: ControlCode,
    pub slow_blink: ControlCode,
    pub rapid_blink: ControlCode,
    pub undo_blink: ControlCode,
    pub conceal: ControlCode,
    pub undo_conceal: ControlCode,
    pub strikethrough: ControlCode,
    pub undo_strikethrough: ControlCode,
    pub framed: ControlCode,
    pub encircled: ControlCode,
    pub undo_framed: ControlCode,
    pub overline: ControlCode,
    pub undo_overline: ControlCode,
    pub save_cursor_position: ControlCode,
    pub restore_cursor_position: ControlCode,
    pub save_cursor: ControlCode,
    pub restore_cursor: ControlCode,
    pub enable_alt_screen: ControlCode,
    pub disable_alt_screen: ControlCode,
    pub erase_to_end: ControlCode,
    pub erase_from_start: ControlCode,
    pub erase_screen: ControlCode,
    pub erase_scrollback: ControlCode,
    pub erase_to_end_of_line: ControlCode,
    pub erase_to_start_of_line: ControlCode,
    pub erase_line: ControlCode,
    pub device_status_report: ControlCode,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ControlCode {
    CSI { params: Vec<Vec<u16>>, intermediates: Vec<u8>, action: char },
    ESC { intermediates: Vec<u8>, byte: u8 },
    __NonExhaustive,
}

impl AsTermInput for ControlCode {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        match self {
            ControlCode::CSI { params, intermediates, action } => {
                buf.extend_from_slice(b"\x1b["); // CSI
                buf.extend_from_slice(intermediates);

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        buf.push(b';');
                    }

                    for (j, subparam) in param.iter().enumerate() {
                        if j != 0 {
                            buf.push(b':');
                        }
                        extend_itoa(buf, *subparam);
                    }
                }

                let mut action_buf = [0; 4];
                buf.extend_from_slice(action.encode_utf8(&mut action_buf).as_bytes());
            }
            ControlCode::ESC { intermediates, byte } => {
                buf.extend_from_slice(b"\x1b"); // ESC
                buf.extend_from_slice(intermediates);
                buf.push(*byte);
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for ControlCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlCode::CSI { params, intermediates, action } => {
                write!(f, "CSI ")?;
                for intermediate in intermediates {
                    write!(f, "{} ", *intermediate as char)?;
                }
                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, "; ")?;
                    }
                    for (j, subparam) in param.iter().enumerate() {
                        if j != 0 {
                            write!(f, ": ")?;
                        }
                        write!(f, "{} ", subparam)?;
                    }
                }
                write!(f, "{}", action)?;
            }
            ControlCode::ESC { intermediates, byte } => {
                write!(f, "ESC ")?;
                for intermediate in intermediates {
                    write!(f, "{} ", *intermediate as char)?;
                }
                write!(f, "{}", byte)?;
            }
            _ => write!(f, "<display unimpl>")?,
        }

        Ok(())
    }
}

impl ControlCode {
    fn fuse_csi<I>(control_codes: I) -> Vec<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let mut fused_codes = vec![];
        let mut current_params = vec![];
        let mut current_intermediates = vec![];
        let mut current_action = None;
        for code in control_codes.into_iter() {
            if let ControlCode::CSI { params, intermediates, action } = code {
                if let Some(cur_action) = current_action {
                    if cur_action == action && current_intermediates == intermediates {
                        current_params.extend(params);
                    } else {
                        fused_codes.push(ControlCode::CSI {
                            params: std::mem::take(&mut current_params),
                            intermediates: std::mem::take(&mut current_intermediates),
                            action: cur_action,
                        });
                        current_action = Some(action);
                        current_intermediates = intermediates;
                        current_params = params;
                    }
                } else {
                    current_action = Some(action);
                    current_intermediates = intermediates;
                    current_params.extend(params);
                }
            } else {
                if let Some(action) = current_action {
                    fused_codes.push(ControlCode::CSI {
                        params: std::mem::take(&mut current_params),
                        intermediates: std::mem::take(&mut current_intermediates),
                        action,
                    });
                    current_action = None;
                }
                fused_codes.push(code);
            }
        }

        if let Some(action) = current_action {
            fused_codes.push(ControlCode::CSI {
                params: std::mem::take(&mut current_params),
                intermediates: std::mem::take(&mut current_intermediates),
                action,
            })
        }

        fused_codes
    }
}

static CONTROL_CODES: OnceLock<ControlCodes> = OnceLock::new();

pub fn control_codes() -> &'static ControlCodes {
    CONTROL_CODES.get_or_init(|| ControlCodes {
        clear_screen: ControlCode::CSI { params: vec![], intermediates: vec![], action: 'J' },
        clear_attrs: ControlCode::CSI { params: vec![], intermediates: vec![], action: 'm' },
        fgcolor_default: ControlCode::CSI {
            params: vec![vec![39]],
            intermediates: vec![],
            action: 'm',
        },
        bgcolor_default: ControlCode::CSI {
            params: vec![vec![49]],
            intermediates: vec![],
            action: 'm',
        },
        underline: ControlCode::CSI { params: vec![vec![4]], intermediates: vec![], action: 'm' },
        double_underline: ControlCode::CSI {
            params: vec![vec![21]],
            intermediates: vec![],
            action: 'm',
        },
        undo_underline: ControlCode::CSI {
            params: vec![vec![24]],
            intermediates: vec![],
            action: 'm',
        },
        bold: ControlCode::CSI { params: vec![vec![1]], intermediates: vec![], action: 'm' },
        faint: ControlCode::CSI { params: vec![vec![2]], intermediates: vec![], action: 'm' },
        reset_font_weight: ControlCode::CSI {
            params: vec![vec![22]],
            intermediates: vec![],
            action: 'm',
        },
        italic: ControlCode::CSI { params: vec![vec![3]], intermediates: vec![], action: 'm' },
        undo_italic: ControlCode::CSI {
            params: vec![vec![23]],
            intermediates: vec![],
            action: 'm',
        },
        inverse: ControlCode::CSI { params: vec![vec![7]], intermediates: vec![], action: 'm' },
        undo_inverse: ControlCode::CSI {
            params: vec![vec![27]],
            intermediates: vec![],
            action: 'm',
        },
        slow_blink: ControlCode::CSI { params: vec![vec![5]], intermediates: vec![], action: 'm' },
        rapid_blink: ControlCode::CSI { params: vec![vec![6]], intermediates: vec![], action: 'm' },
        undo_blink: ControlCode::CSI { params: vec![vec![25]], intermediates: vec![], action: 'm' },
        conceal: ControlCode::CSI { params: vec![vec![8]], intermediates: vec![], action: 'm' },
        undo_conceal: ControlCode::CSI {
            params: vec![vec![28]],
            intermediates: vec![],
            action: 'm',
        },
        strikethrough: ControlCode::CSI {
            params: vec![vec![9]],
            intermediates: vec![],
            action: 'm',
        },
        undo_strikethrough: ControlCode::CSI {
            params: vec![vec![29]],
            intermediates: vec![],
            action: 'm',
        },
        framed: ControlCode::CSI { params: vec![vec![51]], intermediates: vec![], action: 'm' },
        encircled: ControlCode::CSI { params: vec![vec![52]], intermediates: vec![], action: 'm' },
        undo_framed: ControlCode::CSI {
            params: vec![vec![54]],
            intermediates: vec![],
            action: 'm',
        },
        overline: ControlCode::CSI { params: vec![vec![53]], intermediates: vec![], action: 'm' },
        undo_overline: ControlCode::CSI {
            params: vec![vec![55]],
            intermediates: vec![],
            action: 'm',
        },
        save_cursor_position: ControlCode::CSI {
            params: vec![],
            intermediates: vec![],
            action: 's',
        },
        restore_cursor_position: ControlCode::CSI {
            params: vec![],
            intermediates: vec![],
            action: 'u',
        },
        save_cursor: ControlCode::ESC { intermediates: vec![], byte: b'7' },
        restore_cursor: ControlCode::ESC { intermediates: vec![], byte: b'8' },
        enable_alt_screen: ControlCode::CSI {
            params: vec![vec![1049]],
            intermediates: vec![b'?'],
            action: 'h',
        },
        disable_alt_screen: ControlCode::CSI {
            params: vec![vec![1049]],
            intermediates: vec![b'?'],
            action: 'l',
        },
        erase_to_end: ControlCode::CSI {
            params: vec![vec![0]],
            intermediates: vec![],
            action: 'J',
        },
        erase_from_start: ControlCode::CSI {
            params: vec![vec![1]],
            intermediates: vec![],
            action: 'J',
        },
        erase_screen: ControlCode::CSI {
            params: vec![vec![2]],
            intermediates: vec![],
            action: 'J',
        },
        erase_scrollback: ControlCode::CSI {
            params: vec![vec![3]],
            intermediates: vec![],
            action: 'J',
        },
        erase_to_end_of_line: ControlCode::CSI {
            params: vec![vec![0]],
            intermediates: vec![],
            action: 'K',
        },
        erase_to_start_of_line: ControlCode::CSI {
            params: vec![vec![1]],
            intermediates: vec![],
            action: 'K',
        },
        erase_line: ControlCode::CSI { params: vec![vec![2]], intermediates: vec![], action: 'K' },
        device_status_report: ControlCode::CSI {
            params: vec![vec![6]],
            intermediates: vec![],
            action: 'n',
        },
    })
}

#[allow(dead_code)]
impl ControlCodes {
    pub fn fgcolor_idx(i: u8) -> ControlCode {
        if i < 8 {
            ControlCode::CSI {
                params: vec![vec![(i + 30) as u16]],
                intermediates: vec![],
                action: 'm',
            }
        } else if i < 16 {
            ControlCode::CSI {
                params: vec![vec![(i + 82) as u16]],
                intermediates: vec![],
                action: 'm',
            }
        } else {
            ControlCode::CSI {
                params: vec![vec![38], vec![5], vec![i as u16]],
                intermediates: vec![],
                action: 'm',
            }
        }
    }

    pub fn fgcolor_rgb(r: u8, g: u8, b: u8) -> ControlCode {
        ControlCode::CSI {
            params: vec![vec![38], vec![2], vec![r as u16], vec![g as u16], vec![b as u16]],
            intermediates: vec![],
            action: 'm',
        }
    }

    pub fn bgcolor_idx(i: u8) -> ControlCode {
        if i < 8 {
            ControlCode::CSI {
                params: vec![vec![(i + 40) as u16]],
                intermediates: vec![],
                action: 'm',
            }
        } else if i < 16 {
            ControlCode::CSI {
                params: vec![vec![(i + 92) as u16]],
                intermediates: vec![],
                action: 'm',
            }
        } else {
            ControlCode::CSI {
                params: vec![vec![48], vec![5], vec![i as u16]],
                intermediates: vec![],
                action: 'm',
            }
        }
    }

    pub fn bgcolor_rgb(r: u8, g: u8, b: u8) -> ControlCode {
        ControlCode::CSI {
            params: vec![vec![48], vec![2], vec![r as u16], vec![g as u16], vec![b as u16]],
            intermediates: vec![],
            action: 'm',
        }
    }

    pub fn cursor_up(n: u16) -> ControlCode {
        Self::move_cursor(n, 'A')
    }

    pub fn cursor_down(n: u16) -> ControlCode {
        Self::move_cursor(n, 'B')
    }

    pub fn cursor_forward(n: u16) -> ControlCode {
        Self::move_cursor(n, 'C')
    }

    pub fn cursor_backwards(n: u16) -> ControlCode {
        Self::move_cursor(n, 'D')
    }

    pub fn cursor_next_line(n: u16) -> ControlCode {
        Self::move_cursor(n, 'E')
    }

    pub fn cursor_prev_line(n: u16) -> ControlCode {
        Self::move_cursor(n, 'F')
    }

    pub fn cursor_position(row: u16, col: u16) -> ControlCode {
        if row == 1 && col == 1 {
            ControlCode::CSI { params: vec![], intermediates: vec![], action: 'H' }
        } else {
            ControlCode::CSI {
                params: vec![vec![row], vec![col]],
                intermediates: vec![],
                action: 'H',
            }
        }
    }

    pub fn cursor_horizontal_absolute(col: u16) -> ControlCode {
        ControlCode::CSI { params: vec![vec![col]], intermediates: vec![], action: 'G' }
    }

    fn move_cursor(n: u16, action: char) -> ControlCode {
        if n == 1 {
            ControlCode::CSI { params: vec![], intermediates: vec![], action }
        } else {
            ControlCode::CSI { params: vec![vec![n]], intermediates: vec![], action }
        }
    }

    pub fn scroll_up(n: u16) -> ControlCode {
        if n == 1 {
            ControlCode::CSI { params: vec![], intermediates: vec![], action: 'S' }
        } else {
            ControlCode::CSI { params: vec![vec![n]], intermediates: vec![], action: 'S' }
        }
    }

    pub fn scroll_down(n: u16) -> ControlCode {
        if n == 1 {
            ControlCode::CSI { params: vec![], intermediates: vec![], action: 'T' }
        } else {
            ControlCode::CSI { params: vec![vec![n]], intermediates: vec![], action: 'T' }
        }
    }
}

/// Represents a foreground or background color for cells.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Color {
    /// The default terminal color.
    Default,

    /// An indexed terminal color.
    Idx(u8),

    /// An RGB terminal color. The parameters are (red, green, blue).
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Self::Default
    }
}

impl Color {
    fn bgcode(&self) -> ControlCode {
        match self {
            Color::Default => control_codes().bgcolor_default.clone(),
            Color::Idx(i) => ControlCodes::bgcolor_idx(*i),
            Color::Rgb(r, g, b) => ControlCodes::bgcolor_rgb(*r, *g, *b),
        }
    }

    fn fgcode(&self) -> ControlCode {
        match self {
            Color::Default => control_codes().fgcolor_default.clone(),
            Color::Idx(i) => ControlCodes::fgcolor_idx(*i),
            Color::Rgb(r, g, b) => ControlCodes::fgcolor_rgb(*r, *g, *b),
        }
    }
}

#[derive(Default, Debug)]
#[must_use = "this struct does nothing unless you call term_input_into"]
pub struct Crlf;

impl AsTermInput for Crlf {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(b"\r\n");
    }
}

fn extend_itoa<I: itoa::Integer>(buf: &mut Vec<u8>, i: I) {
    let mut itoa_buf = itoa::Buffer::new();
    buf.extend_from_slice(itoa_buf.format(i).as_bytes());
}
