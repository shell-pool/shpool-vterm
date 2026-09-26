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

//! Throws random input at the terminal to check that nothing makes it panic,
//! and that restoring whatever state it ends up in gives back that state.

use shpool_vterm::{ContentRegion, Size, Term};
use unicode_width::UnicodeWidthChar;

/// A small, fast and deterministic PRNG (splitmix64), so that a failing case
/// can be reproduced from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

const TEXT: &[&str] = &[
    "a",
    "b",
    "q",
    "x",
    " ",
    "hello",
    "#",
    "~",
    "中",
    "😊",
    "é",
    "\u{301}",
    "\u{200d}",
    "\u{fe0f}",
    "\u{1f3fb}",
    "\u{0}",
];

const C0: &[&str] =
    &["\r", "\n", "\t", "\x08", "\x0b", "\x0c", "\x0e", "\x0f", "\x07", "\r\n", "\x1b", "\x7f"];

const ESC: &[&str] = &[
    "\x1b7", "\x1b8", "\x1bD", "\x1bE", "\x1bM", "\x1bc", "\x1b(0", "\x1b)0", "\x1b*0", "\x1b+0",
    "\x1b(B", "\x1b(A", "\x1bN", "\x1bO", "\x1bn", "\x1bo", "\x1b=", "\x1b>", "\x1bH", "\x1b#8",
    "\x1b\\",
];

const CSI_ACTIONS: &[char] = &[
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'P', 'S', 'T', 'X', 'Z',
    '`', 'a', 'b', 'd', 'e', 'f', 'g', 'h', 'l', 'm', 'n', 'q', 'r', 's', 'u', 't', 'c', 'x',
];

const PRIVATE_MODES: &[u16] = &[
    1, 3, 5, 6, 7, 9, 12, 25, 45, 47, 66, 69, 1000, 1002, 1003, 1004, 1005, 1006, 1007, 1015, 1016,
    1034, 1047, 1048, 1049, 2004, 2026,
];

fn param(rng: &mut Rng) -> String {
    match rng.below(10) {
        0 => String::new(),
        1 => "0".into(),
        2 => "65535".into(),
        3 => format!("{}", rng.below(100_000)),
        _ => format!("{}", rng.below(12)),
    }
}

fn csi(rng: &mut Rng) -> String {
    match rng.below(6) {
        0 => {
            let mode = rng.pick(PRIVATE_MODES);
            let action = rng.pick(&['h', 'l', 's', 'r']);
            format!("\x1b[?{mode}{action}")
        }
        1 => {
            // SGR, with the extended color and underline forms.
            let mut params = vec![];
            for _ in 0..rng.below(4) + 1 {
                params.push(match rng.below(8) {
                    0 => format!("38;5;{}", rng.below(256)),
                    1 => format!("48;2;{};{};{}", rng.below(256), rng.below(256), rng.below(256)),
                    2 => format!("4:{}", rng.below(6)),
                    3 => format!("58:2::{}:{}:{}", rng.below(256), rng.below(256), rng.below(256)),
                    _ => format!("{}", rng.below(110)),
                });
            }
            format!("\x1b[{}m", params.join(";"))
        }
        2 => {
            let prefix = rng.pick(&["", ">", "<", "=", "!", " ", "$", "\""]);
            format!("\x1b[{prefix}{}{}", param(rng), rng.pick(&['p', 'q', 'u', 'm', 'c', 't']))
        }
        _ => {
            let mut params = vec![];
            for _ in 0..rng.below(3) {
                params.push(param(rng));
            }
            format!("\x1b[{}{}", params.join(";"), rng.pick(CSI_ACTIONS))
        }
    }
}

