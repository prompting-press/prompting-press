// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Derive / merge integration tests:
//! - derive() Replace default (non-breaking, immutability of base)
//! - derive_with Merge (unions variables/variants/metadata)
//! - immutability of base in both cases

use garde::Validate;
use prompting_press::{
    DeriveOptions, GuardConfig, MergeStrategy, Prompt, PromptOverlay, PromptVariable,
};
use serde::Serialize;
use std::collections::HashMap;

fn base_prompt() -> Prompt {
    Prompt::from_yaml(
        r#"
name: base
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
"#,
    )
    .unwrap()
}

fn parse_variables(json: &str) -> HashMap<String, PromptVariable> {
    serde_json::from_str(json).expect("valid variable map JSON")
}

// ── Replace (default) ─────────────────────────────────────────────────────────

#[test]
fn derive_replace_default_changes_body() {
    let base = base_prompt();
    let derived = base
        .derive(PromptOverlay {
            body: Some("Hello {{ name }}!".to_string()),
            ..Default::default()
        })
        .expect("derive must succeed");

    assert_eq!(derived.body(), "Hello {{ name }}!");
}

#[test]
fn derive_replace_leaves_base_unchanged() {
    let base = base_prompt();
    let original_body = base.body().to_string();

    let _derived = base
        .derive(PromptOverlay {
            body: Some("Different body {{ name }}".to_string()),
            ..Default::default()
        })
        .unwrap();

    // Immutability: base is untouched.
    assert_eq!(base.body(), original_body);
}

#[test]
fn derive_replace_drops_base_variables_not_in_overlay() {
    // Base has two variables; overlay replaces 'variables' with only one.
    let base = Prompt::from_yaml(
        r#"
name: base
role: user
body: "{{ a }} {{ b }}"
variables:
  a: { type: string, trusted: true }
  b: { type: string, trusted: true }
"#,
    )
    .unwrap();

    let new_vars = parse_variables(r#"{"a":{"type":"string","trusted":true}}"#);
    // Replace body too so the agreement check passes (b removed from body).
    let derived = base
        .derive(PromptOverlay {
            variables: Some(new_vars),
            body: Some("{{ a }}".to_string()),
            ..Default::default()
        })
        .expect("derive must succeed");

    assert!(derived.variables().contains_key("a"), "a present");
    assert!(
        !derived.variables().contains_key("b"),
        "b dropped by Replace"
    );
}

#[test]
fn derive_replace_undeclared_variable_fails_construction() {
    let base = base_prompt();
    let err = base
        .derive(PromptOverlay {
            body: Some("{{ name }} {{ ghost }}".to_string()),
            ..Default::default()
        })
        .expect_err("undeclared var must fail");

    assert!(
        matches!(err, prompting_press::ConsumerError::Kernel(_)),
        "expected Kernel error, got {err:?}"
    );
}

// ── Merge ─────────────────────────────────────────────────────────────────────

#[test]
fn derive_merge_unions_variables_and_renders_both() {
    let base = base_prompt();
    let extra_vars = parse_variables(r#"{"count":{"type":"integer","trusted":true}}"#);

    let derived = base
        .derive_with(
            PromptOverlay {
                body: Some("Hi {{ name }}, you have {{ count }} messages".to_string()),
                variables: Some(extra_vars),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        )
        .expect("Merge derive must succeed");

    assert!(
        derived.variables().contains_key("name"),
        "base var retained"
    );
    assert!(
        derived.variables().contains_key("count"),
        "overlay var added"
    );
    assert_eq!(derived.variables().len(), 2);

    // Render to confirm both variables are present and the template renders correctly.
    #[derive(Serialize, Validate)]
    struct MergedVars {
        #[garde(skip)]
        name: String,
        #[garde(skip)]
        count: i64,
    }
    let vars = MergedVars {
        name: "Ada".into(),
        count: 3,
    };
    let result = derived
        .render(&vars, None, &GuardConfig::default(), false)
        .expect("merged render must succeed");

    assert_eq!(result.text, "Hi Ada, you have 3 messages");
}

#[test]
fn derive_merge_leaves_base_unchanged() {
    let base = base_prompt();
    let extra_vars = parse_variables(r#"{"count":{"type":"integer","trusted":true}}"#);

    let _derived = base
        .derive_with(
            PromptOverlay {
                variables: Some(extra_vars),
                body: Some("Hi {{ name }} {{ count }}".to_string()),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        )
        .unwrap();

    // Base must have only 'name' — untouched.
    assert_eq!(base.variables().len(), 1);
    assert!(!base.variables().contains_key("count"));
}

#[test]
fn derive_merge_child_wins_on_key_collision() {
    let base = Prompt::from_yaml(
        r#"
name: base
role: user
body: "{{ field }}"
variables:
  field: { type: string, trusted: true }
"#,
    )
    .unwrap();

    // Overlay replaces 'field' with trusted: false.
    let overlay_vars = parse_variables(r#"{"field":{"type":"string","trusted":false}}"#);
    let derived = base
        .derive_with(
            PromptOverlay {
                variables: Some(overlay_vars),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        )
        .expect("child-wins Merge must succeed");

    let field = derived.variables().get("field").expect("field present");
    assert!(!field.trusted, "overlay's trusted=false wins");
    assert_eq!(derived.variables().len(), 1);
}

#[test]
fn derive_merge_unions_variants() {
    let base = Prompt::from_yaml(
        r#"
name: base
role: user
body: "{{ name }}"
variables:
  name: { type: string, trusted: true }
variants:
  v1:
    body: "v1: {{ name }}"
"#,
    )
    .unwrap();

    let overlay_variants: HashMap<String, prompting_press::PromptVariant> =
        serde_json::from_str(r#"{"v2":{"body":"v2: {{ name }}"}}"#).unwrap();

    let derived = base
        .derive_with(
            PromptOverlay {
                variants: Some(overlay_variants),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        )
        .expect("Merge unions variants");

    assert!(
        derived.variants().contains_key("v1"),
        "base variant retained"
    );
    assert!(
        derived.variants().contains_key("v2"),
        "overlay variant added"
    );
}

#[test]
fn derive_and_derive_with_default_are_identical() {
    let base = base_prompt();
    let overlay = PromptOverlay {
        body: Some("Hello {{ name }}!".to_string()),
        ..Default::default()
    };

    let via_derive = base.derive(overlay.clone()).expect("derive");
    let via_derive_with = base
        .derive_with(overlay, DeriveOptions::default())
        .expect("derive_with default");

    assert_eq!(via_derive.body(), via_derive_with.body());
}
