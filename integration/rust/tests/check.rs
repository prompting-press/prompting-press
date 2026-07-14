// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Agreement check integration tests: prompt.check() → CheckReport findings.

use prompting_press::{CheckReport, FindingKind, Prompt};

// ── passing check ─────────────────────────────────────────────────────────────

#[test]
fn check_passes_for_trusted_only_variables() {
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

    let report = prompt.check();
    assert!(report.passed(), "trusted-only prompt must pass check");
    assert!(report.is_empty(), "findings must be empty");
}

#[test]
fn check_passes_for_no_variables() {
    let prompt = Prompt::from_yaml(
        r#"
name: static_prompt
role: system
body: "You are a helpful assistant."
variables: {}
"#,
    )
    .unwrap();

    let report: CheckReport = prompt.check();
    assert!(report.passed());
}

// ── UntrustedWithoutGuard finding ────────────────────────────────────────────

#[test]
fn check_finds_untrusted_variable_without_guard() {
    let prompt = Prompt::from_yaml(
        r#"
name: unguarded
role: user
body: "Process: {{ payload }}"
variables:
  payload: { type: string, trusted: false }
"#,
    )
    .unwrap();

    let report = prompt.check();
    assert!(
        !report.passed(),
        "untrusted without guard must produce finding"
    );
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].prompt, "unguarded");
    assert!(matches!(
        &report.findings[0].kind,
        FindingKind::UntrustedWithoutGuard { field } if field == "payload"
    ));
}

#[test]
fn check_passes_for_untrusted_variable_with_guard_in_metadata() {
    let prompt = Prompt::from_yaml(
        r#"
name: guarded
role: user
body: "Process: {{ payload }}"
variables:
  payload: { type: string, trusted: false }
metadata:
  guard: { enabled: true }
"#,
    )
    .unwrap();

    let report = prompt.check();
    assert!(report.passed(), "guard present → check must pass");
}

// ── multiple untrusted variables ──────────────────────────────────────────────

#[test]
fn check_emits_one_finding_per_untrusted_variable_without_guard() {
    let prompt = Prompt::from_yaml(
        r#"
name: multi_untrusted
role: user
body: "{{ a }} {{ b }}"
variables:
  a: { type: string, trusted: false }
  b: { type: string, trusted: false }
"#,
    )
    .unwrap();

    let report = prompt.check();
    assert!(!report.passed());
    assert_eq!(report.findings.len(), 2, "one finding per untrusted field");
    let fields: std::collections::BTreeSet<&str> = report
        .findings
        .iter()
        .map(|f| {
            let FindingKind::UntrustedWithoutGuard { field } = &f.kind;
            field.as_str()
        })
        .collect();
    assert!(fields.contains("a"), "finding for 'a'");
    assert!(fields.contains("b"), "finding for 'b'");
}

// ── check is pure ─────────────────────────────────────────────────────────────

#[test]
fn check_is_idempotent_and_pure() {
    let prompt = Prompt::from_yaml(
        r#"
name: unguarded
role: user
body: "{{ payload }}"
variables:
  payload: { type: string, trusted: false }
"#,
    )
    .unwrap();

    let r1 = prompt.check();
    let r2 = prompt.check();
    assert_eq!(r1, r2, "check must be idempotent");
    // Prompt itself is unchanged (pure).
    assert_eq!(prompt.name(), "unguarded");
}

// ── finding detail is non-empty ───────────────────────────────────────────────

#[test]
fn check_finding_detail_is_non_empty_and_names_field() {
    let prompt = Prompt::from_yaml(
        r#"
name: unguarded
role: user
body: "{{ user_data }}"
variables:
  user_data: { type: string, trusted: false }
"#,
    )
    .unwrap();

    let report = prompt.check();
    let finding = &report.findings[0];
    assert!(
        !finding.detail.is_empty(),
        "finding detail must not be empty"
    );
    assert!(
        finding.detail.contains("user_data"),
        "detail must name the offending field"
    );
}
