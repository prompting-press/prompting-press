// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! US2 agreement-purity suite (spec 002, T022): V2.5 — analysis mutates nothing.
//!
//! The agreement analysis MUST be pure: it must not mutate the prompt definition, the
//! values, or any output, and must not render as a side effect (FR-018, SC-006). The
//! template body lives in a JSON data fixture, never inlined here (self-referential-grep
//! mitigation, see `tests/fixtures/README.md`).

mod common;

use common::load_def_fixture;
use prompting_press_core::required_roots;

/// V2.5 — after `required_roots`, the `def` and the `values` are byte-for-byte unchanged,
/// and the analysis returns only the required-roots set (no rendered text exists to leak).
///
/// Purity is checked structurally: `required_roots` takes `&PromptDefinition` (a shared
/// borrow — it cannot mutate the def through the type system) and never takes `values` at
/// all (it does not render). This test additionally snapshots both inputs and asserts
/// equality after the call, so a future signature change that introduced mutation would
/// fail loudly. [FR-018, SC-006]
#[test]
fn v2_5_analysis_is_pure_inputs_unchanged() {
    let def = load_def_fixture("agreement-purity");

    // A `values` map the analysis must NOT touch (it does not render). Snapshot it.
    let values = minijinja::Value::from_serialize(serde_json::json!({
        "alpha": "A",
        "beta": { "gamma": "G" },
        "delta": ["d1", "d2"],
    }));
    let values_before = values.clone();

    // Snapshot the def by serializing it to canonical JSON before analysis.
    let def_before =
        serde_json::to_string(&def).expect("def must serialize for the before-snapshot");

    let agreement = required_roots(&def, None).expect("analysis must succeed");

    // The def is unchanged (re-serialize and compare to the snapshot).
    let def_after = serde_json::to_string(&def).expect("def must serialize for the after-snapshot");
    assert_eq!(
        def_before, def_after,
        "the prompt definition must be unchanged after analysis (FR-018)",
    );

    // The values are unchanged — the analysis never rendered, so it never read them.
    assert_eq!(
        values, values_before,
        "the values must be unchanged after analysis (FR-018: no render side effect)",
    );

    // The analysis produced ONLY the required-roots set — there is no rendered `text`
    // field on `Agreement` to carry a side-effect render. The roots are the real ones.
    assert_eq!(agreement.variant, "default");
    let expected: std::collections::BTreeSet<String> = ["alpha", "beta", "delta"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        agreement.required_roots, expected,
        "roots are the external roots only; `gamma` (nested) and `x` (loop local) excluded",
    );
}
