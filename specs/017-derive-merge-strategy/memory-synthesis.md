# Memory Synthesis

## Current Scope

Spec 017 — add a `merge` strategy (`Replace` default | `Shallow`) to the immutable
`Prompt.derive` primitive, across the Rust consumer (`prompting-press`) and the Python + TS
bindings that delegate to it. Consumer/binding layer only; `prompting-press-core` kernel and the
JSON Schema are untouched. Carries the v3.0.0 constitution repositioning statement + a Principle
VI clarification.

## Relevant Decisions

- **D1 — Cross-binding type parity via canonical serialized form** (Reason Included: 017's
  parity tests must assert equal merged definitions across bindings; compare canonical serialized
  form, not native objects. Status: active. Source: docs/memory/decisions/2026-06-28-canonical-serialized-form-marshaling.md)
- **Spec-008 §9b shallow-replace derive** (Reason Included: 017 extends this exact primitive; the
  default `Replace` path must stay byte-identical to §9b behavior. Status: active/shipped. Source:
  docs/research/registry-value-and-object-model.md §9)

## Active Architecture Constraints

- **Bindings are marshaling shells; ALL logic lives in `prompting_press::Prompt`** (Reason
  Included: merge logic MUST be implemented once in the Rust consumer's `derive`; Py/TS delegate
  via `inner.derive(...)`; no per-binding merge logic. Source: crates/prompting-press-{py,node}/src/prompt.rs
  — verified this session; Principle I/C-01).
- **Kernel is validation-blind + I/O-free; construction re-validates the merged whole** (Reason
  Included: merge runs before `Self::new(merged)`, which enforces agreement/parse/reserved-name;
  merge MUST NOT bypass it. Source: crates/prompting-press/src/prompt.rs:334; Principles III/IV).

## Accepted Deviations

- _(none applicable)_

## Relevant Security Constraints

- _(none directly — 017 adds no I/O and no new error surface carrying bound values; the
  re-validation path and its error scrubbing (D2/D3) are unchanged by merge.)_

## Related Historical Lessons

- **C-11 options-object/keyword-only call shape** (Reason Included: the `merge` selector must be a
  named/keyword param, never a positional mode boolean; Rust uses a `Default` options struct — the
  clarify decision aligns with the C-11 Rust threshold. Source: DECISIONS.md 2026-06-28.)
- **Principle VI compile-time-vs-runtime coverage asymmetry** (Reason Included: `validation_required`
  coverage under merge raises at construction in Py/TS but is a compile-time guarantee in Rust; MUST
  NOT add a Rust runtime coverage throw. Source: constitution Principle VI; DECISIONS.md.)

## Conflict Warnings

- **Soft:** 017 reverses spec-008 §9b's "replace is the only semantic" by adding `Shallow`. Resolved
  by keeping `Replace` the default (non-breaking) and gating `Shallow` behind the explicit selector.
  No hard conflict: §9b chose replace to avoid deep-merge's delete-expressibility problem; 017
  excludes `deep` for the same reason, so the §9b rationale is honored.
- No hard conflicts with constitution principles: 017 stays in-boundary (no I/O, no kernel change,
  agreement check preserved). The Principle VI edit is an additive clarification, not a reversal.

## Retrieval Notes

- Index entries considered: 4 (D1, D2, D3, A1) + INDEX. Read: INDEX.md, D1 (parity). A1 (loader) is
  relevant to spec 019, not 017 — excluded. D2/D3 (error scrubbing) noted as unaffected by merge.
- Governance read: constitution v2.0.0 Principles I/III/IV/VI, DECISIONS.md (C-11 lineage), roadmap
  C-06/C-08/C-11 — already in session context.
- Budget: within limits (2 decisions, 2 architecture constraints, 0 deviations, 0 security, 2
  lessons, <900 words). memory-md MCP tools unavailable this session → read INDEX + entries directly
  per skill fallback.
