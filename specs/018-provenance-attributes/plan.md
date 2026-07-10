# Implementation Plan: Provenance attributes helper

**Branch**: `018-provenance-attributes` | **Date**: 2026-07-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/018-provenance-attributes/spec.md`

## Summary

Add a `provenance_attributes()` projection method to the render result in each binding, returning a
flat string→string map of the four content-identity provenance fields (`name`, `variant`,
`template_hash`, `render_hash`) under fixed `prompting_press.prompt.*` keys — for one-call attachment to a
telemetry span. Pure getter: no I/O, no callback, no telemetry dependency, no emission. Rejects the
issue's `ProvenanceSink`/`OtelSink` (Principle V: no telemetry sink/coupling). Cites the spec-017
v3.0.0 repositioning statement; its own edit softens Principle V to permit *formatting* provenance
(still data on the return value). No kernel behavior change.

## Technical Context

**Language/Version**: Rust consumer + PyO3 + napi bindings; Python (dict) + TypeScript (Record)
facades. Existing workspace pins; unchanged.

**Primary Dependencies**: none new. Explicitly **no** telemetry/OTel dependency (FR-006).

**Storage**: N/A (no I/O).

**Testing**: cargo test (consumer + py/node), pytest, node:test, conformance corpus (parity of the
emitted map).

**Target Platform**: library (crate + wheel + npm addon). Unchanged.

**Project Type**: multi-binding library over a shared Rust core.

**Performance Goals**: N/A — building a 4-entry map from existing fields; not on a hot path.

**Constraints**: binding-layer only; the four fields already exist on the kernel `RenderResult`
(`crates/prompting-press-core/src/engine.rs:121-127`). No kernel behavior change (FR-010); no
telemetry dep (FR-006); metadata/text/guard excluded (FR-007).

**Scale/Scope**: one method × three bindings; 4 fixed keys; 1 conformance case.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (structural parity)** — ✅ PASS. The 4 keys + map shape are defined once (shared key
  constants, research R1) so every binding emits the identical map; a conformance case guards it.
- **Principle II (FFI isolation)** — ✅ PASS. Pure marshaling/formatting in the binding + consumer
  layer; no FFI crate in the kernel.
- **Principle III (minimal boundary)** — ✅ PASS. No I/O, no LLM, no request assembly, **no
  telemetry emission**. A projection of existing return-value fields.
- **Principle V (provenance)** — ✅ PASS + **amended here (softened)**. Provenance stays data on the
  return value; the library MAY now *format* it into an attribute map. Still **no sink, no push, no
  telemetry dependency**. The two hashes + their semantics are unchanged. Cites spec-017's v3.0.0
  repositioning statement.
- **Principle VI (idiom)** — ✅ PASS. `dict` / `Record` / `BTreeMap` per language; a method.
- **Principle VII (schema)** — ✅ PASS. No schema change (an API-surface method, not a
  prompt-definition field).
- **Scope Discipline / C-08** — ✅ PASS. No config knob (fixed keys, 4 fields); no pluggable sink.

**Gate result: PASS.** The only judgment point is R1 (where the shared key constants live) — a
parity implementation detail, not a principle risk.

## Project Structure

### Documentation (this feature)

```text
specs/018-provenance-attributes/
├── plan.md · spec.md · memory-synthesis.md · research.md · data-model.md · quickstart.md
├── contracts/provenance-attributes.md
└── tasks.md   (Phase 2 — /speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── prompting-press-core/   # RenderResult data struct (fields already present) — NO behavior change
├── prompting-press/        # consumer — home for the shared prompting_press.prompt.* key constants +
│                           #   a provenance_attributes(&RenderResult)->BTreeMap helper (research R1)
├── prompting-press-py/     # render.rs — add provenance_attributes() method to the RenderResult pyclass
└── prompting-press-node/   # render.rs — add provenanceAttributes() to the napi RenderResult type

packages/
├── python/                 # RenderResult surfaced 1:1 — method exposed via the pyclass
└── typescript/src/index.ts # add provenanceAttributes(): Record<string,string> to the RenderResult type

conformance/                # add a provenance-attributes parity case (identical map across bindings)
.specify/memory/            # constitution v3.0.0 Principle V softening + DECISIONS.md (cites spec-017)
```

**Structure Decision**: Define the four `prompting_press.prompt.*` key strings and the map-building **once**
in the consumer crate (`prompting-press`) as `pub const`s + a small helper over the fields; each
binding's `RenderResult` method calls/mirrors it so the emitted map is identical (Principle I),
without a kernel behavior change (FR-010). See research R1 for the alternative considered.

## Complexity Tracking

> No Constitution Check violations. Section intentionally empty.
