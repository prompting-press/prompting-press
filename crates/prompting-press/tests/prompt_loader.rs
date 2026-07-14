// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Spec 019 — `PromptLoader` integration tests (T006).
//!
//! Covers:
//! - `FileSystemLoader`: hit + miss (`load_not_found`)
//! - Traversal guard: `../secret`, absolute, NUL, backslash, escaping symlink,
//!   key="", key=".", empty-suffix, intermediate "." (SC-008)
//! - Read cap: exceed `max_bytes` → `load_io` (SC-009)
//! - `MemoryLoader`: hit + miss
//! - Closure-as-loader (blanket impl)
//! - Load error is DISTINCT from parse error (SC-010)
//! - Error messages are scrubbed (no path/secret leak)

use std::collections::HashMap;

use prompting_press::error::code;
use prompting_press::loader::{FileSystemLoader, MemoryLoader, PromptLoader};
use prompting_press::{ConsumerError, Prompt, PromptLoadError};

// ─── minimal valid YAML fixture ─────────────────────────────────────────────

const VALID_YAML: &str = "name: test\nrole: user\nbody: \"Hello {{ name }}\"\nvariables:\n  name: { type: string, trusted: true }\n";

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Create a temp directory with a single prompt file `{key}.yaml` containing `contents`.
/// Returns the temp dir (auto-deleted on drop) and its path.
fn temp_dir_with_file(key: &str, contents: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(format!("{key}.yaml"));
    std::fs::write(&path, contents).expect("write fixture");
    let base = dir.path().to_path_buf();
    (dir, base)
}

// ─── FileSystemLoader: happy path ────────────────────────────────────────────

#[test]
fn filesystem_loader_hit() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let raw = loader.load("greet").expect("hit");
    assert!(
        raw.contains("name: test"),
        "raw text contains the YAML content"
    );
}

#[test]
fn filesystem_loader_miss_returns_not_found() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("missing_key").expect_err("miss");
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "missing key → NotFound, got {err:?}"
    );
    let row = err.to_field_error();
    assert_eq!(row.code, code::LOAD_NOT_FOUND);
}

// ─── FileSystemLoader: traversal guard (SC-008) ──────────────────────────────

#[test]
fn traversal_guard_parent_dir_component() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("../secret").expect_err("traversal rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
    // Scrubbing check: the error message must carry the logical key but not the base path.
    let row = err.to_field_error();
    assert!(
        row.message.contains("../secret"),
        "message contains key: {}",
        row.message
    );
    assert!(
        !row.message.contains(&base.display().to_string()),
        "message must not leak base path: {}",
        row.message
    );
}

