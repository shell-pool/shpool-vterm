//! Runs the libvterm test suite in `tests/data/libvterm` against
//! shpool-vterm.
//!
//! libvterm drives its `.test` files with `run-test.pl` and `harness.c`.
//! This reimplements just enough of that pair to run them against a `Term`.
//! The files are made up of a few kinds of lines:
//!
//! - `!Name` starts a section. Sections are not independent: each one picks up
//!   the terminal state that the sections before it left behind.
//! - Uppercase commands, like `RESET` or `PUSH "ABC"`, drive the terminal.
//! - Lowercase expectations, like `putglyph 0x41 1 0,0`, check the callbacks
//!   that libvterm fires. shpool-vterm has no such callbacks, so these get
//!   skipped.
//! - `?query = want` assertions, like `?cursor = 0,3`, check the terminal
//!   state. These are what we actually test.
//!
//! Plenty of the suite covers things shpool-vterm does not model, so only
//! the sections listed in the allowlist below get checked. Every command in
//! a file still gets run, so that allowlisted sections start from the same
//! state they would under libvterm.

use std::{fs, path::PathBuf};

use shpool_vterm::{Size, Term};

/// The size `harness.c` creates its terminal with.
const INIT_SIZE: Size = Size { width: 80, height: 25 };

/// libvterm's screen keeps no scrollback, and none of the queries look at
/// it, so we keep the minimum.
const SCROLLBACK_LINES: usize = 0;

/// Defines a test for each libvterm test file, which checks the sections
/// listed for it.
macro_rules! libvterm_tests {
    ( $( $test_name:ident : $file:literal => [ $( $section:literal ),* $(,)? ] ),* $(,)? ) => {
        $(
            #[test]
            fn $test_name() {
                run_test_file($file, &[ $( $section ),* ]);
            }
        )*
    };
}

// The allowlist. Each entry maps a test file (without the `.test`
// extension) to the names of the sections in it that should pass.
// Sections that fail are left in, commented out.
libvterm_tests! {
    state_movecursor: "11state_movecursor" => [
        "Implicit",
        "Backspace",
        "Horizontal Tab",
        "Carriage Return",
        "Linefeed",
        "Backspace bounded by lefthand edge",
        "Backspace cancels phantom",
        "HT bounded by righthand edge",
        "Index",
        "Reverse Index",
        "Newline",
        "Cursor Forward",
        "Cursor Down",
        "Cursor Up",
        "Cursor Backward",
        "Cursor Next Line",
        "Cursor Previous Line",
        "Cursor Horizonal Absolute",
        "Cursor Position",
        "Cursor Position cancels phantom",
        "Bounds Checking",
        "Horizontal Position Absolute",
        "Horizontal Position Relative",
        // "Horizontal Position Backward", // want 0,4, got 0,7
        "Horizontal and Vertical Position",
        "Vertical Position Absolute",
        "Vertical Position Relative",
        // "Vertical Position Backward", // want 4,2, got 6,2
        "Cursor Horizontal Tab",
        "Cursor Backward Tab",
    ],
    state_scroll: "12state_scroll" => [
        "Linefeed",
        "Linefeed in DECSTBM",
        "Linefeed outside DECSTBM",
        "Index in DECSTBM",
        "Reverse Index in DECSTBM",
        "Scroll Down",
        "Scroll Up",
        "SD/SU in DECSTBM+DECSLRM",
        "Scroll Down move+erase emulation",
        "Scroll Up move+erase emulation",
        "DECSTBM resets cursor position",
    ],
    state_edit: "13state_edit" => [
        "ICH",
        "DCH",
        "ECH",
        // "IL", // want 1,1, got 1,0
        "DL",
        "EL 0",
        "EL 1",
        "EL 2",
        "SEL",
        "ED 0",
        "ED 1",
        "ED 2",
        "SED",
        "ICH move+erase emuation",
        "DCH move+erase emulation",
    ],
    state_mode: "15state_mode" => [
        "Insert/Replace Mode",
        // "Newline/Linefeed mode", // want 2,0, got 2,4
        "DEC origin mode",
        "Origin mode with DECSLRM",
        "Origin mode bounds cursor to scrolling region",
        "Origin mode without scroll region",
    ],
    state_resize: "16state_resize" => [
        "Resize",
        "Resize without reset",
        // "Resize shrink moves cursor", // want 0,79, got 1,5
        "Resize grow doesn't cancel phantom",
    ],
    state_wrapping: "20state_wrapping" => [
        "79th Column",
        "80th Column Phantom",
        "Line Wraparound",
        "Line Wraparound during combined write",
        "DEC Auto Wrap Mode",
        "80th column causes linefeed on wraparound",
        "80th column phantom linefeed phantom cancelled by explicit cursor move",
    ],
    state_tabstops: "21state_tabstops" => [
        "Initial",
        "HTS",
        "TBC 0",
        "TBC 3",
        "Tabstops after resize",
    ],
    state_save: "22state_save" => [
        "Save/restore using DECSC/DECRC",
        "Save twice, restore twice happens on both edge transitions",
    ],
    state_reset: "27state_reset" => [
        "RIS homes cursor",
        "RIS clears tabstops",
    ],
    state_dbl_wh: "28state_dbl_wh" => [
        // "Double Width, Single Height", // want 1,1, got 0,41
        "Double Height",
    ],
    screen_resize: "63screen_resize" => [
        "Resize shorter does not send the cursor to a negative row",
    ],
    screen_reflow: "69screen_reflow" => [
        "Cursor goes missing",
    ],
}

