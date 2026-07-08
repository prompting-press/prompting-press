# Phase 1 Data Model: pluggable prompt loader

No prompt-definition schema change. New surface: the loader interface, two built-ins, an error type.

## Entity: PromptLoader (interface)
- One operation: `load(key) -> raw text` (sync Rust/Python; `Promise<string>` TS).
- Rust: object-safe `trait` + blanket impl for `Fn(&str)->Result<String,LoadError>`.
- Python: `runtime_checkable Protocol`; a plain callable coerces to a loader.
- TS: `interface`; a function coerces to a loader.
- Returns **text only** (never a Prompt; never parses). No registration to use a custom impl.

## Entity: FileSystemLoader (built-in)
| Field | Meaning | Default |
|---|---|---|
| `base` | base directory | (required) |
| `suffix` | file suffix appended to the key | `.yaml` |
| `max_bytes` | max file size read; exceed → `load_io` | (sane default) |
- `load(key)` reads `{base}/{key}{suffix}`. Key is relative under `base`.
- **Traversal guard:** a key resolving outside `base` (`..`/absolute) → `PromptLoadError` (SC-008).

## Entity: MemoryLoader (built-in)
- Constructed from a `key → text` map. `load(key)` returns the mapped text; miss → `PromptLoadError`.
- Primary testing/embedding loader (no filesystem).

## Entity: PromptLoadError (NEW type)
- A NEW exception class (paralleling `PromptRenderError`/`PromptValidationError`), NOT the existing `LoadError` (which is the parse/shape error). Normalized into `[{field, code, message}]` (C-06); NEW codes **`load_io`** / **`load_not_found`** added to the closed vocab (a compatibility-surface expansion recorded in the amendment).
- **Distinct** from construct-from-text (parse/validation) errors (FR-007).
- Built-ins raise it; custom loaders documented-to (their own errors propagate as-is; FR-008).

## Behavior / Invariants
- **INV-1 (text leaf):** `load` returns raw text; no parse, no format detection, no Prompt (FR-005).
- **INV-2 (structured miss):** missing key → `PromptLoadError`, never null/empty/partial (FR-006).
- **INV-3 (distinct surfaces):** load error ≠ parse error (FR-007).
- **INV-4 (traversal-safe):** guard validates the FINAL path incl. suffix vs canonicalized `base`; rejects absolute/`..`/escaping-symlink; `key=""`/`key="."`/empty-suffix defined; canonicalize-fail-on-missing → `load_not_found` (SC-008).
- **INV-4a (read cap):** exceed `max_bytes` → `PromptLoadError` `load_io` (SC-009).
- **INV-5 (not fused):** no construction path performs I/O; caller composes load + from_yaml (FR-011).
- **INV-6 (no container):** no name-keyed holder/cache/lint object in this feature (FR-012).
- **INV-7 (kernel untouched):** prompting-press-core has no diff; no new standard-pkg dep (SC-005/006).
