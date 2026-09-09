#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

frag! {
    scrollback_insert_line_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_lines_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_many_lines { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(10)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_medium_lines { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_past_text { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"),
       term::ControlCodes::cursor_position(4, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(4, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_with_scroll_region_reassembly { scrollback_lines: 100, width: 5, height: 6 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"), term::Crlf::default(),
       term::Raw::from("66"),
       term::ControlCodes::set_scroll_region(2, 5),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Raw::from("66"),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_line_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_lines_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_many_lines { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::delete_lines(10)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_past_text { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"),
       term::ControlCodes::cursor_position(4, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(4, 1),
            term::control_codes().clear_attrs
}

frag! {
    insert_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1 23"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    insert_char_shift_off { scrollback_lines: 100, width: 3, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from(" 12"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    insert_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("  123"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    delete_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("13   "),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    delete_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::delete_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("3    "),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    delete_char_at_end_of_line { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("12   "),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    delete_char_with_backfill_attrs { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::fgcolor_idx(1),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("13  "),
            term::ControlCodes::fgcolor_idx(1),
            term::Raw::from(" "),
            term::control_codes().fgcolor_default,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::fgcolor_idx(1)
}

frag! {
    erase_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1 3"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    erase_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("12345"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::erase_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1   5"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    erase_char_with_attrs { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::fgcolor_idx(1),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1"),
            term::ControlCodes::fgcolor_idx(1),
            term::Raw::from(" "),
            term::control_codes().fgcolor_default,
            term::Raw::from("3"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::fgcolor_idx(1)
}

frag! {
    erase_char_at_right_margin { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("12345"),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("12345"),
            term::ControlCodes::cursor_position(1, 6),
            term::control_codes().clear_attrs
}

frag! {
    insert_mode_shifts_text { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().enable_insert_mode,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1X23"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}

frag! {
    insert_mode_shifts_off_right_margin { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("12345"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().enable_insert_mode,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1X234"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}

frag! {
    insert_mode_multi_char { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().enable_insert_mode,
       term::Raw::from("XY")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1XY23"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}

frag! {
    insert_mode_switch_to_replace { scrollback_lines: 100, width: 5, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().enable_insert_mode,
       term::Raw::from("X"),
       term::control_codes().disable_insert_mode,
       term::Raw::from("Y")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1XY3"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    insert_mode_repeat_char { scrollback_lines: 100, width: 6, height: 4 }
    <= term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::control_codes().enable_insert_mode,
       term::Raw::from("X"),
       term::ControlCodes::repeat_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1XXX23"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs,
            term::control_codes().enable_insert_mode
}
