# Implementation Plan: Pluggable prompt loader

**Branch**: `019-pluggable-loader` | **Date**: 2026-07-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/019-pluggable-loader/spec.md`

## Summary

Add a language-native `PromptLoader` interface (`load(key) -> raw text`) plus two built-in
implementations — `FileSystemLoader` (base dir + suffix; path-traversal-guarded) and
`MemoryLoader` (key→text map) — in each binding's standard package. The loader is a pure I/O leaf:
it returns text, never a `Prompt`, and is composed by the caller with the existing text factories
(`Prompt.from_yaml(loader.load(key))`). No fusion into construction, no name-keyed container, no
cross-FFI loader. Missing key → `PromptLoadError` normalized into the common `[{field,code,message}]`
family, distinct from parse errors. Carries the MAJOR boundary amendment: reintroduces the Loader
seam (C-08) and softens Principle III's "no I/O" for a caller-invoked, language-side loader while
the **kernel and construction stay I/O-free**. Cites spec-017's v3.0.0 repositioning statement.

## Technical Context

**Language/Version**: Rust consumer (`trait PromptLoader`, object-safe, blanket `Fn` impl) + Python
(`Protocol` + callable coercion) + TypeScript (`interface` + function coercion). Existing pins.

**Primary Dependencies**: none new in the standard package (filesystem via std). Heavier backends
(fsspec / object_store / S3) explicitly **deferred** to opt-in extras — not added here.

**Storage**: filesystem read (the loader's whole point) — confined to the loader leaf, invoked by
the caller. The **kernel and prompt construction perform no I/O** (Principle III preserved there).

**Testing**: cargo test (consumer), pytest, node:test; per-binding loader + traversal-guard tests;
custom-loader example test; error-distinction test. (Loaders are language-side; not a conformance-
corpus/ FFI-marshaling concern — per-binding suites own them.)

**Target Platform**: library (crate + wheel + npm addon).

**Project Type**: multi-binding library; loaders are **per-language ecosystems by construction**
(a Python loader ≠ a TS loader) — no cross-FFI loader concept.

**Performance Goals**: N/A — a file read / map lookup per `load` call.

**Constraints**: kernel untouched (SC-005); no name-keyed container (FR-012); not fused into
construction (FR-011); no new standard-package dependency (SC-006); traversal-guarded (SC-008).

**Scale/Scope**: 1 interface + 2 built-ins × 3 bindings; sync in Py/Rust, async in TS.

## Constitution Check

*GATE: Must pass before Phase 0. Re-check after Phase 1.*

- **Principle I (parity)** — ✅ PASS (bounded). Loaders are language-side by design; there is no
  shared-core behavior to keep byte-identical (they produce text the existing factories consume).
  The *capability* is uniform; the implementations are native (C-06). Not a conformance-corpus item.
- **Principle II (FFI isolation)** — ✅ PASS. Loaders live entirely in the binding/consumer layer;
  the kernel never learns about them; no FFI crate added to the kernel.
- **Principle III (minimal boundary)** — ⚠️ **AMENDED (softened here — the MAJOR edit)**. Today: "no
  I/O … no storage layer." This feature permits a **caller-invoked, language-side loader seam** that
  reads bytes. Preserved: the **kernel performs no I/O and stays validation-blind**, and **prompt
  construction remains I/O-free** (the loader is separate from `from_yaml`; FR-011). The relaxation
  is scoped to an opt-in leaf the caller invokes, not the kernel. Cites spec-017 repositioning.
- **Principle VII (schema)** — ✅ PASS. No schema change (loaders are an API-surface concern).
- **Scope Discipline / C-08** — ⚠️ **AMENDED**. The "Loader" seam (eliminated in R1) is reintroduced
  as an **earned** opt-in seam — earned by the real second consumer (Bellwether), which is exactly
  C-08's "earned by a second concrete consumer" bar. The "Never: I/O / storage adapters" roadmap
  entry is updated accordingly. Heavier backends stay deferred until further demand.
- **C-06 (idiom + error normalization)** — ✅ PASS. `LoadError` normalizes into `[{field,code,
  message}]`; native error types don't cross FFI.

**Gate result: PASS with a ratified MAJOR amendment** (Principle III softening + C-08 Loader
re-scope). This is the intended repositioning, recorded in DECISIONS.md, not an unjustified
violation. Complexity Tracking: the amendment IS the complexity, justified by the second consumer.

## Project Structure

### Documentation (this feature)

```text
specs/019-pluggable-loader/
├── plan.md · spec.md · memory-synthesis.md · research.md · data-model.md · quickstart.md
├── contracts/loader.md
└── tasks.md   (Phase 2 — /speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── prompting-press-core/   # UNTOUCHED (kernel stays I/O-free; SC-005)
├── prompting-press/        # Rust consumer — NEW loader module: trait PromptLoader (object-safe,
│                           #   blanket Fn impl), FileSystemLoader{base,suffix}, MemoryLoader, LoadError
├── prompting-press-py/     # expose loaders to Python (PyO3) OR provide them in the Python facade
└── prompting-press-node/   # expose loaders to Node (napi) OR provide them in the TS facade

packages/
├── python/                 # PromptLoader Protocol + FileSystemLoader + MemoryLoader (callable coercion)
│                           #   — sync `load(key) -> str`
└── typescript/src/         # PromptLoader interface + FileSystemLoader (node fs) + MemoryLoader
                            #   — async `load(key): Promise<string>` (function coercion)

.specify/memory/            # constitution v3.0.0: Principle III softening + C-08 Loader re-scope;
                            #   DECISIONS.md; roadmap "Never: I/O" + Scope-Discipline entries updated
```

**Structure Decision**: Loaders are **per-language** (three parallel ecosystems). The Rust consumer
gets the canonical `trait`/built-ins; Python and TS get native protocol/interface + built-ins in
their facades (sync Rust/Py, async TS). Whether Python/TS reuse the Rust built-ins via FFI or
implement natively is a plan-time detail (research R2) — but the *interface* is defined natively in
each so custom loaders are idiomatic. Kernel + construction remain I/O-free.

## Complexity Tracking

| Item | Why needed | Simpler alternative rejected because |
|------|-----------|--------------------------------------|
| Principle III softening + C-08 Loader re-scope (MAJOR amendment) | The deliberate repositioning (batteries-included, opt-in seams) with a real second consumer (Bellwether) needing swappable storage | Keeping the pure-core-only stance was rejected by the user's explicit repositioning decision; the loader is opt-in and the kernel/construction stay I/O-free, so the relaxation is bounded |
