# Memory synthesis — spec 017 (derive merge strategy)

Compact planning context distilled from the governance layer (constitution v2.0.0,
DECISIONS.md, roadmap v1.4.0) and the shipped `derive` code. Source of truth; ≤900 words.

## What already ships (verified in code)

- `Prompt` is an **immutable value object** (spec 008). The sole mutator is
  **`Prompt::derive(overlay) -> Result<Prompt>`** (`crates/prompting-press/src/prompt.rs:307`),
  exposed as `derive(overlay, validators=)` in Python (`crates/prompting-press-py/src/prompt.rs:445`)
  and `with(overlay, validators?)` in TypeScript (`packages/typescript/src/index.ts`).
- Today's `derive` semantics: **shallow REPLACE per top-level field** — each `Some`/present
  overlay field replaces that whole field; absent fields untouched. Then the **whole merged
  definition is re-validated through the same constructor** (`Self::new(merged)`): reserved-name
  check, per-arm `required_roots` parse, agreement check (referenced roots ⊆ declared variables).
- `PromptOverlay` (Rust) carries **only data fields** (`name, role, body, variables, variants,
  output_model, metadata`) — no runtime validator object (Rust validator is the generic `V`).
- Python/TS validator **carry-forward rule (R6)**: overlay supplies validators ⇒ use them; else
  carry forward `self.validators`. Coverage of `validation_required` vars is re-checked against the
  derived definition with the effective validator (`check_validator_coverage`).
- `metadata`/`variants` are **library-opaque** (`additionalProperties: true`); stored + echoed,
  never interpreted (Principle III, schema).

## What spec 017 adds

- A **keyword-only `merge` strategy** param on `derive`: `"replace"` (default = today) | `"shallow"`.
- `"shallow"`: the three **map** fields (`variables`, `variants`, `metadata`) **union at top-level
  keys, child-wins whole-entry** (`{...base, ...overlay}` per map; NO recursion into an entry).
  Scalar fields (`name, role, body, output_model`) **always replace** when overlay-present.
- Merged whole re-validated through the **same** path (unchanged). Coverage re-checked against the
  **merged** variable set (Py/TS runtime raise/throw; Rust compile-time via `V`).

## Governing constraints (must hold)

- **Principle III / C-03** — no I/O, no kernel change. 017 is **consumer/binding-layer only**;
  `prompting-press-core` is untouched. Merge is a pure in-memory data operation before re-validation.
- **Principle IV / C-04** — the agreement check stays sound; it runs on the merged whole, unchanged.
- **Principle VI / C-06** — native idiom + the **compile-time-vs-runtime coverage asymmetry**:
  Python (Pydantic) / TS (Zod) enforce `validation_required` coverage at construction (raise/throw);
  Rust guarantees it **structurally at compile time** via `V` — MUST NOT add a runtime coverage
  throw. This asymmetry is endorsed, not a defect (DECISIONS 2026-06-28).
- **C-11** — the `merge` strategy is a **keyword-only / named param** (Python `*, merge=`; TS options
  object; Rust — a single added param is below the 2+ options-struct threshold, stays positional/
  named-enum), NOT a positional mode boolean. No `null`-soup.
- **C-07** — no schema change (no `extends:` field). The prompt-definition JSON Schema is untouched;
  no codegen re-run needed.
- **C-08 / Scope Discipline** — `deep` and `none` strategies are **deliberately excluded**; the enum
  axis is reserved for a future consumer-earned addition (no new method needed). Excluding them is
  the discipline, not a gap.

## Constitution amendment carried by 017

- **v3.0.0 repositioning statement (one-time, canonical here):** Prompting Press deliberately
  relaxes its "minimal core that never grows into a framework" thesis → "a minimal, validation-blind
  core PLUS **earned, opt-in seams**" (driven by a real second consumer, Bellwether). Specs 018
  (#270 provenance-attributes) and 019 (#268 pluggable loader) **cite** this statement while making
  their own per-principle edits.
- **017's own concrete edit:** a **Principle VI clarification** — `derive` gains a merge-strategy
  axis; the coverage asymmetry is preserved. Recorded in DECISIONS.md. Version bump reflects the
  repositioning statement (MAJOR → v3.0.0), not 017's code size.

## Motivating consumer

Bellwether `trend_value` strategy: a base prompt declares shared `extraction` (untrusted) variable;
children (`bull`, `bear`, `valuation`) add their own (e.g. `sentiment`) via
`base.derive({variables: {sentiment: {...}}}, merge="shallow")` → `{extraction, sentiment}`.
Today this requires manual spread `{...base.variables, sentiment}`; `shallow` makes it the semantic.

## Open questions for clarify

1. Rust surface for the strategy: a `MergeStrategy` enum param on `derive`, or a second method
   `derive_merged`? (Lean: enum param — keeps one method, matches Py/TS.)
2. Does `derive` today expose `merge` via the TS options object already present (`with(overlay, validators?)`)
   — i.e. does it become `with(overlay, { validators?, merge? })`? (C-11 options-object.)
3. Default confirmation: `"replace"` default is non-breaking; confirm we don't flip default to shallow.
