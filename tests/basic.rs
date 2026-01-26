#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

frag! {
    simple_str { scrollback_lines: 100, width: 100, height: 100 }
    <= term::Raw::from("foobar")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("foobar"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("foobar"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(10) =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("foobar"),
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
}

frag! {
    newline2line { scrollback_lines: 100, width: 10, height: 1 }
    <= term::Raw::from("foo\r\nbar")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("foo"),
            term::Crlf::default(),
            term::Raw::from("bar"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("bar"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(1) =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("bar"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    wide_char { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Raw::from("😊"),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 7),
            term::control_codes().clear_attrs
}

frag! {
    erase_screen_no_scrollback { scrollback_lines: 10, width: 5, height: 2 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    erase_scrollback { scrollback_lines: 10, width: 5, height: 2 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::control_codes().erase_scrollback
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
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
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
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

frag! {
    dsr_ignored { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::control_codes().device_status_report,
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("AB"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    erase_display_to_end_with_decom { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_to_end
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Raw::from("22222"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}

frag! {
    erase_display_from_start_with_decom { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_from_start
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("   33"),
            term::Crlf::default(),
            term::Raw::from("44444"),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}

frag! {
    erase_screen_with_decom { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}
