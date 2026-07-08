# Tasks: Pluggable prompt loader

**Feature**: 019-pluggable-loader | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no incomplete deps)
- **[Story]**: US1 (filesystem behind interface), US2 (memory loader for tests), US3 (custom loader)

## Path Conventions

- Rust consumer: `crates/prompting-press/src/` (new loader module) + `error.rs`
- Python facade: `packages/python/python/prompting_press/`
- TS facade: `packages/typescript/src/`
- Governance: `.specify/memory/{constitution.md,DECISIONS.md,roadmap.md}`

---

## Phase 1: Setup

- [ ] T001 Confirm baseline green on `019-pluggable-loader`. Confirm the existing text factories (`crates/prompting-press/src/prompt.rs:108+` from_yaml/json/toml) and the error family (`error.rs:97-109` `ConsumerError::Load` + `[{field,code,message}]`) — the loader composes with / normalizes into these.

## Phase 2: Foundational (Rust interface + error + built-ins)

**⚠️ The canonical interface lands here.**

- [ ] T002 Add `LoadError` in the Rust consumer (`crates/prompting-press/src/error.rs` or loader module) that normalizes into the common `[{field,code,message}]` family (codes `not_found`/`io`), distinct from parse/validation errors (FR-007/FR-008). Ensure default messages are scrubbed (no file contents / secret-looking values — D2/D3 lineage).
- [ ] T003 Add the loader module in `crates/prompting-press/src/`: object-safe `trait PromptLoader { fn load(&self,key:&str)->Result<String,LoadError>; }` + blanket impl for `Fn(&str)->Result<String,LoadError>+Send+Sync` (FR-001).
- [ ] T004 Implement `FileSystemLoader { base, suffix=".yaml" }`: `load(key)` reads `{base}/{key}{suffix}`; **reject keys escaping `base`** (`..`/absolute) with `LoadError` via canonicalize + prefix-check (FR-002/FR-002a/SC-008). Missing file → `LoadError` (FR-006).
- [ ] T005 Implement `MemoryLoader` from a `key→text` map; miss → `LoadError` (FR-003/FR-006).
- [ ] T006 [P] Rust tests: FileSystemLoader hit + miss; **traversal guard** (`../secret` → LoadError, no outside read, SC-008); MemoryLoader hit + miss; closure-as-loader (blanket impl); load-error ≠ parse-error (compose with `from_yaml` and show distinct surfaces, FR-007); scrubbed error message.
- [ ] T007 [P] Rustdoc: interface contract, `load` returns raw text (not a Prompt), missing-key/error behavior, traversal guard, sync nature, and the "compose with from_yaml; not fused" note (FR-005/FR-011).

**Checkpoint**: Rust interface + built-ins + LoadError exist, traversal-guarded, tested.

## Phase 3: User Story 1 — Filesystem behind a swappable interface (Priority: P1) 🎯 MVP

- [ ] T008 [US1] Python: `PromptLoader` `runtime_checkable Protocol` + `FileSystemLoader(base, suffix=".yaml")` + callable coercion, in `packages/python/python/prompting_press/`. Sync `load(key)->str`; raises `LoadError`; traversal-guarded. (Native impl or thin wrapper over the Rust built-in — research R2; keep it a language-side leaf.)
- [ ] T009 [US1] TypeScript: `PromptLoader` interface (`load(key): Promise<string>`) + `FileSystemLoader` (node `fs`, base+suffix, async) + function coercion, in `packages/typescript/src/`. Raises/rejects `LoadError`; traversal-guarded.
- [ ] T010 [P] [US1] Python tests: FileSystemLoader hit/miss, traversal guard, callable coercion, compose `Prompt.from_yaml(loader.load(k))`, load-vs-parse error distinction.
- [ ] T011 [P] [US1] TS tests: FileSystemLoader hit/miss (async), traversal guard, function coercion, compose `Prompt.fromYaml(await loader.load(k))`, error distinction.

**Checkpoint**: filesystem loading behind the interface works in all three bindings; swap-without-changing-call-sites demonstrated.

## Phase 4: User Story 2 — Memory loader for tests/embedding (Priority: P1)

