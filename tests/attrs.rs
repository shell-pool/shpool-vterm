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
            reset_codes,
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
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("abcde"),
            term::control_codes().end_link,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("f"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    link_jump { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("a"),
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("b")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("a"),
            term::control_codes().end_link,
            term::Crlf::default(),
            term::Raw::from(" "),
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("b"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    redundant_bold { scrollback_lines: 100, width: 100, height: 100 }
    <= term::control_codes().bold,
       term::Raw::from("a"),
       term::control_codes().bold,
       term::Raw::from("b")
    => ContentRegion::All =>
            reset_codes,
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
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("ab"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    underline { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().underline,
       term::Raw::from("b"),
       term::control_codes().undo_underline,
       term::Raw::from("a")
    => ContentRegion::All =>
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
            term::control_codes().bold,
            term::Raw::from("AC"),
            term::control_codes().reset_font_weight,
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().bold,
            term::control_codes().save_cursor,
            term::control_codes().reset_font_weight,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().hide_cursor
}

frag! {
    show_cursor { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().hide_cursor,
       term::control_codes().show_cursor
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    hide_cursor_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().hide_cursor,
       term::Raw::from("abc")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().hide_cursor
}

frag! {
    application_cursor_keys { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_cursor_keys
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_application_cursor_keys
}

frag! {
    disable_application_cursor_keys { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_cursor_keys,
       term::control_codes().disable_application_cursor_keys
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    application_keypad_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_keypad_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_application_keypad_mode
}

frag! {
    disable_application_keypad_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_keypad_mode,
       term::control_codes().disable_application_keypad_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    paste_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_paste_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_paste_mode
}

frag! {
    disable_paste_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_paste_mode,
       term::control_codes().disable_paste_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    report_focus { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_report_focus
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_report_focus
}

frag! {
    disable_report_focus { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_report_focus,
       term::control_codes().disable_report_focus
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    report_focus_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_report_focus,
       term::Raw::from("abc")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().enable_report_focus
}

frag! {
    report_focus_and_paste_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_paste_mode,
       term::control_codes().enable_report_focus
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_report_focus,
            term::control_codes().enable_paste_mode
}

// Mouse tracking and mouse encoding modes.
//
// These change how the client encodes mouse events onto the pty, so if a
// reattach does not restore them the client and the application disagree
// about the wire format: either the application stops seeing mouse input, or
// the client sends reports the application decodes as keystrokes.
//
// 1000/1002/1003 select how much the client reports (press only, press plus
// drag, or all motion) and 1006 selects SGR encoding, which is orthogonal to
// the other three.

frag! {
    mouse_tracking_normal { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1000])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::dec_private_modes_set(&[1000])
}

frag! {
    mouse_tracking_button_event { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1002])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::dec_private_modes_set(&[1002])
}

frag! {
    mouse_tracking_any_event { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1003])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::dec_private_modes_set(&[1003])
}

frag! {
    mouse_sgr_encoding { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1006])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::dec_private_modes_set(&[1006])
}

// The combination every modern full screen application actually sends.
frag! {
    mouse_tracking_and_sgr_encoding { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1002, 1006])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::dec_private_modes_set(&[1002]),
            term::ControlCodes::dec_private_modes_set(&[1006])
}

frag! {
    disable_mouse_tracking { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1002, 1006]),
       term::ControlCodes::dec_private_modes_reset(&[1002, 1006])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// An unrecognised parameter must not discard the rest of the list. The DECSET
// handler bails out of the whole sequence on the first parameter it does not
// know, so an unsupported mode silently takes every mode after it down too.
// Here 1004 is supported and restored on its own, but is lost when it follows
// a mode the handler does not recognise.
frag! {
    unknown_mode_does_not_discard_later_params { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_set(&[1000, 1004])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_report_focus,
            term::ControlCodes::dec_private_modes_set(&[1000])
}

// DECCKM (CSI ? 1 h, application cursor keys) and DECKPAM (ESC =, application
// keypad) are independent modes covering different key groups: DECCKM changes
// what the arrow keys send, DECKPAM changes the numeric keypad. Setting or
// clearing one must leave the other alone.
frag! {
    application_cursor_keys_and_keypad_are_independent
        { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_cursor_keys,
       term::control_codes().enable_application_keypad_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_application_cursor_keys,
            term::control_codes().enable_application_keypad_mode
}

// Resetting the keypad must not clear the cursor key mode as well.
frag! {
    disabling_keypad_leaves_cursor_keys_set { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_application_cursor_keys,
       term::control_codes().enable_application_keypad_mode,
       term::control_codes().disable_application_keypad_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_application_cursor_keys
}

frag! {
    cursor_style_blink_block { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::BlinkBlock
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::BlinkBlock
}

frag! {
    cursor_style_steady_block { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBlock
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBlock
}

frag! {
    cursor_style_blink_underline { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::BlinkUnderline
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::BlinkUnderline
}

frag! {
    cursor_style_steady_underline { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyUnderline
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyUnderline
}

frag! {
    cursor_style_blink_bar { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::BlinkBar
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::BlinkBar
}

frag! {
    cursor_style_steady_bar { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBar
}

frag! {
    cursor_style_switch { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::BlinkBlock,
       term::CursorStyle::SteadyBar
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBar
}

frag! {
    cursor_style_reset { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar,
       term::CursorStyle::Default
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    cursor_style_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar,
       term::Raw::from("abc")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBar
}

frag! {
    cursor_style_and_hide_cursor { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar,
       term::control_codes().hide_cursor
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBar,
            term::control_codes().hide_cursor
}

frag! {
    cursor_style_no_params { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar,
       term::ControlCodes::cursor_style(None)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    cursor_style_out_of_bounds { scrollback_lines: 10, width: 10, height: 10 }
    <= term::CursorStyle::SteadyBar,
       term::ControlCodes::cursor_style(Some(99))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::CursorStyle::SteadyBar
}

frag! {
    link_dangling_cursor_not_restored { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("text")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("text"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    decstr_resets_cursor_attrs { scrollback_lines: 10, width: 20, height: 10 }
    <= term::control_codes().bold,
       term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::control_codes().soft_reset,
       term::Raw::from("plain")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("plain"),
            term::ControlCodes::cursor_position(1, 6),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_cursor_attrs { scrollback_lines: 10, width: 20, height: 10 }
    <= term::control_codes().bold,
       term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::control_codes().hard_reset,
       term::Raw::from("plain")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("plain"),
            term::ControlCodes::cursor_position(1, 6),
            term::control_codes().clear_attrs
}

#[test]
fn link_dump_resets_link_at_start() {
    use shpool_vterm::term::AsTermInput;

    let term = shpool_vterm::Term::new(10, shpool_vterm::Size { width: 10, height: 10 });
    let mut want = vec![];
    crate::support::frag::reset_codes.term_input_into(&mut want);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut want);
    term::control_codes().clear_attrs.term_input_into(&mut want);

    let contents = term.contents(ContentRegion::All);
    assert_eq!(contents, want);
    // Explicitly verify the raw byte sequence of end_link is standard OSC 8 ;;
    // ST ("\x1b]8;;\x1b\\")
    assert!(contents.starts_with(b"\x1b]8;;\x1b\\"));
}

#[test]
fn end_link_wire_format() {
    use shpool_vterm::term::AsTermInput;

    let mut buf = vec![];
    term::control_codes().end_link.term_input_into(&mut buf);
    assert_eq!(buf, b"\x1b]8;;\x1b\\");
}

frag! {
    link_multiline_dangling { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("line1"),
       term::Crlf::default(),
       term::Raw::from("line2")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("line1"),
            term::control_codes().end_link,
            term::Crlf::default(),
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("line2"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(2, 6),
            term::control_codes().clear_attrs
}

frag! {
    link_closed_with_other_cursor_attrs { scrollback_lines: 10, width: 20, height: 10 }
    <= term::control_codes().bold,
       term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("link"),
       term::control_codes().end_link,
       term::Raw::from("plain")
    => ContentRegion::All =>
            reset_codes,
            term::control_codes().bold,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("link"),
            term::control_codes().end_link,
            term::Raw::from("plain"),
            term::control_codes().reset_font_weight,
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs,
            term::control_codes().bold
}

frag! {
    link_with_params_dangling_cursor_not_restored { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![b'i', b'd', b'=', b'1'], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("link")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![b'i', b'd', b'=', b'1'], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("link"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    save_restore_cursor_attrs_link { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("A"),
       term::control_codes().save_cursor,
       term::control_codes().end_link,
       term::Raw::from("B"),
       term::control_codes().restore_cursor,
       term::Raw::from("C")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("AC"),
            term::control_codes().end_link,
            // Like for the cursor attrs, the link is left out.
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    underline_colors_basic { scrollback_lines: 20, width: 20, height: 20 }
    <= term::Raw::from("a"),
       term::ControlCodes::underline_color_idx(1),
       term::Raw::from("b"),
       term::ControlCodes::underline_color_idx(100),
       term::Raw::from("c"),
       term::ControlCodes::underline_color_rgb(10, 20, 30),
       term::Raw::from("d"),
       term::control_codes().underline_color_default,
       term::Raw::from("e")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::ControlCodes::underline_color_idx(1),
            term::Raw::from("b"),
            term::ControlCodes::underline_color_idx(100),
            term::Raw::from("c"),
            term::ControlCodes::underline_color_rgb(10, 20, 30),
            term::Raw::from("d"),
            term::control_codes().underline_color_default,
            term::Raw::from("e"),
            term::ControlCodes::cursor_position(1, 6),
            term::control_codes().clear_attrs
}

frag! {
    underline_color_cursor_restore { scrollback_lines: 10, width: 20, height: 10 }
    <= term::Raw::from("a"),
       term::ControlCodes::underline_color_rgb(255, 128, 0)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::underline_color_rgb(255, 128, 0)
}

frag! {
    underline_color_colon_syntax { scrollback_lines: 10, width: 20, height: 10 }
    <= term::Raw::from("\x1b[58:2:0:255:192:185ma\x1b[58:2:10:20:30mb\x1b[58:5:42mc\x1b[59md")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::underline_color_rgb(255, 192, 185),
            term::Raw::from("a"),
            term::ControlCodes::underline_color_rgb(10, 20, 30),
            term::Raw::from("b"),
            term::ControlCodes::underline_color_idx(42),
            term::Raw::from("c"),
            term::control_codes().underline_color_default,
            term::Raw::from("d"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    underline_color_reset_clear_attrs { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::underline_color_rgb(100, 150, 200),
       term::Raw::from("colored"),
       term::control_codes().clear_attrs,
       term::Raw::from("plain")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::underline_color_rgb(100, 150, 200),
            term::Raw::from("colored"),
            term::control_codes().underline_color_default,
            term::Raw::from("plain"),
            term::ControlCodes::cursor_position(1, 13),
            term::control_codes().clear_attrs
}

frag! {
    save_restore_cursor_attrs_underline_color { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::underline_color_rgb(1, 2, 3),
       term::Raw::from("A"),
       term::control_codes().save_cursor,
       term::control_codes().underline_color_default,
       term::ControlCodes::cursor_forward(1),
       term::Raw::from("B"),
       term::control_codes().restore_cursor,
       term::Raw::from("C")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::underline_color_rgb(1, 2, 3),
            term::Raw::from("AC"),
            term::control_codes().underline_color_default,
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::ControlCodes::underline_color_rgb(1, 2, 3),
            term::control_codes().save_cursor,
            term::control_codes().underline_color_default,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::underline_color_rgb(1, 2, 3)
}

frag! {
    decstr_resets_underline_color { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::underline_color_rgb(1, 2, 3),
       term::control_codes().soft_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_underline_color { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::underline_color_rgb(1, 2, 3),
       term::control_codes().hard_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    enable_cursor_blink { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_cursor_blink
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_cursor_blink
}

frag! {
    disable_cursor_blink { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().disable_cursor_blink
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().disable_cursor_blink
}

frag! {
    cursor_blink_toggle { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_cursor_blink,
       term::control_codes().disable_cursor_blink
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().disable_cursor_blink
}

frag! {
    cursor_blink_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().disable_cursor_blink,
       term::Raw::from("abc")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().disable_cursor_blink
}

frag! {
    cursor_blink_and_hide_cursor { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().disable_cursor_blink,
       term::control_codes().hide_cursor
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().hide_cursor,
            term::control_codes().disable_cursor_blink
}

frag! {
    decstr_resets_cursor_blink { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().disable_cursor_blink,
       term::control_codes().soft_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_cursor_blink { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_cursor_blink,
       term::control_codes().hard_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    enable_insert_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_insert_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}

frag! {
    disable_insert_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_insert_mode,
       term::control_codes().disable_insert_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    insert_mode_with_text { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_insert_mode,
       term::Raw::from("abc")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}

frag! {
    decstr_resets_insert_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_insert_mode,
       term::control_codes().soft_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    ris_resets_insert_mode { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_insert_mode,
       term::control_codes().hard_reset
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    smooth_scroll_mode_ignored { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_smooth_scroll_mode,
       term::control_codes().disable_smooth_scroll_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    deccolm_mode_ignored { scrollback_lines: 10, width: 10, height: 10 }
    <= term::control_codes().enable_132_column_mode,
       term::control_codes().disable_132_column_mode
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    deccolm_and_decsclm_batched_ignored { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::dec_private_modes_reset(&[3, 4]),
       term::ControlCodes::dec_private_modes_set(&[3, 4])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    link_url_with_semicolons { scrollback_lines: 10, width: 20, height: 10 }
    <= term::Raw::from("\x1b]8;id=1;http://a/?b=1;c=2\x1b\\"),
       term::Raw::from("ab")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(b"id=1"[..].into(), b"http://a/?b=1;c=2"[..].into()),
            term::Raw::from("ab"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    link_empty_url_with_params_ends_link { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("a"),
       term::ControlCodes::start_link(b"id=1"[..].into(), smallvec![]),
       term::Raw::from("b")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("a"),
            term::control_codes().end_link,
            term::Raw::from("b"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    link_over_long_url_ignored { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(smallvec![], format!("http://a/{}", "b".repeat(3000)).as_bytes().into()),
       term::Raw::from("a")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("a"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    link_over_long_params_dropped { scrollback_lines: 10, width: 20, height: 10 }
    <= term::ControlCodes::start_link(format!("id={}", "b".repeat(300)).as_bytes().into(), smallvec![b'h', b't', b't', b'p']),
       term::Raw::from("a")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::start_link(smallvec![], smallvec![b'h', b't', b't', b'p']),
            term::Raw::from("a"),
            term::control_codes().end_link,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}
