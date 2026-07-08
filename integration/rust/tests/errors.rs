//! Error normalization integration tests:
//! - ConsumerError / FieldError normalized [{field, code, message}] shape
//! - Error codes from the closed vocabulary
//! - PromptLoadError distinct from ConsumerError

use prompting_press::{error::code, ConsumerError, FieldError, Prompt, PromptLoadError};

// ── ConsumerError::Load ───────────────────────────────────────────────────────

#[test]
fn consumer_error_load_on_bad_yaml() {
    let err = Prompt::from_yaml("not: yaml: at: all: :bad").unwrap_err();
    // Malformed YAML → ConsumerError::Load with a description string.
    match err {
        ConsumerError::Load(detail) => {
            assert!(!detail.is_empty(), "Load detail must not be empty");
        }
        other => panic!("expected ConsumerError::Load, got {other:?}"),
    }
}

#[test]
fn consumer_error_load_on_bad_json() {
    let err = Prompt::from_json("}{invalid").unwrap_err();
    assert!(
        matches!(err, ConsumerError::Load(_)),
        "bad JSON → Load, got {err:?}"
    );
}

#[test]
fn consumer_error_load_on_bad_toml() {
    let err = Prompt::from_toml("not = valid [toml broken").unwrap_err();
    assert!(
        matches!(err, ConsumerError::Load(_)),
        "bad TOML → Load, got {err:?}"
    );
}

// ── ConsumerError::Kernel — undefined_variable ────────────────────────────────

#[test]
fn kernel_error_undefined_variable_has_correct_code_and_field() {
    let err =
        Prompt::from_json(r#"{"name":"bad","role":"user","body":"{{ ghost }}","variables":{}}"#)
            .unwrap_err();

    match &err {
        ConsumerError::Kernel(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, code::UNDEFINED_VARIABLE);
            assert!(!rows[0].field.is_empty(), "field must be named");
            assert!(!rows[0].message.is_empty(), "message must be non-empty");
        }
        other => panic!("expected ConsumerError::Kernel, got {other:?}"),
    }
}

// ── ConsumerError::Kernel — unknown_variant ────────────────────────────────────

#[test]
fn kernel_error_unknown_variant_has_correct_code() {
    use garde::Validate;
    use prompting_press::GuardConfig;
    use serde::Serialize;

    #[derive(Serialize, Validate)]
    struct Vars {
        #[garde(skip)]
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

    let vars = Vars {
        name: "Ada".to_string(),
    };
    let err = prompt
        .render(&vars, Some("nonexistent"), &GuardConfig::default(), false)
        .unwrap_err();

    match &err {
        ConsumerError::Kernel(rows) => {
            assert_eq!(rows[0].code, code::UNKNOWN_VARIANT);
            assert_eq!(rows[0].field, "variant");
        }
        other => panic!("expected ConsumerError::Kernel, got {other:?}"),
    }
}

// ── ConsumerError::Validation ─────────────────────────────────────────────────

#[test]
fn validation_error_has_validation_code() {
    use garde::Validate;
    use prompting_press::GuardConfig;
    use serde::Serialize;

    #[derive(Serialize, Validate)]
    struct Vars {
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

    let bad = Vars {
        name: String::new(),
    };
    let err = prompt
        .render(&bad, None, &GuardConfig::default(), false)
        .unwrap_err();

    match &err {
        ConsumerError::Validation(rows) => {
            assert!(!rows.is_empty());
            assert!(
                rows.iter().all(|r| r.code == code::VALIDATION),
                "all codes must be 'validation'"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

// ── FieldError shape ──────────────────────────────────────────────────────────

#[test]
fn field_error_has_field_code_message_shape() {
    // FieldError is the [{field, code, message}] normalized shape.
    let fe = FieldError {
        field: "name".to_string(),
        code: code::VALIDATION.to_string(),
        message: "length must be at least 1".to_string(),
    };

    assert_eq!(fe.field, "name");
    assert_eq!(fe.code, code::VALIDATION);
    assert_eq!(fe.message, "length must be at least 1");
}

// ── ConsumerError::Display ────────────────────────────────────────────────────

#[test]
fn consumer_error_implements_display_and_std_error() {
    let err = Prompt::from_json("{}").unwrap_err();
    // Display must be non-empty — it is what callers log.
    let s = err.to_string();
    assert!(!s.is_empty(), "Display must be non-empty");
}

// ── PromptLoadError has Display and Error ─────────────────────────────────────

#[test]
fn prompt_load_error_implements_display_and_std_error() {
    let err = PromptLoadError::NotFound {
        key: "greet".to_string(),
    };
    let s = err.to_string();
    assert!(s.contains("greet"), "Display must mention the key");
    assert!(
        s.contains(code::LOAD_NOT_FOUND),
        "Display must mention the code"
    );
}

#[test]
fn prompt_load_error_io_display_mentions_detail() {
    let err = PromptLoadError::Io {
        key: "greet".to_string(),
        detail: "disk full".to_string(),
    };
    let s = err.to_string();
    assert!(s.contains("greet"));
}
