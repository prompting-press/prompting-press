# Quickstart / Validation: derive() merge strategy

Runnable scenarios proving the feature end-to-end. Full test code lives in `tasks.md` /
implementation; this is the validation guide. See [contracts/derive-merge.md](./contracts/derive-merge.md)
and [data-model.md](./data-model.md) for the surface + algorithm.

## Prerequisites

- Workspace builds green (`cargo build`, the Python wheel, the npm addon) on the feature branch.
- No new dependencies required.

## Scenario 1 — Merge strategy unions variables (US1, the motivating case)

**Setup:** a base prompt declaring `variables: { extraction }`; an overlay adding
`variables: { sentiment }` and a body referencing both.

**Run:** derive with `Merge`.

- Rust: `base.derive_with(overlay, DeriveOptions { strategy: MergeStrategy::Merge })?`
- Python: `base.derive(overlay, strategy=MergeStrategy.MERGE)`
- TS: `base.derive(overlay, { strategy: MergeStrategy.Merge })`

**Expected:** derived prompt declares `{ extraction, sentiment }`; base still declares
`{ extraction }` (immutability); rendering the child with both variables succeeds.

## Scenario 2 — Default is Replace (US2, non-breaking)

**Setup:** same base; overlay `variables: { sentiment }`.

**Run:** derive with NO strategy (Rust `derive(overlay)`, Python `derive(overlay)`, TS
`derive(overlay)`).

**Expected:** derived prompt declares exactly `{ sentiment }` (base's `extraction` dropped) —
identical to today. Existing `derive` test suites pass unchanged.

## Scenario 3 — Child wins whole-entry on key collision (US1 scenario 2 / INV-4)

**Setup:** base `variables: { extraction: {type: string, trusted: false} }`; overlay
`variables: { extraction: {type: string, trusted: true} }`.

**Run:** derive with `Merge`.

**Expected:** derived `extraction` == the overlay's entry (`trusted: true`), whole-entry — no
field-level merge inside the entry.

## Scenario 4 — Merge that breaks agreement fails at construction (SC-004 / INV-3)

**Setup:** base body references `{{ extraction }}`; overlay `body: "... {{ missing }}"` with no
`missing` in base or overlay variables; `Merge`.

**Expected:** construction fails with the structured agreement error (referenced root not in the
merged declared variables) — no silent acceptance.

## Scenario 5 — validation_required coverage against the merged set (FR-009)

**Setup (Python/TS):** overlay adds a `validation_required` variable the effective validator does
not cover; `Merge`.

**Expected:** raises/throws at construction (coverage evaluated over the merged variable set).
**Rust:** the analogous mismatch is a compile-time error at the `render::<V>` site — NO runtime
coverage throw.

## Scenario 6 — Cross-binding parity (SC-003 / R7)

**Run:** the same base + overlay + `Merge` through all three bindings; capture the merged
definition (canonical serialized form) + render + `template_hash`/`render_hash`.

**Expected:** all three produce equal serialized merged definitions and equal hashes (conformance
corpus case).

## Boundary checks (SC-006)

- `git diff` shows **no change** to `crates/prompting-press-core/` or
  `schemas/jsonschema/prompt-definition.schema.json`.
- No new dependency in any manifest.

## Amendment check (SC-007)

- Constitution reflects the v3.0.0 repositioning statement + Principle VI merge-strategy
  clarification; `DECISIONS.md` records the amendment; rendered `CLAUDE.md`/`AGENTS.md` in sync.
