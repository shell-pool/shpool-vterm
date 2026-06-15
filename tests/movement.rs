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

frag! {
    tab_stop_default { scrollback_lines: 100, width: 20, height: 10 }
    <= term::Raw::from("A\tB")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from("       "), // 7 spaces
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_clamp { scrollback_lines: 100, width: 10, height: 10 }
    <= term::Raw::from("A\tB\tC")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Raw::from("       "), // 7 spaces
            term::Raw::from("B"),
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(1, 11),
            term::control_codes().clear_attrs
}

#[test]
fn tab_stop_resize() {
    use shpool_vterm::term::AsTermInput;

    let mut term = shpool_vterm::Term::new(100, shpool_vterm::Size { width: 10, height: 10 });
    term.process(b"A\tB");

    let mut expected = vec![];
    term::control_codes().clear_attrs.term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected);
    term::control_codes().clear_screen.term_input_into(&mut expected);
    term::Raw::from("A").term_input_into(&mut expected);
    term::Raw::from("       ").term_input_into(&mut expected);
    term::Raw::from("B").term_input_into(&mut expected);
    term::ControlCodes::cursor_position(1, 10).term_input_into(&mut expected);
    term::control_codes().clear_attrs.term_input_into(&mut expected);

    assert_eq!(term.contents(ContentRegion::All), expected);

    term.resize(shpool_vterm::Size { width: 20, height: 10 });

    term.process(b"C\tD");

    let mut expected2 = vec![];
    term::control_codes().clear_attrs.term_input_into(&mut expected2);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut expected2);
    term::control_codes().clear_screen.term_input_into(&mut expected2);
    term::Raw::from("A").term_input_into(&mut expected2);
    term::Raw::from("       ").term_input_into(&mut expected2);
    term::Raw::from("B").term_input_into(&mut expected2);
    term::Raw::from("C").term_input_into(&mut expected2);
    term::Raw::from("      ").term_input_into(&mut expected2);
    term::Raw::from("D").term_input_into(&mut expected2);
    term::ControlCodes::cursor_position(1, 18).term_input_into(&mut expected2);
    term::control_codes().clear_attrs.term_input_into(&mut expected2);

    assert_eq!(term.contents(ContentRegion::All), expected2);
}

frag! {
    tab_stop_set_clear { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::cursor_horizontal_absolute(6),
       term::control_codes().horizontal_tab_set,
       term::Raw::from("\r\tB"),
       term::ControlCodes::cursor_horizontal_absolute(6),
       term::ControlCodes::tab_clear(None),
       term::Raw::from("\r\tC")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("     "), // 5 spaces
            term::Raw::from("B"),
            term::Raw::from("  "), // 2 spaces
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_ctc_set_clear { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::cursor_horizontal_absolute(6),
       term::ControlCodes::cursor_tab_control(Some(0)),
       term::Raw::from("\r\tB"),
       term::ControlCodes::cursor_horizontal_absolute(6),
       term::ControlCodes::cursor_tab_control(Some(2)),
       term::Raw::from("\r\tC")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("     "), // 5 spaces
            term::Raw::from("B"),
            term::Raw::from("  "), // 2 spaces
            term::Raw::from("C"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_clear_all_tbc { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::tab_clear(Some(3)),
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("                   "), // 19 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 21),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_clear_all_ctc { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::cursor_tab_control(Some(5)),
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("                   "), // 19 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 21),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_reset_decstr { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::tab_clear(Some(3)),
       term::control_codes().soft_reset,
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("        "), // 8 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}

frag! {
    tab_stop_reset_ris { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::tab_clear(Some(3)),
       term::control_codes().hard_reset,
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("        "), // 8 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}
