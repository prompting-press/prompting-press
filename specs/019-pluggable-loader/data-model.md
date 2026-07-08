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
- `load(key)` reads `{base}/{key}{suffix}`. Key is relative under `base`.
- **Traversal guard:** a key resolving outside `base` (`..`/absolute) → `LoadError` (SC-008).

## Entity: MemoryLoader (built-in)
- Constructed from a `key → text` map. `load(key)` returns the mapped text; miss → `LoadError`.
- Primary testing/embedding loader (no filesystem).

## Entity: LoadError
- Normalized into the common `[{field, code, message}]` family (C-06); code `not_found`/`io`.
- **Distinct** from construct-from-text (parse/validation) errors (FR-007).
- Built-ins raise it; custom loaders documented-to (their own errors propagate as-is; FR-008).

## Behavior / Invariants
- **INV-1 (text leaf):** `load` returns raw text; no parse, no format detection, no Prompt (FR-005).
- **INV-2 (structured miss):** missing key → `LoadError`, never null/empty/partial (FR-006).
- **INV-3 (distinct surfaces):** load error ≠ parse error (FR-007).
- **INV-4 (traversal-safe):** FileSystemLoader never reads outside `base` (SC-008).
- **INV-5 (not fused):** no construction path performs I/O; caller composes load + from_yaml (FR-011).
- **INV-6 (no container):** no name-keyed holder/cache/lint object in this feature (FR-012).
- **INV-7 (kernel untouched):** prompting-press-core has no diff; no new standard-pkg dep (SC-005/006).
