# Tasks: Prompt `derive()` merge strategy

**Feature**: 017-derive-merge-strategy | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 (shallow union), US2 (replace-default parity), US3 (add-a-var ergonomics)

## Path Conventions

- Rust consumer (single source of merge logic): `crates/prompting-press/src/prompt.rs`
- PyO3 binding: `crates/prompting-press-py/src/prompt.rs`; Python facade/exports: `packages/python/python/prompting_press/`
- napi binding: `crates/prompting-press-node/src/prompt.rs`; TS facade: `packages/typescript/src/index.ts`
- Conformance corpus: `conformance/`
- Governance: `.specify/memory/constitution.md`, `.specify/memory/DECISIONS.md`, `.specify/memory/roadmap.md`

---

## Phase 1: Setup (Shared Infrastructure)

- [ ] T001 Confirm baseline is green on branch `017-derive-merge-strategy` (build Rust workspace + Python wheel + npm addon; run existing `derive` tests) so pre-change behavior is captured for the SC-002 parity assertion.
- [ ] T002 Record the exact current `derive` signatures + behavior (Rust `crates/prompting-press/src/prompt.rs:307`, Py `crates/prompting-press-py/src/prompt.rs:445`, TS `packages/typescript/src/index.ts:692`) as the "replace/default" golden reference in `specs/017-derive-merge-strategy/quickstart.md` Scenario 2 baseline notes.

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ Blocks all user stories — the shared merge core + the value type land here.**

