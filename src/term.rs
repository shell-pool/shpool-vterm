// The MIT License (MIT)
//
// Copyright (c) 2016 Jesse Luehrs
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
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

/// A position that the terminal is writing at. Includes attributes that
/// have been previously set via control codes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cursor {
    pub pos: Pos,
    pub attrs: Attrs,
}

impl Cursor {
    pub fn new(pos: Pos) -> Self {
        Cursor {
            pos,
            attrs: Attrs::default(),
        }
    }

    /// Ensure that the cursor is within the given region
    /// by moving to the closest edge if it is out of bounds.
    pub fn clamp_to<R>(&mut self, region: R)
    where
        R: Region,
    {
        let (low_row, high_row) = region.row_bounds();
        if self.pos.row < low_row {
            self.pos.row = low_row;
        }
        if self.pos.row >= high_row {
            self.pos.row = high_row - 1;
        }

        let (low_col, high_col) = region.col_bounds();
        if self.pos.col < low_col {
            self.pos.col = low_col;
        }
        if self.pos.col >= high_col {
            self.pos.col = high_col - 1;
        }
    }
}

/// A position within the terminal. Generally, this refers to a grid
/// mode view of the terminal, not the underlying logical lines mode
/// that we actually store the data in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
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

#[derive(Default, Debug)]
#[must_use = "this struct does nothing unless you call term_input_into"]
pub struct ClearScreen;

impl AsTermInput for ClearScreen {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(b"\x1b[H\x1b[J");
    }
}

#[derive(Default, Debug)]
#[must_use = "this struct does nothing unless you call term_input_into"]
pub struct ClearAttrs;

impl AsTermInput for ClearAttrs {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(b"\x1b[m");
    }
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
        Raw {
            inner: Vec::from(value.as_bytes()),
        }
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
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl std::fmt::Display for Attrs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = &self.fgcolor {
            write!(f, "<FG {c:?}>")?;
        }

        if let Some(c) = &self.bgcolor {
            write!(f, "<BG {c:?}>")?;
        }

        if self.bold {
            write!(f, "b")?;
        }
        if self.italic {
            write!(f, "i")?;
        }
        if self.underline {
            write!(f, "_")?;
        }
        if self.inverse {
            write!(f, "<")?;
        }

        Ok(())
    }
}

impl Attrs {
    pub fn has_attrs(&self) -> bool {
        self.fgcolor.is_some()
            || self.bgcolor.is_some()
            || self.bold
            || self.italic
            || self.underline
            || self.inverse
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

        if self.bold && !next.bold {
            codes.push(controls.undo_bold.clone());
        } else if !self.bold && next.bold {
            codes.push(controls.bold.clone());
        }

        if self.italic && !next.italic {
            codes.push(controls.undo_italic.clone());
        } else if !self.italic && next.italic {
            codes.push(controls.italic.clone());
        }

        if self.underline && !next.underline {
            codes.push(controls.undo_underline.clone());
        } else if !self.italic && next.underline {
            codes.push(controls.underline.clone());
        }

        if self.inverse && !next.inverse {
            codes.push(controls.undo_inverse.clone());
        } else if !self.inverse && next.inverse {
            codes.push(controls.inverse.clone());
        }

        ControlCode::fuse_csi(codes)
    }
}

