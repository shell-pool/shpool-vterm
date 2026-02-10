#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    link_basic { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p', b':', b'/', b'/', b'a', b'.', b'c']),
       term::Raw::from("link"),
       term::control_codes().end_link
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p', b':', b'/', b'/', b'a', b'.', b'c']),
            term::Raw::from("link"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    link_wrap { scrollback_lines: 10, width: 5, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("abcdef")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("abcde"),
            term::control_codes().end_link,
            term::Crlf::default(),
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("f"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p'])
}

frag! {
    link_jump { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("a"),
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("b")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("a"),
            term::control_codes().end_link,
            term::Crlf::default(),
            term::Raw::from(" "),
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("b"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p'])
}

frag! {
    redundant_bold { scrollback_lines: 100, width: 100, height: 100 }
    <= term::control_codes().bold,
       term::Raw::from("a"),
       term::control_codes().bold,
       term::Raw::from("b")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::control_codes().bold,
            term::Raw::from("ab"),
            term::control_codes().reset_font_weight,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().bold
}

frag! {
    redundant_link { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("a"),
       term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("b")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("ab"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p'])
}

frag! {
    underline { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().underline,
       term::Raw::from("b"),
       term::control_codes().undo_underline,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().underline,
            term::Raw::from("b"),
            term::control_codes().undo_underline,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    bold { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().bold,
       term::Raw::from("b"),
       term::control_codes().reset_font_weight,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().bold,
            term::Raw::from("b"),
            term::control_codes().reset_font_weight,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    italic { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().italic,
       term::Raw::from("b"),
       term::control_codes().undo_italic,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().italic,
            term::Raw::from("b"),
            term::control_codes().undo_italic,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    inverse { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().inverse,
       term::Raw::from("b"),
       term::control_codes().undo_inverse,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().inverse,
            term::Raw::from("b"),
            term::control_codes().undo_inverse,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    faint { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().faint,
       term::Raw::from("b"),
       term::control_codes().reset_font_weight,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().faint,
            term::Raw::from("b"),
            term::control_codes().reset_font_weight,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    blink { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().slow_blink,
       term::Raw::from("b"),
       term::control_codes().undo_blink,
       term::Raw::from("a"),
       term::control_codes().rapid_blink,
       term::Raw::from("c")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().slow_blink,
            term::Raw::from("b"),
            term::control_codes().undo_blink,
            term::Raw::from("a"),
            term::control_codes().rapid_blink,
            term::Raw::from("c"),
            term::control_codes().undo_blink,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs,
            term::control_codes().rapid_blink
}

frag! {
    conceal { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().conceal,
       term::Raw::from("b"),
       term::control_codes().undo_conceal,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().conceal,
            term::Raw::from("b"),
            term::control_codes().undo_conceal,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    strikethrough { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().strikethrough,
       term::Raw::from("b"),
       term::control_codes().undo_strikethrough,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().strikethrough,
            term::Raw::from("b"),
            term::control_codes().undo_strikethrough,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    framed { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().framed,
       term::Raw::from("b"),
       term::control_codes().undo_framed,
       term::Raw::from("a"),
       term::control_codes().encircled,
       term::Raw::from("c")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().framed,
            term::Raw::from("b"),
            term::control_codes().undo_framed,
            term::Raw::from("a"),
            term::control_codes().encircled,
            term::Raw::from("c"),
            term::control_codes().undo_framed,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs,
            term::control_codes().encircled
}

frag! {
    overline { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().overline,
       term::Raw::from("b"),
       term::control_codes().undo_overline,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().overline,
            term::Raw::from("b"),
            term::control_codes().undo_overline,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    double_underline { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().double_underline,
       term::Raw::from("b"),
       term::control_codes().undo_underline,
       term::Raw::from("a")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::control_codes().double_underline,
            term::Raw::from("b"),
            term::control_codes().undo_underline,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    save_restore_cursor_attrs { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().bold,
       term::Raw::from("A"),
       term::control_codes().save_cursor,
       term::control_codes().reset_font_weight,
       term::ControlCodes::cursor_forward(1),
       term::Raw::from("B"),
       term::control_codes().restore_cursor,
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::control_codes().bold,
            term::Raw::from("AC"),
            term::control_codes().reset_font_weight,
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().bold
}

frag! {
    fg_colors { scrollback_lines: 20, width: 20, height: 20 }
    <= term::Raw::from("a"),
       term::ControlCodes::fgcolor_idx(1),
       term::Raw::from("b"),
       term::ControlCodes::fgcolor_idx(9),
       term::Raw::from("c"),
       term::ControlCodes::fgcolor_idx(100),
       term::Raw::from("d"),
       term::ControlCodes::fgcolor_rgb(10, 20, 30),
       term::Raw::from("e"),
       term::control_codes().fgcolor_default,
       term::Raw::from("f")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::ControlCodes::fgcolor_idx(1),
            term::Raw::from("b"),
            term::ControlCodes::fgcolor_idx(9),
            term::Raw::from("c"),
            term::ControlCodes::fgcolor_idx(100),
            term::Raw::from("d"),
            term::ControlCodes::fgcolor_rgb(10, 20, 30),
            term::Raw::from("e"),
            term::control_codes().fgcolor_default,
            term::Raw::from("f"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
}

frag! {
    bg_colors { scrollback_lines: 20, width: 20, height: 20 }
    <= term::Raw::from("a"),
       term::ControlCodes::bgcolor_idx(2),
       term::Raw::from("b"),
       term::ControlCodes::bgcolor_idx(10),
       term::Raw::from("c"),
       term::ControlCodes::bgcolor_idx(200),
       term::Raw::from("d"),
       term::ControlCodes::bgcolor_rgb(40, 50, 60),
       term::Raw::from("e"),
       term::control_codes().bgcolor_default,
       term::Raw::from("f")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("a"),
            term::ControlCodes::bgcolor_idx(2),
            term::Raw::from("b"),
            term::ControlCodes::bgcolor_idx(10),
            term::Raw::from("c"),
            term::ControlCodes::bgcolor_idx(200),
            term::Raw::from("d"),
            term::ControlCodes::bgcolor_rgb(40, 50, 60),
            term::Raw::from("e"),
            term::control_codes().bgcolor_default,
            term::Raw::from("f"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
}

frag! {
    hide_cursor { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().hide_cursor
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().hide_cursor
}

frag! {
    show_cursor { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().hide_cursor,
       term::control_codes().show_cursor
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    hide_cursor_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().hide_cursor,
       term::Raw::from("abc")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().hide_cursor
}

frag! {
    application_keypad_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_keypad_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_application_keypad_mode
}

frag! {
    disable_application_keypad_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_keypad_mode,
       term::control_codes().disable_application_keypad_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    paste_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_paste_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_paste_mode
}

frag! {
    disable_paste_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_paste_mode,
       term::control_codes().disable_paste_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}
