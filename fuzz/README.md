# Fuzzing

The `restore` target throws sessions of random input and resizes at the
terminal. For each session it checks that the terminal does not panic, that
restoring it into a fresh terminal gives back what it had, and that restoring
again from there changes nothing more. The session and the checks live in
`tests/support/fuzz.rs`.

## Setup

You need a nightly toolchain, cargo-fuzz and a C++ compiler, since libFuzzer
gets built from source. Linux and macOS both work.

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

The repo's `rust-toolchain.toml` pins the stable toolchain that `cargo test`
uses, so every `cargo fuzz` command below needs `+nightly`.

## Running

From the root of the repo:

```sh
cargo +nightly fuzz run restore -- -dict=fuzz/restore.dict
```

This keeps going until it finds a failure or you stop it. Everything after
`--` goes to libFuzzer. `fuzz/restore.dict` gives it pieces of the control
codes that the terminal understands, so that it does not have to find each of
them one byte at a time.

```sh
# Stop after ten minutes.
cargo +nightly fuzz run restore -- -dict=fuzz/restore.dict -max_total_time=600
# Run eight jobs in parallel, without AddressSanitizer.
cargo +nightly fuzz run -j 8 -s none restore -- -dict=fuzz/restore.dict
```

shpool-vterm itself has no unsafe code, so AddressSanitizer mostly watches
over its dependencies. Turning it off with `-s none` makes runs faster.

The inputs that reach new code pile up in `fuzz/corpus/restore`, and the next
run starts from them. `cargo +nightly fuzz cmin restore` throws out the ones
that don't add anything.

## When it finds something

The input that failed gets saved in `fuzz/artifacts/restore`. To run it again,
shrink it, and print the session it decodes to:

```sh
cargo +nightly fuzz run restore fuzz/artifacts/restore/crash-<hash>
cargo +nightly fuzz tmin restore fuzz/artifacts/restore/crash-<hash>
cargo +nightly fuzz fmt restore fuzz/artifacts/restore/minimized-from-<hash>
```

`fmt` prints the screen size, the resizes and each chunk of input as a byte
string, which makes it easy to turn into a `frag!` test.

Once the bug is fixed, copy the input into `tests/data/fuzz/restore`, named
after what it caught. `cargo test --test fuzz` replays everything in there,
without needing nightly.

Changing `Session` or `Step` changes what every saved input decodes to, so the
ones in `tests/data/fuzz/restore` would stop testing what they used to.
