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

frag! {
    alt_screen_scroll_region { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_region_with_content { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("A"),
       term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_region_unset { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().unset_scroll_region
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    scroll_region_persistence { scrollback_lines: 100, width: 10, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(3, 6),
       term::control_codes().disable_alt_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_display_to_end { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_to_end
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_display_from_start { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_from_start
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Raw::from("   22"),
            term::Crlf::default(),
            term::Raw::from("33333"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_screen { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_display_to_end_with_decom { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_to_end
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Raw::from("22222"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_display_from_start_with_decom { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_from_start
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("   33"),
            term::Crlf::default(),
            term::Raw::from("44444"),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_screen_with_decom { scrollback_lines: 100, width: 10, height: 10 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11111"), term::Crlf::default(),
       term::Raw::from("22222"), term::Crlf::default(),
       term::Raw::from("33333"), term::Crlf::default(),
       term::Raw::from("44444"), term::Crlf::default(),
       term::Raw::from("55555"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().erase_screen
    => ContentRegion::All =>
            term::control_codes().clear_attrs,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55555"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().enable_scroll_region_origin_mode,
            term::control_codes().clear_attrs
}