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

//! The session that the `restore` fuzz target in `fuzz/` runs, shared with
//! `tests/fuzz.rs` so that an input decodes to the same session in both.
//!
//! Changing `Session` or `Step` changes what every saved input decodes to, so
//! the ones in `tests/data/fuzz` would stop testing what they used to.

use arbitrary::{Arbitrary, Unstructured};
use shpool_vterm::{ContentRegion, Size, Term};
use unicode_width::UnicodeWidthChar;

/// A terminal of some size, and the things that happen to it.
#[derive(Arbitrary, Debug)]
pub struct Session {
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=19))]
    scrollback_lines: usize,
    #[arbitrary(with = small_size)]
    size: Size,
    steps: Vec<Step>,
}

/// One thing that happens to the terminal.
#[derive(Arbitrary, Debug)]
enum Step {
    /// A chunk of input, as it might come off the pty.
    Input(Bytes),
    Resize(#[arbitrary(with = small_size)] Size),
}

/// Screens this small wrap and scroll after just a few chars, which is where
/// the bugs tend to be. Screens with no rows or columns are in the mix too.
fn small_size(u: &mut Unstructured) -> arbitrary::Result<Size> {
    Ok(Size { width: u.int_in_range(0..=12)?, height: u.int_in_range(0..=8)? })
}

/// Bytes that print as a byte string, which is far easier to read than a
/// list of numbers once escape codes are involved.
#[derive(Arbitrary)]
struct Bytes(Vec<u8>);

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b\"{}\"", self.0.escape_ascii())
    }
}

impl Session {
    /// Run the session, panicking if the terminal does, or if restoring what
    /// it ends up with does not give that back.
    pub fn check(&self) {
        let mut size = self.size;
        let mut term = Term::new(self.scrollback_lines, size);
        for step in &self.steps {
            match step {
                Step::Input(bytes) => term.process(&bytes.0),
                Step::Resize(new_size) => {
                    size = *new_size;
                    term.resize(size);
                }
            }
        }

        for region in [ContentRegion::All, ContentRegion::Screen, ContentRegion::BottomLines(3)] {
            let _ = term.contents(region);
        }

        // There is nothing to restore into a screen with no rows or columns.
        if size.width == 0 || size.height == 0 {
            return;
        }

        let dump = term.contents(ContentRegion::All);
        let redump = restore(&dump, size);
        let lenient = prints_blank_for_wrap(&dump);
        assert!(
            normalize(&redump, lenient) == normalize(&dump, lenient),
            "restore is not faithful\ndump:   {:?}\nredump: {:?}",
            String::from_utf8_lossy(&dump),
            String::from_utf8_lossy(&redump)
        );

        // Whatever the first restore did to the dump, restoring what came out
        // of it should give back exactly the same thing.
        let reredump = restore(&redump, size);
        assert!(
            reredump == redump,
            "restore is not stable\nredump:   {:?}\nreredump: {:?}",
            String::from_utf8_lossy(&redump),
            String::from_utf8_lossy(&reredump)
        );
    }
}

/// Restore `dump` into a fresh terminal of `size`, and dump that one.
fn restore(dump: &[u8], size: Size) -> Vec<u8> {
    // The terminal we restore into has its own idea of how much scrollback
    // to keep, and it had better be enough.
    let mut term = Term::new(10_000, size);
    term.process(dump);
    term.contents(ContentRegion::All)
}

/// Paper over what restoring a cursor that is waiting to wrap does to the
/// terminal we restore into.
///
/// There is no way to set the pending wrap flag other than to print the char
/// in the last column, and when that is a blank, the terminal we restore into
/// ends up with a space there, which it stores just like any other text. That
/// can even add a row to the bottom of a screen that is not full, so blank
/// rows at the bottom get dropped too, but only when the dump shows signs of
/// that having happened, since they matter once there is scrollback.
///
/// The same goes for the spaces in front of a wide char. The row with the
/// blank in its last column might have wrapped because the wide char at the
/// start of the next one did not fit there, which makes the space part of the
/// same line as the wide char.
fn normalize(dump: &[u8], lenient: bool) -> Vec<u8> {
    let dump = drop_before_line_end(dump, b" ", true);
    if lenient {
        drop_blanks_before_wide_chars(&drop_before_line_end(&dump, b"\r\n", false))
    } else {
        dump
    }
}

/// Drop the runs of spaces that come before a wide char, along with any in
/// between the codes that set the attrs of the chars.
fn drop_blanks_before_wide_chars(dump: &[u8]) -> Vec<u8> {
    let dump = String::from_utf8_lossy(dump);
    let mut out = String::with_capacity(dump.len());
    for (i, c) in dump.char_indices() {
        if c == ' ' && starts_with_wide_char(&dump[i..]) {
            continue;
        }
        out.push(c);
    }
    out.into_bytes()
}

/// Whether `s` starts with a wide char, once the spaces, SGR codes and links
/// at the start of it are out of the way.
fn starts_with_wide_char(mut s: &str) -> bool {
    loop {
        let rest = s.trim_start_matches(' ');
        let rest = if let Some(sgr) = rest.strip_prefix("\x1b[") {
            let n = sgr.bytes().take_while(|b| b.is_ascii_digit() || matches!(b, b';' | b':'));
            sgr[n.count()..].strip_prefix('m').unwrap_or(rest)
        } else if let Some(link) = rest.strip_prefix("\x1b]8;") {
            link.find("\x1b\\").map_or(rest, |end| &link[end + 2..])
        } else {
            rest
        };
        if rest.len() == s.len() {
            return s.chars().next().is_some_and(|c| UnicodeWidthChar::width(c) == Some(2));
        }
        s = rest;
    }
}

/// Drop the runs of `unit` that come right before the end of the painting,
/// and also the ones right before the end of a line if `in_line` is set.
fn drop_before_line_end(dump: &[u8], unit: &[u8], in_line: bool) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(dump.len());
    let mut i = 0;
    while i < dump.len() {
        if dump[i..].starts_with(unit) {
            let mut end = i;
            while dump[end..].starts_with(unit) {
                end += unit.len();
            }
            let rest = &dump[end..];
            if !((in_line && rest.starts_with(b"\r\n")) || ends_painting(rest)) {
                out.extend_from_slice(&dump[i..end]);
            }
            i = end;
        } else {
            out.push(dump[i]);
            i += 1;
        }
    }
    out
}

/// Whether the dump prints a blank to get a pending wrap back, which is when
/// it puts the cursor somewhere and then prints a single space, followed by
/// some other control code.
fn prints_blank_for_wrap(dump: &[u8]) -> bool {
    (0..dump.len()).any(|i| {
        let rest = &dump[i..];
        if !rest.starts_with(b"\x1b[") {
            return false;
        }
        let n = rest[2..].iter().take_while(|b| b.is_ascii_digit() || **b == b';').count();
        rest[2 + n..].starts_with(b"H ") && rest.get(4 + n).is_some_and(|b| b.is_ascii_control())
    })
}

/// Whether `rest` starts with one of the codes that come right after the
/// lines of a dump: cursor positioning, a mode or the scroll region.
fn ends_painting(rest: &[u8]) -> bool {
    if !rest.starts_with(b"\x1b[") {
        return false;
    }
    let params = rest[2..].iter().take_while(|b| b.is_ascii_digit() || **b == b';' || **b == b'?');
    let n = params.count();
    matches!(rest.get(2 + n), Some(b'H' | b'h' | b'r'))
}
