# Phase 0 Research: pluggable prompt loader

No open NEEDS CLARIFICATION. Decisions from grill + 2026-07-08 clarify defaults.

## R1 — Interface + error contract
- **Decision:** `load(key) -> raw text`; failure → `PromptLoadError` normalized into the crate's common
  `[{field, code, message}]` family (reuse `ConsumerError::Load` lineage, `crates/prompting-press/src/error.rs:109`),
  NEW class `PromptLoadError` + NEW codes `load_io`/`load_not_found` (NOT a reuse of the existing parse-error `LoadError`/`ConsumerError::Load` — that collided). Recorded as a compatibility-surface expansion (FR-018). Distinct at the CLASS level from parse errors (FR-007/SC-010).
- **Rationale:** consistency with existing error normalization (C-06); no bespoke parallel hierarchy.
- **Alternatives rejected:** nullable return (silent empty → confusing downstream parse error);
  brand-new exception tree (fragments error handling).

## R2 — Per-language interface + built-ins
- **Decision:** Rust `trait PromptLoader { fn load(&self,key:&str)->Result<String,LoadError>; }`
  (object-safe; `dyn`) + blanket impl for `Fn(&str)->Result<String,LoadError>`. Python
  `runtime_checkable Protocol` + callable coercion. TS `interface { load(key):Promise<string> }` +
  function coercion. Built-ins `FileSystemLoader` + `MemoryLoader` in each.
- **Sync/async:** sync in Rust/Python; **async (`Promise<string>`) in TS** (Node fs idiom; cloud
  backends need async) — C-06 native idiom, accepted divergence.
- **Open (plan/tasks):** whether Python/TS built-ins wrap the Rust ones via FFI or are native. Lean:
  **native per language** (FileSystemLoader is a few lines of fs code; avoids marshaling a string
  round-trip through FFI for a trivial read, and keeps the loader a pure language-side leaf). The
  *interface* is native regardless so custom loaders are idiomatic.

## R3 — FileSystemLoader mapping + traversal guard (security)
- **Decision:** `base` dir + `suffix` (default `.yaml`); `load(key)` reads `{base}/{key}{suffix}`.
  Keys are relative under `base`; a key escaping `base` (`..`, absolute) → `PromptLoadError` (SC-008).
- **Rationale:** logical keys not physical paths (centralization value); traversal guard prevents
  reading arbitrary files when keys are attacker-influenced. Canonicalize + prefix-check under base.

## R4 — Not fused into construction; no container (scope guards)
- **Decision:** loader stays separate from `from_yaml`/`from_json`/`from_toml`
  (`crates/prompting-press/src/prompt.rs:108+`); caller composes `from_yaml(loader.load(key))`
  (FR-011). NO `Prompt.load(loader,...)`. NO name-keyed container/registry (FR-012) — deferred.
- **Rationale:** keeps construction I/O-free (Principle III preserved there); the container is the
  deferred "query-capable registry" and must be ratified on its own.

## R5 — Per-language ecosystems; heavier backends deferred
- **Decision:** ship FileSystemLoader + MemoryLoader only, in the standard package. fsspec (Py) /
  object_store (Rust) / S3-GCS (TS) DEFERRED to opt-in extras (FR-004). Loaders don't cross FFI —
  a Python loader ≠ a TS loader (three parallel ecosystems by construction).
- **Rationale:** the interface is the extension point; the ecosystem grows per language. Adding
  cloud deps now is unearned (C-08).

## R6 — Amendment (MAJOR)
- **Decision:** soften Principle III for a caller-invoked language-side loader seam (kernel +
  construction stay I/O-free); re-scope C-08 to admit the Loader seam as earned by Bellwether; update
  roadmap "Never: I/O/storage adapters" + Scope-Discipline entries. Cite spec-017 v3.0.0 repositioning.
- **Rationale:** the deliberate repositioning; bounded (opt-in leaf, kernel untouched).