#[test]
fn parse_all_test_files() {
    let mut paths: Vec<PathBuf> = fs::read_dir(data_dir())
        .expect("reading libvterm test dir")
        .map(|entry| entry.expect("reading libvterm test dir entry").path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "test"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no libvterm test files found");

    for path in paths {
        let src = fs::read_to_string(&path).expect("reading libvterm test file");
        if let Err(e) = parse(&src) {
            panic!("{}: {e}", path.display());
        }
    }
}

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("libvterm")
}

/// Run every command in the given test file, checking the queries in the
/// allowlisted sections.
fn run_test_file(file: &str, allowlist: &[&str]) {
    let path = data_dir().join(format!("{file}.test"));
    let src =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let lines = parse(&src).unwrap_or_else(|e| panic!("{file}: {e}"));

    for section in allowlist {
        let found = lines
            .iter()
            .any(|line| matches!(&line.directive, Directive::Section(name) if name == section));
        assert!(found, "{file}: allowlisted section '{section}' not found");
    }

    let mut term: Option<Term> = None;
    let mut size = INIT_SIZE;
    // Commands before the first section set things up for all of them,
    // so they always need to be supported.
    let mut checking = true;
    let mut section_name = String::from("<preamble>");
    let mut section_queries = 0;
    let mut failures = vec![];

    for Line { num, directive } in lines {
        match directive {
            Directive::Section(name) => {
                if checking && section_name != "<preamble>" && section_queries == 0 {
                    failures.push(format!("section '{section_name}' has no queries to check"));
                }
                checking = allowlist.contains(&name.as_str());
                section_name = name;
                section_queries = 0;
            }
            Directive::Init => {
                if term.is_none() {
                    term = Some(Term::new(SCROLLBACK_LINES, size));
                }
            }
            // Turning on libvterm's callbacks does not change the terminal
            // state, and shpool-vterm is always in UTF-8 mode.
            Directive::WantState | Directive::WantScreen | Directive::Utf8(true) => {}
            Directive::Reset => {
                if term.is_some() {
                    term = Some(Term::new(SCROLLBACK_LINES, size));
                }
            }
            Directive::Push(bytes) => match term.as_mut() {
                Some(term) => term.process(&bytes),
                None => failures.push(format!("line {num}: PUSH before INIT")),
            },
            Directive::Resize(new_size) => {
                size = new_size;
                if let Some(term) = term.as_mut() {
                    term.resize(size);
                }
            }
            Directive::Utf8(false) => {
                if checking {
                    failures.push(format!("line {num}: unsupported command: UTF8 0"));
                }
            }
            Directive::Unsupported(cmd) => {
                if checking {
                    failures.push(format!("line {num}: unsupported command: {cmd}"));
                }
            }
            Directive::Expectation(_) => {}
            Directive::Query { query, want } => {
                if !checking {
                    continue;
                }
                section_queries += 1;
                let Some(term) = term.as_ref() else {
                    failures.push(format!("line {num}: ?{query} before INIT"));
                    continue;
                };
                match answer(term, &query) {
                    Ok(got) if got == want => {}
                    Ok(got) => failures.push(format!(
                        "line {num}: ?{query} failed\n    want: {want}\n    got:  {got}\n{term}"
                    )),
                    Err(e) => failures.push(format!("line {num}: {e}")),
                }
            }
        }
    }
    if checking && section_queries == 0 {
        failures.push(format!("section '{section_name}' has no queries to check"));
    }

    if !failures.is_empty() {
        panic!("{file}: {} failure(s):\n{}", failures.len(), failures.join("\n"));
    }
}

