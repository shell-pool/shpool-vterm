# shpool-vterm

This crate is a (for now experimental) attempt at replacing `shpool_vt100` with
something a bit more fit to purpose. [This issue](https://github.com/shell-pool/shpool/issues/46)
goes into detail about what is wrong with our current virtual terminal
implementation. It boils down to two things: bugs we don't know how to fix
and a structural issue with memory usage. shpool should be a lightweight
program, and with the current `shpool_vt100` crate, that is impossible because
of how terminal state is represented.

## Related Projects

* The vt100 crate.
* [libvterm](https://www.leonerd.org.uk/code/libvterm/) (has some great test files we should try to re-use)
