# Quickstart / Validation: provenance attributes helper

Runnable scenarios; full test code in tasks.md. See [contracts](./contracts/provenance-attributes.md).

## Prerequisites
- Workspace builds green; no new dependencies (verify manifest diff).

## Scenario 1 — One-call span attachment (US1)
Render a prompt; call the helper; assert a 4-entry map under `gen_ai.prompt.{name,variant,
template_hash,render_hash}` whose values equal the result fields.
- Rust: `let attrs = result.provenance_attributes();`
- Python: `span.set_attributes(result.provenance_attributes())`
- TS: `span.setAttributes(result.provenanceAttributes())`
Expected: four attributes present, values match; no other keys.

## Scenario 2 — No telemetry dependency / no emission (US2)
Inspect manifests before/after: no OTel/telemetry dep added (FR-006). Assert render fires no
callback and has no side effect beyond returning the result (FR-005).

## Scenario 3 — Exclusions (FR-007)
Assert the map never contains rendered `text`, `guard` text, metadata, or `output_model` — exactly
four keys.

## Scenario 4 — Default variant
Render with no variant selected → `gen_ai.prompt.variant == "default"` (never empty/absent).

## Scenario 5 — Custom keys still possible (US3)
Read the four public fields directly and build a custom-keyed map without the helper (additive).

## Scenario 6 — Cross-binding parity (SC-003)
Same render through all three bindings → identical map (canonical serialized values, D1).

## Boundary checks (SC-002/SC-006)
- No telemetry/observability dependency in any manifest.
- `prompting-press-core` unchanged (no diff).

## Amendment check (SC-007)
- Constitution Principle V softened (formatting allowed; no sink/push/dep), cites spec-017
  repositioning; recorded in DECISIONS.md; rendered copies in sync.
