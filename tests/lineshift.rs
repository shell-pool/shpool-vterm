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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(4, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_insert_with_scroll_region_reassembly { scrollback_lines: 100, width: 5, height: 6 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"), term::Crlf::default(),
       term::Raw::from("66"),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_delete_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
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
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::ControlCodes::cursor_position(4, 1),
            term::control_codes().clear_attrs
}
