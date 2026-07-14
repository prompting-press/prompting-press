// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Pluggable prompt loader (spec 019).
//!
//! [`PromptLoader`] is an object-safe trait whose single operation [`PromptLoader::load`]
//! takes a logical key and returns the prompt source as **raw text** — never a
//! [`crate::Prompt`], never parsed. Construction and parsing remain separate, composable
//! steps (FR-005/FR-011).
//!
//! ## Built-in loaders
//!
//! - [`FileSystemLoader`]: maps a key to `{base}/{key}{suffix}`, traversal-guarded and
//!   read-capped. Default suffix `.yaml`, default cap
//!   [`FileSystemLoader::DEFAULT_MAX_BYTES`].
//! - [`MemoryLoader`]: maps a key against an in-memory `HashMap<String, String>`.
//!
//! ## Composing with [`crate::Prompt`]
//!
//! The loader is a pure I/O leaf. To load and construct a prompt:
//!
//! ```no_run
//! # use std::collections::HashMap;
//! # use prompting_press::loader::{MemoryLoader, PromptLoader};
//! # use prompting_press::Prompt;
//! let mut map = HashMap::new();
//! map.insert("greet".to_string(), r#"name: greet
//! role: user
//! body: "Hello {{ name }}"
//! variables:
//!   name: { type: string, trusted: true }
//! "#.to_string());
//! let loader = MemoryLoader::new(map);
//! let raw = loader.load("greet").expect("key present");
//! let prompt = Prompt::from_yaml(&raw).expect("valid YAML");
//! assert_eq!(prompt.name(), "greet");
//! ```
//!
//! ## Custom loaders
//!
//! Implement [`PromptLoader`] on a struct, or pass a closure directly:
//!
//! ```
//! # use prompting_press::loader::PromptLoader;
//! # use prompting_press::PromptLoadError;
//! let loader: &dyn PromptLoader = &|key: &str| -> Result<String, PromptLoadError> {
//!     Err(PromptLoadError::NotFound { key: key.to_string() })
//! };
//! assert!(loader.load("anything").is_err());
//! ```
//!
//! ## Error taxonomy
//!
//! [`PromptLoadError`](crate::PromptLoadError) is distinct from [`crate::ConsumerError`] at the
//! type level (FR-007/FR-008):
//! - [`crate::error::code::LOAD_NOT_FOUND`]: key not in backing store.
//! - [`crate::error::code::LOAD_IO`]: I/O error or `max_bytes` exceeded.
//!
//! ## Security: traversal guard + read cap (FR-002a/FR-002b/FR-016/SC-008/SC-009)
//!
//! [`FileSystemLoader`] validates the final resolved path (including suffix) against a
//! canonicalized base directory. Keys with `..` components, absolute paths, embedded NUL
//! bytes, cross-platform separators, or symlinks that escape the base are all rejected with
//! `load_not_found`. A missing-target canonicalize failure also produces `load_not_found`
//! (not `load_io`). Reading a file beyond `max_bytes` produces `load_io`.
//!
//! ## Sync nature
//!
//! This Rust trait is **synchronous** (the idiomatic Rust default for library code). The
//! TypeScript binding exposes an async interface (`Promise<string>`) to match the Node.js
//! ecosystem's native I/O idiom (FR-009).

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use crate::PromptLoadError;

// ─── PromptLoader trait ──────────────────────────────────────────────────────

/// A pluggable source of raw prompt text (spec 019, FR-001).
///
/// The single operation [`PromptLoader::load`] maps a logical key to the raw text of a
/// prompt definition. The returned text is **never parsed or validated** — that belongs to
/// the construct-from-text path ([`crate::Prompt::from_yaml`] etc.). Loading and
/// construction are always separate, composable steps (FR-005/FR-011).
///
/// The trait is **object-safe** (`dyn PromptLoader` is valid) and `Send + Sync` so loaders
/// can be stored in shared state without additional wrappers.
///
/// A blanket impl allows any `Fn(&str) -> Result<String, PromptLoadError>` closure to be
/// used as a loader without defining a struct (FR-001).
pub trait PromptLoader: Send + Sync {
    /// Load the prompt source for the given logical `key`, returning raw text.
    ///
    /// # Errors
    ///
    /// - [`PromptLoadError::NotFound`] — the key does not exist in the backing store.
    /// - [`PromptLoadError::Io`] — an I/O failure occurred, or a resource cap was exceeded.
    fn load(&self, key: &str) -> Result<String, PromptLoadError>;
}

