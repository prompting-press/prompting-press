//! US1 happy-path render contract (spec 003, T007).
//!
//! Exercises the consumer's `render` wrapper end to end with a real garde-validated,
//! serde-serializable Vars struct:
//!
//! - **V1.1** valid vars → a [`RenderResult`] with non-empty `text` and 64-hex
//!   `template_hash` / `render_hash` (the kernel's provenance, surfaced unchanged).
//! - **V1.5** the same prompt + same valid vars rendered twice → byte-identical `text`
//!   and equal hashes (the kernel's determinism, surfaced through the consumer; SC-001).
//! - **F5 guard plumbing** the consumer PLUMBS a [`GuardConfig`] through to the kernel and
//!   surfaces the resulting `guard` field: `enabled: true` (over a prompt declaring an
//!   untrusted field) → `guard.is_some()`; `GuardConfig::default()` (disabled) →
//!   `guard.is_none()`. Guard *wording / expansion* is the kernel's concern (spec 002) and
//!   is NOT re-tested here (FR-009 / F5).

use garde::Validate;
use prompting_press::error::code;
use prompting_press::{get_source, render, ConsumerError, Registry};
use prompting_press_core::GuardConfig;
use serde::Serialize;

/// A non-negative-bounded custom validator (garde custom-validator signature is
/// `fn(&T, &Ctx) -> garde::Result`). `n` must be at most 100.
fn at_most_100(value: &u32, _ctx: &()) -> garde::Result {
    if *value <= 100 {
        Ok(())
    } else {
        Err(garde::Error::new("n must be at most 100"))
    }
}

/// Typed Vars deriving BOTH `serde::Serialize` and `garde::Validate` (Context = `()`,
/// so plain `validate()` works). One `#[garde(custom)]` field and one `#[garde(length)]`
/// field. Field names match the prompt's declared `variables` (`name`, `n`).
#[derive(Debug, Serialize, Validate)]
struct Vars {
    /// 1..=20 chars — a built-in `length` validator.
    #[garde(length(min = 1, max = 20))]
    name: String,
    /// At most 100 — a custom validator.
    #[garde(custom(at_most_100))]
    n: u32,
}

/// Build a registry holding one prompt whose root body references `name` and `n`, with
/// matching `variables` entries. `name` is declared `untrusted` so the opt-in guard has a
/// field to name (the kernel's `build_guard_text` returns `None` over an empty
/// untrusted∪external union, so the guard-plumb assertion needs a tagged field).
fn registry_with_greeting() -> Registry {
    let mut reg = Registry::new();
    let def = serde_json::from_value(serde_json::json!({
        "name": "greeting",
        "role": "user",
        "body": "Hi {{ name }}, n={{ n }}",
        "variables": {
            "name": { "type": "string",  "provenance": "untrusted" },
            "n":    { "type": "integer", "provenance": "trusted" }
        }
    }))
    .expect("valid prompt definition");
    reg.insert(def);
    reg
}

/// V1.1 — valid vars produce a `RenderResult` with non-empty text and 64-hex hashes.
#[test]
fn valid_vars_render_with_provenance() {
    let reg = registry_with_greeting();
    let vars = Vars {
        name: "Ada".to_string(),
        n: 7,
    };

    let result = render(&reg, "greeting", &vars, None, &GuardConfig::default())
        .expect("valid vars must render");

    assert_eq!(result.name, "greeting");
    assert_eq!(result.variant, "default");
    assert_eq!(
        result.text, "Hi Ada, n=7",
        "rendered text must interpolate both vars"
    );
    assert!(!result.text.is_empty(), "render text must be non-empty");

    // Provenance hashes are lowercase 64-hex (SHA256), surfaced unchanged from the kernel.
    assert!(
        is_sha256_hex(&result.template_hash),
        "template_hash must be 64-hex"
    );
    assert!(
        is_sha256_hex(&result.render_hash),
        "render_hash must be 64-hex"
    );
}

/// V1.5 — the same prompt + same vars rendered twice is byte-identical (kernel determinism).
#[test]
fn render_is_deterministic() {
    let reg = registry_with_greeting();
    let vars = Vars {
        name: "Grace".to_string(),
        n: 42,
    };

    let first = render(&reg, "greeting", &vars, None, &GuardConfig::default()).expect("render 1");
    let second = render(&reg, "greeting", &vars, None, &GuardConfig::default()).expect("render 2");

    assert_eq!(
        first.text, second.text,
        "text must be byte-identical across renders"
    );
    assert_eq!(first.template_hash, second.template_hash);
    assert_eq!(first.render_hash, second.render_hash);
}

/// F5 — the consumer PLUMBS `GuardConfig` through to the kernel and surfaces the `guard`
/// field. Enabled (over a prompt declaring an untrusted field) → `Some`; default
/// (disabled) → `None`. We assert ONLY plumbing (some/none), not the guard wording — that
/// is the kernel's contract (spec 002).
#[test]
fn guard_config_is_plumbed_through() {
    let reg = registry_with_greeting();
    let vars = Vars {
        name: "Ada".to_string(),
        n: 1,
    };

    // Disabled (default) → no guard field surfaced.
    let disabled = render(&reg, "greeting", &vars, None, &GuardConfig::default())
        .expect("render with disabled guard");
    assert!(
        disabled.guard.is_none(),
        "disabled GuardConfig must surface guard = None"
    );

    // Enabled → guard field surfaced (the prompt declares `name` as untrusted, so the
    // kernel's guard text has a field to name).
    let enabled_cfg = GuardConfig {
        enabled: true,
        template: None,
    };
    let enabled =
        render(&reg, "greeting", &vars, None, &enabled_cfg).expect("render with enabled guard");
    assert!(
        enabled.guard.is_some(),
        "enabled GuardConfig must surface guard = Some (plumbed through to the kernel)"
    );

    // Plumbing must be purely additive: the rendered body is unchanged by the guard.
    assert_eq!(
        disabled.text, enabled.text,
        "guard must not alter rendered text"
    );
}