/// Answer a query (without the leading `?`) in the format `harness.c` does.
fn answer(term: &Term, query: &str) -> Result<String, String> {
    match query {
        "cursor" => {
            let pos = term.cursor();
            Ok(format!("{},{}", pos.row, pos.col))
        }
        _ => Err(format!("unsupported query: ?{query}")),
    }
}

#[derive(Debug)]
struct Line {
    /// The 1-based line number in the test file.
    num: usize,
    directive: Directive,
}

#[derive(Debug)]
enum Directive {
    /// `!Name`
    Section(String),
    /// `INIT`
    Init,
    /// `UTF8 1` or `UTF8 0`
    Utf8(bool),
    /// `WANTSTATE <flags>`
    WantState,
    /// `WANTSCREEN <flags>`
    WantScreen,
    /// `RESET`
    Reset,
    /// `PUSH <perl string expression>`
    Push(Vec<u8>),
    /// `RESIZE rows,cols`
    Resize(Size),
    /// A command the harness does not model, like `INKEY`.
    Unsupported(String),
    /// A lowercase callback expectation, like `putglyph 0x41 1 0,0`.
    #[allow(dead_code)]
    Expectation(String),
    /// `?query = want`
    Query { query: String, want: String },
}

/// Parse a test file the way `run-test.pl` reads it, with `$SEQ` and `$REP`
/// lines already expanded.
fn parse(src: &str) -> Result<Vec<Line>, String> {
    let mut lines = vec![];
    for (i, line) in src.lines().enumerate() {
        let num = i + 1;
        let line = line.trim();
        if line == "__END__" {
            break;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parse_line(num, line, &mut lines).map_err(|e| format!("line {num}: {e}"))?;
    }
    Ok(lines)
}

fn parse_line(num: usize, line: &str, into: &mut Vec<Line>) -> Result<(), String> {
    if let Some(rest) = line.strip_prefix("$SEQ") {
        let (range, inner) = rest.split_once(':').ok_or("$SEQ without ':'")?;
        let (low, high) = match range.split_whitespace().collect::<Vec<_>>().as_slice() {
            [low, high] => (parse_num(low)?, parse_num(high)?),
            _ => return Err(format!("bad $SEQ range '{range}'")),
        };
        for val in low..=high {
            parse_line(num, inner.trim().replace("\\#", &val.to_string()).as_str(), into)?;
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("$REP") {
        let (count, inner) = rest.split_once(':').ok_or("$REP without ':'")?;
        for _ in 0..parse_num(count.trim())? {
            parse_line(num, inner.trim(), into)?;
        }
        return Ok(());
    }

    let directive = if let Some(name) = line.strip_prefix('!') {
        Directive::Section(name.to_string())
    } else if let Some(rest) = line.strip_prefix('?') {
        let (query, want) = rest.split_once('=').ok_or("query without '='")?;
        Directive::Query { query: query.trim().to_string(), want: want.trim().to_string() }
    } else if line.starts_with(|c: char| c.is_ascii_uppercase()) {
        parse_command(line)?
    } else if line.starts_with(|c: char| c.is_ascii_lowercase()) {
        Directive::Expectation(line.to_string())
    } else {
        return Err(format!("unrecognised line '{line}'"));
    };
    into.push(Line { num, directive });
    Ok(())
}

fn parse_command(line: &str) -> Result<Directive, String> {
    let (cmd, args) = match line.split_once(' ') {
        Some((cmd, args)) => (cmd, args.trim()),
        None => (line, ""),
    };
    Ok(match cmd {
        "INIT" => Directive::Init,
        "UTF8" => Directive::Utf8(parse_num(args)? != 0),
        "WANTSTATE" => Directive::WantState,
        "WANTSCREEN" => Directive::WantScreen,
        "RESET" => Directive::Reset,
        "PUSH" => Directive::Push(eval_perl_string(args)?),
        "RESIZE" => {
            let (rows, cols) = args.split_once(',').ok_or("RESIZE without ','")?;
            Directive::Resize(Size {
                width: parse_num(cols.trim())?,
                height: parse_num(rows.trim())?,
            })
        }
        _ => Directive::Unsupported(line.to_string()),
    })
}

fn parse_num(s: &str) -> Result<usize, String> {
    s.parse().map_err(|e| format!("bad number '{s}': {e}"))
}

/// Evaluate the subset of perl string expressions that the test files use
/// (which `run-test.pl` passes to `eval`): double quoted strings, joined
/// with `.` and repeated with `x`, like `"e" . "\xCC\x81" x 10`.
fn eval_perl_string(expr: &str) -> Result<Vec<u8>, String> {
    let mut out = vec![];
    let mut rest = expr.trim_start();
    loop {
        let (s, after) = parse_perl_string_literal(rest)?;
        rest = after.trim_start();
        if let Some(after) = rest.strip_prefix('x') {
            let after = after.trim_start();
            let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
            out.extend(s.repeat(parse_num(&after[..end])?));
            rest = after[end..].trim_start();
        } else {
            out.extend(s);
        }

        if rest.is_empty() {
            return Ok(out);
        }
        rest = rest
            .strip_prefix('.')
            .ok_or_else(|| format!("unexpected '{rest}' in '{expr}'"))?
            .trim_start();
    }
}

/// Parse a double quoted perl string from the start of `s`, returning its
/// bytes and whatever comes after it.
fn parse_perl_string_literal(s: &str) -> Result<(Vec<u8>, &str), String> {
    let mut chars =
        s.strip_prefix('"').ok_or_else(|| format!("expected '\"' at '{s}'"))?.char_indices();
    let mut out = vec![];
    fn push_char(out: &mut Vec<u8>, c: char) {
        out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
    }

    while let Some((i, c)) = chars.next() {
        match c {
            // Skip the opening quote as well as this one.
            '"' => return Ok((out, &s[i + 2..])),
            '\\' => {
                let (_, esc) = chars.next().ok_or("unterminated escape")?;
                match esc {
                    'e' => out.push(0x1b),
                    'n' => out.push(b'\n'),
                    'r' => out.push(b'\r'),
                    't' => out.push(b'\t'),
                    'b' => out.push(0x08),
                    'a' => out.push(0x07),
                    '0' => {
                        let digits: String = chars
                            .clone()
                            .take(2)
                            .map(|(_, c)| c)
                            .take_while(|c| c.is_digit(8))
                            .collect();
                        for _ in 0..digits.len() {
                            chars.next();
                        }
                        out.push(u8::from_str_radix(&format!("0{digits}"), 8).unwrap());
                    }
                    'x' => {
                        let digits: String = if chars.clone().next().map(|(_, c)| c) == Some('{') {
                            chars.next();
                            let digits: String =
                                chars.by_ref().map(|(_, c)| c).take_while(|c| *c != '}').collect();
                            digits
                        } else {
                            let digits: String = chars
                                .clone()
                                .take(2)
                                .map(|(_, c)| c)
                                .take_while(|c| c.is_ascii_hexdigit())
                                .collect();
                            for _ in 0..digits.len() {
                                chars.next();
                            }
                            digits
                        };
                        let val = u32::from_str_radix(&digits, 16)
                            .map_err(|e| format!("bad \\x escape '{digits}': {e}"))?;
                        match u8::try_from(val) {
                            Ok(byte) => out.push(byte),
                            Err(_) => push_char(
                                &mut out,
                                char::from_u32(val).ok_or(format!("bad codepoint {val:#x}"))?,
                            ),
                        }
                    }
                    // Anything else, like `\"` or `\$`, is just the char.
                    c => push_char(&mut out, c),
                }
            }
            c => push_char(&mut out, c),
        }
    }
    Err(format!("unterminated string '{s}'"))
}

#[test]
fn eval_perl_string_forms() {
    assert_eq!(eval_perl_string(r#""ABC""#).unwrap(), b"ABC");
    assert_eq!(eval_perl_string(r#""\e[4;2H""#).unwrap(), b"\x1b[4;2H");
    assert_eq!(eval_perl_string(r#""A\e[1\"qB""#).unwrap(), b"A\x1b[1\"qB");
    assert_eq!(eval_perl_string(r#""\xC3\x81""#).unwrap(), b"\xc3\x81");
    assert_eq!(eval_perl_string(r#""\x{9d}1\x{100}""#).unwrap(), b"\x9d1\xc4\x80");
    assert_eq!(eval_perl_string(r#""\0\012""#).unwrap(), b"\x00\x0a");
    assert_eq!(eval_perl_string(r#""\n"x3"#).unwrap(), b"\n\n\n");
    assert_eq!(eval_perl_string(r#""e" . "\xCC\x81" x 2"#).unwrap(), b"e\xcc\x81\xcc\x81");
    assert_eq!(
        eval_perl_string(r#""\e]52;c;" . "LS0t"x2 . "\e\\""#).unwrap(),
        b"\x1b]52;c;LS0tLS0t\x1b\\"
    );
    assert!(eval_perl_string(r#""unterminated"#).is_err());
}
