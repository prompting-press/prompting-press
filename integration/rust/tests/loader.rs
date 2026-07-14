// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Loader integration tests:
//! - MemoryLoader hit/miss
//! - FileSystemLoader::with_base hit + traversal-guard rejection + read-cap
//! - Custom loader via closure impl
//! - PromptLoadError distinct from ConsumerError
//! - load returns raw text (compose with from_yaml, NOT fused)

use prompting_press::{
    error::code, ConsumerError, FileSystemLoader, MemoryLoader, Prompt, PromptLoadError,
    PromptLoader,
};
use std::collections::HashMap;

// ── MemoryLoader ──────────────────────────────────────────────────────────────

const GREET_YAML: &str = r#"
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
"#;

fn memory_loader() -> MemoryLoader {
    let mut map = HashMap::new();
    map.insert("greet".to_string(), GREET_YAML.to_string());
    MemoryLoader::new(map)
}

#[test]
fn memory_loader_hit_returns_raw_text() {
    let loader = memory_loader();
    let raw = loader.load("greet").expect("key present");
    assert!(raw.contains("name: greet"), "raw text contains prompt name");
}

#[test]
fn memory_loader_miss_returns_not_found_error() {
    let loader = memory_loader();
    let err = loader.load("missing").expect_err("missing key must error");
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "expected NotFound, got {err:?}"
    );
}

#[test]
fn memory_loader_not_found_code_is_load_not_found() {
    let loader = memory_loader();
    let err = loader.load("missing").unwrap_err();
    let field_err = err.to_field_error();
    assert_eq!(field_err.code, code::LOAD_NOT_FOUND);
}

#[test]
fn memory_loader_raw_text_composes_with_from_yaml() {
    // load returns raw text only; construction is a separate step (FR-005/FR-011).
    let loader = memory_loader();
    let raw = loader.load("greet").unwrap();
    let prompt = Prompt::from_yaml(&raw).expect("loaded text must construct");
    assert_eq!(prompt.name(), "greet");
}

// ── FileSystemLoader ──────────────────────────────────────────────────────────

// Each call returns a UNIQUE temp dir (pid + a process-wide atomic counter), so the
// filesystem tests — which cargo runs in parallel threads by default — never share a
// directory or race on `greet.yaml` (one test sets max_bytes=1, another reads the file;
// a shared dir made the hit test flaky on CI).
fn temp_dir_with_prompt() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ppress-integ-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("greet.yaml"), GREET_YAML).unwrap();
    dir
}

#[test]
fn fs_loader_with_base_hit_returns_raw_text() {
    let dir = temp_dir_with_prompt();
    let loader = FileSystemLoader::with_base(&dir).expect("base exists");
    let raw = loader.load("greet").expect("greet.yaml present");
    assert!(raw.contains("name: greet"));
}

#[test]
fn fs_loader_miss_returns_not_found() {
    let dir = temp_dir_with_prompt();
    let loader = FileSystemLoader::with_base(&dir).unwrap();
    let err = loader
        .load("nonexistent")
        .expect_err("missing file must error");
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "expected NotFound, got {err:?}"
    );
}

#[test]
fn fs_loader_traversal_guard_rejects_dotdot_key() {
    let dir = temp_dir_with_prompt();
    let loader = FileSystemLoader::with_base(&dir).unwrap();
    // A key with .. components attempts path traversal — must be rejected.
    let err = loader
        .load("../etc/passwd")
        .expect_err("traversal must be rejected");
    // The error is NotFound (not Io) — the traversal guard surfaces as not-found.
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "expected NotFound for traversal attempt, got {err:?}"
    );
}

#[test]
fn fs_loader_traversal_guard_rejects_absolute_key() {
    let dir = temp_dir_with_prompt();
    let loader = FileSystemLoader::with_base(&dir).unwrap();
    let err = loader
        .load("/etc/passwd")
        .expect_err("absolute key must be rejected");
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "expected NotFound for absolute key, got {err:?}"
    );
}

#[test]
fn fs_loader_read_cap_exceeded_returns_io_error() {
    let dir = temp_dir_with_prompt();
    // Set max_bytes to 1 so any real file exceeds the cap.
    let loader = FileSystemLoader::new(&dir, ".yaml", 1).expect("base exists");
    let err = loader.load("greet").expect_err("oversized file must error");
    assert!(
        matches!(err, PromptLoadError::Io { .. }),
        "expected Io error for exceeded cap, got {err:?}"
    );
}

#[test]
fn fs_loader_nonexistent_base_returns_not_found() {
    let result = FileSystemLoader::with_base("/nonexistent/path/that/does/not/exist/xyz");
    assert!(
        result.is_err(),
        "nonexistent base must fail at construction"
    );
}

// ── Custom loader via closure ─────────────────────────────────────────────────

#[test]
fn closure_loader_implements_prompt_loader_trait() {
    // A closure satisfying Fn(&str) -> Result<String, PromptLoadError> is a PromptLoader.
    let loader: &dyn PromptLoader = &|key: &str| -> Result<String, PromptLoadError> {
        if key == "greet" {
            Ok(GREET_YAML.to_string())
        } else {
            Err(PromptLoadError::NotFound {
                key: key.to_string(),
            })
        }
    };

    let raw = loader.load("greet").expect("closure hit");
    assert!(raw.contains("name: greet"));

    let err = loader.load("other").expect_err("closure miss");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

// ── PromptLoadError is distinct from ConsumerError ────────────────────────────

#[test]
fn prompt_load_error_is_distinct_from_consumer_error() {
    // PromptLoadError and ConsumerError are different types — the type system enforces this.
    // This test is a compile-time proof: it would not compile if they were the same type.
    let load_err = PromptLoadError::NotFound {
        key: "k".to_string(),
    };
    let _consumer_err: ConsumerError = ConsumerError::Load("not a loader error".to_string());

    // They are different: load_err is not a ConsumerError.
    // We verify by asserting the field_error conversion works and has the expected code.
    let field_err = load_err.to_field_error();
    assert_eq!(field_err.code, code::LOAD_NOT_FOUND);
}

#[test]
fn prompt_load_error_io_has_load_io_code() {
    let err = PromptLoadError::Io {
        key: "k".to_string(),
        detail: "disk full".to_string(),
    };
    let field_err = err.to_field_error();
    assert_eq!(field_err.code, code::LOAD_IO);
}
