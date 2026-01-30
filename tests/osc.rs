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
