# Tasks: Provenance attributes helper

**Feature**: 018-provenance-attributes | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 (one-call span attach), US2 (no telemetry coupling), US3 (custom keys preserved)

## Path Conventions

- Consumer (shared keys + helper): `crates/prompting-press/src/` (render/result module)
- PyO3 binding: `crates/prompting-press-py/src/render.rs`; Python facade: `packages/python/python/prompting_press/`
- napi binding: `crates/prompting-press-node/src/render.rs`; TS facade: `packages/typescript/src/index.ts`
- Conformance: `conformance/`
- Governance: `.specify/memory/{constitution.md,DECISIONS.md,roadmap.md}`

---

## Phase 1: Setup

- [ ] T001 Confirm baseline green on `018-provenance-attributes` (build Rust workspace + wheel + addon; run existing render tests). Confirm kernel `RenderResult` carries `name`/`variant`/`template_hash`/`render_hash` (`crates/prompting-press-core/src/engine.rs:121-127`) — no kernel change will be made.

## Phase 2: Foundational (shared key constants + helper)

**⚠️ Blocks all bindings — define the keys + map shape ONCE (Principle I, research R1).**

- [ ] T002 Add the four `pub const` key strings (`prompting_press.prompt.name`, `.variant`, `.template_hash`, `.render_hash`) in the consumer crate `crates/prompting-press/src/` (render/result module).
- [ ] T003 Add a shared **free function** `provenance_attributes_of(name, variant, template_hash, render_hash) -> BTreeMap<String,String>` in the consumer building exactly the four entries. **NOT an inherent `impl RenderResult` method** — the consumer re-exports the kernel `RenderResult` (lib.rs:178), so that is E0116. Also add an optional **extension trait** `ProvenanceExt` impl'd for `RenderResult` so Rust callers can write `result.provenance_attributes()` (requires `use prompting_press::ProvenanceExt`). Pure: no I/O, no mutation (FR-004). Unit-test both (exactly 4 entries; values match; deterministic order).

**Checkpoint**: keys + map builder exist once; bindings mirror/delegate.

## Phase 3: User Story 1 — One-call span attachment (Priority: P1) 🎯 MVP

**Independent Test**: render → helper → 4-entry `prompting_press.prompt.*` map whose values equal the result fields; passable to a span's bulk set-attributes.

- [ ] T004 [US1] Add `provenance_attributes()` to the Python `RenderResult` pyclass in `crates/prompting-press-py/src/render.rs`, returning `dict[str,str]` from the shared keys/values. Ensure it's exposed on the Python-visible type.
- [ ] T005 [US1] Add `provenanceAttributes()` to the napi `RenderResult` in `crates/prompting-press-node/src/render.rs`, returning a `Record<string,string>`-marshaled map from the shared keys/values.
- [ ] T006 [US1] Surface `provenanceAttributes(): Record<string,string>` on the TS `RenderResult` type in `packages/typescript/src/index.ts` (+ any `.d.ts`/type re-export).
- [ ] T007 [P] [US1] Rust consumer test: `result.provenance_attributes()` returns the 4 expected keys/values; default-variant render yields `prompting_press.prompt.variant == "default"` (INV-3).
- [ ] T008 [P] [US1] Python test (`packages/python`): `result.provenance_attributes()` is a 4-entry dict with the fixed keys; values equal `result.name/variant/template_hash/render_hash`.
- [ ] T009 [P] [US1] TS test (`packages/typescript`): `result.provenanceAttributes()` is a 4-entry record with the fixed keys/values.
- [ ] T010 [US1] Conformance case in `conformance/`: identical render through all three bindings → identical provenance map (canonical serialized form, D1) (SC-003).

**Checkpoint**: US1 works in all three bindings with parity proven.

## Phase 4: User Story 2 — No telemetry coupling / no emission (Priority: P1)

