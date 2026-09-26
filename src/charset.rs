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

//! The ISO 2022 style charset switching that terminals inherited from the
//! VT100.
//!
//! Apps, curses ones in particular, draw lines and boxes by designating the
//! DEC special graphics set into one of the G0-G3 slots, invoking that slot
//! and then printing plain ascii letters, which the terminal displays as
//! line drawing chars. We translate chars as they get printed and store the
//! result, so the screen always holds the chars that were displayed and
//! dumping it does not depend on the charset state at all.

use crate::term::{self, AsTermInput, ControlCodes};

/// A set of 94 chars which can be designated into one of the G0-G3 slots.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum Charset {
    #[default]
    Ascii,
    /// The UK national set, which is ascii with `#` swapped out for `£`.
    Uk,
    /// The DEC special graphics set, which swaps the lowercase letters and
    /// a few symbols out for line drawing and other graphical chars.
    DecSpecialGraphics,
}

impl Charset {
    /// The charset selected by the final byte of an `ESC ( F` style
    /// designation. We treat the sets we don't support as ascii, which is
    /// what the national sets mostly are, rather than leaving the old set
    /// in place.
    pub fn from_designator(designator: u8) -> Self {
        match designator {
            b'A' => Charset::Uk,
            b'0' => Charset::DecSpecialGraphics,
            _ => Charset::Ascii,
        }
    }

    /// The final byte of the escape sequence that designates this charset.
    pub fn designator(self) -> u8 {
        match self {
            Charset::Ascii => b'B',
            Charset::Uk => b'A',
            Charset::DecSpecialGraphics => b'0',
        }
    }

    /// The char that gets displayed when `c` is printed through this charset.
    pub fn translate(self, c: char) -> char {
        match self {
            Charset::Ascii => c,
            Charset::Uk => match c {
                '#' => '£',
                _ => c,
            },
            Charset::DecSpecialGraphics => match c {
                '_' => ' ',
                '`' => '◆',
                'a' => '▒',
                'b' => '␉',
                'c' => '␌',
                'd' => '␍',
                'e' => '␊',
                'f' => '°',
                'g' => '±',
                'h' => '␤',
                'i' => '␋',
                'j' => '┘',
                'k' => '┐',
                'l' => '┌',
                'm' => '└',
                'n' => '┼',
                'o' => '⎺',
                'p' => '⎻',
                'q' => '─',
                'r' => '⎼',
                's' => '⎽',
                't' => '├',
                'u' => '┤',
                'v' => '┴',
                'w' => '┬',
                'x' => '│',
                'y' => '≤',
                'z' => '≥',
                '{' => 'π',
                '|' => '≠',
                '}' => '£',
                '~' => '·',
                _ => c,
            },
        }
    }
}

/// The charset state of a terminal: which charset is designated into each
/// of the G0-G3 slots, and which slot printed chars get translated through.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Charsets {
    /// The charsets designated into G0 through G3.
    slots: [Charset; 4],
    /// The slot invoked into GL by a locking shift (SI, SO, LS2 or LS3).
    /// This is the slot printed chars normally come from.
    gl: usize,
    /// A slot invoked for just the next printed char by a single shift (SS2
    /// or SS3).
    single_shift: Option<usize>,
}

impl Charsets {
    /// Designate a charset into one of the G0-G3 slots.
    pub fn designate(&mut self, slot: usize, charset: Charset) {
        if let Some(s) = self.slots.get_mut(slot) {
            *s = charset;
        }
    }

