# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
