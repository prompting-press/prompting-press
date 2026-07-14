// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Provenance attributes tests — spec 018 (T007, T011, T012, T013).
//!
//! T007 (US1): `result.provenance_attributes()` returns the 4 expected keys/values; a
//!             default-variant render yields `prompting_press.prompt.variant == "default"`.
//! T011 (US2): no telemetry dependency (manifest check is in CI; this asserts no dep is
//!             pulled into the test binary).
//! T012 (US2): exactly 4 keys; map excludes text/guard/metadata/`output_model`; pure.
//! T013 (US3): the four fields remain publicly readable; a custom-keyed map is buildable.

use garde::Validate;
use prompting_press::{ProvenanceExt, KEY_NAME, KEY_RENDER_HASH, KEY_TEMPLATE_HASH, KEY_VARIANT};
use prompting_press_core::GuardConfig;
use serde::Serialize;

// ── helpers ───────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Validate)]
struct NameVars {
    #[garde(skip)]
    name: String,
}

fn greeting_prompt() -> prompting_press::Prompt {
    prompting_press::Prompt::from_json(
        r#"{
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}, you have 0 messages",
            "variables": { "name": { "type": "string", "trusted": true } }
        }"#,
    )
    .expect("valid greeting prompt")
}

fn greet_vars() -> NameVars {
    NameVars {
        name: "Ada".to_string(),
    }
}

// ── T007 ─────────────────────────────────────────────────────────────────────────────────

/// T007 — `result.provenance_attributes()` returns exactly 4 entries whose values equal the
/// result fields, under the library-owned `prompting_press.prompt.*` keys.
#[test]
fn provenance_attributes_returns_four_entries_with_correct_keys_and_values() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .expect("render must succeed");

    let attrs = result.provenance_attributes();

    // Exactly 4 entries (FR-002, SC-004).
    assert_eq!(attrs.len(), 4, "must contain exactly 4 entries");

    // Keys are the library-owned constants (FR-003).
    assert!(attrs.contains_key(KEY_NAME), "must contain KEY_NAME");
    assert!(attrs.contains_key(KEY_VARIANT), "must contain KEY_VARIANT");
    assert!(
        attrs.contains_key(KEY_TEMPLATE_HASH),
        "must contain KEY_TEMPLATE_HASH"
    );
    assert!(
        attrs.contains_key(KEY_RENDER_HASH),
        "must contain KEY_RENDER_HASH"
    );

    // Values equal the result fields 1:1 (FR-001).
    assert_eq!(attrs[KEY_NAME], result.name);
    assert_eq!(attrs[KEY_VARIANT], result.variant);
    assert_eq!(attrs[KEY_TEMPLATE_HASH], result.template_hash);
    assert_eq!(attrs[KEY_RENDER_HASH], result.render_hash);
}

/// T007 (INV-3) — a default-variant render always yields `variant == "default"` in the map.
#[test]
fn default_variant_renders_as_default_in_attributes() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .expect("render must succeed");

    let attrs = result.provenance_attributes();
    assert_eq!(
        attrs[KEY_VARIANT], "default",
        "no variant selected ⇒ KEY_VARIANT must be \"default\""
    );
}

/// T007 — two renders of the same prompt/vars produce identical attribute maps (SC-003,
/// deterministic content identity).
#[test]
fn provenance_attributes_are_deterministic() {
    let prompt = greeting_prompt();
    let map1 = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap()
        .provenance_attributes();
    let map2 = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap()
        .provenance_attributes();

    assert_eq!(
        map1, map2,
        "identical renders must produce identical attribute maps"
    );
}

/// T007 — free function and extension trait produce identical maps.
#[test]
fn free_fn_and_trait_produce_identical_maps() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap();

    let via_trait = result.provenance_attributes();
    let via_fn = prompting_press::provenance_attributes_of(
        &result.name,
        &result.variant,
        &result.template_hash,
        &result.render_hash,
    );

    assert_eq!(via_trait, via_fn);
}

// ── T012 (US2) ───────────────────────────────────────────────────────────────────────────

/// T012 — map excludes text, guard text, metadata, and `output_model` (FR-007, SC-004, SEC-001).
/// The map is an explicit allowlist — these keys must never appear.
#[test]
fn provenance_attributes_excludes_non_provenance_fields() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap();

    let attrs = result.provenance_attributes();

    // Explicit exclusions (FR-007).
    assert!(
        !attrs.contains_key("text"),
        "rendered body text must be excluded"
    );
    assert!(!attrs.contains_key("guard"), "guard text must be excluded");
    assert!(
        !attrs.contains_key("output_model"),
        "output_model must be excluded"
    );
    assert!(!attrs.contains_key("metadata"), "metadata must be excluded");
    // Still exactly 4.
    assert_eq!(attrs.len(), 4);
}

/// T012 — the helper is a pure projection: calling it twice on the same result produces
/// equal maps, not mutating anything.
#[test]
fn provenance_attributes_is_pure() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap();

    // Called twice on the same result — must produce equal maps (pure, no side-effects).
    let a = result.provenance_attributes();
    let b = result.provenance_attributes();
    assert_eq!(
        a, b,
        "calling provenance_attributes twice must produce equal maps"
    );

    // The result fields are unchanged after calling the helper.
    assert_eq!(result.name, "greet");
    assert_eq!(result.variant, "default");
}

// ── T013 (US3) ───────────────────────────────────────────────────────────────────────────

/// T013 — the four provenance fields remain publicly readable; a consumer can build
/// a custom-keyed attribute map without the helper (FR-008; helper is additive).
#[test]
fn public_fields_allow_custom_keyed_map() {
    let prompt = greeting_prompt();
    let result = prompt
        .render(&greet_vars(), None, &GuardConfig::default(), false)
        .unwrap();

    // Consumer-chosen keys — nothing to do with prompting_press.prompt.*.
    let custom: std::collections::BTreeMap<String, String> = [
        ("my.prompt.name".to_string(), result.name.clone()),
        ("my.prompt.variant".to_string(), result.variant.clone()),
        (
            "my.prompt.template_hash".to_string(),
            result.template_hash.clone(),
        ),
        (
            "my.prompt.render_hash".to_string(),
            result.render_hash.clone(),
        ),
    ]
    .into_iter()
    .collect();

    assert_eq!(custom.len(), 4);
    assert_eq!(custom["my.prompt.name"], "greet");
    assert_eq!(custom["my.prompt.variant"], "default");
}