- [ ] T003 Add `MergeStrategy` enum to the Rust consumer in `crates/prompting-press/src/prompt.rs`: `enum MergeStrategy { Replace, Shallow }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]` and `#[default] Replace`.
- [ ] T004 Add `#[derive(Debug, Clone, Default)] struct DeriveOptions { pub merge: MergeStrategy }` in `crates/prompting-press/src/prompt.rs` (forward-extensible options tail per research R2).
- [ ] T005 Implement the strategy-aware merge in the Rust consumer `crates/prompting-press/src/prompt.rs`: a private helper that, given `&self.def`, `overlay: PromptOverlay`, and `MergeStrategy`, produces the merged `PromptDefinition` per the data-model algorithm — scalars replace when present; under `Shallow` the three map fields (`variables`, `variants`, `metadata`) union top-level keys child-wins-whole-entry (`{...base, ...overlay}`, NO recursion); under `Replace` map fields replace wholesale (today's behavior). Route the merged def through the existing `Self::new(merged)` validating constructor (unchanged).
- [ ] T006 Wire the public Rust surface in `crates/prompting-press/src/prompt.rs`: keep `derive(&self, overlay) -> Result<Self>` meaning `Replace` (delegates to the helper with `DeriveOptions::default()`), and add `derive_with(&self, overlay, options: DeriveOptions) -> Result<Self>` (delegates with the given strategy). Preserve the exact existing error classes and immutability (INV-1).
- [ ] T007 [P] Rustdoc on `MergeStrategy`, `DeriveOptions`, `derive`, `derive_with` in `crates/prompting-press/src/prompt.rs`: document the two strategies, whole-entry-no-recursion semantics, `deep`/`none` deliberately excluded (C-08 reserved axis), and that the merged whole is re-validated.

**Checkpoint**: Rust consumer compiles; merge logic + value type exist once (Principle I satisfied). Bindings can now delegate.

---

## Phase 3: User Story 2 — Replace is the default, non-breaking (Priority: P1) 🎯 MVP guard

**Goal**: Existing behavior is byte-identical when no strategy is passed; this is the safety net that must hold before shipping the new path.

**Independent Test**: derive with an overlay replacing `variables` and no strategy → base's other-map entries dropped, identical to pre-change output; existing `derive` suites pass unchanged.

- [ ] T008 [US2] Add Rust consumer tests in `crates/prompting-press/src/prompt.rs` (test module): `derive(overlay)` and `derive_with(overlay, DeriveOptions::default())` both produce output equal to the pre-change replace behavior for scalar-replace and map-replace cases (SC-002, INV-2).
- [ ] T009 [US2] Verify the full existing `derive` test suite passes unmodified across the Rust consumer + `-py` + `-node` crates (no test edits needed for the default path); fix any incidental breakage from the T006 refactor.

**Checkpoint**: Default/replace parity proven in Rust before touching bindings.

---

## Phase 4: User Story 1 — Shallow merge unions variables across all bindings (Priority: P1) 🎯 MVP

**Goal**: A consumer can derive a child that declares the union of base + overlay variables in one call, in all three bindings.

**Independent Test**: base `{extraction}` + overlay `{sentiment}` + `Shallow` → `{extraction, sentiment}`; base unchanged; child renders with both.

- [ ] T010 [US1] Rust consumer tests in `crates/prompting-press/src/prompt.rs`: `Shallow` unions `variables` (US1 scenario 1), child-wins whole-entry on key collision (INV-4 / US1 scenario 2), body referencing merged vars constructs (US1 scenario 3), empty overlay map leaves base map unchanged (INV-5), and a `Shallow` merge that breaks agreement fails at construction (SC-004 / INV-3).
- [ ] T011 [US1] PyO3 binding in `crates/prompting-press-py/src/prompt.rs`: add a keyword-only `merge` arg to `derive` (`#[pyo3(signature = (overlay, *, validators = None, merge = MergeStrategy::Replace))]` equivalent), marshal the `MergeStrategy` value, and delegate to the Rust `derive`/`derive_with`. Re-check `validation_required` coverage against the MERGED variable set with the effective validator (existing R6 carry-forward); raise the structured error when uncovered (FR-009).
- [ ] T012 [P] [US1] Export `MergeStrategy` from the Python facade in `packages/python/python/prompting_press/__init__.py` (importable enum `REPLACE`/`SHALLOW`); ensure it round-trips through the PyO3 marshaling.
- [ ] T013 [US1] napi binding in `crates/prompting-press-node/src/prompt.rs`: accept a `MergeStrategy` value on the derive path, marshal it, and delegate to the Rust `derive`/`derive_with`. Re-check `validation_required` coverage against the merged set (FR-009).
- [ ] T014 [US1] TS facade in `packages/typescript/src/index.ts`: move `derive`'s optional tail into an options object — `derive(overlay, options?: { validators?: ValidatorMap; merge?: MergeStrategy })` (C-11, research R4; BREAKING at 0.x — `validators` moves inside `{}`). Export a `MergeStrategy` const/union. Default `Replace`.
- [ ] T015 [US1] Python parity tests in `packages/python` (pytest): base `{extraction}` + overlay `{sentiment}` + `MergeStrategy.SHALLOW` → `{extraction, sentiment}`; immutability of base; uncovered `validation_required` under merge raises.
- [ ] T016 [US1] TS parity tests in `packages/typescript` (node:test): same shallow-union + immutability + coverage-throw scenarios via the options-object surface.
- [ ] T017 [US1] Add a `conformance/` case exercising `Shallow` merge; assert equal merged definition (canonical serialized form — decision D1), render, and `template_hash`/`render_hash` across Rust/Python/TS (SC-003, research R7).

**Checkpoint**: US1 works end-to-end in all three bindings with cross-binding parity proven.

---

## Phase 5: User Story 3 — Add-a-variable ergonomics (Priority: P2)

**Goal**: `Shallow` add-a-key equals the manual-spread `Replace` result — the strategy does the spread, removing the drop-inherited-vars footgun.

**Independent Test**: `derive({variables:{sentiment}}, Shallow)` equals `derive({variables:{extraction, sentiment}}, Replace)`.

- [ ] T018 [P] [US3] Equivalence test (Rust + one binding) in `crates/prompting-press/src/prompt.rs` and `packages/python` tests: assert the `Shallow` add-a-key result equals the manual-spread `Replace` result for the same added variable (US3 scenario 1).

---

## Phase 6: Constitution amendment (v3.0.0)

**⚠️ Governance work carried by this spec (FR-015/016/017). Must land with the code.**

- [ ] T019 Amend `.specify/memory/constitution.md`: add the one-time **v3.0.0 repositioning statement** ("minimal, validation-blind core PLUS earned, opt-in seams", motivated by second consumer Bellwether) and the **Principle VI clarification** (`derive` gains a merge-strategy axis; compile-time-vs-runtime coverage asymmetry preserved). Update the version line to 3.0.0 and add the sync-impact report comment.
- [ ] T020 Record the amendment in `.specify/memory/DECISIONS.md` (rationale, MAJOR bump → v3.0.0, propagation list) per the Governance amendment policy.
- [ ] T021 [P] Update `.specify/memory/roadmap.md`: add spec 017 entry (status → in-progress/implemented as appropriate) and note it carries the v3.0.0 repositioning statement that 018/019 cite.
- [ ] T022 Regenerate the APM-rendered `CLAUDE.md` + `AGENTS.md` constitution copies (via `apm compile` or the project's regen path) so the v3.0.0 body + version match the source (SC-007).

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T023 [P] Boundary assertion: confirm `git diff` shows NO change to `crates/prompting-press-core/` and `schemas/jsonschema/prompt-definition.schema.json`, and no new dependency in any manifest (SC-006). Add a note/CI check if a cheap one exists.
- [ ] T024 [P] Docs: update the derive/inheritance guidance in the docs site + each binding quickstart to show the `merge` strategy, `Shallow` semantics, and the `deep`/`none`-excluded rationale (docs-are-product rule — current behavior only).
- [ ] T025 [P] Update `specs/017-derive-merge-strategy/quickstart.md` if any surface detail shifted during implementation (keep it an accurate validation guide).
- [ ] T026 Run the full workspace test + conformance gate; confirm all green including the new `Shallow` conformance case.

---

## Dependencies & Execution Order

- **Phase 1 (Setup)** → **Phase 2 (Foundational)**: T003–T007 are the shared core; everything else depends on them.
- **Phase 3 (US2 parity)** should complete right after Phase 2 — it's the non-breaking safety net (guard the MVP before extending).
- **Phase 4 (US1)** depends on Phase 2; T011/T013 (bindings) depend on T005/T006; T012 depends on T011; T014 depends on T013; T015/T016 depend on their bindings; T017 depends on all three bindings.
- **Phase 5 (US3)** depends on Phase 4 (Shallow must exist).
- **Phase 6 (amendment)** can proceed in parallel with code (T019–T022) but MUST land in the same change set.
- **Phase 7 (polish)** last; T026 gates completion.

## Parallel Opportunities

- T007 (rustdoc) ∥ T008 (US2 tests) once T006 lands.
- T012 (Python export) ∥ T013 (napi binding) — different files.
- T015 ∥ T016 (Python vs TS parity tests) — different packages.
- T019–T022 (governance) ∥ the code phases — different files.
- T023/T024/T025 polish tasks are mutually parallel.

## Implementation Strategy

- **MVP = Phase 2 + Phase 3 + Phase 4** (shared core + replace-parity guard + shallow union across bindings). That delivers the motivating Bellwether use case with the non-breaking guarantee proven.
- US3 (Phase 5) is a thin equivalence proof; the amendment (Phase 6) is mandatory governance; polish (Phase 7) closes boundary + docs.
- Rust-first: implement + test the merge in the consumer before any binding touches it (Principle I — one source of logic).

## Task Summary

- **Total tasks**: 26
- **US1 (shallow union)**: T010–T017 (8) — the core capability, all bindings + conformance
- **US2 (replace parity)**: T008–T009 (2) — non-breaking guard
- **US3 (ergonomics)**: T018 (1)
- **Setup/Foundational**: T001–T007 (7) — shared merge core + value type
- **Amendment**: T019–T022 (4)
- **Polish**: T023–T026 (4)
- **Suggested MVP scope**: Phases 2–4 (US1 + US2).
