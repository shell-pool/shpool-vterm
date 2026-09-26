// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Throws sessions of random input and resizes at the terminal, checking that
//! nothing makes it panic and that restoring whatever state it ends up in
//! gives back that state. Run it from the root of the repo with:
//!
//! ```text
//! cargo +nightly fuzz run restore -- -dict=fuzz/restore.dict
//! ```
//!
//! A failing input gets saved in `fuzz/artifacts/restore`. `cargo fuzz tmin`
//! shrinks it, and `cargo fuzz fmt` prints the session it decodes to. Once
//! the bug is fixed, add the input to `tests/data/fuzz/restore`, which
//! `cargo test` replays.

#![no_main]

#[path = "../../tests/support/fuzz.rs"]
mod fuzz;

libfuzzer_sys::fuzz_target!(|session: fuzz::Session| session.check());
