// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Composition integration tests: Composition + Message, multiple (Prompt, vars) → Messages.

use garde::Validate;
use prompting_press::{Composition, Message, Prompt};
use serde::Serialize;

#[derive(Serialize, Validate)]
struct SimpleVars {
    #[garde(skip)]
    name: String,
}

fn system_prompt() -> Prompt {
    Prompt::from_yaml(
        r#"
name: system_intro
role: system
body: "You are a helpful assistant."
variables: {}
"#,
    )
    .unwrap()
}

fn user_prompt() -> Prompt {
    Prompt::from_yaml(
        r#"
name: user_greeting
role: user
body: "Hello {{ name }}!"
variables:
  name: { type: string, trusted: true }
"#,
    )
    .unwrap()
}

// ── empty composition ─────────────────────────────────────────────────────────

#[test]
fn empty_composition_resolves_to_empty_vec() {
    let comp = Composition::new();
    assert!(comp.is_empty());
    let messages = comp.resolve().expect("empty resolve must succeed");
    assert!(messages.is_empty());
}

// ── single-entry composition ──────────────────────────────────────────────────

#[test]
fn single_entry_composition_resolves_to_one_message() {
    let prompt = user_prompt();
    let vars = SimpleVars {
        name: "Ada".to_string(),
    };

    let mut comp = Composition::new();
    comp.append(&prompt, &vars, None)
        .expect("append must succeed");

    let messages = comp.resolve().expect("resolve must succeed");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "Hello Ada!");
    assert_eq!(messages[0].role, "user");
}

// ── multi-entry composition ───────────────────────────────────────────────────

#[test]
fn multi_entry_composition_preserves_append_order() {
    // system + user entries in order.
    let sys = system_prompt();
    let usr = user_prompt();

    #[derive(Serialize, Validate)]
    struct Empty {}

    let vars_empty = Empty {};
    let vars_name = SimpleVars {
        name: "Ada".to_string(),
    };

    let mut comp = Composition::new();
    comp.append(&sys, &vars_empty, None).expect("system append");
    comp.append(&usr, &vars_name, None).expect("user append");

    let messages = comp.resolve().expect("resolve");
    assert_eq!(messages.len(), 2);
    // Order must match append order.
    assert_eq!(messages[0].role, "system");
    assert_eq!(messages[0].text, "You are a helpful assistant.");
    assert_eq!(messages[1].role, "user");
    assert_eq!(messages[1].text, "Hello Ada!");
}

// ── Message shape ─────────────────────────────────────────────────────────────

#[test]
fn resolved_message_has_role_and_text_fields() {
    let prompt = user_prompt();
    let vars = SimpleVars {
        name: "Bob".to_string(),
    };

    let mut comp = Composition::new();
    comp.append(&prompt, &vars, None).unwrap();
    let messages = comp.resolve().unwrap();

    let msg: &Message = &messages[0];
    assert_eq!(msg.role, "user");
    assert_eq!(msg.text, "Hello Bob!");
}

// ── validation failure at append ──────────────────────────────────────────────

#[test]
fn append_validation_failure_does_not_store_entry() {
    #[derive(Serialize, Validate)]
    struct ValidatedVars {
        #[garde(length(min = 1))]
        name: String,
    }

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

    let bad_vars = ValidatedVars {
        name: String::new(), // fails garde length(min=1)
    };

    let mut comp = Composition::new();
    let err = comp.append(&prompt, &bad_vars, None);
    assert!(err.is_err(), "invalid vars must fail at append");
    // Composition is unchanged — the bad entry was not stored.
    assert_eq!(
        comp.len(),
        0,
        "composition must be empty after failed append"
    );
}

// ── named variant in composition ──────────────────────────────────────────────

#[test]
fn composition_with_named_variant_renders_variant_body() {
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

    let mut comp = Composition::new();
    comp.append(&prompt, &vars, Some("brief"))
        .expect("append with variant");
    let messages = comp.resolve().unwrap();

    assert_eq!(messages[0].text, "Hi Ada");
}

// ── len() ─────────────────────────────────────────────────────────────────────

#[test]
fn composition_len_reflects_appended_entries() {
    let prompt = user_prompt();
    let vars = SimpleVars {
        name: "Ada".to_string(),
    };

    let mut comp = Composition::new();
    assert_eq!(comp.len(), 0);
    comp.append(&prompt, &vars, None).unwrap();
    assert_eq!(comp.len(), 1);
    comp.append(&prompt, &vars, None).unwrap();
    assert_eq!(comp.len(), 2);
}
