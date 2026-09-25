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
            reset_codes,
            // The scrollback is restored first, so that the shell history
            // is still there when the user exits the alt screen.
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            // Switching screens doesn't move the cursor, so the B went in
            // right next to where the A is on the main screen.
            term::Raw::from(" B"),
            term::Crlf::default(),
            // It filled the last column, which leaves a wrap pending.
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("B"),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_isolation { scrollback_lines: 100, width: 2, height: 2 }
    <= term::Raw::from("A"),
       term::control_codes().enable_alt_screen,
       term::Raw::from("B"),
       term::control_codes().disable_alt_screen
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_mode_restored { scrollback_lines: 100, width: 2, height: 2 }
    <= term::Raw::from("A"),
       term::control_codes().enable_alt_screen
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::Crlf::default(),
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
            reset_codes,
            term::Raw::from("A"),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            // Switching screens doesn't move the cursor, so the B went in
            // right next to where the A is on the main screen.
            term::Raw::from(" B"),
            term::Crlf::default(),
            // It filled the last column, which leaves a wrap pending.
            term::ControlCodes::cursor_position(1, 2),
            term::Raw::from("B"),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_region { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(2, 5)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("A"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 5),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_region_unset { scrollback_lines: 100, width: 10, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().unset_scroll_region
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// A real terminal only has the one scroll region, so the one the app set up
// on the alt screen is still in place once it is back on the main screen.
frag! {
    scroll_region_shared_between_screens { scrollback_lines: 100, width: 10, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 5),
       term::control_codes().enable_alt_screen,
       term::ControlCodes::set_scroll_region(3, 6),
       term::control_codes().disable_alt_screen
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(3, 5),
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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

// ED works on the whole screen. Neither the scroll region nor origin mode
// limit what it erases.
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11111"),
            term::Crlf::default(),
            term::Raw::from("22222"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 3),
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 3),
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
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_line_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_line_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_line_many { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(10)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_line_medium { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::insert_lines(2)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"), term::Crlf::default(),
            term::Raw::from("22"), term::Crlf::default(),
            term::Raw::from("33"), term::Crlf::default(),
            term::Raw::from("44"), term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_line_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"), term::Crlf::default(),
            term::Raw::from("33"), term::Crlf::default(),
            term::Raw::from("44"), term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_lines_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(3, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"), term::Crlf::default(),
            term::Raw::from("22"), term::Crlf::default(),
            term::Raw::from("44"), term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(3, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_many_lines { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"),
       term::ControlCodes::cursor_position(2, 1),
       term::ControlCodes::delete_lines(10)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_outside_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("11"), term::Crlf::default(),
       term::Raw::from("22"), term::Crlf::default(),
       term::Raw::from("33"), term::Crlf::default(),
       term::Raw::from("44"), term::Crlf::default(),
       term::Raw::from("55"),
       term::ControlCodes::set_scroll_region(2, 4),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::delete_lines(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("11"),
            term::Crlf::default(),
            term::Raw::from("22"),
            term::Crlf::default(),
            term::Raw::from("33"),
            term::Crlf::default(),
            term::Raw::from("44"),
            term::Crlf::default(),
            term::Raw::from("55"),
            term::ControlCodes::set_scroll_region(2, 4),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("1 23"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_char_shift_off { scrollback_lines: 100, width: 3, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from(" 12"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_insert_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::insert_character(2)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("  123"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("13   "),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::delete_character(2)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("3    "),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_char_at_end_of_line { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 3),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("12   "),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_delete_char_with_backfill_attrs { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::delete_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("13  "),
            term::ControlCodes::bgcolor_idx(4),
            term::Raw::from(" "),
            term::control_codes().bgcolor_default,
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_erase_char_basic { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("1 3"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_many_chars { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("12345"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::erase_character(3)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("1   5"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_erase_char_with_attrs { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("123"),
       term::ControlCodes::cursor_position(1, 2),
       term::ControlCodes::bgcolor_idx(4),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("1"),
            term::ControlCodes::bgcolor_idx(4),
            term::Raw::from(" "),
            term::control_codes().bgcolor_default,
            term::Raw::from("3"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs,
            term::ControlCodes::bgcolor_idx(4)
}

frag! {
    alt_screen_erase_char_at_end_of_line { scrollback_lines: 100, width: 5, height: 4 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("12345"),
       term::ControlCodes::cursor_position(1, 5),
       term::ControlCodes::erase_character(1)
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("1234 "),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_on_newline { scrollback_lines: 100, width: 5, height: 3 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("1"), term::Crlf::default(),
       term::Raw::from("2"), term::Crlf::default(),
       term::Raw::from("3"), term::Crlf::default(),
       term::Raw::from("4")
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::Crlf::default(),
            term::Raw::from("4"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

frag! {
    alt_screen_scroll_region_on_newline { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("0"), term::Crlf::default(),
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
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
    alt_screen_newline_below_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("0"), term::Crlf::default(),
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
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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
    alt_screen_newline_above_scroll_region { scrollback_lines: 100, width: 5, height: 5 }
    <= term::control_codes().enable_alt_screen,
       term::Raw::from("0"), term::Crlf::default(),
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
            empty_scrollback,
            term::control_codes().enable_alt_screen,
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

// Ensure that the alt screen wins when margins conflict. The alt screen takes
// the margins over from the main screen, but here the app drops them again.
frag! {
    alt_screen_resets_scrollback_scroll_region_and_origin_mode
        { scrollback_lines: 100, width: 5, height: 5 }
    <= term::ControlCodes::set_scroll_region(2, 4),
       term::control_codes().enable_scroll_region_origin_mode,
       term::control_codes().enable_alt_screen,
       term::control_codes().unset_scroll_region,
       term::control_codes().disable_scroll_region_origin_mode,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            // The scrollback restore leaves the terminal with margins and
            // origin mode set.
            term::ControlCodes::set_scroll_region(2, 4),
            term::control_codes().enable_scroll_region_origin_mode,
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().enable_alt_screen,
            // The alt screen has neither, so it must say so explicitly.
            term::control_codes().unset_scroll_region,
            term::control_codes().disable_scroll_region_origin_mode,
            term::Raw::from("X"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 2),
            term::control_codes().clear_attrs
}

// ED 3 drops the scrollback of the main screen even while the alt screen is
// up, and leaves both screens alone.
frag! {
    alt_screen_erase_scrollback { scrollback_lines: 10, width: 5, height: 2 }
    <= term::Raw::from("1\r\n2\r\n3"),
       term::control_codes().enable_alt_screen,
       term::Raw::from("alt"),
       term::control_codes().erase_scrollback
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("2"),
            term::Crlf::default(),
            term::Raw::from("3"),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::Crlf::default(),
            term::Raw::from(" alt"),
            term::ControlCodes::cursor_position(2, 5),
            term::control_codes().clear_attrs
}

// Switching to the alt screen leaves the cursor where the restore of the main
// screen put it, so the restore has to home it before painting the alt screen
// or everything would land in the wrong spot.
frag! {
    alt_screen_painted_from_the_top { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("$ vi"),
       term::Crlf::default(),
       term::control_codes().enable_alt_screen,
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("text"),
       term::ControlCodes::cursor_position(3, 1),
       term::Raw::from("~")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("$ vi"),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("text"),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from("~"),
            term::ControlCodes::cursor_position(3, 2),
            term::control_codes().clear_attrs
}

// Resetting the scroll region already homes the cursor.
frag! {
    alt_screen_painted_from_the_top_after_region_reset { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("$ vi"),
       term::ControlCodes::set_scroll_region(1, 2),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().enable_alt_screen,
       term::ControlCodes::cursor_position(1, 1),
       term::Raw::from("text")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("$ vi"),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(2, 3),
            term::control_codes().enable_alt_screen,
            term::control_codes().unset_scroll_region,
            term::Raw::from("text"),
            term::Crlf::default(),
            term::Crlf::default(),
            // The alt screen took the scroll region over from the main
            // screen, so it gets set up again once the paint is done.
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

// The alt screen starts out with the scroll region of the main screen.
frag! {
    alt_screen_inherits_scroll_region { scrollback_lines: 100, width: 3, height: 3 }
    <= term::ControlCodes::set_scroll_region(1, 2),
       term::control_codes().enable_alt_screen,
       term::Raw::from("a\r\nb\r\nc")
    => ContentRegion::All =>
            reset_codes,
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().enable_alt_screen,
            term::control_codes().unset_scroll_region,
            term::Raw::from("b"),
            term::Crlf::default(),
            term::Raw::from("c"),
            term::Crlf::default(),
            term::ControlCodes::set_scroll_region(1, 2),
            term::ControlCodes::cursor_position(2, 2),
            term::control_codes().clear_attrs
}

// Leaving the alt screen restores the cursor the way DECRC does, which
// brings back the attrs and charsets it was saved with too.
frag! {
    alt_screen_exit_restores_cursor_state { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("A"),
       term::ControlCodes::fgcolor_idx(1),
       term::control_codes().enable_alt_screen,
       term::control_codes().clear_attrs,
       term::ControlCodes::designate_charset(0, b'0'),
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().disable_alt_screen,
       term::Raw::from("q")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("A\x1b[31mq\x1b[39m"),
            term::ControlCodes::cursor_position(1, 3),
            term::control_codes().clear_attrs,
            term::ControlCodes::fgcolor_idx(1)
}

// The alt screen keeps its own saved cursor while the main screen is up.
frag! {
    alt_screen_saved_cursor_survives_exit { scrollback_lines: 100, width: 5, height: 2 }
    <= term::control_codes().enable_alt_screen,
       term::ControlCodes::cursor_position(2, 3),
       term::control_codes().save_cursor,
       term::control_codes().disable_alt_screen,
       term::control_codes().enable_alt_screen,
       term::control_codes().restore_cursor,
       term::Raw::from("X")
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Crlf::default(),
            term::Raw::from("  X"),
            term::ControlCodes::cursor_position(2, 4),
            term::control_codes().clear_attrs
}

// Mode 47 switches screens without saving the cursor or erasing anything,
// so the alt screen comes back the way it was left.
frag! {
    alt_screen_47_keeps_contents { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("$ "),
       term::ControlCodes::dec_private_modes_set(&[47]),
       term::Raw::from("alt"),
       term::ControlCodes::dec_private_modes_reset(&[47]),
       term::Raw::from("\r\n"),
       term::ControlCodes::dec_private_modes_set(&[47])
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("$ "),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().enable_alt_screen,
            term::ControlCodes::cursor_position(1, 1),
            term::Raw::from("  alt"),
            term::Crlf::default(),
            term::ControlCodes::cursor_position(2, 1),
            term::control_codes().clear_attrs
}

// Leaving with mode 47 leaves the cursor where the alt screen had it.
frag! {
    alt_screen_47_exit_keeps_cursor { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("$ "),
       term::ControlCodes::dec_private_modes_set(&[47]),
       term::ControlCodes::cursor_position(3, 2),
       term::ControlCodes::dec_private_modes_reset(&[47]),
       term::Raw::from("x")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("$ "),
            term::Crlf::default(),
            term::Crlf::default(),
            term::Raw::from(" x"),
            term::ControlCodes::cursor_position(3, 3),
            term::control_codes().clear_attrs
}

// The rxvt terminfo entry saves and restores the cursor around mode 47
// itself.
frag! {
    alt_screen_47_with_decsc { scrollback_lines: 100, width: 5, height: 3 }
    <= term::Raw::from("$ "),
       term::control_codes().save_cursor,
       term::ControlCodes::dec_private_modes_set(&[47]),
       term::ControlCodes::cursor_position(3, 2),
       term::Raw::from("vi"),
       term::control_codes().erase_screen,
       term::ControlCodes::dec_private_modes_reset(&[47]),
       term::control_codes().restore_cursor,
       term::Raw::from("ls")
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("$ ls"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

// Leaving with mode 1047 erases the alt screen first.
frag! {
    alt_screen_1047_erases_on_exit { scrollback_lines: 100, width: 5, height: 2 }
    <= term::ControlCodes::dec_private_modes_set(&[1047]),
       term::Raw::from("alt"),
       term::ControlCodes::dec_private_modes_reset(&[1047]),
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::dec_private_modes_set(&[1047])
    => ContentRegion::All =>
            reset_codes,
            empty_scrollback,
            term::control_codes().enable_alt_screen,
            term::Crlf::default(),
            term::ControlCodes::cursor_position(1, 1),
            term::control_codes().clear_attrs
}

// Leaving with mode 1047 when the alt screen isn't up erases nothing.
frag! {
    alt_screen_1047_exit_from_main_screen { scrollback_lines: 100, width: 5, height: 2 }
    <= term::Raw::from("main"),
       term::ControlCodes::dec_private_modes_reset(&[1047])
    => ContentRegion::All =>
            reset_codes,
            term::Raw::from("main"),
            term::ControlCodes::cursor_position(1, 5),
            term::control_codes().clear_attrs
}

// Mode 1048 saves and restores the cursor just like DECSC and DECRC.
frag! {
    save_cursor_mode { scrollback_lines: 100, width: 5, height: 2 }
    <= term::ControlCodes::cursor_position(2, 3),
       term::ControlCodes::fgcolor_idx(1),
       term::ControlCodes::dec_private_modes_set(&[1048]),
       term::control_codes().clear_attrs,
       term::ControlCodes::cursor_position(1, 1),
       term::ControlCodes::dec_private_modes_reset(&[1048]),
       term::Raw::from("x")
    => ContentRegion::All =>
            reset_codes,
            term::Crlf::default(),
            term::Raw::from("  \x1b[31mx\x1b[39m"),
            term::ControlCodes::cursor_position(2, 4),
            term::control_codes().clear_attrs,
            term::ControlCodes::fgcolor_idx(1)
}
