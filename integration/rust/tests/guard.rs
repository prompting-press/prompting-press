// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Guard integration tests: GuardConfig on/off, untrusted delimiting,
//! render_hash differs when guard is on vs off.

use garde::Validate;
use prompting_press::{GuardConfig, Prompt};
use serde::Serialize;

#[derive(Serialize, Validate)]
struct UserInputVars {
    #[garde(skip)]
    user_input: String,
}

const UNTRUSTED_PROMPT_YAML: &str = r#"
name: guarded
role: user
body: "Process: {{ user_input }}"
variables:
  user_input: { type: string, trusted: false }
metadata:
  guard: { enabled: true }
"#;

// ── guard off (default) ───────────────────────────────────────────────────────

#[test]
fn guard_off_body_is_plain_rendered_text() {
    let prompt = Prompt::from_yaml(UNTRUSTED_PROMPT_YAML).unwrap();
    let vars = UserInputVars {
        user_input: "hello".to_string(),
    };

    let result = prompt
        .render(&vars, None, &GuardConfig::default(), false)
        .expect("render with guard off must succeed");

    // GuardConfig::default() has enabled=false — plain render, no delimiter tags.
    assert_eq!(result.text, "Process: hello");
}

// ── guard on ─────────────────────────────────────────────────────────────────

#[test]
fn guard_on_body_contains_untrusted_delimiter_tags() {
    let prompt = Prompt::from_yaml(UNTRUSTED_PROMPT_YAML).unwrap();
    let vars = UserInputVars {
        user_input: "hello".to_string(),
    };

    let guard_on = GuardConfig {
        enabled: true,
        advisory: None,
    };
    let result = prompt
        .render(&vars, None, &guard_on, false)
        .expect("render with guard on must succeed");

    // The untrusted value must be wrapped in <untrusted>…</untrusted> markers.
    assert!(
        result.text.contains("<untrusted>"),
        "guard-on body must contain <untrusted> tag, got: {:?}",
        result.text
    );
    assert!(
        result.text.contains("</untrusted>"),
        "guard-on body must contain </untrusted> tag, got: {:?}",
        result.text
    );
    assert!(
        result.text.contains("hello"),
        "value content must still be present, got: {:?}",
        result.text
    );
}

// ── render_hash differs guard on vs off ──────────────────────────────────────

#[test]
fn render_hash_differs_guard_on_vs_off() {
    let prompt = Prompt::from_yaml(UNTRUSTED_PROMPT_YAML).unwrap();
    let vars = UserInputVars {
        user_input: "hello".to_string(),
    };

    let guard_off = GuardConfig::default();
    let guard_on = GuardConfig {
        enabled: true,
        advisory: None,
    };

    let r_off = prompt
        .render(&vars, None, &guard_off, false)
        .expect("guard off render");
    let r_on = prompt
        .render(&vars, None, &guard_on, false)
        .expect("guard on render");

    // render_hash is SHA256(rendered output); body differs → hashes differ.
    assert_ne!(
        r_off.render_hash, r_on.render_hash,
        "render_hash must differ when guard changes the body"
    );
    // template_hash is over the source text, which is unaffected by guard mode.
    assert_eq!(
        r_off.template_hash, r_on.template_hash,
        "template_hash must be identical (template source is unchanged)"
    );
}

// ── entity-escaping inside untrusted span ─────────────────────────────────────

#[test]
fn guard_on_entity_escapes_angle_brackets_in_value() {
    let prompt = Prompt::from_yaml(UNTRUSTED_PROMPT_YAML).unwrap();
    let vars = UserInputVars {
        // Angle brackets in the value must be entity-escaped so they cannot break the marker tags.
        user_input: "<script>alert(1)</script>".to_string(),
    };

    let guard_on = GuardConfig {
        enabled: true,
        advisory: None,
    };
    let result = prompt
        .render(&vars, None, &guard_on, false)
        .expect("render with special chars must succeed");

    // The outer <untrusted> tag is present.
    assert!(
        result.text.contains("<untrusted>"),
        "guard markers must be present, got: {:?}",
        result.text
    );
    // The raw '<script>' must NOT appear unescaped inside the rendered body —
    // it is entity-escaped per the spec-015 contract.
    assert!(
        !result.text.contains("<script>"),
        "raw < must be entity-escaped inside the guard span, got: {:?}",
        result.text
    );
}

// ── guard on trusted-only prompt ─────────────────────────────────────────────

#[test]
fn guard_on_trusted_variable_produces_no_delimiter_tags() {
    // A prompt with only trusted variables — guard-on should not wrap trusted fields.
    let prompt = Prompt::from_yaml(
        r#"
name: trusted_only
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
"#,
    )
    .unwrap();

    #[derive(Serialize, Validate)]
    struct Vars {
        #[garde(skip)]
        name: String,
    }

    let vars = Vars {
        name: "Ada".to_string(),
    };
    let guard_on = GuardConfig {
        enabled: true,
        advisory: None,
    };
    let result = prompt
        .render(&vars, None, &guard_on, false)
        .expect("trusted-only render with guard must succeed");

    // Trusted variables are not delimited.
    assert!(
        !result.text.contains("<untrusted>"),
        "trusted-only prompt must have no delimiter tags, got: {:?}",
        result.text
    );
    assert_eq!(result.text, "Hi Ada");
}
