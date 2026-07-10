// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Version integration tests: core_version() returns a non-empty string.

use prompting_press::core_version;

#[test]
fn core_version_returns_non_empty_string() {
    let v = core_version();
    assert!(!v.is_empty(), "core_version must not be empty");
}

#[test]
fn core_version_is_stable_across_calls() {
    // The version is a static string — it must be identical across calls.
    assert_eq!(core_version(), core_version());
}
