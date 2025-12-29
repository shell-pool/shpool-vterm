# shpool-vterm

This crate is a (for now experimental) attempt at replacing `shpool_vt100` with
something a bit more fit to purpose. [This issue](https://github.com/shell-pool/shpool/issues/46)
goes into detail about what is wrong with our current virtual terminal
implementation. It boils down to two things: bugs we don't know how to fix
and a structural issue with memory usage. shpool should be a lightweight
program, and with the current `shpool_vt100` crate, that is impossible because
of how terminal state is represented.

## Goals

shpool-vterm aims to implement an in-memory virtual terminal that supports just
a few top-line operations:

* Writing new input to the terminal. In order to maintain terminal state, we
  need to be able to handle the raw output of a linux pty and continually parse
  it into some sort of in-memory state that will allow us to perform the other
  operations we need to be able to handle.
* Correctly handle re-sizing. The terminal needs to be able to correctly handle
  dynamic changes both to the scrollback length and to the visible window size.
* Dump terminal contents as a blob of terminal control codes that can be input
  to another terminal to make it reflect the same thing that our in-memory
  terminal does. The basic idea is to start with a reset code, then just rebuild
  everything. We need to start with a reset because we can't know what state
  the other terminal is in and we need determinism.

## Related Projects

* The vt100 crate.
* [libvterm](https://www.leonerd.org.uk/code/libvterm/) (has some great test files we should try to re-use)

## License

Some of this code is original and some is copied directly from vt100. The
vt100 code is copyright Jesse Luehrs and re-used under the MIT license
and the original code is copyright Google LLC and provided via the Apache-2
license. The test data in tests/data/libvterm is copyright Paul Evans and
is used under the MIT license.
