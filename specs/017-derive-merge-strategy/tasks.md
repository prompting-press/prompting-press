# Tasks: Prompt `derive()` merge strategy

**Feature**: 017-derive-merge-strategy | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 (merge union), US2 (replace-default parity), US3 (add-a-var ergonomics)

## Path Conventions

- Rust consumer (single source of merge logic): `crates/prompting-press/src/prompt.rs`
- PyO3 binding: `crates/prompting-press-py/src/prompt.rs`; Python facade/exports: `packages/python/python/prompting_press/`
- napi binding: `crates/prompting-press-node/src/prompt.rs`; TS facade: `packages/typescript/src/index.ts`
- Conformance corpus: `conformance/`
- Governance: `.specify/memory/constitution.md`, `.specify/memory/DECISIONS.md`, `.specify/memory/roadmap.md`

---

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Confirm baseline is green on branch `017-derive-merge-strategy` (build Rust workspace + Python wheel + npm addon; run existing `derive` tests) so pre-change behavior is captured for the SC-002 parity assertion.
- [X] T002 Record the exact current `derive` signatures + behavior as the "replace/default" golden reference. **DONE (via tests, not a written note):** the default-path parity intent is captured executably by the SC-002 tests (T008/T009 — `derive(overlay)` / `derive_with(.., Replace)` produce byte-identical output to the pre-change behavior; existing suites pass unchanged), which is stronger than a prose baseline. quickstart Scenario 2 documents the default-is-Replace guarantee.

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ Blocks all user stories — the shared merge core + the value type land here.**

