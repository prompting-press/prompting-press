# Feature-local memory — spec 017 (derive merge strategy)

Transient notes for this feature. Durable decisions promote to `.specify/memory/DECISIONS.md`
via the amendment (FR-015/016/017) and to `docs/memory/decisions/` at archive time.

## Design provenance (grill session 2026-07-08)

- Issue #271 offered three options: (A) YAML `extends:` var-merge, (B) Jinja `{% extends %}`,
  (C) `with()`/derive already ships. **B killed** (Principle IV excludes extends/import/include).
  **A rejected** — user does not want YAML-declarative inheritance; also drags in base-resolution
  = the deferred registry/container (Q4). **Chosen:** extend the existing `derive` primitive with a
  merge-strategy param (a refinement of C). Code-side only.
- Resolution model for `extends` (had it been A) would have needed an explicit base object /
  def-map / loader — all rejected. `derive` sidesteps this: base is the receiver object.
- Merge shape: **Shape 1** (one method + strategy param), NOT Shape 2 (separate `extend` method).
  Strategy values: **`replace` (default) + `shallow` only**. `deep`/`none` excluded.

## Key facts to preserve

- Shipped method is named **`derive`** (Rust `Prompt::derive`, Py `derive`), TS exposes **`with`**.
  Plan must confirm the TS name and whether the strategy rides its existing options object.
- Rust `PromptOverlay` = data fields only; no runtime validator (validator is generic `V`).
- Py/TS validator carry-forward = R6 (`crates/prompting-press-py/src/prompt.rs:474`).
- Re-validation entry = `Self::new(merged)` (`crates/prompting-press/src/prompt.rs:334`).

## Amendment sequencing

- 017 carries the canonical v3.0.0 **repositioning statement**; 018 (#270) + 019 (#268) cite it.
- 017's own edit = Principle VI clarification (merge-strategy axis; coverage asymmetry preserved).
- Anchoring the MAJOR bump in 017 (first to land) is deliberate — thesis on record before any seam.

## Resolved at clarify (Session 2026-07-08)

1. RESOLVED — Rust surface: a `Default`-implementing **`DeriveOptions { merge: MergeStrategy }`**
   struct (`..Default::default()`), NOT a bare enum param and NOT a `derive_merged` method.
   Non-breaking + forward-extensible + idiomatic defaultable config.
2. RESOLVED — value type: a shared first-class **`MergeStrategy` enum/const** in every binding
   (Py `MergeStrategy.SHALLOW`, TS `MergeStrategy.Shallow`). Python keyword-only `merge=`; TS
   inside the existing derive options object.
3. RESOLVED — default stays `Replace` (non-breaking) in all three.

## Still open → plan time (fine-grained, non-blocking)

4. `variants`/`metadata` union is consistency-driven (primary consumer is `variables`); note in plan.
5. Exact identifiers (struct/field names, TS enum vs const-object) — plan-time naming.
