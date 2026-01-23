#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    cursor_left { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_backwards(1),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_left_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABC"),
       term::ControlCodes::cursor_backwards(2),
       term::Raw::from("X")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from("X"),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    cursor_right_gap { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().inverse,
       term::Raw::from("A"),
       term::ControlCodes::cursor_backwards(1),
       term::Raw::from("B"),
       term::ControlCodes::cursor_forward(1),
       term::Raw::from("C"),
       term::control_codes().undo_inverse
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::control_codes().inverse,
            term::Raw::from("B"),
            term::control_codes().undo_inverse,
            term::Raw::from(" "),
            term::control_codes().inverse,
            term::Raw::from("C"),
            term::control_codes().undo_inverse,
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    cursor_right_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_forward(2),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from(" "),
            term::Raw::from(" "),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    cursor_down { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_down(1),
       term::ControlCodes::cursor_backwards(1),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_down_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_down(2),
       term::ControlCodes::cursor_backwards(1),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_up { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::ControlCodes::cursor_up(1),
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    cursor_up_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::ControlCodes::cursor_up(2),
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    cursor_next_line { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_next_line(1),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_next_line_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_next_line(2),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_prev_line { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::ControlCodes::cursor_prev_line(1),
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_prev_line_multi { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::ControlCodes::cursor_prev_line(2),
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    cursor_position { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_position(3, 3),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("  B"),
            term::ControlCodes::cursor_position(3, 4),
            term::control_codes().clear_attrs
}

frag! {
    scp_rcp { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::control_codes().save_cursor_position,
       term::Crlf::default(),
       term::Raw::from("B"),
       term::control_codes().restore_cursor_position,
       term::Raw::from("C")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("AC"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    cursor_horizontal_absolute { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_horizontal_absolute(3),
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from(" "),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    cursor_position_no_params { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("123"),
       // cursor position with no params (should be the same as (1,1)).
       term::ControlCode::CSI {
           params: smallvec![],
           intermediates: smallvec![],
           action: 'H',
       },
       term::Raw::from("X")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("X23"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_up { scrollback_lines: 100, width: 10, height: 2 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::Crlf::default(),
       term::Raw::from("C"),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::scroll_up(1),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(50) =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::scroll_up(1),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_down { scrollback_lines: 100, width: 10, height: 2 }
    <= term::Raw::from("A\n\rB\n\rC\n\rD"),
       term::ControlCodes::scroll_up(2),
       term::ControlCodes::scroll_down(1)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Raw::from("D"),
            term::ControlCodes::scroll_up(1),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(50) =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::Raw::from("D"),
            term::ControlCodes::scroll_up(1),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_with_content { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_unset { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().unset_scroll_region
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_clamp_top { scrollback_lines: 100, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::cursor_up(1),
       term::control_codes().disable_scroll_region_origin_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_clamp_bottom { scrollback_lines: 100, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::cursor_down(1),
       term::control_codes().disable_scroll_region_origin_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_cup_translation { scrollback_lines: 4, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("X"),
       term::control_codes().disable_scroll_region_origin_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_sets_bit { scrollback_lines: 4, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("X")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_cup_clamp { scrollback_lines: 4, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(1000, 2),
       term::Raw::from("X"),
       term::control_codes().disable_scroll_region_origin_mode
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}