/// An unknown prompt name resolves to a structured `UnknownPrompt`, never a panic
/// (FR-008a) — the registry-miss path of the render contract.
#[test]
fn unknown_prompt_is_structured_error() {
    let reg = registry_with_greeting();
    let vars = Vars {
        name: "Ada".to_string(),
        n: 1,
    };
    let err = render(&reg, "does-not-exist", &vars, None, &GuardConfig::default())
        .expect_err("absent name must error");
    match err {
        ConsumerError::UnknownPrompt(name) => assert_eq!(name, "does-not-exist"),
        other => panic!("expected UnknownPrompt, got {other:?}"),
    }
}

/// A multi-variant registry: a root body plus a named `concise` variant, each referencing
/// only the declared `name` variable. Used by the named-variant render + `get_source` tests.
fn registry_with_variants() -> Registry {
    let mut reg = Registry::new();
    let def = serde_json::from_value(serde_json::json!({
        "name": "greet",
        "role": "user",
        "body": "Hello there, {{ name }}!",
        "variants": {
            "concise": { "body": "Hi {{ name }}" }
        },
        "variables": {
            "name": { "type": "string", "provenance": "trusted" }
        }
    }))
    .expect("valid prompt definition");
    reg.insert(def);
    reg
}

/// A single-field Vars struct (`name`) for the multi-variant prompt.
#[derive(Debug, Serialize, Validate)]
struct NameVars {
    #[garde(length(min = 1, max = 50))]
    name: String,
}

/// TS-1(a) — `get_source(reg, name, None)` returns the root body's unrendered template
/// source (the exact string the kernel hashes into `template_hash`).
#[test]
fn get_source_returns_root_body() {
    let reg = registry_with_variants();
    let src = get_source(&reg, "greet", None).expect("root source must resolve");
    assert_eq!(
        src, "Hello there, {{ name }}!",
        "must return the root body source"
    );
}

/// TS-1(a, variant) — `get_source` with a declared variant returns that arm's source.
#[test]
fn get_source_returns_named_variant_body() {
    let reg = registry_with_variants();
    let src = get_source(&reg, "greet", Some("concise")).expect("variant source must resolve");
    assert_eq!(
        src, "Hi {{ name }}",
        "must return the named variant's body source"
    );
}

/// TS-1(b) — `get_source` for an absent prompt name resolves to `UnknownPrompt`, not a panic.
#[test]
fn get_source_unknown_prompt_is_structured_error() {
    let reg = registry_with_variants();
    let err = get_source(&reg, "does-not-exist", None).expect_err("absent name must error");
    match err {
        ConsumerError::UnknownPrompt(name) => assert_eq!(name, "does-not-exist"),
        other => panic!("expected UnknownPrompt, got {other:?}"),
    }
}

/// TS-1(c) — `get_source` for an unknown variant resolves to a normalized `Kernel` error
/// carrying `code::UNKNOWN_VARIANT` (the kernel rejects the lookup; the consumer normalizes).
#[test]
fn get_source_unknown_variant_is_kernel_error() {
    let reg = registry_with_variants();
    let err = get_source(&reg, "greet", Some("nope")).expect_err("unknown variant must error");
    match err {
        ConsumerError::Kernel(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(
                rows[0].code,
                code::UNKNOWN_VARIANT,
                "must map to unknown_variant"
            );
        }
        other => panic!("expected ConsumerError::Kernel, got {other:?}"),
    }
}

/// TS-2(a) — rendering a declared named variant selects that arm: the `RenderResult.variant`
/// is the variant name and the `text` comes from the variant's body.
#[test]
fn named_variant_render_selects_that_arm() {
    let reg = registry_with_variants();
    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let result = render(
        &reg,
        "greet",
        &vars,
        Some("concise"),
        &GuardConfig::default(),
    )
    .expect("named variant must render");

    assert_eq!(result.name, "greet");
    assert_eq!(
        result.variant, "concise",
        "the selected variant must be surfaced"
    );
    assert_eq!(
        result.text, "Hi Ada",
        "text must come from the variant's body"
    );
}

/// TS-2(b) — rendering an unknown variant resolves to a normalized `Kernel` error carrying
/// `code::UNKNOWN_VARIANT`.
#[test]
fn render_unknown_variant_is_kernel_error() {
    let reg = registry_with_variants();
    let vars = NameVars {
        name: "Ada".to_string(),
    };
    let err = render(&reg, "greet", &vars, Some("nope"), &GuardConfig::default())
        .expect_err("unknown variant must error");
    match err {
        ConsumerError::Kernel(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(
                rows[0].code,
                code::UNKNOWN_VARIANT,
                "must map to unknown_variant"
            );
        }
        other => panic!("expected ConsumerError::Kernel, got {other:?}"),
    }
}

/// Lowercase 64-char hex (a SHA256 digest), with no allocation.
fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
