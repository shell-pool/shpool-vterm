#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

frag! {
    default { scrollback_lines: 100, width: 20, height: 10 }
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
    clamp { scrollback_lines: 100, width: 10, height: 10 }
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
fn resize() {
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
    set_clear { scrollback_lines: 100, width: 20, height: 10 }
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
    ctc_set_clear { scrollback_lines: 100, width: 20, height: 10 }
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
    clear_all_tbc { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::tab_clear(Some(3)),
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::tab_clear(Some(3)),
            term::Raw::from("                   "), // 19 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 21),
            term::control_codes().clear_attrs
}

frag! {
    clear_all_ctc { scrollback_lines: 100, width: 20, height: 10 }
    <= term::ControlCodes::cursor_tab_control(Some(5)),
       term::Raw::from("\tA")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::tab_clear(Some(3)),
            term::Raw::from("                   "), // 19 spaces
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 21),
            term::control_codes().clear_attrs
}

frag! {
    reset_decstr { scrollback_lines: 100, width: 20, height: 10 }
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
    reset_ris { scrollback_lines: 100, width: 20, height: 10 }
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
