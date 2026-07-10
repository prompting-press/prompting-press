# Cross-spec conflict check — 017 / 018 / 019 (2026-07-08)

Run manually (the three specs are on separate branches; `sync.conflicts` reads one active feature
dir, so compared via `git show` across branch tips). Tips: 017 `28635bd`, 018 `3d08989`, 019 `8cc2555`.

## Verdict: NO HARD CONFLICTS. One hard merge-order constraint; clean additive merges.

## Constitution version layering — COHERENT
- 017 → **v3.0.0**: writes the one-time repositioning statement + Principle VI clarification + spec-008
  FR-017(b) redefinition.
- 018 → **v3.1.0**: CITES 017's repositioning; adds ONLY the Principle V softening (format provenance;
  no sink/dep).
- 019 → **v3.2.0**: CITES 017's repositioning; adds ONLY Principle III softening + C-08 Loader re-scope
  + the error-taxonomy compatibility-surface expansion.
- Each edits a DISTINCT principle (VI / V / III+C-08). The repositioning statement is authored once
  (017) and only cited by 018/019 — no double-authoring, no contradictory edits.

## Merge-order constraint (HARD, documented in all three)
**017 → 018 → 019.** Each later spec's amendment assumes the prior version line. If merged out of
order, the citing spec's anchor dangles and the version line conflicts. Each rebases the version line
on merge.

## Shared code surface — NO FUNCTIONAL OVERLAP
- 017: `merge_definitions` / `MergeStrategy` / `DeriveOptions` / `derive_with` in prompt.rs.
- 018: `provenance_attributes_of` + `ProvenanceExt` + `prompting_press.prompt.*` consts (render/result).
- 019: loader module + `PromptLoadError` + `load_io`/`load_not_found` in error.rs.
- Only 019 touches the closed `code` vocab (018's keys are attribute strings, not error codes) — no
  code-name collision. `lib.rs` export additions are additive across all three. Merges clean in order.

## Residual risk
- Purely mechanical: on merge, each spec rebases the constitution version line + re-runs `apm compile`
  to regenerate CLAUDE.md/AGENTS.md from the new source. Not a design conflict.
