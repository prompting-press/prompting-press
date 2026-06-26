//! US4 composition (spec 003, T021; FR-012/FR-013).
//!
//! Pins the multi-message composition contract: an explicit ORDERED sequence of
//! `(prompt-name, vars, variant)` entries that resolves to an ordered `Vec<Message>` where
//! each `Message { role, text }` is the named prompt rendered with its own validated vars,
//! in append order. No fluent `.chain()` API (FR-013).
//!
//! - **V4.1** N entries over registered prompts, each with its own valid vars → `resolve`
//!   yields exactly N `Message`s in APPEND ORDER, each `text` = that prompt rendered with its
//!   vars and each `role` = that prompt definition's role (SC-008).
//! - **V4.2** one entry's vars violate a validator → the failing operation (`append`)
//!   returns `Err(ConsumerError::Validation(..))` naming the field, and NO partial `Vec` is
//!   returned as success (US4 scenario 3).
//! - **V4.3** a fragment rendered with its own vars, its `.text` passed as a declared
//!   variable into a parent prompt → composition-by-value works with NO template include
//!   (US4 scenario 4).
//! - **V4.4** an empty `Composition` → `resolve` returns `Ok(vec![])` (edge case F7).

use garde::Validate;
use prompting_press::error::code;
use prompting_press::{render, Composition, ConsumerError, Message, Registry};
use prompting_press_core::GuardConfig;
use serde::Serialize;

/// Greeting vars (system prompt) — `name`, length-bounded.
#[derive(Debug, Serialize, Validate)]
struct GreetVars {
    #[garde(length(min = 1, max = 20))]
    name: String,
}

/// Question vars (user prompt) — `topic`, length-bounded.
#[derive(Debug, Serialize, Validate)]
struct AskVars {
    #[garde(length(min = 1, max = 50))]
    topic: String,
}

/// Answer vars (assistant prompt) — `answer`, length-bounded.
#[derive(Debug, Serialize, Validate)]
struct AnswerVars {
    #[garde(length(min = 1, max = 50))]
    answer: String,
}

/// A registry with one prompt per role, so `Message.role` can be asserted distinctly.
fn registry() -> Registry {
    let mut reg = Registry::new();
    reg.insert(
        serde_json::from_value(serde_json::json!({
            "name": "greet",
            "role": "system",
            "body": "You are talking to {{ name }}.",
            "variables": { "name": { "type": "string", "provenance": "trusted" } }
        }))
        .expect("valid greet def"),
    );
    reg.insert(
        serde_json::from_value(serde_json::json!({
            "name": "ask",
            "role": "user",
            "body": "Tell me about {{ topic }}.",
            "variables": { "topic": { "type": "string", "provenance": "trusted" } }
        }))
        .expect("valid ask def"),
    );
    reg.insert(
        serde_json::from_value(serde_json::json!({
            "name": "answer",
            "role": "assistant",
            "body": "Here is what I know: {{ answer }}",
            "variables": { "answer": { "type": "string", "provenance": "trusted" } }
        }))
        .expect("valid answer def"),
    );
    reg
}

/// V4.1 — N ordered entries resolve to exactly N messages, in append order, each rendered
/// with its own vars and tagged with its prompt's role (SC-008).
#[test]
fn ordered_composition_resolves_n_to_n() {
    let reg = registry();

    let mut comp = Composition::new();
    comp.append(
        "greet",
        &GreetVars {
            name: "Ada".to_string(),
        },
        None,
    )
    .expect("greet vars valid");
    comp.append(
        "ask",
        &AskVars {
            topic: "rust".to_string(),
        },
        None,
    )
    .expect("ask vars valid");
    comp.append(
        "answer",
        &AnswerVars {
            answer: "it is fast".to_string(),
        },
        None,
    )
    .expect("answer vars valid");

    let messages = comp.resolve(&reg).expect("all entries valid → resolves");

    // Exactly N (3) messages, in APPEND ORDER (SC-008).
    assert_eq!(messages.len(), 3, "N entries → exactly N messages");

    // Each message's role matches its prompt definition's role, and text = that prompt
    // rendered with its own vars (cross-checked against a direct `render`).
    let expected: [(&str, &str); 3] = [
        ("system", "You are talking to Ada."),
        ("user", "Tell me about rust."),
        ("assistant", "Here is what I know: it is fast"),
    ];
    for (msg, (role, text)) in messages.iter().zip(expected.iter()) {
        assert_eq!(&msg.role, role, "role matches the prompt def's role");
        assert_eq!(&msg.text, text, "text = prompt rendered with its own vars");
    }

    // Direct cross-check: the first message must equal a standalone render of `greet`.
    let direct = render(
        &reg,
        "greet",
        &GreetVars {
            name: "Ada".to_string(),
        },
        None,
        &GuardConfig::default(),
    )
    .expect("direct render");
    assert_eq!(messages[0].text, direct.text);
    assert_eq!(messages[0].role, direct_role(&reg, "greet"));
}