- [X] T003 Add `MergeStrategy` enum to the Rust consumer in `crates/prompting-press/src/prompt.rs`: `enum MergeStrategy { Replace, Merge }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]` and `#[default] Replace`. Export it from the consumer crate's public API (bindings marshal it).
- [X] T004 Add `#[derive(Debug, Clone, Default)] struct DeriveOptions { pub strategy: MergeStrategy }` in `crates/prompting-press/src/prompt.rs` (forward-extensible options tail per research R2; field named `strategy`, not `merge`).
- [X] T005 Implement the **shared single-source** `merge_definitions(base: serde_json::Value, overlay: serde_json::Value, strategy: MergeStrategy) -> Result<serde_json::Value, ...>` helper in `crates/prompting-press/src/prompt.rs` (or a sibling module), operating in `serde_json::Value` space (FR-018, research R8): scalars replace when the overlay supplies them; under `Merge` the three map fields (`variables`, `variants`, `metadata`) union top-level keys child-wins-whole-entry (`{...base, ...overlay}`, NO recursion); under `Replace` map fields replace wholesale (today's behavior). This is the ONE union algorithm both the typed path and Node call. Unit-test the helper directly.
- [X] T006 Wire the public Rust surface in `crates/prompting-press/src/prompt.rs`: keep `derive(&self, overlay: PromptOverlay) -> Result<Self>` with its EXACT current signature and `Replace` semantics (serialize def+overlay → `merge_definitions(.., Replace)` → `Self::new`); add `derive_with(&self, overlay: PromptOverlay, options: DeriveOptions) -> Result<Self>` routing through `merge_definitions(.., options.strategy)`. Preserve existing error classes and immutability (INV-1); default path stays byte-identical (INV-2/SC-002).
- [X] T007 [P] Rustdoc on `MergeStrategy`, `DeriveOptions`, `derive`, `derive_with`, `merge_definitions` in `crates/prompting-press/src/prompt.rs`: document the two strategies (merge/replace industry-standard pair), whole-entry-no-recursion semantics, `deep`/`none` deliberately excluded (C-08 reserved axis), the name-only soundness boundary (FR-019), and that the merged whole is re-validated.

**Checkpoint**: Rust consumer compiles; the ONE `merge_definitions` helper + value type exist once (Principle I satisfied). Both the typed path and the Node binding can now call the shared helper.

---

## Phase 3: User Story 2 — Replace is the default, non-breaking (Priority: P1) 🎯 MVP guard

**Goal**: Existing behavior is byte-identical when no strategy is passed; this is the safety net that must hold before shipping the new path.

**Independent Test**: derive with an overlay replacing `variables` and no strategy → base's other-map entries dropped, identical to pre-change output; existing `derive` suites pass unchanged.

- [X] T008 [US2] Add Rust consumer tests in `crates/prompting-press/src/prompt.rs` (test module): `derive(overlay)` and `derive_with(overlay, DeriveOptions::default())` both produce output equal to the pre-change replace behavior for scalar-replace and map-replace cases (SC-002, INV-2).
- [X] T009 [US2] Verify the full existing `derive` test suite passes unmodified across the Rust consumer + `-py` + `-node` crates (no test edits needed for the default path); fix any incidental breakage from the T006 refactor.

**Checkpoint**: Default/replace parity proven in Rust before touching bindings.

---

## Phase 4: User Story 1 — Merge strategy unions variables across all bindings (Priority: P1) 🎯 MVP

**Goal**: A consumer can derive a child that declares the union of base + overlay variables in one call, in all three bindings.

**Independent Test**: base `{extraction}` + overlay `{sentiment}` + `Merge` → `{extraction, sentiment}`; base unchanged; child renders with both.

- [X] T010 [US1] Rust consumer tests in `crates/prompting-press/src/prompt.rs`: `Merge` unions `variables` (US1 scenario 1), child-wins whole-entry on key collision (INV-4 / US1 scenario 2), body referencing merged vars constructs (US1 scenario 3), empty overlay map leaves base map unchanged (INV-5), and a `Merge` that breaks agreement fails at construction (SC-004 / INV-3).
- [X] T010a [US1] Rust soundness tests (FR-019) in `crates/prompting-press/src/prompt.rs`: (a) a `Merge` whose union **removes** a variable that a base **variant** body references → construction FAILS via the agreement check (name-removal caught, across arms); (b) a `Merge` that **replaces** a variable's declaration (type/trust swap) referenced by a base arm → construction SUCCEEDS (accepted; name-only boundary documented).
- [X] T010b [US1] Rust map-coverage tests in `crates/prompting-press/src/prompt.rs`: `Merge` unions `variants` (base variant + overlay variant → both present) and `metadata` (base keys + overlay keys unioned; a `guard`-key collision replaces the base's whole guard entry, whole-entry child-wins — documents metadata inheritance).
- [X] T010c [US1] Rust error-scrubbing test (SEC-001) in `crates/prompting-press/src/prompt.rs`: a failed `Merge` construction yields the SAME scrubbed error class as a failed plain `derive` — no overlay value content leaks into the default error message.
- [X] T011 [US1] PyO3 binding in `crates/prompting-press-py/src/prompt.rs`: add a keyword-only `strategy` arg to `derive` (`#[pyo3(signature = (overlay, *, validators = None, strategy = ...))]`, default `Replace`), marshal the `MergeStrategy` value, and delegate to the Rust `derive`/`derive_with`. Re-check `validation_required` coverage against the MERGED variable set with the effective validator (existing R6 carry-forward); raise the structured error when uncovered (FR-009).
- [X] T012 [P] [US1] Export `MergeStrategy` from the Python facade in `packages/python/python/prompting_press/__init__.py` (importable enum `REPLACE`/`MERGE`); ensure it round-trips through the PyO3 marshaling.
- [X] T013 [US1] **Refactor** the napi binding `derive_prompt` in `crates/prompting-press-node/src/prompt.rs`: accept a `MergeStrategy` value, and **replace the private `shallow_merge_json`** with a call to the shared consumer `merge_definitions(base_json, overlay_json, strategy)` helper (FR-018) → `Prompt::from_json`. Keep the no-`Deserialize` property (helper takes `serde_json::Value`). Re-check `validation_required` coverage against the merged set (FR-009). Remove the now-dead `shallow_merge_json` + `IntoObject` trait if unused.
- [X] T014 [US1] TS facade in `packages/typescript/src/index.ts`: move `derive`'s optional tail into an options object — `derive(overlay, options?: { validators?: ValidatorMap; strategy?: MergeStrategy })` (C-11, research R4; BREAKING at 0.x — `validators` moves inside `{}`). Export a `MergeStrategy` const/union (`Replace`/`Merge`). Default `Replace`. Grep the repo for existing positional `derive(overlay, validators)` call sites (samples/tests) and migrate them.
- [X] T015 [US1] Python parity tests in `packages/python` (pytest): base `{extraction}` + overlay `{sentiment}` + `MergeStrategy.MERGE` → `{extraction, sentiment}`; immutability of base; uncovered `validation_required` under merge raises; a `variants` union and a `metadata` union case.
- [X] T016 [US1] TS parity tests in `packages/typescript` (node:test): same union + immutability + coverage-throw scenarios via the options-object surface (`{ strategy: MergeStrategy.Merge }`), incl. a `variants` and `metadata` union case.
- [X] T017 [US1] Add a `conformance/` case exercising `Merge`; assert equal merged definition (canonical serialized form — decision D1), render, and `template_hash`/`render_hash` across Rust/Python/TS (SC-003, research R7). Include a `metadata` value with a date/decimal to exercise the D1 marshaling the shared helper must keep identical.

**Checkpoint**: US1 works end-to-end in all three bindings via the single shared helper, with cross-binding parity proven.

---

## Phase 5: User Story 3 — Add-a-variable ergonomics (Priority: P2)

**Goal**: `Merge` add-a-key equals the manual-spread `Replace` result — the strategy does the spread, removing the drop-inherited-vars footgun.

**Independent Test**: `derive({variables:{sentiment}}, Merge)` equals `derive({variables:{extraction, sentiment}}, Replace)`.

- [X] T018 [P] [US3] Equivalence test (Rust + one binding) in `crates/prompting-press/src/prompt.rs` and `packages/python` tests: assert the `Merge` add-a-key result equals the manual-spread `Replace` result for the same added variable (US3 scenario 1).

---

## Phase 6: Constitution amendment (v3.0.0)

**⚠️ Governance work carried by this spec (FR-015/016/017). Must land with the code.**

- [X] T019 Amend `.specify/memory/constitution.md`: add the one-time **v3.0.0 repositioning statement** ("minimal, validation-blind core PLUS earned, opt-in seams", motivated by second consumer Bellwether) and the **Principle VI clarification** (`derive` gains a merge-strategy axis; compile-time-vs-runtime coverage asymmetry preserved). Update the version line to 3.0.0 and add the sync-impact report comment (which MUST note the spec-008 FR-017(b) redefinition below).
- [X] T020 Record the amendment in `.specify/memory/DECISIONS.md` (rationale, MAJOR bump → v3.0.0, propagation list) per the Governance amendment policy. **MUST explicitly record the redefinition of spec-008 FR-017(b)** (FR-016.2): the shipped "shallow-replaces each supplied top-level field (no deep merge) … the only way to vary a prompt" is superseded by "overlay MAY union under `MergeStrategy::Merge`" — a backward-incompatible FR redefinition (analogous to spec-015 redefining spec-002's guard invariant), the driver for the MAJOR bump — not merely a Principle VI clause.
- [X] T021 [P] Update `.specify/memory/roadmap.md`: add spec 017 entry (status → in-progress/implemented as appropriate) and note it carries the v3.0.0 repositioning statement that 018/019 cite.
- [X] T022 Regenerate the APM-rendered `CLAUDE.md` + `AGENTS.md` constitution copies (via `apm compile`) so the v3.0.0 body + version match the source (SC-007). **DONE**: `apm compile` (0.23.1) run on the consolidated `017-derive-merge-strategy` branch → both regenerated to v3.0.0 (hash `c459af94f644`); unrelated APM asset regen was restored so only CLAUDE.md/AGENTS.md changed.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T023 [P] Boundary assertion: confirm `git diff` shows NO change to `crates/prompting-press-core/` and `schemas/jsonschema/prompt-definition.schema.json`, and no new dependency in any manifest (SC-006). Add a note/CI check if a cheap one exists.
- [X] T024 [P] Docs: update the derive/inheritance guidance in the docs site + each binding quickstart to show the `strategy` param + `MergeStrategy { Replace, Merge }`, the union semantics, the `deep`/`none`-excluded rationale, the name-only soundness boundary (FR-019), and that `metadata` (incl. a `guard` key the library reads for presence) is inherited/replaced under `Merge` (SEC-002). Docs-are-product rule — current behavior only. Note the TS breaking change (validators → options object) in the changelog/migration note.
- [X] T025 [P] Update `specs/017-derive-merge-strategy/quickstart.md` if any surface detail shifted during implementation (keep it an accurate validation guide). **DONE**: quickstart already reflects the final surface (`derive_with(overlay, DeriveOptions { strategy })`, `strategy=MergeStrategy.MERGE`, `{ strategy: MergeStrategy.Merge }`) from the iterate; verified accurate against the implementation.
- [X] T026 Run the full workspace test + conformance gate; confirm all green including the new `Merge` conformance case.

---

## Dependencies & Execution Order

- **Phase 1 (Setup)** → **Phase 2 (Foundational)**: T003–T007 are the shared core; everything else depends on them.
- **Phase 3 (US2 parity)** should complete right after Phase 2 — it's the non-breaking safety net (guard the MVP before extending).
- **Phase 4 (US1)** depends on Phase 2; T011/T013 (bindings) depend on T005/T006; T012 depends on T011; T014 depends on T013; T015/T016 depend on their bindings; T017 depends on all three bindings.
- **Phase 5 (US3)** depends on Phase 4 (`Merge` must exist).
- **Phase 6 (amendment)** can proceed in parallel with code (T019–T022) but MUST land in the same change set.
- **Phase 7 (polish)** last; T026 gates completion.

## Parallel Opportunities

- T007 (rustdoc) ∥ T008 (US2 tests) once T006 lands.
- T012 (Python export) ∥ T013 (napi binding) — different files.
- T015 ∥ T016 (Python vs TS parity tests) — different packages.
- T019–T022 (governance) ∥ the code phases — different files.
- T023/T024/T025 polish tasks are mutually parallel.

## Implementation Strategy

- **MVP = Phase 2 + Phase 3 + Phase 4** (shared core + replace-parity guard + merge union across bindings). That delivers the motivating Bellwether use case with the non-breaking guarantee proven.
- US3 (Phase 5) is a thin equivalence proof; the amendment (Phase 6) is mandatory governance; polish (Phase 7) closes boundary + docs.
- Rust-first: implement + test the merge in the consumer before any binding touches it (Principle I — one source of logic).

## Task Summary

- **Total tasks**: 26
- **US1 (merge union)**: T010–T017 + T010a/b/c (11) — the core capability, all bindings + conformance
- **US2 (replace parity)**: T008–T009 (2) — non-breaking guard
- **US3 (ergonomics)**: T018 (1)
- **Setup/Foundational**: T001–T007 (7) — shared merge core + value type
- **Amendment**: T019–T022 (4)
- **Polish**: T023–T026 (4)
- **Suggested MVP scope**: Phases 2–4 (US1 + US2).