/// Blanket impl: any `Fn(&str) -> Result<String, PromptLoadError>` closure is a loader.
///
/// This allows passing a closure directly wherever a [`PromptLoader`] reference is expected,
/// without defining a struct.
impl<F> PromptLoader for F
where
    F: Fn(&str) -> Result<String, PromptLoadError> + Send + Sync,
{
    fn load(&self, key: &str) -> Result<String, PromptLoadError> {
        self(key)
    }
}

// ─── FileSystemLoader ────────────────────────────────────────────────────────

/// A loader that reads prompt files from a configured base directory (spec 019, FR-002).
///
/// Maps a logical key to `{base}/{key}{suffix}` and returns the file's raw text.
///
/// ## Traversal guard (FR-002a/FR-002b/SC-008)
///
/// Keys are treated as relative paths under `base`. The guard rejects:
/// - keys with `..` path components
/// - absolute keys (start with `/` or a Windows drive prefix)
/// - keys containing embedded NUL bytes
/// - keys containing backslash (`\`) or UNC path patterns
/// - `key=""` or `key="."` (resolve to the base directory, not a file)
/// - any intermediate `"."` component (e.g. `foo/./bar`)
/// - symlinks that point outside the canonicalized `base`
///
/// A canonicalize failure on a **missing target** returns `load_not_found` (not `load_io`).
///
/// ## Read cap (FR-016/SC-009)
///
/// Reading a file exceeding [`max_bytes`](FileSystemLoader) returns
/// `PromptLoadError::Io` with code `load_io`.
pub struct FileSystemLoader {
    /// The canonicalized base directory (resolved at construction time).
    base: PathBuf,
    /// File name suffix appended to every key (default `.yaml`).
    suffix: String,
    /// Maximum bytes to read from a file.
    pub max_bytes: usize,
}

impl FileSystemLoader {
    /// The default maximum file size in bytes (1 MiB).
    pub const DEFAULT_MAX_BYTES: usize = 1 << 20; // 1 MiB

    /// Construct a `FileSystemLoader` rooted at `base` with the given `suffix` and `max_bytes`.
    ///
    /// `base` is canonicalized at construction time. A non-existent `base` returns
    /// `PromptLoadError::NotFound`; other OS failures return `PromptLoadError::Io`.
    ///
    /// # Errors
    ///
    /// - [`PromptLoadError::NotFound`] if `base` does not exist.
    /// - [`PromptLoadError::Io`] on other canonicalize failures (e.g. permission error).
    pub fn new(
        base: impl Into<PathBuf>,
        suffix: impl Into<String>,
        max_bytes: usize,
    ) -> Result<Self, PromptLoadError> {
        let raw: PathBuf = base.into();
        let canonical = canonicalize_dir(&raw)?;
        Ok(Self {
            base: canonical,
            suffix: suffix.into(),
            max_bytes,
        })
    }

    /// Convenience constructor with default suffix (`.yaml`) and
    /// [`DEFAULT_MAX_BYTES`](FileSystemLoader::DEFAULT_MAX_BYTES).
    ///
    /// # Errors
    ///
    /// - [`PromptLoadError::NotFound`] if `base` does not exist.
    /// - [`PromptLoadError::Io`] on other canonicalize failures.
    pub fn with_base(base: impl Into<PathBuf>) -> Result<Self, PromptLoadError> {
        Self::new(base, ".yaml", Self::DEFAULT_MAX_BYTES)
    }
}

/// Canonicalize a **base directory** path, mapping OS errors to [`PromptLoadError`].
fn canonicalize_dir(path: &Path) -> Result<PathBuf, PromptLoadError> {
    path.canonicalize().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PromptLoadError::NotFound {
                key: path.display().to_string(),
            }
        } else {
            PromptLoadError::Io {
                key: path.display().to_string(),
                detail: "cannot canonicalize base directory".to_string(),
            }
        }
    })
}