    /// Invoke one of the G0-G3 slots into GL until the next locking shift.
    pub fn lock_shift(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.gl = slot;
        }
    }

    /// Invoke one of the G0-G3 slots for just the next printed char.
    pub fn single_shift(&mut self, slot: usize) {
        if slot < self.slots.len() {
            self.single_shift = Some(slot);
        }
    }

    /// The part of the charset state that DECSC saves: the designations and
    /// the locking shift. Like in xterm, a pending single shift is not part
    /// of it.
    pub fn saved(&self) -> Charsets {
        Charsets { single_shift: None, ..self.clone() }
    }

    /// Put back a charset state that `saved` saved. A pending single shift
    /// stays pending, since it still applies to the next printed char.
    pub fn restore(&mut self, saved: &Charsets) {
        self.slots = saved.slots;
        self.gl = saved.gl;
    }

    /// Translate a printed char into the char that gets displayed, using up
    /// any pending single shift.
    pub fn translate(&mut self, c: char) -> char {
        let slot = self.single_shift.take().unwrap_or(self.gl);
        self.slots[slot].translate(c)
    }

    /// Emit the control codes that put a terminal whose charsets have just
    /// been reset into this charset state.
    ///
    /// This has to come after all the contents have been painted, since the
    /// contents hold chars that have already been translated.
    pub fn dump_into(&self, buf: &mut Vec<u8>) {
        for (slot, charset) in self.slots.iter().enumerate() {
            if *charset != Charset::Ascii {
                ControlCodes::designate_charset(slot, charset.designator()).term_input_into(buf);
            }
        }

        let controls = term::control_codes();
        match self.gl {
            1 => buf.push(term::SHIFT_OUT),
            2 => controls.locking_shift_2.term_input_into(buf),
            3 => controls.locking_shift_3.term_input_into(buf),
            _ => {}
        }
        match self.single_shift {
            Some(2) => controls.single_shift_2.term_input_into(buf),
            Some(3) => controls.single_shift_3.term_input_into(buf),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_untouched() {
        let mut charsets = Charsets::default();
        for c in [' ', '#', 'a', 'q', 'x', '~', 'é', '中'] {
            assert_eq!(charsets.translate(c), c);
        }
    }

    #[test]
    fn line_drawing() {
        let mut charsets = Charsets::default();
        charsets.designate(0, Charset::DecSpecialGraphics);
        let drawn: String = "lqwqk tqnqu mqvqj x".chars().map(|c| charsets.translate(c)).collect();
        assert_eq!(drawn, "┌─┬─┐ ├─┼─┤ └─┴─┘ │");
        // Only 0x5f through 0x7e get swapped out.
        let untouched: String = "AZ09#^é".chars().map(|c| charsets.translate(c)).collect();
        assert_eq!(untouched, "AZ09#^é");
    }

    #[test]
    fn uk() {
        let mut charsets = Charsets::default();
        charsets.designate(0, Charset::Uk);
        assert_eq!(charsets.translate('#'), '£');
        assert_eq!(charsets.translate('q'), 'q');
    }

    #[test]
    fn locking_shifts() {
        let mut charsets = Charsets::default();
        charsets.designate(1, Charset::DecSpecialGraphics);
        assert_eq!(charsets.translate('q'), 'q');
        charsets.lock_shift(1);
        assert_eq!(charsets.translate('q'), '─');
        assert_eq!(charsets.translate('q'), '─');
        charsets.lock_shift(0);
        assert_eq!(charsets.translate('q'), 'q');
    }

    #[test]
    fn single_shift_lasts_for_one_char() {
        let mut charsets = Charsets::default();
        charsets.designate(3, Charset::DecSpecialGraphics);
        charsets.single_shift(3);
        assert_eq!(charsets.translate('q'), '─');
        assert_eq!(charsets.translate('q'), 'q');
    }

    #[test]
    fn out_of_range_slots_are_ignored() {
        let mut charsets = Charsets::default();
        charsets.designate(4, Charset::DecSpecialGraphics);
        charsets.lock_shift(4);
        charsets.single_shift(4);
        assert_eq!(charsets, Charsets::default());
    }

    #[test]
    fn unsupported_sets_are_ascii() {
        assert_eq!(Charset::from_designator(b'K'), Charset::Ascii);
        assert_eq!(Charset::from_designator(b'<'), Charset::Ascii);
        for charset in [Charset::Ascii, Charset::Uk, Charset::DecSpecialGraphics] {
            assert_eq!(Charset::from_designator(charset.designator()), charset);
        }
    }

    #[test]
    fn save_and_restore_leave_a_single_shift_alone() {
        let mut charsets = Charsets::default();
        charsets.designate(2, Charset::DecSpecialGraphics);
        charsets.single_shift(2);
        let saved = charsets.saved();
        assert_eq!(charsets.translate('q'), '─');

        charsets.restore(&saved);
        assert_eq!(charsets.translate('q'), 'q');
        charsets.single_shift(2);
        charsets.restore(&saved);
        assert_eq!(charsets.translate('q'), '─');
    }

    #[test]
    fn dump() {
        let mut buf = vec![];
        Charsets::default().dump_into(&mut buf);
        assert_eq!(buf, b"");

        let mut charsets = Charsets::default();
        charsets.designate(1, Charset::DecSpecialGraphics);
        charsets.designate(2, Charset::Uk);
        charsets.lock_shift(1);
        charsets.dump_into(&mut buf);
        assert_eq!(buf, b"\x1b)0\x1b*A\x0e");

        buf.clear();
        charsets.lock_shift(3);
        charsets.single_shift(2);
        charsets.dump_into(&mut buf);
        assert_eq!(buf, b"\x1b)0\x1b*A\x1bo\x1bN");
    }
}
