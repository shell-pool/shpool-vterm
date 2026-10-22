# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.9](https://github.com/shell-pool/shpool-vterm/compare/v0.2.8...v0.2.9) - 2026-10-22

### Added

- add reverse screen mode support
- track fg color for blank cells
- add DECALN support
- add LF/NL mode support
- add support for all vterm test query modes
- add test driver for libvterm tests

### Other

- update README
- mark DECSCA intentionally unsupported
- enable resize taller libvterm test
- ignore some more libvterm tests
- mark alt fonts ignored
- mark reflow divergence intentional
- mark 13state_edit/IL intentionally ignored
- mark HPB/VPB intentionally unsupported
- clean up callback only vterm tests

## [0.2.8](https://github.com/shell-pool/shpool-vterm/compare/v0.2.7...v0.2.8) - 2026-10-14

### Added

- restore the saved cursor on reattach
- support IND, NEL, HPR, VPR and CHT
- support the 47, 1047 and 1048 alt screen modes
- support the DEC special graphics and UK charsets
- support auto-wrap mode (DECAWM)

### Fixed

- restore a wrap pending after a wide char shown as a blank
- restore every palette color, not just the first seven
- skip restoring a saved cursor that only differs by its link
- leave trailing blanks out of the restore
- put restored bottom lines back on the rows they came from
- survive a screen with no rows or columns
- restore to home when no cursor was saved, even after a resize
- leave a pending single shift alone on DECSC and DECRC
- make CSI s and CSI u save the same state as DECSC and DECRC
- save origin mode with the cursor and clamp it on restore
- make RIS and DECSTR reset everything they are supposed to
- drop blank rows under the cursor when the screen gets shorter
- stop CUU and CUD at the scroll region margins
- move the cursor to the start of the line on IL and DL
- enter the alt screen the way xterm does
- share the cursor, scroll region and origin mode between screens
- home the cursor before painting the alt screen on restore
- drop the scroll region when the terminal is resized
- set the scroll region and origin mode the way xterm does
- make ED erase the whole screen and ED 3 keep it
- parse OSC strings the way applications actually send them
- erase, insert and scroll with the current background color
- reset the modes a restore depends on before painting it
- don't drop scrollback lines on resize
- keep wide chars, blank space and the cursor intact across reflow
- blank out wide chars that are only partly overwritten or erased
- implement the pending wrap state of the last column
- scroll the whole screen when the main screen is partly filled
- don't treat CSI sequences with private markers as plain commands

### Other

- explain how to run the fuzz target
- fuzz the terminal with cargo-fuzz
- use named control codes instead of raw escapes
- pack State's on/off modes into a bitflags word

## [0.2.7](https://github.com/shell-pool/shpool-vterm/compare/v0.2.6...v0.2.7) - 2026-10-05

### Added

- add Term::title() getter

## [0.2.6](https://github.com/shell-pool/shpool-vterm/compare/v0.2.5...v0.2.6) - 2026-09-23

### Fixed

- scroll polarity was swapped
- track positions across resize
- bug where resizing would clobber blank lines

## [0.2.5](https://github.com/shell-pool/shpool-vterm/compare/v0.2.4...v0.2.5) - 2026-09-18

### Added

- restore scrollback when in altscreen mode
- restore alt mode
- properly restore input modes

### Fixed

- flesh out reset sequence some more
- origin mode homing bug
- guard against some panics
- scroll region resize bug
- zero width char panic
- delete char panic

## [0.2.4](https://github.com/shell-pool/shpool-vterm/compare/v0.2.3...v0.2.4) - 2026-09-14

### Fixed

- subtraction underflow panic in RI handler

## [0.2.3](https://github.com/shell-pool/shpool-vterm/compare/v0.2.2...v0.2.3) - 2026-09-09

### Added

- add insert mode support
- add backward tab support
- add cursor blinking support
- more fully support underline color
- add RI support

### Fixed

- more thoroughly ignore queries
- avoid restoring half-open links

### Other

- ignore DECCOLM

## [0.2.2](https://github.com/shell-pool/shpool-vterm/compare/v0.2.1...v0.2.2) - 2026-08-28

### Added

- add optional log tagging

## [0.2.1](https://github.com/shell-pool/shpool-vterm/compare/v0.2.0...v0.2.1) - 2026-08-27

### Added

- add HVP support
- add title stack support
- add cursor style support
- add VPA support
- add REP support
- add HPA support
- suppress ascii charset config codes
- add report focus tracking support

### Other

- ignore DA queries
- suppress query control code logs

## [0.2.0](https://github.com/shell-pool/shpool-vterm/compare/v0.1.0...v0.2.0) - 2026-08-14

### Added

- add presubmit checks
- configure release-plz
- add ECH support
- ignore DEC{SET,RST} 2026
- add support for esc app keypad mode controls
- add backsapce support
- restore tabstop state
- add tabstop control command support
- add tab support

### Fixed

- publish workflow
- publish pipeline
- out of bounds on newline

### Other

- [**breaking**] hide internal types better
- ignore non-actionable codes
# [0.1.0] - 2026-02-10

Initial version. Lots is probably still broken but we are at a point
where we can try integrating with shpool on an experimental basis.
