# Implementation Plan: Prompt `derive()` merge strategy

**Branch**: `017-derive-merge-strategy` | **Date**: 2026-07-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/017-derive-merge-strategy/spec.md`

## Summary

Add a `merge` strategy to the immutable `Prompt.derive` primitive so a derived prompt can
**union** its base's map-typed fields (`variables`, `variants`, `metadata`) with the overlay's
(child-wins, whole-entry) instead of only wholesale-replacing them. Two strategies: `Replace`
(default, today's behavior — non-breaking) and `Merge` (new union). Scalar fields always
replace. The merged definition is re-validated through the existing single validating
constructor (`Prompt::new(merged)`), so the agreement check and `validation_required` coverage
run over the merged whole. The union algorithm is implemented **once** in the Rust consumer as a
shared `merge_definitions` helper; the Python binding delegates through `derive`, and the Node
binding calls the same helper (replacing its private JSON merge) — genuine single-source parity
(Principle I). No kernel change, no JSON-Schema change, no I/O. Carries the v3.0.0 constitution
repositioning statement, a Principle VI clarification, **and a redefinition of spec-008 FR-017(b)**
(overlay may union under `Merge`, superseding "wholesale-replace only").

## Technical Context

**Language/Version**: Rust (stable, workspace-pinned) for `prompting-press` consumer +
`prompting-press-py` (PyO3) + `prompting-press-node` (napi); Python 3.x (Pydantic) facade;
TypeScript (Zod) facade. Versions per existing workspace pins — unchanged by this feature.

**Primary Dependencies**: existing only — MiniJinja (kernel, untouched), garde (Rust validator),
Pydantic (Python), Zod (TS), PyO3, napi-rs. **No new dependency added.**

**Storage**: N/A (no I/O; Principle III).

**Testing**: `cargo test` (consumer + py/node crates), pytest (Python), node:test (TS),
plus the shared `conformance/` corpus for cross-binding parity.

**Target Platform**: library — Rust crate + PyPI wheel + npm native addon. Unchanged.

**Project Type**: multi-binding library over a shared Rust core (crate layout is load-bearing:
`prompting-press-core` kernel / `prompting-press` consumer / `-py` / `-node`).

**Performance Goals**: N/A — merge is an in-memory map union over a handful of fields at
construction; not on any hot path. Determinism (equal inputs → equal merged definition + hashes)
is the only relevant property.

**Constraints**: consumer/binding-layer ONLY — `prompting-press-core` and
`schemas/jsonschema/prompt-definition.schema.json` MUST show no diff (SC-006). No I/O. Default
strategy byte-identical to today (SC-002). Cross-binding semantic parity (D1: compare canonical
serialized form).

**Scale/Scope**: 3 map fields unioned; 4 scalar fields replaced; 2 strategy values; 3 bindings.
Small, bounded surface.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against constitution v2.0.0 (→ v3.0.0 after this feature's amendment):

- **Principle I (Shared core, structural parity)** — ✅ PASS **via a shared helper (corrected)**.
  The union algorithm lands **once** in the consumer as `merge_definitions(base, overlay, strategy)`
  operating in `serde_json::Value` space. **Correction (adversarial review):** the Python binding
  delegates to `inner.derive` (`crates/prompting-press-py/src/prompt.rs:454`), but the **Node
  binding does NOT** — it has a private `shallow_merge_json` → `Prompt::from_json`
  (`crates/prompting-press-node/src/prompt.rs:245-262,335-349`), deliberately avoiding a
  `Deserialize` dep on `PromptOverlay`. The prior draft wrongly claimed "node delegates via
  `inner.derive` (verified)". Resolution (FR-018): both the typed Rust `derive`/`derive_with` and
  Node's construction path call the single `merge_definitions` helper — genuine single-source
  parity, byte-identical by construction (honors D1: per-binding date/decimal serialization would
  otherwise let a JSON-space union and a typed-map union diverge). No second union implementation.
- **Principle II (FFI isolation)** — ✅ PASS. No FFI crate touches the kernel; merge is in the
  consumer + marshaling in the bindings. `MergeStrategy` marshals as a small enum/string across FFI.
- **Principle III (Minimal boundary)** — ✅ PASS. No I/O, no LLM, no request assembly. Merge is a
  pure in-memory transform before re-validation. Kernel untouched.
- **Principle IV (Sound agreement check)** — ✅ PASS. Merge runs BEFORE `Prompt::new(merged)`,
  which runs the agreement check over the merged variable set (FR-007). The check is unchanged and
  stays sound; no new template feature introduced.
- **Principle V (Repo canonical / provenance)** — ✅ PASS (untouched). No version axis, no hash
  change to the mechanism (a merged prompt hashes its resolved body as usual).
- **Principle VI (Per-language idiom)** — ✅ PASS + **amended here**. `MergeStrategy` is native in
  each language (Python enum keyword-only; TS enum/const in an options object; Rust enum in a
  `Default` options struct — C-11). `validation_required` coverage under merge: Py/TS raise at
  construction, Rust guarantees at compile time (asymmetry preserved). The amendment ADDS a
  merge-strategy axis clause; it does not reverse the principle.
- **Principle VII (JSON Schema single source)** — ✅ PASS. **No schema change** → no codegen change.
  `MergeStrategy` is an API-surface type, not a prompt-definition field.
- **Scope Discipline / C-08** — ✅ PASS. `deep`/`none` excluded; the enum axis is reserved for a
  future consumer-earned value without a new method. No new pluggable interface.
- **C-11 (options/keyword call shape)** — ✅ PASS. Selector is keyword-only (Py) / options object
  (TS) / Default options struct (Rust), never a positional mode boolean.

**Amendment note (in-scope governance work, not a violation):** this feature carries the
v3.0.0 repositioning statement, a Principle VI clarification, AND a redefinition of spec-008
FR-017(b) — overlay may union under `Merge`, superseding its "wholesale-replace only / no deep
merge" wording (FR-015/016/017). Recorded in
DECISIONS.md; no principle is violated — one is additively clarified.

**Gate result: PASS.** No violations; Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/017-derive-merge-strategy/
├── plan.md              # This file
├── spec.md              # Feature spec (+ Clarifications)
├── memory.md            # Feature-local notes
├── memory-synthesis.md  # Memory synthesis (planning input)
├── research.md          # Phase 0 output (this command)
├── data-model.md        # Phase 1 output (this command)
├── quickstart.md        # Phase 1 output (this command)
├── contracts/           # Phase 1 output (this command)
│   └── derive-merge.md  # Per-binding derive surface contract
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

```text
crates/
├── prompting-press-core/        # KERNEL — untouched by 017 (SC-006 asserts no diff)
├── prompting-press/             # Rust consumer — PRIMARY change site
│   └── src/prompt.rs            #   NEW: MergeStrategy, DeriveOptions{strategy}, derive_with,
│                                #        and the SHARED merge_definitions(base,overlay,strategy)
│                                #        helper (serde_json::Value space) — the single source
├── prompting-press-py/          # PyO3 binding — add strategy= keyword-only to derive; MergeStrategy enum
│   └── src/prompt.rs            #   derive(.., *, validators=None, strategy=Replace); delegates to consumer
└── prompting-press-node/        # napi binding — REFACTOR derive_prompt to call the shared helper
    └── src/prompt.rs            #   replace private shallow_merge_json; marshal MergeStrategy; call helper

packages/
├── python/                      # Pydantic facade — export MergeStrategy; keyword-only merge=
└── typescript/
    └── src/index.ts             # Zod facade — derive(overlay, { validators?, merge? }); export MergeStrategy

schemas/jsonschema/              # UNTOUCHED (no schema change; SC-006)
conformance/                     # add a derive-merge parity fixture/case (cross-binding, D1)
.specify/memory/                 # constitution v3.0.0 edit + DECISIONS.md entry (amendment)
```

**Structure Decision**: The change is concentrated in `crates/prompting-press/src/prompt.rs`
(the single source of merge logic) plus thin marshaling additions in each binding + the two
language facades. The kernel crate and the JSON Schema are explicitly out of the change set.

## Complexity Tracking

> No Constitution Check violations. Section intentionally empty.
