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

use shpool_vterm::{
    term::{Attrs, Color, FontWeight, UnderlineStyle},
    CellView, Size, Term,
};

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
        // xterm ignores HPB (CSI Pn j) and VPB (CSI Pn k). Its parse table
        // for CSI sequences maps the final bytes j and k to
        // CASE_GROUND_STATE, which does nothing:
        // https://github.com/ThomasDickey/xterm-snapshots/blob/5c75ead9a75d4b2e666ca06fa94503b5766ed93f/VTPrsTbl.c#L936-L937
        // "Horizontal Position Backward",
        "Horizontal and Vertical Position",
        "Vertical Position Absolute",
        "Vertical Position Relative",
        // xterm ignores VPB (CSI Pn k), see HPB above.
        // "Vertical Position Backward",
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
        // libvterm keeps the cursor column on IL, but xterm moves it to the
        // left margin:
        // https://github.com/ThomasDickey/xterm-snapshots/blob/5c75ead9a75d4b2e666ca06fa94503b5766ed93f/util.c#L1650
        // "IL",
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
        "Newline/Linefeed mode",
        "DEC origin mode",
        "Origin mode with DECSLRM",
        "Origin mode bounds cursor to scrolling region",
        "Origin mode without scroll region",
    ],
    state_resize: "16state_resize" => [
        "Resize",
        "Resize without reset",
        // shpool-vterm reflows on resize, so the cursor follows the text onto the
        // next row rather than being clamped. libvterm's state layer doesn't reflow.
        // "Resize shrink moves cursor",
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
        "Set up state",
        "Change state",
        "Restore",
        "Save/restore using DECSC/DECRC",
        "Save twice, restore twice happens on both edge transitions",
    ],
    state_reset: "27state_reset" => [
        "RIS homes cursor",
        "RIS clears tabstops",
    ],
    state_dbl_wh: "28state_dbl_wh" => [
        // Double width/height lines (ESC # 3 through 6) are supported by
        // xterm, but most terminals (VTE, kitty, Alacritty) ignore them, so
        // we do too.
        // "Double Width, Single Height",
        "Double Height",
    ],
    state_pen: "30state_pen" => [
        "Reset",
        "Bold",
        "Underline",
        "Italic",
        "Blink",
        "Reverse",
        // "Font Selection", // alternate fonts (SGR 10-19) unsupported
        "Foreground",
        "Background",
        // "Bold+ANSI colour == highbright", // harness.c turns on bold_highbright
        // "Super/Subscript", // super/subscript (SGR 73-75) unsupported
        "DECSTR resets pen attributes",
    ],
    state_flow: "32state_flow" => [
        "Spillover text marks continuation on second line",
        "CRLF in column 80 does not mark continuation",
        "EL cancels continuation of following line",
    ],
    screen_ascii: "60screen_ascii" => [
        "Get",
        "Erase",
        "Copycell",
        "Space padding",
        "Linefeed padding",
        "Altscreen",
    ],
    screen_unicode: "61screen_unicode" => [
        "Single width UTF-8",
        "Wide char",
        "Combining char",
        // "10 combining accents should not crash", // libvterm keeps 5 combining chars, we keep 16
        // libvterm keeps 5 combining chars, we keep 16
        // "40 combining accents in two split writes of 20 should not crash",
        "Outputing CJK doublewidth in 80th column should wraparound to next line and not crash\"",
    ],
    screen_damage: "62screen_damage" => [
        "Merge scroll*2 with damage",
    ],
    screen_resize: "63screen_resize" => [
        "Resize wider preserves cells",
        "Resize wider allows print in new area",
        "Resize shorter with blanks just truncates",
        "Resize shorter with content must scroll",
        "Resize shorter does not lose line with cursor",
        "Resize shorter does not send the cursor to a negative row",
        // "Resize taller attempts to pop scrollback", // harness.c makes up the scrollback it pops
        "Resize can operate on altscreen",
    ],
    screen_pen: "64screen_pen" => [
        "Plain",
        "Bold",
        "Italic",
        "Underline",
        "Reset",
        // "Font", // alternate fonts (SGR 10-19) unsupported
        "Foreground",
        "Background",
        // "Super/subscript", // super/subscript (SGR 73-75) unsupported
        // erased cells don't keep the fg color
        // "EL sets only colours to end of line, not other attrs",
        // "DECSCNM xors reverse for entire screen", // DECSCNM unsupported
        // "Set default colours", // SETDEFAULTCOL is libvterm API
    ],
    screen_protect: "65screen_protect" => [
        // "Selective erase", // DECSCA unsupported
        "Non-selective erase",
    ],
    screen_extent: "66screen_extent" => [
        "Bold extent",
    ],
    screen_dbl_wh: "67screen_dbl_wh" => [
        "Single Width, Single Height",
        // Double width/height lines are unsupported, see 28state_dbl_wh.
        // "Double Width, Single Height",
        // "Double Height",
        // "Late change",
        // "DWL doesn't spill over on scroll",
    ],
    screen_reflow: "69screen_reflow" => [
        "Resize wider reflows wide lines",
        "Resize narrower can create continuation lines",
        "Shell wrapped prompt behaviour",
        "Cursor goes missing",
    ],
    vttest_01_movement_1: "90vttest_01-movement-1" => [
        "Output",
    ],
    vttest_01_movement_2: "90vttest_01-movement-2" => [
        "Output",
    ],
    vttest_01_movement_3: "90vttest_01-movement-3" => [
        "Output",
    ],
    vttest_01_movement_4: "90vttest_01-movement-4" => [
        "Output",
    ],
    vttest_02_screen_1: "90vttest_02-screen-1" => [
        "Output",
    ],
    vttest_02_screen_2: "90vttest_02-screen-2" => [
        "Output",
    ],
    vttest_02_screen_3: "90vttest_02-screen-3" => [
        "Output",
    ],
    vttest_02_screen_4: "90vttest_02-screen-4" => [
        "Output",
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
            // Configuring libvterm's callbacks does not change the terminal
            // state, and shpool-vterm is always in UTF-8 mode.
            Directive::WantState
            | Directive::WantScreen
            | Directive::Damage
            | Directive::Utf8(true) => {}
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
                match (answer(term, &query), normalize_want(&query, &want)) {
                    (Ok(got), Ok(want)) if got == want => {}
                    (Ok(got), Ok(want)) => failures.push(format!(
                        "line {num}: ?{query} failed\n    want: {want}\n    got:  {got}\n{term}"
                    )),
                    (Err(e), _) | (_, Err(e)) => failures.push(format!("line {num}: {e}")),
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
    let (name, args) = match query.split_once(' ') {
        Some((name, args)) => (name, args.trim()),
        None => (query, ""),
    };
    match name {
        "cursor" => {
            let pos = term.cursor();
            Ok(format!("{},{}", pos.row, pos.col))
        }
        "pen" => answer_pen(term.cursor_attrs(), args),
        "lineinfo" => {
            // We can't see the line above the top of the screen, so the top
            // row never counts as a continuation.
            let row = parse_num(args)?;
            Ok(if row > 0 && term.row_wrapped(row - 1) { "cont" } else { "" }.to_string())
        }
        "screen_chars" => {
            let chars = screen_chars(term, parse_rect(term, args)?)?;
            Ok(hex_list(chars.into_iter().map(u32::from)))
        }
        // `run-test.pl` answers this one with `?screen_chars <row>`.
        "screen_row" => {
            let row = parse_num(args)?;
            let rect =
                Rect { start_row: row, start_col: 0, end_row: row + 1, end_col: term.size().width };
            Ok(hex_list(screen_chars(term, rect)?.into_iter().map(u32::from)))
        }
        "screen_text" => {
            let text: String = screen_chars(term, parse_rect(term, args)?)?.into_iter().collect();
            Ok(hex_list(text.bytes().map(u32::from)))
        }
        "screen_cell" => {
            let (row, col) = parse_pos(args)?;
            answer_screen_cell(&get_cell(term, row, col)?)
        }
        "screen_eol" => {
            let (row, col) = parse_pos(args)?;
            for col in col..term.size().width {
                let cell = get_cell(term, row, col)?;
                if cell.wide_padding || !cell.chars.is_empty() {
                    return Ok("0".to_string());
                }
            }
            Ok("1".to_string())
        }
        "screen_attrs_extent" => {
            let (row, col) = parse_pos(args)?;
            let target = get_cell(term, row, col)?.attrs;
            let same = |col| get_cell(term, row, col).map(|cell| cell.attrs == target);
            let mut start = col;
            while start > 0 && same(start - 1)? {
                start -= 1;
            }
            let mut end = col;
            while end + 1 < term.size().width && same(end + 1)? {
                end += 1;
            }
            Ok(format!("{row},{start}-{},{end}", row + 1))
        }
        _ => Err(format!("unsupported query: ?{query}")),
    }
}

/// Put the want of a query into the same form `answer` produces, the way
/// `run-test.pl` does.
fn normalize_want(query: &str, want: &str) -> Result<String, String> {
    match query.split_whitespace().next() {
        Some("screen_chars" | "screen_row") if want.starts_with('"') => {
            Ok(hex_list(eval_perl_string(want)?.into_iter().map(u32::from)))
        }
        Some("screen_row") => {
            let chars = want
                .split(',')
                .filter(|c| !c.trim().is_empty())
                .map(|c| {
                    let c = c.trim();
                    u32::from_str_radix(c.trim_start_matches("0x"), 16)
                        .map_err(|e| format!("bad char '{c}': {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(hex_list(chars.into_iter()))
        }
        _ => Ok(want.to_string()),
    }
}

fn hex_list(vals: impl Iterator<Item = u32>) -> String {
    vals.map(|v| format!("0x{v:02x}")).collect::<Vec<_>>().join(",")
}

/// A rectangle of cells. The ends are exclusive.
struct Rect {
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
}

/// Parse either `start_row,start_col,end_row,end_col` or just a row, which
/// means the whole row.
fn parse_rect(term: &Term, args: &str) -> Result<Rect, String> {
    let nums = args.split(',').map(|n| parse_num(n.trim())).collect::<Result<Vec<_>, _>>()?;
    match nums.as_slice() {
        [start_row, start_col, end_row, end_col] => Ok(Rect {
            start_row: *start_row,
            start_col: *start_col,
            end_row: *end_row,
            end_col: *end_col,
        }),
        [row] => {
            Ok(Rect { start_row: *row, start_col: 0, end_row: row + 1, end_col: term.size().width })
        }
        _ => Err(format!("bad rect '{args}'")),
    }
}

/// Parse `row,col`.
fn parse_pos(args: &str) -> Result<(usize, usize), String> {
    let (row, col) = args.split_once(',').ok_or_else(|| format!("bad position '{args}'"))?;
    Ok((parse_num(row.trim())?, parse_num(col.trim())?))
}

fn get_cell(term: &Term, row: usize, col: usize) -> Result<CellView, String> {
    term.cell(row, col).ok_or_else(|| format!("{row},{col} is off the screen"))
}

/// The chars in the given rect, the way libvterm's `vterm_screen_get_chars`
/// gets them: blanks at the end of each row get dropped, blanks before a
/// char become spaces, and rows get joined with newlines.
fn screen_chars(term: &Term, rect: Rect) -> Result<Vec<char>, String> {
    let mut chars = vec![];
    for row in rect.start_row..rect.end_row {
        let mut blanks = 0;
        for col in rect.start_col..rect.end_col {
            let cell = get_cell(term, row, col)?;
            if cell.wide_padding {
                continue;
            }
            if cell.chars.is_empty() {
                blanks += 1;
                continue;
            }
            chars.extend(std::iter::repeat(' ').take(blanks));
            blanks = 0;
            chars.extend(cell.chars);
        }
        if row + 1 < rect.end_row {
            chars.push('\n');
        }
    }
    Ok(chars)
}

fn answer_screen_cell(cell: &CellView) -> Result<String, String> {
    // libvterm marks the padding after a wide char with a char of -1.
    let chars = if cell.wide_padding {
        String::from("0xffffffff")
    } else {
        cell.chars.iter().map(|c| format!("{:#x}", *c as u32)).collect::<Vec<_>>().join(",")
    };
    let width = if cell.width == 2 { 2 } else { 1 };

    let attrs = &cell.attrs;
    let mut flags = String::new();
    if attrs.font_weight == Some(FontWeight::Bold) {
        flags.push('B');
    }
    if attrs.underline.is_some() {
        flags.push_str(&format!("U{}", underline_num(attrs)));
    }
    if attrs.italic {
        flags.push('I');
    }
    if attrs.blink.is_some() {
        flags.push('K');
    }
    if attrs.inverse {
        flags.push('R');
    }

    Ok(format!(
        "{{{chars}}} width={width} attrs={{{flags}}} fg={} bg={}",
        screen_rgb(attrs.fgcolor, DEFAULT_FG)?,
        screen_rgb(attrs.bgcolor, DEFAULT_BG)?,
    ))
}

fn answer_pen(attrs: &Attrs, attr: &str) -> Result<String, String> {
    let on_off = |on: bool| String::from(if on { "on" } else { "off" });
    Ok(match attr {
        "bold" => on_off(attrs.font_weight == Some(FontWeight::Bold)),
        "underline" => underline_num(attrs).to_string(),
        "italic" => on_off(attrs.italic),
        "blink" => on_off(attrs.blink.is_some()),
        "reverse" => on_off(attrs.inverse),
        // shpool-vterm ignores the SGR codes for alternate fonts (10-19)
        // and super/subscript (73-75), so these never change.
        "font" => String::from("0"),
        "small" => on_off(false),
        "baseline" => String::from("normal"),
        "foreground" => pen_color(attrs.fgcolor, DEFAULT_FG, "is_default_fg"),
        "background" => pen_color(attrs.bgcolor, DEFAULT_BG, "is_default_bg"),
        _ => return Err(format!("unsupported query: ?pen {attr}")),
    })
}

/// libvterm's numbering for underline styles. It has no dotted or dashed
/// underlines, and treats them as single ones.
fn underline_num(attrs: &Attrs) -> u8 {
    match attrs.underline {
        None => 0,
        Some(UnderlineStyle::Double) => 2,
        Some(UnderlineStyle::Curly) => 3,
        Some(UnderlineStyle::Single | UnderlineStyle::Dotted | UnderlineStyle::Dashed) => 1,
    }
}

/// libvterm's default colors.
const DEFAULT_FG: (u8, u8, u8) = (240, 240, 240);
const DEFAULT_BG: (u8, u8, u8) = (0, 0, 0);

/// libvterm's palette for the 16 ANSI colors.
const ANSI_PALETTE: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (224, 0, 0),
    (0, 224, 0),
    (224, 224, 0),
    (0, 0, 224),
    (224, 0, 224),
    (0, 224, 224),
    (224, 224, 224),
    (128, 128, 128),
    (255, 64, 64),
    (64, 255, 64),
    (255, 255, 64),
    (64, 64, 255),
    (255, 64, 255),
    (64, 255, 255),
    (255, 255, 255),
];

/// A pen color, the way libvterm reports it before it resolves it to RGB.
fn pen_color(color: Color, default: (u8, u8, u8), default_flag: &str) -> String {
    match color {
        Color::Default => format!("rgb({},{},{},{default_flag})", default.0, default.1, default.2),
        Color::Idx(idx) => format!("idx({idx})"),
        Color::Rgb(r, g, b) => format!("rgb({r},{g},{b})"),
    }
}

/// A cell color, the way libvterm reports it after resolving it to RGB.
fn screen_rgb(color: Color, default: (u8, u8, u8)) -> Result<String, String> {
    let (r, g, b) = match color {
        Color::Default => default,
        Color::Idx(idx) => *ANSI_PALETTE
            .get(idx as usize)
            .ok_or_else(|| format!("no libvterm palette entry for color {idx}"))?,
        Color::Rgb(r, g, b) => (r, g, b),
    };
    Ok(format!("rgb({r},{g},{b})"))
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
    /// `DAMAGEMERGE <mode>` or `DAMAGEFLUSH`, which control how libvterm
    /// batches up its damage callbacks.
    Damage,
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
        "DAMAGEMERGE" | "DAMAGEFLUSH" => Directive::Damage,
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