- [ ] T011 [US2] Assert (test + manifest inspection) that NO telemetry/observability dependency is added in any package/crate manifest (FR-006, SC-002). Add a cheap CI/manifest check if one exists.
- [ ] T012 [US2] Test that the helper is a pure projection: no I/O, mutates nothing, and render invokes no caller callback / has no side effect beyond returning the result (FR-004/FR-005). Assert the map excludes `text`/`guard`/metadata/`output_model` — EXACTLY 4 keys, **per binding**; document the helper is an explicit allowlist, NOT a reflection (a future `RenderResult` field cannot leak) (FR-007, SC-004, security SEC-001).

## Phase 5: User Story 3 — Custom keys preserved (Priority: P2)

- [ ] T013 [P] [US3] Test (one binding) that the four provenance fields remain publicly readable and a consumer can build a custom-keyed map without the helper (FR-008; helper is additive).

## Phase 6: Constitution amendment (additive → v3.1.0, on 017's v3.0.0 baseline)

**⚠️ Governance (FR-012/013/014). Cites spec-017's repositioning statement.**

- [ ] T014 Amend `.specify/memory/constitution.md` Principle V: the library MAY **format** provenance into a flat attribute map (still data on the return value); still **no telemetry sink, no push/emit, no telemetry dependency**; the two hashes + semantics unchanged. **Cite the spec-017 v3.0.0 repositioning statement** (do not re-derive it). Bump the version → **v3.1.0** (additive Principle V softening on top of 017's v3.0.0; 019 is v3.2.0). Do NOT re-declare v3.0.0. Update the sync-impact note. Merge AFTER 017.
- [ ] T015 Record in `.specify/memory/DECISIONS.md`: Principle V softening rationale + that it cites spec-017's repositioning; note it does NOT add a sink (rejects issue #270's `ProvenanceSink`/`OtelSink`).
- [ ] T016 [P] Update `.specify/memory/roadmap.md`: add spec 018 entry (status) noting it cites 017's v3.0.0 repositioning and lands as additive v3.1.0.
- [ ] T017 Regenerate the APM-rendered `CLAUDE.md`/`AGENTS.md` constitution copies so they match the amended source (SC-007); if `apm compile` isn't runnable, note for the reviewer.

## Phase 7: Polish & Cross-Cutting

- [ ] T018 [P] Boundary assertion: `git diff` shows NO change to `crates/prompting-press-core/` (SC-006) and no new dependency in any manifest.
- [ ] T019 [P] Docs: document `provenance_attributes()`/`provenanceAttributes()` in the docs site + each binding quickstart, incl. the fixed library-owned `prompting_press.prompt.*` keys, that they are NOT OTel-convention keys (a consumer may remap onto their tracer's convention), that `render_hash` is a content-identifier not the content (SEC-002), and the Rust `use ProvenanceExt` requirement. Docs-are-product — current behavior only.
- [ ] T020 Run full workspace test + conformance gate; confirm green including the new parity case.

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 (shared keys+helper) → everything.
- Phase 3 bindings (T004/T005/T006) depend on T002/T003; their tests (T007-T009) follow; T010 depends on all three.
- Phase 4/5 depend on the helper existing.
- Phase 6 (amendment) parallel with code but lands in the same change set; additive v3.1.0 on 017's v3.0.0; merge after 017.
- Phase 7 last; T020 gates completion.

## Parallel Opportunities

- T007/T008/T009 (per-binding tests) mutually parallel.
- T014-T016 (governance) parallel with code.
- T018/T019 polish parallel.

## Implementation Strategy

- **MVP = Phase 2 + Phase 3** (shared helper + all-binding method + parity). Delivers the one-call
  span attachment. Phase 4 is the boundary guard (no dep/no emission); Phase 6 the mandatory amendment.

## Task Summary

- **Total**: 20 tasks
- **US1**: T004–T010 (7) — the helper across bindings + parity
- **US2**: T011–T012 (2) — no-dep / no-emission / exclusions
- **US3**: T013 (1)
- **Setup/Foundational**: T001–T003 (3)
- **Amendment**: T014–T017 (4)
- **Polish**: T018–T020 (3)
- **MVP scope**: Phases 2–3.