- [ ] T012 [US2] Python `MemoryLoader(prompts: dict[str,str])` + tests (hit/miss). 
- [ ] T013 [US2] TS `MemoryLoader(Record<string,string>)` + tests (hit/miss, async).
- [ ] T014 [P] [US2] Demonstrate dependency-injection: same consuming code runs against FileSystemLoader and MemoryLoader unchanged (SC-002) — one test per binding.

## Phase 5: User Story 3 — Custom loader against the contract (Priority: P2)

- [ ] T015 [P] [US3] Per binding: an example/test custom loader (implement the interface or pass a callable/function) used interchangeably with the built-ins, NO registration (SC-004); its failure surfaces as a load error distinct from parse errors.

## Phase 6: Constitution amendment (v3.0.0 — MAJOR boundary)

**⚠️ The MAJOR amendment of the trio (FR-016/017/018). Cites spec-017's repositioning.**

- [ ] T016 Amend `.specify/memory/constitution.md`: **soften Principle III** to permit a caller-invoked, language-side loader seam (kernel + construction stay I/O-free) and **re-scope Scope-Discipline/C-08** so the Loader seam is an earned opt-in seam (earned by second consumer Bellwether). Cite the spec-017 v3.0.0 repositioning statement; keep the version line at v3.0.0 (coordinate with 017/018 under the same major line); update the sync-impact report.
- [ ] T017 Update `.specify/memory/roadmap.md`: the "**Never: I/O / storage adapters**" entry and the Scope-Discipline "Loader → eliminated" entry — reflect the earned opt-in loader seam; add spec 019 entry. Note heavier backends still deferred.
- [ ] T018 Record in `.specify/memory/DECISIONS.md`: the Principle III softening + C-08 re-scope rationale, MAJOR bump, that it cites spec-017's repositioning, and the bounded scope (opt-in leaf; kernel/construction I/O-free).
- [ ] T019 Regenerate the APM-rendered `CLAUDE.md`/`AGENTS.md` constitution copies to match (SC-007); if `apm compile` isn't runnable, note for reviewer.

## Phase 7: Polish & Cross-Cutting

- [ ] T020 [P] Boundary assertion: `git diff` shows NO change to `crates/prompting-press-core/` (SC-005); no cloud/third-party storage dependency added; both built-ins in the standard package (SC-006).
- [ ] T021 [P] Docs: document the loader interface, both built-ins, the custom-loader contract (incl. error/traversal behavior, sync/async per language), and — honestly — that the loader's value is swappable/testable/centralized storage, NOT "you couldn't read a file before" (opt-in; single-file callers can still use read_text). Docs-are-product — current behavior only.
- [ ] T022 Run full workspace + per-binding test suites; confirm green.

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 (Rust interface/built-ins/error) → Phases 3–5 (bindings + custom).
- T008/T009 depend on T002-T005; their tests (T010/T011) follow. T012/T013 (memory) depend on the interface.
- Phase 6 (amendment) is the gating governance work — the MAJOR boundary change; parallel with code but lands in the same change set; coordinate v3.0.0 with 017/018.
- Phase 7 last; T022 gates completion.

## Parallel Opportunities

- T006/T007 (Rust tests/doc) parallel once built-ins land.
- T010 ∥ T011 (Python vs TS tests). T012 ∥ T013 (memory built-ins).
- T016-T018 (governance) parallel with code.
- T020/T021 polish parallel.

## Implementation Strategy

- **MVP = Phase 2 + Phase 3 + Phase 4** (Rust interface + FileSystem + Memory across bindings). Delivers
  Bellwether's swappable + disk-free-test story. Phase 5 = custom-loader proof; Phase 6 = the mandatory
  MAJOR amendment (must land with the code).
- Rust-first: canonical interface + built-ins + traversal guard in the consumer, then per-language facades.

## Task Summary

- **Total**: 22 tasks
- **US1 (filesystem)**: T008–T011 (4)
- **US2 (memory)**: T012–T014 (3)
- **US3 (custom)**: T015 (1)
- **Setup/Foundational**: T001–T007 (7) — Rust interface, built-ins, LoadError, traversal guard
- **Amendment (MAJOR)**: T016–T019 (4)
- **Polish**: T020–T022 (3)
- **MVP scope**: Phases 2–4.
