#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::term;

frag! {
    underline { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("a"),
       term::control_codes().underline,
       term::Raw::from("b"),
       term::control_codes().undo_underline,
       term::Raw::from("a")
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
    => term::control_codes().clear_attrs,
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
