#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    title_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b't', b'i', b't', b'l', b'e'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b't', b'i', b't', b'l', b'e'])
}

frag! {
    icon_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_icon_name(smallvec![b'i', b'c', b'o', b'n'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_icon_name(smallvec![b'i', b'c', b'o', b'n'])
}

frag! {
    title_and_icon_same { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b's', b'a', b'm', b'e'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title_and_icon_name(smallvec![b's', b'a', b'm', b'e'])
}

frag! {
    title_and_icon_diff { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b'A']),
       term::ControlCodes::set_title(smallvec![b'B'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'B']),
            term::ControlCodes::set_icon_name(smallvec![b'A'])
}

frag! {
    title_push_pop_basic { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::save_title(Some(2)),
       term::ControlCodes::set_title(smallvec![b'B']),
       term::ControlCodes::restore_title(Some(2))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'A'])
}

frag! {
    icon_push_pop_basic { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_icon_name(smallvec![b'A']),
       term::ControlCodes::save_title(Some(1)),
       term::ControlCodes::set_icon_name(smallvec![b'B']),
       term::ControlCodes::restore_title(Some(1))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_icon_name(smallvec![b'A'])
}

frag! {
    title_and_icon_push_pop { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b'A']),
       term::ControlCodes::save_title(Some(0)),
       term::ControlCodes::set_title_and_icon_name(smallvec![b'B']),
       term::ControlCodes::restore_title(Some(0))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title_and_icon_name(smallvec![b'A'])
}

frag! {
    title_and_icon_push_pop_default_param { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b'A']),
       term::ControlCodes::save_title(None),
       term::ControlCodes::set_title_and_icon_name(smallvec![b'B']),
       term::ControlCodes::restore_title(None)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title_and_icon_name(smallvec![b'A'])
}

frag! {
    title_push_pop_multiple_levels { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::save_title(Some(2)),
       term::ControlCodes::set_title(smallvec![b'B']),
       term::ControlCodes::save_title(Some(2)),
       term::ControlCodes::set_title(smallvec![b'C']),
       term::ControlCodes::restore_title(Some(2))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'B'])
}

frag! {
    title_push_pop_stack_isolation { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::set_icon_name(smallvec![b'1']),
       term::ControlCodes::save_title(Some(2)),
       term::ControlCodes::set_title(smallvec![b'B']),
       term::ControlCodes::set_icon_name(smallvec![b'2']),
       term::ControlCodes::restore_title(Some(2))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'A']),
            term::ControlCodes::set_icon_name(smallvec![b'2'])
}

frag! {
    icon_push_pop_stack_isolation { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::set_icon_name(smallvec![b'1']),
       term::ControlCodes::save_title(Some(1)),
       term::ControlCodes::set_title(smallvec![b'B']),
       term::ControlCodes::set_icon_name(smallvec![b'2']),
       term::ControlCodes::restore_title(Some(1))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'B']),
            term::ControlCodes::set_icon_name(smallvec![b'1'])
}

frag! {
    title_pop_empty_stack { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::restore_title(None),
       term::ControlCodes::set_title(smallvec![b'A'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'A'])
}

frag! {
    title_push_empty_stack { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::save_title(None),
       term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::restore_title(None)
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    dimension_queries_ignored { scrollback_lines: 10, width: 10, height: 10 }
    <= term::Raw::from("\x1b[14t\x1b[16t\x1b[18t\x1b[19t"),
       term::ControlCodes::set_title(smallvec![b'A'])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'A'])
}

#[test]
fn title_stack_depth_limit() {
    use shpool_vterm::term::AsTermInput;

    let mut term = shpool_vterm::Term::new(10, shpool_vterm::Size { width: 10, height: 10 });

    // Set initial title A
    term.process(b"\x1b]2;A\x1b\\");

    // Push 20 times (exceeding MAX_TITLE_STACK_DEPTH = 16)
    for _ in 0..20 {
        term.process(b"\x1b[22;2t");
    }

    // Set title B
    term.process(b"\x1b]2;B\x1b\\");

    // Pop once -> should restore A
    term.process(b"\x1b[23;2t");

    let mut want = vec![];
    support::frag::reset_codes.term_input_into(&mut want);
    term::ControlCodes::cursor_position(1, 1).term_input_into(&mut want);
    term::control_codes().clear_attrs.term_input_into(&mut want);
    term::ControlCodes::set_title(smallvec![b'A']).term_input_into(&mut want);

    assert_eq!(term.contents(ContentRegion::All), want);
}

frag! {
    title_push_empty_stack_title_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::save_title(Some(2)),
       term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::set_icon_name(smallvec![b'B']),
       term::ControlCodes::restore_title(Some(2))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_icon_name(smallvec![b'B'])
}

frag! {
    icon_push_empty_stack_icon_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::save_title(Some(1)),
       term::ControlCodes::set_title(smallvec![b'A']),
       term::ControlCodes::set_icon_name(smallvec![b'B']),
       term::ControlCodes::restore_title(Some(1))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'A'])
}