fn osc(rng: &mut Rng) -> String {
    let term = if rng.below(2) == 0 { "\x07" } else { "\x1b\\" };
    let body = match rng.below(8) {
        0 => format!("0;title{}", rng.below(10)),
        1 => format!("2;{}", rng.pick(&["", "t", "x".repeat(40).as_str()])),
        2 => format!("4;{};rgb:ff/00/{:02x}", rng.below(300), rng.below(256)),
        3 => format!("8;;http://x/{}", rng.below(3)),
        4 => "8;;".into(),
        5 => format!("{};#{:06x}", 10 + rng.below(10), rng.below(0xffffff)),
        6 => format!("7;file://host/dir{}", rng.below(3)),
        _ => format!("{}", rng.pick(&["22;1", "22;0", "23;1", "23;0", "104", "110", "52;c;YQ=="])),
    };
    format!("\x1b]{body}{term}")
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

/// One thing that happens to the terminal.
#[derive(Clone, Debug)]
enum Step {
    Resize(Size),
    /// A piece of input, generated as a unit (some text, a control code...)
    /// so that shrinking a failing case does not tear escape codes apart.
    Input(Vec<u8>),
}

fn atom(rng: &mut Rng) -> Vec<u8> {
    match rng.below(16) {
        0..=4 => rng.pick(TEXT).as_bytes().to_vec(),
        5 | 6 => rng.pick(C0).as_bytes().to_vec(),
        7 | 8 => rng.pick(ESC).as_bytes().to_vec(),
        9..=12 => csi(rng).into_bytes(),
        13 => osc(rng).into_bytes(),
        14 => b"\x1bP1$rstuff\x1b\\".to_vec(),
        _ => vec![rng.next() as u8],
    }
}

fn size(rng: &mut Rng) -> Size {
    Size { width: rng.below(13), height: rng.below(9) }
}

struct Case {
    scrollback_lines: usize,
    size: Size,
    steps: Vec<Step>,
}

fn gen_case(seed: u64) -> Case {
    let mut rng = Rng(seed);
    let scrollback_lines = rng.below(20);
    let size = size(&mut rng);
    let mut steps = vec![];
    for _ in 0..rng.below(8) + 1 {
        if rng.below(4) == 0 {
            let mut size = self::size(&mut rng);
            if rng.below(3) == 0 {
                size.width = std::cmp::max(size.width, 1);
                size.height = std::cmp::max(size.height, 1);
            }
            steps.push(Step::Resize(size));
        }
        for _ in 0..rng.below(40) {
            steps.push(Step::Input(atom(&mut rng)));
        }
    }
    Case { scrollback_lines, size, steps }
}

impl Case {
    fn describe(&self) -> String {
        let mut out = format!("scrollback {} size {:?}\n", self.scrollback_lines, self.size);
        let mut input = vec![];
        for step in &self.steps {
            match step {
                Step::Resize(size) => {
                    if !input.is_empty() {
                        out += &format!("{:?}\n", String::from_utf8_lossy(&input));
                        input.clear();
                    }
                    out += &format!("resize {size:?}\n");
                }
                Step::Input(bytes) => input.extend_from_slice(bytes),
            }
        }
        if !input.is_empty() {
            out += &format!("{:?}\n", String::from_utf8_lossy(&input));
        }
        out
    }

    /// Run the case, returning a description of what went wrong, if
    /// anything did.
    fn check(&self, check_restore: bool) -> Option<String> {
        let result = std::panic::catch_unwind(|| {
            let mut size = self.size;
            let mut term = Term::new(self.scrollback_lines, size);
            for step in &self.steps {
                match step {
                    Step::Resize(new_size) => {
                        size = *new_size;
                        term.resize(size);
                    }
                    Step::Input(bytes) => term.process(bytes),
                }
            }

            for region in [ContentRegion::All, ContentRegion::Screen, ContentRegion::BottomLines(3)]
            {
                let _ = term.contents(region);
            }

            if check_restore && size.width > 0 && size.height > 0 {
                let dump = term.contents(ContentRegion::All);
                // The terminal we restore into has its own idea of how much
                // scrollback to keep, and it had better be enough.
                let mut restored = Term::new(10_000, size);
                restored.process(&dump);
                let redump = restored.contents(ContentRegion::All);
                let lenient = prints_blank_for_wrap(&dump);
                if normalize(&redump, lenient) != normalize(&dump, lenient) {
                    return Some(format!(
                        "restore is not faithful\ndump:   {:?}\nredump: {:?}",
                        String::from_utf8_lossy(&dump),
                        String::from_utf8_lossy(&redump)
                    ));
                }

                // Whatever the first restore did to the dump, restoring what
                // came out of it should give back exactly the same thing.
                let mut rerestored = Term::new(10_000, size);
                rerestored.process(&redump);
                let reredump = rerestored.contents(ContentRegion::All);
                if reredump != redump {
                    return Some(format!(
                        "restore is not stable\nredump:   {:?}\nreredump: {:?}",
                        String::from_utf8_lossy(&redump),
                        String::from_utf8_lossy(&reredump)
                    ));
                }
            }
            None
        });
        match result {
            Ok(problem) => problem,
            Err(e) => Some(format!(
                "panic: {}",
                e.downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default()
            )),
        }
    }

    /// Throw away as much of a failing case as possible while keeping it
    /// failing, so that it is easier to see what is going on.
    fn shrink(mut self, check_restore: bool) -> Case {
        let mut chunk = self.steps.len() / 2;
        while chunk > 0 {
            let mut i = 0;
            while i < self.steps.len() {
                let mut steps = self.steps.clone();
                steps.drain(i..std::cmp::min(i + chunk, steps.len()));
                let candidate =
                    Case { scrollback_lines: self.scrollback_lines, size: self.size, steps };
                if candidate.check(check_restore).is_some() {
                    self = candidate;
                } else {
                    i += chunk;
                }
            }
            chunk /= 2;
        }
        self
    }
}

/// A quick run over a fixed set of seeds, to catch regressions.
#[test]
fn random_input() {
    for seed in 0..1000 {
        if let Some(problem) = gen_case(seed).check(true) {
            panic!("seed {seed}: {problem}");
        }
    }
}

/// A much longer run, for hunting down new bugs:
///
/// ```text
/// FUZZ_RESTORE=1 FUZZ_START=0 FUZZ_N=1000000 \
///     cargo test --release --test fuzz -- --ignored random_input_long
/// ```
///
/// `FUZZ_START` and `FUZZ_N` pick the seeds to try, `FUZZ_RESTORE` adds the
/// restore checks to the check for panics, and `FUZZ_MAX_FAILURES` is how
/// many failing cases to shrink and print before stopping.
#[test]
#[ignore]
fn random_input_long() {
    let n: u64 = std::env::var("FUZZ_N").ok().and_then(|s| s.parse().ok()).unwrap_or(200_000);
    let start: u64 = std::env::var("FUZZ_START").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let check_restore = std::env::var("FUZZ_RESTORE").is_ok();
    let max_failures: usize =
        std::env::var("FUZZ_MAX_FAILURES").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    std::panic::set_hook(Box::new(|_| {}));
    let mut failures = 0;
    for seed in start..start + n {
        let case = gen_case(seed);
        if case.check(check_restore).is_some() {
            let case = case.shrink(check_restore);
            let problem = case.check(check_restore).unwrap_or_default();
            eprintln!("FAILED SEED {seed}:\n{}{problem}\n", case.describe());
            failures += 1;
            if failures >= max_failures {
                break;
            }
        }
    }
    let _ = std::panic::take_hook();
    assert_eq!(failures, 0);
}
