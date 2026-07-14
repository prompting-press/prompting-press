// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Construction integration tests: from_yaml / from_json / from_toml (valid + invalid).
//!
//! Exercises Prompt::from_yaml, from_json, from_toml on well-formed input and verifies
//! that invalid input (bad shape, undeclared variable, excluded feature, reserved name)
//! surfaces ConsumerError::Load or ConsumerError::Kernel with the expected codes.

use prompting_press::{error::code, ConsumerError, Prompt};

// ── shared fixture ─────────────────────────────────────────────────────────────

const YAML_BASIC: &str = r#"
name: base
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
"#;

const JSON_BASIC: &str = r#"{
  "name": "base",
  "role": "user",
  "body": "Hi {{ name }}",
  "variables": { "name": { "type": "string", "trusted": true } }
}"#;

const TOML_BASIC: &str = r#"
name = "base"
role = "user"
body = "Hi {{ name }}"

[variables.name]
type = "string"
trusted = true
"#;

// ── from_yaml ─────────────────────────────────────────────────────────────────

#[test]
fn from_yaml_valid_constructs_prompt() {
    let p = Prompt::from_yaml(YAML_BASIC).expect("valid YAML must construct");
    assert_eq!(p.name(), "base");
    assert_eq!(p.body(), "Hi {{ name }}");
    assert!(p.variables().contains_key("name"));
    assert!(p.variants().is_empty());
}

#[test]
fn from_yaml_bad_shape_returns_load_error() {
    // Missing required 'role' field → serde shape error.
    let bad = r#"name: test
body: "Hi"
variables: {}
"#;
    let err = Prompt::from_yaml(bad).expect_err("bad shape must fail");
    assert!(
        matches!(err, ConsumerError::Load(_)),
        "expected Load, got {err:?}"
    );
}

#[test]
fn from_yaml_undeclared_variable_returns_kernel_error() {
    let bad = r#"
name: bad
role: user
body: "{{ ghost }}"
variables:
  name: { type: string, trusted: true }
"#;
    let err = Prompt::from_yaml(bad).expect_err("undeclared var must fail");
    match &err {
        ConsumerError::Kernel(rows) => {
            assert!(
                rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                "expected undefined_variable code, got {rows:?}"
            );
        }
        other => panic!("expected Kernel error, got {other:?}"),
    }
}

#[test]
fn from_yaml_excluded_feature_returns_kernel_error() {
    let bad = r#"
name: bad
role: user
body: '{% include "x" %}'
variables: {}
"#;
    let err = Prompt::from_yaml(bad).expect_err("excluded feature must fail");
    assert!(
        matches!(&err, ConsumerError::Kernel(_)),
        "expected Kernel error, got {err:?}"
    );
}

#[test]
fn from_yaml_reserved_variant_name_returns_kernel_error() {
    let bad = r#"
name: bad
role: user
body: "Hi"
variables: {}
variants:
  default:
    body: "shadowed"
"#;
    let err = Prompt::from_yaml(bad).expect_err("reserved variant name must fail");
    match &err {
        ConsumerError::Kernel(rows) => {
            assert_eq!(rows[0].field, "variant");
        }
        other => panic!("expected Kernel error, got {other:?}"),
    }
}

// ── from_json ─────────────────────────────────────────────────────────────────

#[test]
fn from_json_valid_constructs_prompt() {
    let p = Prompt::from_json(JSON_BASIC).expect("valid JSON must construct");
    assert_eq!(p.name(), "base");
    assert!(p.variables().contains_key("name"));
}

#[test]
fn from_json_malformed_returns_load_error() {
    let bad = "not json at all";
    let err = Prompt::from_json(bad).expect_err("bad JSON must fail");
    assert!(
        matches!(err, ConsumerError::Load(_)),
        "expected Load, got {err:?}"
    );
}

#[test]
fn from_json_undeclared_variable_returns_kernel_error() {
    let bad = r#"{"name":"bad","role":"user","body":"{{ ghost }}","variables":{"name":{"type":"string","trusted":true}}}"#;
    let err = Prompt::from_json(bad).expect_err("undeclared var must fail");
    match &err {
        ConsumerError::Kernel(rows) => {
            assert!(
                rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                "expected undefined_variable, got {rows:?}"
            );
        }
        other => panic!("expected Kernel error, got {other:?}"),
    }
}

// ── from_toml ─────────────────────────────────────────────────────────────────

#[test]
fn from_toml_valid_constructs_prompt() {
    let p = Prompt::from_toml(TOML_BASIC).expect("valid TOML must construct");
    assert_eq!(p.name(), "base");
    assert_eq!(p.body(), "Hi {{ name }}");
    assert!(p.variables().contains_key("name"));
}

#[test]
fn from_toml_malformed_returns_load_error() {
    let bad = "not = toml [broken";
    let err = Prompt::from_toml(bad).expect_err("bad TOML must fail");
    assert!(
        matches!(err, ConsumerError::Load(_)),
        "expected Load, got {err:?}"
    );
}

#[test]
fn from_toml_undeclared_variable_returns_kernel_error() {
    let bad = r#"
name = "bad"
role = "user"
body = "{{ ghost }}"
[variables.name]
type = "string"
trusted = true
"#;
    let err = Prompt::from_toml(bad).expect_err("undeclared var must fail");
    match &err {
        ConsumerError::Kernel(rows) => {
            assert!(
                rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                "expected undefined_variable, got {rows:?}"
            );
        }
        other => panic!("expected Kernel error, got {other:?}"),
    }
}
