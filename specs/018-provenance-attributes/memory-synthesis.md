# Memory Synthesis

## Current Scope

Spec 018 — add a `provenance_attributes()` projection method to `RenderResult` in each binding,
returning a flat `gen_ai.prompt.*` string→string map of the four content-identity fields. Binding/
consumer layer only; kernel `RenderResult` data unchanged. Cites spec-017's v3.0.0 repositioning;
its own edit softens Principle V (formatting allowed; still no sink/push/dependency).

## Relevant Decisions

- **D1 — Cross-binding parity via canonical serialized form** (Reason: the emitted map's values are
  the already-serialized `template_hash`/`render_hash`/`name`/`variant`; the conformance parity case
  compares serialized form. Status: active. Source: docs/memory/decisions/2026-06-28-…marshaling.md)
- **Spec-017 v3.0.0 repositioning statement** (Reason: 018 cites it as the shared anchor for
  softening Principle V; it does not re-introduce the rationale. Status: pending/landing with 017.)

## Active Architecture Constraints

- **Kernel `RenderResult` already carries the 4 fields** (`crates/prompting-press-core/src/engine.rs:121-127`);
  its rustdoc says "no telemetry sink, no tracing coupling" — 018 preserves that (formatting only).
- **Each binding surfaces its own `RenderResult`** (py pyclass, napi struct, TS type) 1:1 from the
  kernel; the helper is added per binding, keys/map defined once in the consumer (Principle I).

## Accepted Deviations

- _(none)_

## Relevant Security Constraints

- **No rendered content in telemetry** — the map excludes `text`/`guard`/metadata (FR-007): avoids a
  data-exposure + span-cardinality foot-gun. Aligns with the D2/D3 scrubbing doctrine's spirit.

## Related Historical Lessons

- **Principle V verbatim in code** — the kernel `RenderResult` rustdoc already encodes "no telemetry
  sink, no tracing coupling"; the softening amendment must be precise (formatting ≠ sink).
- **C-08 no speculative config** — fixed keys, no key-mapping knob; consumers wanting variation read
  the public fields.

## Conflict Warnings

- **Soft:** issue #270 proposed a `ProvenanceSink` + built-in `OtelSink` — that WOULD hard-conflict
  with Principle V. Resolved by shipping the projection helper instead (no sink, no dep). No hard
  conflict remains.

## Retrieval Notes

- Read: docs/memory/INDEX.md, D1; kernel engine.rs (fields), binding render.rs (surfacing), TS
  index.ts. Governance: constitution Principle V, C-08. memory-md MCP unavailable → direct reads.
- Budget: within limits (2 decisions, 2 architecture, 0 deviations, 1 security, 2 lessons).
