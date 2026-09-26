#[macro_use]
#[path = "support/mod.rs"]
mod support;

use shpool_vterm::{term, ContentRegion};

// Private markers and intermediates turn a final byte into an entirely
// different command, so none of these should be mistaken for the plain
// command that shares their final byte.

frag! {
    xtmodkeys_is_not_sgr { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("A"),
       // XTMODKEYS, not underline + faint
       term::ControlCodes::xtmodkeys(4, 2),
       term::Raw::from("B")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("AB"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    xtsmgraphics_is_not_scroll_up { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("A\r\nB"),
       // XTSMGRAPHICS query, not SU
       term::ControlCodes::xtsmgraphics(1, 1, 0)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    mouse_highlight_tracking_is_not_scroll_down { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("A\r\nB"),
       term::ControlCodes::highlight_mouse_tracking(1, 1, 1, 1, 1)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Raw::from("B"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

frag! {
    kitty_keyboard_is_not_restore_cursor { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().save_cursor_position,
       term::Raw::from("AB"),
       // push, pop, set and query the kitty keyboard flags
       term::ControlCodes::kitty_keyboard_push(1),
       term::ControlCodes::kitty_keyboard_pop(None),
       term::ControlCodes::kitty_keyboard_set(1, 1),
       term::control_codes().kitty_keyboard_query,
       // DECSMBV (set margin bell volume)
       term::ControlCodes::decsmbv(8),
       term::Raw::from("C")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("ABC"),
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    xtsave_is_not_save_cursor { scrollback_lines: 100, width: 10, height: 5 }
    <= term::ControlCodes::cursor_position(2, 2),
       term::control_codes().save_cursor_position,
       term::ControlCodes::cursor_position(3, 3),
       // XTSAVE, not SCP
       term::ControlCodes::xtsave(&[25]),
       term::ControlCodes::cursor_position(1, 1),
       term::control_codes().restore_cursor_position,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from(" X"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().save_cursor,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    deccara_is_not_set_scroll_region { scrollback_lines: 100, width: 10, height: 5 }
    <= term::ControlCodes::deccara(1, 1, 2, 2, &[1])
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    decst8c_resets_tab_stops { scrollback_lines: 100, width: 20, height: 5 }
    <= term::ControlCodes::tab_clear(Some(3)),
       term::control_codes().decst8c,
       term::Raw::from("\tA")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("        A"),
            term::ControlCodes::cursor_position(1, 10),
            term::control_codes().clear_attrs
}

frag! {
    decsed_erases_like_ed { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("ABC"),
       term::ControlCodes::decsed(Some(2))
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::cursor_position(1, 4),
            term::control_codes().clear_attrs
}

frag! {
    decsel_erases_like_el { scrollback_lines: 100, width: 10, height: 5 }
    <= term::Raw::from("ABC"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::decsel(None)
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}
