#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::term;

frag! {
    simple_str { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("foobar")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("foobar"),
       term::ControlCodes::cursor_position(1, 7),
       term::control_codes().clear_attrs
}

frag! {
    newline2line { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("foo\r\nbar")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("foo"),
       term::Crlf::default(),
       term::Raw::from("bar"),
       term::ControlCodes::cursor_position(2, 4),
       term::control_codes().clear_attrs
}

frag! {
    wide_char { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Raw::from("😊"),
       term::Raw::from("B")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("A😊B"),
       term::ControlCodes::cursor_position(1, 5),
       term::control_codes().clear_attrs
}

frag! {
    wide_char_wrap { scrollback_lines: 100, width: 2, height: 10 }
    <= term::Raw::from("A"),
       term::Raw::from("😊")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("😊"),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().clear_attrs
}

frag! {
    wide_char_wrap_mid { scrollback_lines: 100, width: 3, height: 10 }
    <= term::Raw::from("a😊b")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("a😊"),
       term::Crlf::default(),
       term::Raw::from("b"),
       term::ControlCodes::cursor_position(2, 2),
       term::control_codes().clear_attrs
}

frag! {
    erase_display_to_end { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABCDEF"),
       term::ControlCodes::cursor_backwards(3),
       term::control_codes().erase_to_end
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("ABC"),
       term::ControlCodes::cursor_position(1, 4),
       term::control_codes().clear_attrs
}

frag! {
    erase_display_from_start { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABCDEF"),
       term::ControlCodes::cursor_backwards(3),
       term::control_codes().erase_from_start
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       // Standard behavior: inclusive of cursor (D cleared)
       term::Raw::from("    EF"),
       term::ControlCodes::cursor_position(1, 4),
       term::control_codes().clear_attrs
}

frag! {
    erase_screen_basic { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABCDEF"),
       term::control_codes().erase_screen
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::ControlCodes::cursor_position(1, 7),
       term::control_codes().clear_attrs
}

frag! {
    erase_screen_no_scrollback { scrollback_lines: 10, width: 5, height: 2 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::control_codes().erase_screen
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::Raw::from("1"),
       term::Crlf::default(),
       term::Crlf::default(),
       term::ControlCodes::cursor_position(2, 2),
       term::control_codes().clear_attrs
}

frag! {
    erase_scrollback { scrollback_lines: 10, width: 5, height: 2 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::control_codes().erase_scrollback
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,
       term::ControlCodes::cursor_position(2, 2),
       term::control_codes().clear_attrs
}

frag! {
    erase_in_line { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABCDE"),
       term::ControlCodes::cursor_backwards(4), // at B (1)
       term::control_codes().erase_to_end_of_line,
       term::Raw::from("X"),

       term::ControlCodes::cursor_next_line(1),
       term::Raw::from("ABCDE"),
       term::ControlCodes::cursor_backwards(2), // at D (3)
       term::control_codes().erase_to_start_of_line,
       term::Raw::from("Y"),

       term::ControlCodes::cursor_next_line(1),
       term::Raw::from("ABCDE"),
       term::ControlCodes::cursor_backwards(2), // at D (3)
       term::control_codes().erase_line,
       term::Raw::from("Z"),

       term::ControlCodes::cursor_next_line(1),
       term::Raw::from("ABCDE"),
       term::ControlCodes::cursor_backwards(4), // at B (1)
       term::Raw::from("\x1b[K"), // Raw CSI K (default 0)
       term::Raw::from("W")
    => term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().clear_screen,

       term::Raw::from("AX"),
       term::Crlf::default(),

       term::Raw::from("   YE"),
       term::Crlf::default(),

       term::Raw::from("   Z"),
       term::Crlf::default(),

       term::Raw::from("AW"),

       term::ControlCodes::cursor_position(4, 3),
       term::control_codes().clear_attrs
}
