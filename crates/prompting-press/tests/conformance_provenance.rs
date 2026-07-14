// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Conformance corpus — provenance-attributes parity runner (spec 018, T010).
//!
//! Drives the `provenance-attributes` marshaling fixture through the consumer's render path
//! and verifies that `provenance_attributes()` produces the four canonical
//! `prompting_press.prompt.*` keys with values matching the committed golden. Because all
//! three bindings (Rust/Python/TypeScript) verify against the SAME golden, cross-binding
//! parity is transitive (SC-003).

mod common;

use common::{build_vars, load_marshaling_fixtures, RawVars};
use prompting_press::Prompt;
use prompting_press::{ProvenanceExt, KEY_NAME, KEY_RENDER_HASH, KEY_TEMPLATE_HASH, KEY_VARIANT};
use prompting_press_core::GuardConfig;

/// T010 — the provenance-attributes case renders identically to the golden AND the
/// provenance map projects the four fields with the correct library-owned keys.
#[test]
fn provenance_attributes_fixture_matches_golden_and_has_four_keys() {
    let fixtures = load_marshaling_fixtures();

    // Locate the provenance-attributes fixture.
    let (_, fx) = fixtures
        .iter()
        .find(|(_, fx)| fx.case == "provenance-attributes")
        .expect("provenance-attributes fixture must exist in conformance/marshaling/");

    let def: prompting_press_core::PromptDefinition = serde_json::from_value(fx.definition.clone())
        .expect("valid prompt definition in provenance-attributes fixture");
    let prompt = Prompt::new(def).expect("Prompt::new must succeed for provenance-attributes");

    let vars = RawVars(build_vars(&fx.input));
    let result = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .expect("render must succeed for provenance-attributes");

    // Verify the golden is populated (guard against un-regenerated fixture).
    assert!(
        !fx.expected.template_hash.is_empty() && !fx.expected.render_hash.is_empty(),
        "golden is empty — run `moon run conformance:regen`"
    );

    // Render parity with the golden (the plain marshaling contract).
    assert_eq!(result.text, fx.expected.text, "text must match golden");
    assert_eq!(
        result.template_hash, fx.expected.template_hash,
        "template_hash must match golden"
    );
    assert_eq!(
        result.render_hash, fx.expected.render_hash,
        "render_hash must match golden"
    );

    // Provenance map contract (spec 018 T010, SC-003).
    let attrs = result.provenance_attributes();

    assert_eq!(
        attrs.len(),
        4,
        "provenance map must contain exactly 4 entries"
    );
    assert_eq!(
        attrs[KEY_NAME], "welcome",
        "KEY_NAME must be the prompt name"
    );
    assert_eq!(
        attrs[KEY_VARIANT], "default",
        "KEY_VARIANT must be 'default'"
    );
    assert_eq!(
        attrs[KEY_TEMPLATE_HASH], fx.expected.template_hash,
        "KEY_TEMPLATE_HASH must match the golden template_hash"
    );
    assert_eq!(
        attrs[KEY_RENDER_HASH], fx.expected.render_hash,
        "KEY_RENDER_HASH must match the golden render_hash"
    );
}