/// Validate `key` for safety before path construction (FR-002a/FR-002b).
///
/// Returns `Err(PromptLoadError::NotFound)` for any suspicious key so that the error code
/// is `load_not_found` (never leaks OS-level path detail).
fn validate_key(key: &str) -> Result<(), PromptLoadError> {
    let reject = || PromptLoadError::NotFound {
        key: key.to_string(),
    };

    // NUL byte — kills a C-string path on POSIX/Windows.
    if key.contains('\0') {
        return Err(reject());
    }
    // Backslash — Windows-style separator or UNC prefix.
    if key.contains('\\') {
        return Err(reject());
    }
    // Empty key or bare "." resolves to the base directory itself.
    if key.is_empty() || key == "." {
        return Err(reject());
    }

    // Walk the path components and reject anything that is not a plain Normal segment.
    for component in Path::new(key).components() {
        match component {
            Component::Normal(_) => {}
            // Absolute root, "..", ".", or a Windows prefix — all rejected.
            _ => return Err(reject()),
        }
    }

    // If the key is absolute per `Path::is_absolute` (belt-and-suspenders after the
    // component walk above).
    if Path::new(key).is_absolute() {
        return Err(reject());
    }

    Ok(())
}

impl PromptLoader for FileSystemLoader {
    fn load(&self, key: &str) -> Result<String, PromptLoadError> {
        // --- traversal guard (FR-002a/FR-002b/SC-008) ---
        validate_key(key)?;

        // Build the candidate path: {base}/{key}{suffix}.
        // Append the suffix as a raw string (not via `set_extension`) to avoid
        // stripping an existing dot in the key or the suffix.
        let candidate_str = format!(
            "{}{}{}",
            self.base.display(),
            std::path::MAIN_SEPARATOR,
            key
        ) + &self.suffix;
        let candidate = PathBuf::from(&candidate_str);

        // Canonicalize to resolve symlinks.  A missing file → not_found.
        let resolved = match candidate.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PromptLoadError::NotFound {
                    key: key.to_string(),
                });
            }
            Err(_) => {
                return Err(PromptLoadError::Io {
                    key: key.to_string(),
                    detail: "path resolution failed".to_string(),
                });
            }
        };

        // Symlink escape check: the resolved path must be a strict descendant of self.base.
        if !resolved.starts_with(&self.base) {
            return Err(PromptLoadError::NotFound {
                key: key.to_string(),
            });
        }

        // --- read cap (FR-016/SC-009) ---
        let metadata = resolved.metadata().map_err(|_| PromptLoadError::Io {
            key: key.to_string(),
            detail: "cannot read file metadata".to_string(),
        })?;
        // Cast is safe: file metadata len fits in usize on any supported 64-bit platform.
        // On a theoretical 32-bit platform, files > 4 GiB would wrap — the cap is << 4 GiB.
        #[allow(clippy::cast_possible_truncation)]
        let file_size = metadata.len() as usize;
        if file_size > self.max_bytes {
            return Err(PromptLoadError::Io {
                key: key.to_string(),
                detail: format!(
                    "file size ({file_size} bytes) exceeds max_bytes ({})",
                    self.max_bytes
                ),
            });
        }

        // --- read ---
        std::fs::read_to_string(&resolved).map_err(|_| PromptLoadError::Io {
            key: key.to_string(),
            detail: "failed to read file".to_string(),
        })
    }
}

// ─── MemoryLoader ────────────────────────────────────────────────────────────

/// A loader backed by an in-memory key→text mapping (spec 019, FR-003).
///
/// The primary use case is **dependency injection in tests**: production code uses a
/// [`FileSystemLoader`] or a custom loader; tests substitute a `MemoryLoader` with
/// hard-coded prompt text. No filesystem access is performed.
///
/// A missing key returns [`PromptLoadError::NotFound`]; there is no I/O path.
#[derive(Default)]
pub struct MemoryLoader {
    map: HashMap<String, String>,
}

impl MemoryLoader {
    /// Construct a `MemoryLoader` from an existing key→text mapping.
    #[must_use]
    pub fn new(map: HashMap<String, String>) -> Self {
        Self { map }
    }
}

impl PromptLoader for MemoryLoader {
    fn load(&self, key: &str) -> Result<String, PromptLoadError> {
        self.map
            .get(key)
            .cloned()
            .ok_or_else(|| PromptLoadError::NotFound {
                key: key.to_string(),
            })
    }
}
