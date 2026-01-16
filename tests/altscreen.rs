#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};
use smallvec::smallvec;

frag! {
    alt_screen_basic { scrollback_lines: 100, width: 2, height: 2 }
    <= term::Raw::from("A"),
       term::control_codes().enable_alt_screen,
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("B"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_isolation { scrollback_lines: 100, width: 2, height: 2 }
    <= term::Raw::from("A"),
       term::control_codes().enable_alt_screen,
       term::Raw::from("B"),
       term::control_codes().disable_alt_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_clears { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("JUNK"),
       term::control_codes().disable_alt_screen,
       term::control_codes().enable_alt_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    fused_alt_screen_enable { scrollback_lines: 100, width: 2, height: 2 }
    <= term::Raw::from("A"),
       term::ControlCode::CSI {
           params: smallvec![smallvec![1049], smallvec![1049]],
           intermediates: smallvec![b'?'],
           action: 'h',
       },
       term::Raw::from("B")
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("B"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}
