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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
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
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("  B"),
            term::ControlCodes::cursor_position(3, 4),
            term::control_codes().clear_attrs
}

frag! {
    horizontal_and_vertical_position { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::horizontal_and_vertical_position(3, 3),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
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
            reset_codes,
            term::Raw::from("AC"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

// Like in xterm, CSI s and CSI u save and restore everything that DECSC and
// DECRC do, not just the position.
frag! {
    scp_rcp_restore_attrs_and_charsets { scrollback_lines: 100, width: 10, height: 3 }
    <= term::control_codes().bold,
       term::ControlCodes::designate_charset(0, b'0'),
       term::control_codes().save_cursor_position,
       term::control_codes().clear_attrs,
       term::control_codes().designate_g0_us_ascii,
       term::control_codes().restore_cursor_position,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::control_codes().bold,
            term::Raw::from("─"),
            term::control_codes().reset_font_weight,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().bold,
            term::ControlCodes::designate_charset(0, b'0'),
            term::control_codes().save_cursor,
            term::control_codes().reset_font_weight,
            term::control_codes().designate_g0_us_ascii,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::control_codes().bold,
            term::ControlCodes::designate_charset(0, b'0')
}

frag! {
    cursor_horizontal_absolute { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_horizontal_absolute(3),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Raw::from(" "),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    horizontal_position_absolute { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::horizontal_position_absolute(3),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Raw::from(" "),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    vertical_position_absolute { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::vertical_position_absolute(3),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" B"),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

frag! {
    vertical_position_absolute_default_one { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::ControlCode::CSI {
           params: smallvec![],
           intermediates: smallvec![],
           action: 'd',
       },
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AX"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    vertical_position_absolute_origin_mode { scrollback_lines: 100, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::vertical_position_absolute(2),
       term::Raw::from("X"),
       term::control_codes().disable_scroll_region_origin_mode
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
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
            reset_codes,
            term::Raw::from("X23"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// SU pushes the top row of the screen into the scrollback and opens a blank
// row at the bottom, so the whole buffer keeps every line while the screen
// shows one fewer.
frag! {
    scroll_up { scrollback_lines: 100, width: 10, height: 2 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::Crlf::default(),
       term::Raw::from("C"),
       term::ControlCodes::scroll_up(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(50) =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            reset_codes,
            term::Raw::from("C"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// SD goes the other way, and it does not pull anything back out of the
// scrollback: it opens a blank row at the top of the screen and drops the row
// that falls off the bottom.
frag! {
    scroll_down { scrollback_lines: 100, width: 10, height: 2 }
    <= term::Raw::from("A\n\rB\n\rC\n\rD"),
       term::ControlCodes::scroll_down(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::BottomLines(50) =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
    => ContentRegion::Screen =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_with_content { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_unset { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().unset_scroll_region
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_clamp_top { scrollback_lines: 100, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::cursor_up(1)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(2, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_clamp_bottom { scrollback_lines: 100, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::cursor_down(1)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(2, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 1),
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
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_sets_bit { scrollback_lines: 4, width: 4, height: 4 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 2),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 3),
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
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    backspace { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Raw::from("\x08"),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    backspace_saturate { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\x08"),
       term::Raw::from("A")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_scroll_region_on_newline { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(4, 1),
       term::Raw::from("\n"),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("0"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(4, 2),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_newline_below_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(5, 1),
       term::Raw::from("\n"),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("0"),
            term::Crlf::default(),
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(5, 2),
            term::control_codes().clear_attrs
}

frag! {
    scrollback_newline_above_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("\n"),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("0"),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_basic { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AAAA"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_default_one { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::repeat_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AA"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_consecutive { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::repeat_character(2),
       term::ControlCodes::repeat_character(1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AAAA"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_newline { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\n"),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_carriage_return { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("ABC\r"),
       term::ControlCodes::repeat_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ABC"),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_backspace { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("AB\x08"),
       term::ControlCodes::repeat_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AB"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_tab { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\t"),
       term::ControlCodes::repeat_character(2)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 9),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_cursor_move { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::cursor_forward(2),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_csi_with_intermediates { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCode::CSI { params: smallvec![smallvec![3]], intermediates: smallvec![b'?'], action: 'b' },
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_esc { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::control_codes().save_cursor,
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    repeat_character_reset_on_osc { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::ControlCodes::set_title(smallvec![b't']),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b't'])
}

frag! {
    repeat_character_reset_on_dcs_hook { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\x1bP+q\x1b\\"),
       term::ControlCodes::repeat_character(3)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_basic { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A"),
       term::Crlf::default(),
       term::Raw::from("B"),
       term::control_codes().reverse_index,
       term::Raw::from("C")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AC"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_scroll_at_top { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"),
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().reverse_index,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// Unlike IL, RI doesn't move the cursor to the start of the line when it
// scrolls.
frag! {
    reverse_index_scroll_keeps_column { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("abc"),
       term::control_codes().reverse_index,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("   X"),
            term::Crlf::default(),
            term::Raw::from("abc"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(2, 1),
       term::control_codes().reverse_index,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("0"),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_above_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"),
       term::ControlCodes::set_scroll_region(3, 5),
       term::ControlCodes::cursor_position(2, 1),
       term::control_codes().reverse_index,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("2"),
            term::ControlCodes::set_scroll_region(3, 5),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_clamp_top_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("A"),
       term::ControlCodes::set_scroll_region(3, 5),
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().reverse_index,
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("B"),
            term::ControlCodes::set_scroll_region(3, 5),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    reverse_index_below_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("0"), term::Crlf::default(),
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"),
       term::ControlCodes::set_scroll_region(1, 3),
       term::ControlCodes::cursor_position(4, 1),
       term::control_codes().reverse_index,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("0"),
            term::Crlf::default(),
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::ControlCodes::set_scroll_region(1, 3),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_homes_cursor { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("abc"),
       term::ControlCodes::cursor_position(5, 5),
       term::ControlCodes::set_scroll_region(2, 4),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("Xbc"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_homes_cursor_in_origin_mode { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(5, 5),
       term::ControlCodes::set_scroll_region(3, 6),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(3, 6),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_cancels_pending_wrap { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("abcde"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("Xbcde"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_top_only { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("\x1b[3r")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(3, 10),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_whole_screen { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::set_scroll_region(1, 10)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// A region has to span at least two rows. Anything else leaves both the
// region and the cursor alone.
frag! {
    scroll_region_invalid_ignored { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::ControlCodes::cursor_position(4, 4),
       term::ControlCodes::set_scroll_region(5, 5),
       term::ControlCodes::set_scroll_region(6, 3),
       term::ControlCodes::set_scroll_region(11, 20)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(4, 4),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_homes_cursor { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(3, 6),
       term::ControlCodes::cursor_position(8, 5),
       term::control_codes().enable_scroll_region_origin_mode,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(3, 6),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    origin_mode_reset_homes_cursor { scrollback_lines: 100, width: 10, height: 10 }
    <= term::ControlCodes::set_scroll_region(3, 6),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 5),
       term::control_codes().disable_scroll_region_origin_mode,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(3, 6),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// VT and FF move down a row just like LF.
frag! {
    vertical_tab_and_form_feed { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\x0bB\x0cC")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from(" B"),
            term::Crlf::default(),
            term::Raw::from("  C"),
            term::ControlCodes::cursor_position(3, 4),
            term::control_codes().clear_attrs
}

// IND moves down a row without going back to the start of it.
frag! {
    index { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\x1bDB")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from(" B"),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

// IND scrolls at the bottom of the scroll region.
frag! {
    index_scrolls_region { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::ControlCodes::set_scroll_region(1, 2),
       term::ControlCodes::cursor_position(2, 1),
       term::Raw::from("\x1bD")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

// NEL goes to the start of the next row.
frag! {
    next_line { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("AB\x1bEC")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AB"),
            term::Crlf::default(),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// NEL scrolls at the bottom of the screen.
frag! {
    next_line_scrolls { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("1\r\n23\x1bE4")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("1"),
            term::Crlf::default(),
            term::Raw::from("23"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// HPR moves right like CUF.
frag! {
    horizontal_position_relative { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\x1b[2aB\x1b[aC\x1b[20aD")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A  B C   D"),
            term::ControlCodes::cursor_position(1, 10),
            term::Raw::from("D"),
            term::control_codes().clear_attrs
}

// VPR moves down like CUD.
frag! {
    vertical_position_relative { scrollback_lines: 100, width: 5, height: 5 }
    <= term::Raw::from("A\x1b[2eB\x1b[eC\x1b[20eD")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" B"),
            term::Crlf::default(),
            term::Raw::from("  C"),
            term::Crlf::default(),
            term::Raw::from("   D"),
            term::ControlCodes::cursor_position(5, 5),
            term::control_codes().clear_attrs
}

// CUU stops at the top of the scroll region when it starts out inside of it,
// even without origin mode.
frag! {
    cursor_up_stops_at_scroll_region_top { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::cursor_up(5),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// It stops there coming from below the scroll region too.
frag! {
    cursor_up_from_below_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::ControlCodes::cursor_position(5, 1),
       term::ControlCodes::cursor_up(5),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// Above the scroll region, CUU can go all the way to the top.
frag! {
    cursor_up_above_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(3, 5),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::cursor_up(5),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(3, 5),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// CUD stops at the bottom of the scroll region when it starts out above it.
frag! {
    cursor_down_stops_at_scroll_region_bottom { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::ControlCodes::cursor_down(5),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

// Below the scroll region, CUD can go all the way to the bottom.
frag! {
    cursor_down_below_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(1, 2),
       term::ControlCodes::cursor_position(4, 1),
       term::ControlCodes::cursor_down(5),
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(5, 2),
            term::control_codes().clear_attrs
}

// CNL and CPL stop at the scroll region the same way.
frag! {
    next_and_prev_line_stop_at_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 3),
       term::Raw::from("\x1b[5EA\x1b[5FB")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("B"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("A"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// DECSC saves origin mode along with the position, and DECRC turns it back on
// without homing the cursor.
frag! {
    restore_cursor_restores_origin_mode { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::control_codes().save_cursor,
       term::control_codes().disable_scroll_region_origin_mode,
       term::control_codes().restore_cursor,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("X"),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().save_cursor,
            term::control_codes().disable_scroll_region_origin_mode,
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// It turns it back off too, and the cursor goes back to where it was, even
// though that is outside of the scroll region.
frag! {
    restore_cursor_turns_origin_mode_off { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 3),
       term::ControlCodes::cursor_position(5, 2),
       term::control_codes().save_cursor,
       term::control_codes().enable_scroll_region_origin_mode,
       term::control_codes().restore_cursor,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::cursor_position(5, 2),
            term::control_codes().save_cursor,
            term::ControlCodes::set_scroll_region(2, 3),
            term::ControlCodes::cursor_position(5, 3),
            term::control_codes().clear_attrs
}

// A position saved in origin mode stays inside the scroll region, even if the
// region has shrunk since.
frag! {
    restore_cursor_stays_in_the_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(4, 1),
       term::control_codes().save_cursor,
       term::ControlCodes::set_scroll_region(2, 3),
       term::control_codes().restore_cursor,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("X"),
            // The saved cursor is outside of the scroll region now, so it
            // has to be restored before the region is.
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(5, 1),
            term::control_codes().save_cursor,
            term::control_codes().disable_scroll_region_origin_mode,
            term::ControlCodes::set_scroll_region(2, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}
