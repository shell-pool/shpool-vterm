#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    osc_title_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title(smallvec![b't', b'i', b't', b'l', b'e'])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b't', b'i', b't', b'l', b'e'])
}

frag! {
    osc_icon_only { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_icon_name(smallvec![b'i', b'c', b'o', b'n'])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_icon_name(smallvec![b'i', b'c', b'o', b'n'])
}

frag! {
    osc_title_and_icon_same { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b's', b'a', b'm', b'e'])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title_and_icon_name(smallvec![b's', b'a', b'm', b'e'])
}

frag! {
    osc_title_and_icon_diff { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_title_and_icon_name(smallvec![b'A']),
       term::ControlCodes::set_title(smallvec![b'B'])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_title(smallvec![b'B']),
            term::ControlCodes::set_icon_name(smallvec![b'A'])
}

frag! {
    osc_working_dir { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(smallvec![b'h', b'o', b's', b't'], smallvec![b'/', b't', b'm', b'p'])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_working_dir(smallvec![b'h', b'o', b's', b't'], smallvec![b'/', b't', b'm', b'p'])
}

frag! {
    osc_set_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd'])))
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd'])))
}

frag! {
    osc_reset_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd']))),
       term::ControlCodes::reset_color_indices(std::iter::once(1))
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_set_multiple_colors { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(vec![
           (1, smallvec![b'r', b'e', b'd']),
           (2, smallvec![b'g', b'r', b'e', b'e', b'n']),
       ])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(vec![
                (1, smallvec![b'r', b'e', b'd']),
                (2, smallvec![b'g', b'r', b'e', b'e', b'n']),
            ])
}

frag! {
    osc_reset_multiple_colors { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(vec![
           (1, smallvec![b'r', b'e', b'd']),
           (2, smallvec![b'g', b'r', b'e', b'e', b'n']),
           (3, smallvec![b'b', b'l', b'u', b'e']),
       ]),
       term::ControlCodes::reset_color_indices(vec![1, 3])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(std::iter::once((2, smallvec![b'g', b'r', b'e', b'e', b'n'])))
}

frag! {
    osc_functional_colors { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"red".as_slice(), b"blue".as_slice()])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"red".as_slice(), b"blue".as_slice()])
}

frag! {
    osc_functional_colors_gaps { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"red".as_slice()]),
       term::ControlCodes::set_functional_color(2, vec![b"blue".as_slice()])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"red".as_slice()]),
            term::ControlCodes::set_functional_color(2, vec![b"blue".as_slice()])
}

frag! {
    osc_functional_colors_empty_param { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"".as_slice()])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"".as_slice()])
}

frag! {
    osc_functional_colors_overflow { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(9, vec![b"red".as_slice(), b"blue".as_slice()])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(9, vec![b"red".as_slice()])
}

frag! {
    osc_functional_colors_query { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"?".as_slice()])
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}