/// Helper: the stringified role of a registered prompt, for cross-checking `Message.role`.
fn direct_role(reg: &Registry, name: &str) -> String {
    reg.get(name).expect("present").role.to_string()
}

/// V4.2 — one entry's vars violate a validator → the failing `append` returns
/// `Err(ConsumerError::Validation(..))` naming the field; NO partial `Vec` is produced.
#[test]
fn invalid_entry_vars_error_no_partial_success() {
    let reg = registry();

    let mut comp = Composition::new();
    // First entry is valid and appends cleanly.
    comp.append(
        "greet",
        &GreetVars {
            name: "Ada".to_string(),
        },
        None,
    )
    .expect("first entry valid");

    // Second entry's vars violate `length(min = 1)` → append fails fast with a normalized
    // Validation error identifying the field. The composition never reaches a resolvable
    // partial state that could be returned as success.
    let err = comp
        .append(
            "ask",
            &AskVars {
                topic: String::new(),
            },
            None,
        )
        .expect_err("empty topic must be rejected at append");

    match err {
        ConsumerError::Validation(rows) => {
            assert_eq!(rows.len(), 1, "exactly the one offending field");
            assert_eq!(rows[0].field, "topic", "the failing field is named");
            assert_eq!(rows[0].code, code::VALIDATION);
        }
        other => panic!("expected ConsumerError::Validation, got {other:?}"),
    }

    // No-partial-as-success: the failing entry is NOT in the composition, so a subsequent
    // resolve yields only the entries that appended successfully (here: just `greet`) — the
    // invalid entry was never accepted. Critically, the failed append did not silently land
    // a partial/empty render.
    let messages = comp.resolve(&reg).expect("only the valid entry remains");
    assert_eq!(messages.len(), 1, "only the successfully-appended entry");
    assert_eq!(messages[0].role, "system");
    assert_eq!(messages[0].text, "You are talking to Ada.");
}

/// V4.3 — fragment-by-composition: render a fragment with its own vars, then pass its
/// `.text` as a declared variable's value into a parent prompt. No template include.
#[test]
fn fragment_by_value_into_parent() {
    let mut reg = Registry::new();
    // The fragment prompt: renders a standalone snippet.
    reg.insert(
        serde_json::from_value(serde_json::json!({
            "name": "fragment",
            "role": "user",
            "body": "the {{ adjective }} fox",
            "variables": { "adjective": { "type": "string", "provenance": "trusted" } }
        }))
        .expect("valid fragment def"),
    );
    // The parent prompt: takes a `fragment` variable (a string), embeds it by value.
    reg.insert(
        serde_json::from_value(serde_json::json!({
            "name": "parent",
            "role": "user",
            "body": "Story: {{ fragment }} jumped.",
            "variables": { "fragment": { "type": "string", "provenance": "trusted" } }
        }))
        .expect("valid parent def"),
    );

    #[derive(Debug, Serialize, Validate)]
    struct FragVars {
        #[garde(length(min = 1))]
        adjective: String,
    }
    #[derive(Debug, Serialize, Validate)]
    struct ParentVars {
        #[garde(length(min = 1))]
        fragment: String,
    }

    // 1. Render the fragment with its OWN vars.
    let frag = render(
        &reg,
        "fragment",
        &FragVars {
            adjective: "quick".to_string(),
        },
        None,
        &GuardConfig::default(),
    )
    .expect("fragment renders");
    assert_eq!(frag.text, "the quick fox");

    // 2. Pass the fragment's TEXT as a declared variable's value into the parent — no
    //    template include, pure composition-by-value.
    let parent = render(
        &reg,
        "parent",
        &ParentVars {
            fragment: frag.text.clone(),
        },
        None,
        &GuardConfig::default(),
    )
    .expect("parent renders with the fragment value");
    assert_eq!(parent.text, "Story: the quick fox jumped.");

    // The same pattern is expressible through a Composition that surfaces both messages in
    // order (the parent already carries the fragment by value).
    let mut comp = Composition::new();
    comp.append(
        "fragment",
        &FragVars {
            adjective: "quick".to_string(),
        },
        None,
    )
    .expect("fragment vars valid");
    comp.append(
        "parent",
        &ParentVars {
            fragment: frag.text.clone(),
        },
        None,
    )
    .expect("parent vars valid");
    let messages = comp.resolve(&reg).expect("composition resolves");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].text, "the quick fox");
    assert_eq!(messages[1].text, "Story: the quick fox jumped.");
}

/// V4.4 — an empty `Composition` resolves to `Ok(vec![])` (edge case F7), never a panic.
#[test]
fn empty_composition_resolves_to_empty_vec() {
    let reg = registry();
    let comp = Composition::new();
    let messages: Vec<Message> = comp.resolve(&reg).expect("empty composition is a pass");
    assert!(messages.is_empty(), "empty composition → empty Vec");
}
