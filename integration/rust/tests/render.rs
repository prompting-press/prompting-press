// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Render integration tests: happy path, variants, missing-variant error,
//! undefined-variable loud error.

use garde::Validate;
use prompting_press::GuardConfig;
use prompting_press::{error::code, ConsumerError, Prompt};
use serde::Serialize;

// ── Vars fixtures ─────────────────────────────────────────────────────────────

#[derive(Serialize, Validate)]
struct NameVars {
    #[garde(length(min = 1))]
    name: String,
}

#[derive(Serialize, Validate)]
struct NameCountVars {
    #[garde(length(min = 1))]
    name: String,
    #[garde(range(min = 0, max = 9999))]
    count: u32,
}

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn render_default_variant_returns_rendered_text() {
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

    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let result = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .expect("render must succeed");

    assert_eq!(result.text, "Hi Ada");
    assert_eq!(result.variant, "default");
    assert_eq!(result.name, "greet");
    // SHA-256 hex is always 64 lowercase chars.
    assert_eq!(result.template_hash.len(), 64);
    assert_eq!(result.render_hash.len(), 64);
}

#[test]
fn render_named_variant_renders_variant_body() {
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

    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let result = prompt
        .render(&vars, Some("brief"), &GuardConfig::default(), false)
        .expect("named variant render must succeed");

    assert_eq!(result.text, "Hi Ada");
    assert_eq!(result.variant, "brief");
}

#[test]
fn render_deterministic_hashes_across_two_calls() {
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

    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let r1 = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .unwrap();
    let r2 = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .unwrap();

    assert_eq!(r1.text, r2.text);
    assert_eq!(r1.template_hash, r2.template_hash);
    assert_eq!(r1.render_hash, r2.render_hash);
}

// ── validation failure ────────────────────────────────────────────────────────

#[test]
fn render_validation_failure_returns_consumer_error_validation() {
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

    // Empty name fails garde length(min=1).
    let bad_vars = NameVars {
        name: String::new(),
    };
    let err = prompt
        .render(&bad_vars, None, &GuardConfig::default(), false)
        .expect_err("invalid vars must fail");

    assert!(
        matches!(err, ConsumerError::Validation(_)),
        "expected Validation error, got {err:?}"
    );
}

// ── missing variant ───────────────────────────────────────────────────────────

#[test]
fn render_missing_variant_returns_unknown_variant_error() {
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

    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let err = prompt
        .render(&vars, Some("nonexistent"), &GuardConfig::default(), false)
        .expect_err("unknown variant must fail");

    match &err {
        ConsumerError::Kernel(rows) => {
            assert!(
                rows.iter().any(|r| r.code == code::UNKNOWN_VARIANT),
                "expected unknown_variant code, got {rows:?}"
            );
        }
        other => panic!("expected Kernel error, got {other:?}"),
    }
}

// ── undefined variable at render ──────────────────────────────────────────────

#[test]
fn render_mismatched_vars_struct_produces_loud_undefined_variable_error() {
    // Prompt declares 'name'; vars struct serializes 'count' instead — a mismatched field
    // that passes garde but triggers the kernel's strict-undefined environment.
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

    // This struct has no 'name' field — it will serialize to {"count": 3} and the kernel
    // will hit a strict-undefined on 'name'.
    #[derive(Serialize, Validate)]
    struct WrongVars {
        #[garde(skip)]
        count: u32,
    }

    let bad_vars = WrongVars { count: 3 };
    let err = prompt
        .render(&bad_vars, None, &GuardConfig::default(), false)
        .expect_err("mismatched vars must produce a loud error");

    // The library guarantees this is never a silent empty render — it is a loud error.
    assert!(
        matches!(err, ConsumerError::Kernel(_)),
        "expected loud Kernel error (undefined_variable), got {err:?}"
    );
}

// ── multi-variable render ─────────────────────────────────────────────────────

#[test]
fn render_multiple_variables_interpolated_correctly() {
    let prompt = Prompt::from_yaml(
        r#"
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name: { type: string, trusted: true }
  count: { type: integer, trusted: true }
"#,
    )
    .unwrap();

    let vars = NameCountVars {
        name: "Ada".to_string(),
        count: 5,
    };
    let result = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .expect("multi-var render must succeed");

    assert_eq!(result.text, "Hi Ada, you have 5 messages");
}