#[test]
fn traversal_guard_absolute_key() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader
        .load("/etc/passwd")
        .expect_err("absolute key rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[test]
fn traversal_guard_nul_byte() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("foo\0bar").expect_err("NUL key rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[test]
fn traversal_guard_backslash() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("foo\\bar").expect_err("backslash key rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[test]
fn traversal_guard_empty_key() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("").expect_err("empty key rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[test]
fn traversal_guard_dot_key() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load(".").expect_err("dot key rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[test]
fn traversal_guard_intermediate_dot_component() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    // "foo/./bar" contains a CurDir component
    let err = loader
        .load("foo/./bar")
        .expect_err("intermediate dot rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

#[cfg(unix)]
#[test]
fn traversal_guard_escaping_symlink() {
    use std::os::unix::fs::symlink;

    let secret_dir = tempfile::tempdir().expect("secret dir");
    std::fs::write(secret_dir.path().join("secret.yaml"), "secret contents").unwrap();

    let base_dir = tempfile::tempdir().expect("base dir");
    // Create a symlink inside the base that points to the secret dir.
    symlink(secret_dir.path(), base_dir.path().join("escape")).expect("symlink");

    let loader = FileSystemLoader::with_base(base_dir.path()).expect("loader");
    // Loading "escape/secret" would follow the symlink out of base.
    let err = loader
        .load("escape/secret")
        .expect_err("symlink escape rejected");
    assert!(
        matches!(err, PromptLoadError::NotFound { .. }),
        "escaping symlink must yield NotFound, got {err:?}"
    );
}

// ─── FileSystemLoader: empty suffix behaviour ─────────────────────────────────

#[test]
fn filesystem_loader_custom_suffix_hit() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("greet.json"), VALID_YAML).unwrap();
    let loader =
        FileSystemLoader::new(dir.path(), ".json", FileSystemLoader::DEFAULT_MAX_BYTES).unwrap();
    let raw = loader.load("greet").expect("hit with .json suffix");
    assert!(!raw.is_empty());
}

#[test]
fn filesystem_loader_empty_suffix_dot_key_rejected() {
    // With empty suffix and key=".", the candidate is "{base}/.". That resolves to base —
    // but the validate_key step already rejects "." before we ever reach the filesystem.
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::new(&base, "", FileSystemLoader::DEFAULT_MAX_BYTES).unwrap();
    let err = loader
        .load(".")
        .expect_err("dot key with empty suffix rejected");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

// ─── FileSystemLoader: read cap (SC-009) ──────────────────────────────────────

#[test]
fn read_cap_exceeded_returns_load_io() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("big.yaml");
    // Write 100 bytes of content.
    let content = "x".repeat(100);
    std::fs::write(&path, &content).unwrap();

    // Set max_bytes to 50 — smaller than the file.
    let loader = FileSystemLoader::new(dir.path(), ".yaml", 50).unwrap();
    let err = loader.load("big").expect_err("read cap exceeded");
    assert!(
        matches!(err, PromptLoadError::Io { .. }),
        "exceeded cap → Io, got {err:?}"
    );
    let row = err.to_field_error();
    assert_eq!(row.code, code::LOAD_IO);
}

#[test]
fn read_cap_exactly_at_limit_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let content = "x".repeat(50);
    std::fs::write(dir.path().join("exact.yaml"), &content).unwrap();
    let loader = FileSystemLoader::new(dir.path(), ".yaml", 50).unwrap();
    // File is exactly at the cap — should succeed.
    loader.load("exact").expect("exactly at cap succeeds");
}

// ─── MemoryLoader ────────────────────────────────────────────────────────────

#[test]
fn memory_loader_hit() {
    let mut map = HashMap::new();
    map.insert("greet".to_string(), VALID_YAML.to_string());
    let loader = MemoryLoader::new(map);
    let raw = loader.load("greet").expect("hit");
    assert_eq!(raw, VALID_YAML);
}

#[test]
fn memory_loader_miss_returns_not_found() {
    let loader = MemoryLoader::default();
    let err = loader.load("missing").expect_err("miss");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
    let row = err.to_field_error();
    assert_eq!(row.code, code::LOAD_NOT_FOUND);
}

// ─── Closure-as-loader (blanket impl) ─────────────────────────────────────────

#[test]
fn closure_works_as_loader() {
    let loader: &dyn PromptLoader = &|key: &str| -> Result<String, PromptLoadError> {
        if key == "greet" {
            Ok(VALID_YAML.to_string())
        } else {
            Err(PromptLoadError::NotFound {
                key: key.to_string(),
            })
        }
    };
    loader.load("greet").expect("closure hit");
    let err = loader.load("other").expect_err("closure miss");
    assert!(matches!(err, PromptLoadError::NotFound { .. }));
}

// ─── SC-010: load error ≠ parse error at the class level ─────────────────────

#[test]
fn load_error_is_distinct_from_parse_error() {
    // A parse error is ConsumerError::Load (the dual-input loader / YAML shape error).
    let parse_err = Prompt::from_yaml("not valid yaml: : : ").expect_err("malformed YAML");
    assert!(
        matches!(parse_err, ConsumerError::Load(_)),
        "malformed YAML → ConsumerError::Load, got {parse_err:?}"
    );

    // A PromptLoadError is entirely separate — it cannot be matched as ConsumerError.
    let load_err = PromptLoadError::NotFound {
        key: "missing".to_string(),
    };
    // They are different types; only verify the codes differ to show the taxonomy is distinct.
    let row = load_err.to_field_error();
    assert_eq!(row.code, code::LOAD_NOT_FOUND);
    // The parse error carries the "load" code (for the dual-input loader), NOT "load_not_found".
    if let ConsumerError::Load(_) = parse_err {
        // Good — distinct enum variant.
    }
}

#[test]
fn compose_load_then_parse() {
    let mut map = HashMap::new();
    map.insert("greet".to_string(), VALID_YAML.to_string());
    let loader = MemoryLoader::new(map);

    let raw = loader.load("greet").expect("load");
    let prompt = Prompt::from_yaml(&raw).expect("parse after load");
    assert_eq!(prompt.name(), "test");
}

// ─── Error message scrubbing (SEC-003) ────────────────────────────────────────

#[test]
fn traversal_error_message_does_not_leak_path() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("../../../etc/passwd").expect_err("traversal");
    let row = err.to_field_error();
    // The message carries the logical key but must NOT leak the canonicalized base path.
    assert!(
        row.message.contains("../../../etc/passwd"),
        "message should carry the logical key: {}",
        row.message
    );
    assert!(
        !row.message.contains(&base.display().to_string()),
        "message must not leak base path: {}",
        row.message
    );
}

#[test]
fn not_found_error_message_contains_logical_key_only() {
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let loader = FileSystemLoader::with_base(&base).expect("loader");
    let err = loader.load("no_such_key").expect_err("miss");
    let row = err.to_field_error();
    // The message must carry the logical key, not an absolute path.
    assert!(
        row.message.contains("no_such_key"),
        "message contains key: {}",
        row.message
    );
    assert!(
        !row.message.contains(&base.display().to_string()),
        "message must not leak base path: {}",
        row.message
    );
}

// ─── Dependency-injection test (SC-002): swap FileSystem ↔ Memory ─────────────

fn load_and_parse(loader: &dyn PromptLoader, key: &str) -> prompting_press::Prompt {
    let raw = loader.load(key).expect("load");
    Prompt::from_yaml(&raw).expect("parse")
}

#[test]
fn swap_filesystem_for_memory_without_changing_call_site() {
    // Production: filesystem loader.
    let (_dir, base) = temp_dir_with_file("greet", VALID_YAML);
    let fs_loader = FileSystemLoader::with_base(&base).expect("fs loader");
    let from_fs = load_and_parse(&fs_loader, "greet");

    // Test: memory loader with the same content.
    let mut map = HashMap::new();
    map.insert("greet".to_string(), VALID_YAML.to_string());
    let mem_loader = MemoryLoader::new(map);
    let from_mem = load_and_parse(&mem_loader, "greet");

    // Both yield the same prompt name — call site `load_and_parse` is unchanged.
    assert_eq!(from_fs.name(), from_mem.name());
}
