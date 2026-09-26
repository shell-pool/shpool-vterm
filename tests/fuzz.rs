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

//! Replays the inputs that once made the `restore` fuzz target in `fuzz/`
//! fail, so that the bugs they found stay fixed.

#[path = "support/fuzz.rs"]
mod fuzz;

use arbitrary::{Arbitrary, Unstructured};
use std::path::Path;

#[test]
fn restore_regressions() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/fuzz/restore");
    // The directory only shows up with the first regression.
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let mut paths: Vec<_> =
        entries.map(|entry| entry.expect("listing regressions").path()).collect();
    paths.sort();

    for path in paths {
        let data = std::fs::read(&path).expect("reading regression");
        // This is how `fuzz_target!` decodes its input too.
        let session = fuzz::Session::arbitrary_take_rest(Unstructured::new(&data))
            .unwrap_or_else(|e| panic!("decoding {}: {e}", path.display()));
        eprintln!("{}: {session:#?}", path.display());
        session.check();
    }
}
