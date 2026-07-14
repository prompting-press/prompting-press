// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Provenance integration tests:
//! - ProvenanceExt::provenance_attributes() → exactly 4 prompting_press.prompt.* keys
//! - provenance_attributes_of free function
//! - values match RenderResult fields
//! - KEY_* constants carry correct strings

use garde::Validate;
use prompting_press::{
    provenance_attributes_of, GuardConfig, Prompt, ProvenanceExt, KEY_NAME, KEY_RENDER_HASH,
    KEY_TEMPLATE_HASH, KEY_VARIANT,
};
use serde::Serialize;

#[derive(Serialize, Validate)]
struct SimpleVars {
    #[garde(skip)]
    name: String,
}

fn render_base() -> (Prompt, prompting_press::RenderResult) {
    let prompt = Prompt::from_yaml(
        r#"
name: greet
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
"#,
    )
    .unwrap();

    let vars = SimpleVars {
        name: "Ada".to_string(),
    };
    let result = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .unwrap();
    (prompt, result)
}

// ── KEY_* constants ───────────────────────────────────────────────────────────

#[test]
fn key_constants_have_expected_string_values() {
    assert_eq!(KEY_NAME, "prompting_press.prompt.name");
    assert_eq!(KEY_VARIANT, "prompting_press.prompt.variant");
    assert_eq!(KEY_TEMPLATE_HASH, "prompting_press.prompt.template_hash");
    assert_eq!(KEY_RENDER_HASH, "prompting_press.prompt.render_hash");
}

// ── extension trait ───────────────────────────────────────────────────────────

#[test]
fn provenance_attributes_returns_exactly_four_entries() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();
    assert_eq!(attrs.len(), 4, "must be exactly 4 provenance keys");
}

#[test]
fn provenance_attributes_keys_match_constants() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();

    assert!(attrs.contains_key(KEY_NAME), "KEY_NAME present");
    assert!(attrs.contains_key(KEY_VARIANT), "KEY_VARIANT present");
    assert!(
        attrs.contains_key(KEY_TEMPLATE_HASH),
        "KEY_TEMPLATE_HASH present"
    );
    assert!(
        attrs.contains_key(KEY_RENDER_HASH),
        "KEY_RENDER_HASH present"
    );
}

#[test]
fn provenance_attributes_values_match_render_result_fields() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();

    assert_eq!(attrs[KEY_NAME], result.name);
    assert_eq!(attrs[KEY_VARIANT], result.variant);
    assert_eq!(attrs[KEY_TEMPLATE_HASH], result.template_hash);
    assert_eq!(attrs[KEY_RENDER_HASH], result.render_hash);
}

#[test]
fn provenance_attributes_name_is_prompt_name() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();
    assert_eq!(attrs[KEY_NAME], "greet");
}

#[test]
fn provenance_attributes_variant_is_default_for_no_variant_render() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();
    assert_eq!(attrs[KEY_VARIANT], "default");
}

#[test]
fn provenance_attributes_hashes_are_64_char_hex() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();

    let th = &attrs[KEY_TEMPLATE_HASH];
    let rh = &attrs[KEY_RENDER_HASH];
    assert_eq!(th.len(), 64, "template_hash must be 64 chars");
    assert_eq!(rh.len(), 64, "render_hash must be 64 chars");
    assert!(
        th.chars().all(|c| c.is_ascii_hexdigit()),
        "template_hash must be hex"
    );
    assert!(
        rh.chars().all(|c| c.is_ascii_hexdigit()),
        "render_hash must be hex"
    );
}

#[test]
fn provenance_attributes_excludes_text_and_other_fields() {
    let (_, result) = render_base();
    let attrs = result.provenance_attributes();

    assert!(!attrs.contains_key("text"), "text must not be in map");
    assert!(!attrs.contains_key("guard"), "guard must not be in map");
}

// ── named variant ─────────────────────────────────────────────────────────────

#[test]
fn provenance_attributes_variant_reflects_named_variant() {
    let prompt = Prompt::from_yaml(
        r#"
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
variants:
  brief:
    body: "Hi {{ name }}"
"#,
    )
    .unwrap();

    let vars = SimpleVars {
        name: "Ada".to_string(),
    };
    let result = prompt
        .render(&vars, Some("brief"), &GuardConfig::default(), false)
        .unwrap();
    let attrs = result.provenance_attributes();

    assert_eq!(attrs[KEY_VARIANT], "brief");
}

// ── free function ─────────────────────────────────────────────────────────────

#[test]
fn provenance_attributes_of_free_fn_matches_extension_trait() {
    let (_, result) = render_base();

    let via_trait = result.provenance_attributes();
    let via_fn = provenance_attributes_of(
        &result.name,
        &result.variant,
        &result.template_hash,
        &result.render_hash,
    );

    assert_eq!(via_trait, via_fn);
}

#[test]
fn provenance_attributes_of_free_fn_returns_exactly_four_entries() {
    let map = provenance_attributes_of("my-prompt", "formal", &"a".repeat(64), &"b".repeat(64));
    assert_eq!(map.len(), 4);
}

#[test]
fn provenance_attributes_of_free_fn_values_are_passed_through() {
    let map = provenance_attributes_of("p", "v", "th", "rh");
    assert_eq!(map[KEY_NAME], "p");
    assert_eq!(map[KEY_VARIANT], "v");
    assert_eq!(map[KEY_TEMPLATE_HASH], "th");
    assert_eq!(map[KEY_RENDER_HASH], "rh");
}
