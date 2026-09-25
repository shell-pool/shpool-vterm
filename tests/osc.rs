#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    osc_working_dir { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(b"file://host/tmp"[..].into())
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_working_dir(b"file://host/tmp"[..].into())
}

frag! {
    osc_working_dir_bel_terminated { scrollback_lines: 10, width: 10, height: 10 }
    <= term::Raw::from("\x1b]7;file://host/tmp\x07")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_working_dir(b"file://host/tmp"[..].into())
}

frag! {
    osc_working_dir_with_semicolon { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(b"file://host/a;b;c"[..].into())
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_working_dir(b"file://host/a;b;c"[..].into())
}

frag! {
    osc_working_dir_update { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(b"file://host/a"[..].into()),
       term::ControlCodes::set_working_dir(b"file://host/b"[..].into())
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_working_dir(b"file://host/b"[..].into())
}

frag! {
    osc_working_dir_empty_unsets { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(b"file://host/tmp"[..].into()),
       term::ControlCodes::set_working_dir(smallvec![])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_working_dir_over_long_ignored { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_working_dir(b"file://host/tmp"[..].into()),
       term::ControlCodes::set_working_dir(format!("file://host/{}", "a".repeat(10000)).as_bytes().into())
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_set_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd'])))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd'])))
}

frag! {
    osc_reset_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(std::iter::once((1, smallvec![b'r', b'e', b'd']))),
       term::ControlCodes::reset_color_indices(std::iter::once(1))
    => ContentRegion::All =>
            reset_codes,
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
            reset_codes,
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
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(std::iter::once((2, smallvec![b'g', b'r', b'e', b'e', b'n'])))
}

frag! {
    osc_functional_colors { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"red".as_slice(), b"blue".as_slice()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"red".as_slice(), b"blue".as_slice()])
}

frag! {
    osc_functional_colors_gaps { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"red".as_slice()]),
       term::ControlCodes::set_functional_color(2, vec![b"blue".as_slice()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"red".as_slice()]),
            term::ControlCodes::set_functional_color(2, vec![b"blue".as_slice()])
}

frag! {
    osc_functional_colors_empty_param { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"".as_slice()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(0, vec![b"".as_slice()])
}

frag! {
    osc_functional_colors_overflow { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(9, vec![b"red".as_slice(), b"blue".as_slice()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(9, vec![b"red".as_slice()])
}

frag! {
    osc_functional_colors_query { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"?".as_slice()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_set_color_out_of_range { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(vec![
           (260, smallvec![b'r', b'e', b'd']),
           (261, smallvec![b'b', b'l', b'u', b'e']),
           (100000, smallvec![b'b', b'l', b'u', b'e']),
       ])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_color_indices(std::iter::once((260, smallvec![b'r', b'e', b'd'])))
}

frag! {
    osc_set_color_over_long_spec { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(std::iter::once((1, "x".repeat(1000).as_bytes().into())))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_reset_all_colors { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(vec![
           (1, smallvec![b'r', b'e', b'd']),
           (2, smallvec![b'g', b'r', b'e', b'e', b'n']),
       ]),
       term::ControlCodes::reset_color_indices(std::iter::empty())
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_reset_all_colors_empty_param { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_color_indices(vec![
           (1, smallvec![b'r', b'e', b'd']),
           (2, smallvec![b'g', b'r', b'e', b'e', b'n']),
       ]),
       term::Raw::from("\x1b]104;\x07")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    osc_reset_functional_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec![b"red".as_slice(), b"blue".as_slice()]),
       term::Raw::from("\x1b]110\x1b\\")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(1, vec![b"blue".as_slice()])
}

frag! {
    osc_reset_last_functional_color { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(8, vec![b"red".as_slice(), b"blue".as_slice()]),
       term::Raw::from("\x1b]119\x07")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs,
            term::ControlCodes::set_functional_color(8, vec![b"red".as_slice()])
}

frag! {
    osc_functional_color_over_long_spec { scrollback_lines: 10, width: 10, height: 10 }
    <= term::ControlCodes::set_functional_color(0, vec!["x".repeat(1000).as_bytes()])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}