// A dictionary of standard control codes. Access codes via the
// control_codes() function. Most are constant struct members.
// Codes with dynamic params are generated on the fly via methods.
#[allow(dead_code)]
pub struct ControlCodes {
    pub fgcolor_default: ControlCode,
    pub bgcolor_default: ControlCode,
    pub underline: ControlCode,
    pub undo_underline: ControlCode,
    pub bold: ControlCode,
    pub undo_bold: ControlCode,
    pub italic: ControlCode,
    pub undo_italic: ControlCode,
    pub inverse: ControlCode,
    pub undo_inverse: ControlCode,
    pub save_cursor_position: ControlCode,
    pub restore_cursor_position: ControlCode,
    pub save_cursor: ControlCode,
    pub restore_cursor: ControlCode,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ControlCode {
    CSI { params: Vec<Vec<u16>>, action: char },
    ESC { intermediates: Vec<u8>, byte: u8 },
    __NonExhaustive,
}

impl AsTermInput for ControlCode {
    fn term_input_into(&self, buf: &mut Vec<u8>) {
        match self {
            ControlCode::CSI { params, action } => {
                buf.extend_from_slice(b"\x1b["); // CSI

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
                action.encode_utf8(&mut action_buf);
                buf.extend_from_slice(&action_buf);
            }
            ControlCode::ESC {
                intermediates,
                byte,
            } => {
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
            ControlCode::CSI { params, action } => {
                write!(f, "CSI ")?;
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
        let mut current_action = None;
        for code in control_codes.into_iter() {
            if let ControlCode::CSI { params, action } = code {
                if let Some(cur_action) = current_action {
                    if cur_action == action {
                        current_params.extend(params);
                    } else {
                        fused_codes.push(ControlCode::CSI {
                            params: std::mem::take(&mut current_params),
                            action,
                        });
                        current_action = Some(action);
                    }
                } else {
                    current_action = Some(action);
                    current_params.extend(params);
                }
            } else {
                if let Some(action) = current_action {
                    fused_codes.push(ControlCode::CSI {
                        params: std::mem::take(&mut current_params),
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
                action,
            })
        }

        fused_codes
    }
}

static CONTROL_CODES: OnceLock<ControlCodes> = OnceLock::new();

pub fn control_codes() -> &'static ControlCodes {
    CONTROL_CODES.get_or_init(|| ControlCodes {
        fgcolor_default: ControlCode::CSI {
            params: vec![vec![39]],
            action: 'm',
        },
        bgcolor_default: ControlCode::CSI {
            params: vec![vec![49]],
            action: 'm',
        },
        underline: ControlCode::CSI {
            params: vec![vec![4]],
            action: 'm',
        },
        undo_underline: ControlCode::CSI {
            params: vec![vec![24]],
            action: 'm',
        },
        bold: ControlCode::CSI {
            params: vec![vec![1]],
            action: 'm',
        },
        undo_bold: ControlCode::CSI {
            params: vec![vec![22]],
            action: 'm',
        },
        italic: ControlCode::CSI {
            params: vec![vec![3]],
            action: 'm',
        },
        undo_italic: ControlCode::CSI {
            params: vec![vec![23]],
            action: 'm',
        },
        inverse: ControlCode::CSI {
            params: vec![vec![7]],
            action: 'm',
        },
        undo_inverse: ControlCode::CSI {
            params: vec![vec![27]],
            action: 'm',
        },
        save_cursor_position: ControlCode::CSI {
            params: vec![],
            action: 's',
        },
        restore_cursor_position: ControlCode::CSI {
            params: vec![],
            action: 'u',
        },
        save_cursor: ControlCode::ESC {
            intermediates: vec![],
            byte: b'7',
        },
        restore_cursor: ControlCode::ESC {
            intermediates: vec![],
            byte: b'8',
        },
    })
}

#[allow(dead_code)]
impl ControlCodes {
    pub fn fgcolor_idx(i: u8) -> ControlCode {
        if i < 8 {
            ControlCode::CSI {
                params: vec![vec![(i + 30) as u16]],
                action: 'm',
            }
        } else if i < 16 {
            ControlCode::CSI {
                params: vec![vec![(i + 82) as u16]],
                action: 'm',
            }
        } else {
            ControlCode::CSI {
                params: vec![vec![38], vec![5], vec![i as u16]],
                action: 'm',
            }
        }
    }

    pub fn fgcolor_rgb(r: u8, g: u8, b: u8) -> ControlCode {
        ControlCode::CSI {
            params: vec![
                vec![38],
                vec![2],
                vec![r as u16],
                vec![g as u16],
                vec![b as u16],
            ],
            action: 'm',
        }
    }

    pub fn bgcolor_idx(i: u8) -> ControlCode {
        if i < 8 {
            ControlCode::CSI {
                params: vec![vec![(i + 40) as u16]],
                action: 'm',
            }
        } else if i < 16 {
            ControlCode::CSI {
                params: vec![vec![(i + 92) as u16]],
                action: 'm',
            }
        } else {
            ControlCode::CSI {
                params: vec![vec![48], vec![5], vec![i as u16]],
                action: 'm',
            }
        }
    }

    pub fn bgcolor_rgb(r: u8, g: u8, b: u8) -> ControlCode {
        ControlCode::CSI {
            params: vec![
                vec![48],
                vec![2],
                vec![r as u16],
                vec![g as u16],
                vec![b as u16],
            ],
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
        ControlCode::CSI {
            params: vec![vec![row], vec![col]],
            action: 'H',
        }
    }

    pub fn cursor_horizontal_absolute(col: u16) -> ControlCode {
        ControlCode::CSI {
            params: vec![vec![col]],
            action: 'G',
        }
    }

    fn move_cursor(n: u16, action: char) -> ControlCode {
        if n == 1 {
            ControlCode::CSI {
                params: vec![],
                action,
            }
        } else {
            ControlCode::CSI {
                params: vec![vec![n]],
                action,
            }
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
