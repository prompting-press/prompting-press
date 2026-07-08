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

- [X] T001 Confirm baseline green on `019-pluggable-loader`. Confirm the existing text factories (`crates/prompting-press/src/prompt.rs:108+` from_yaml/json/toml) and the error family (`error.rs:97-109` `ConsumerError::Load` + `[{field,code,message}]`) — the loader composes with / normalizes into these.

## Phase 2: Foundational (Rust interface + error + built-ins)

**⚠️ The canonical interface lands here.**

- [X] T002 **Error-taxonomy expansion (compatibility-surface change — FR-008/FR-018):** add a NEW `PromptLoadError` exception type in the Python + TS bindings (via the existing `create_exception!` / PromptingPressError hierarchy — paralleling `PromptRenderError`), and add NEW `code` constants **`load_io`** and **`load_not_found`** to the closed vocab in `crates/prompting-press/src/error.rs`. Do NOT reuse `LoadError` (that is the parse error) or `ConsumerError::Load`. Add the Rust-side carrier the loader returns (a new `PromptLoadError`-mapping variant/type) and wire BOTH FFI mappers (`prompting-press-node/src/error.rs`, `prompting-press-py/src/error.rs`) + the per-binding routing switches to map `load_io`/`load_not_found` → `PromptLoadError`. Default messages scrubbed (logical key + code only; NO file contents / full absolute path / secrets — D2/D3, security SEC-003).
- [X] T002a **Native error-raise path (FR-008a):** expose a binding-level constructor/factory so a pure-Python / pure-TS loader can raise `PromptLoadError` with a populated `[{field,code,message}]` payload (since `create_exception!` types carry no typed Rust field).
- [X] T003 Add the loader module in `crates/prompting-press/src/`: object-safe `trait PromptLoader { fn load(&self,key:&str)->Result<String, /* PromptLoadError carrier */>; }` + blanket impl for the closure form (FR-001).
- [X] T004 Implement `FileSystemLoader { base, suffix=".yaml", max_bytes }`: `load(key)` reads `{base}/{key}{suffix}`. **Traversal guard (FR-002a/FR-002b/SC-008):** validate the FINAL resolved path incl. suffix against a canonicalized `base`; reject absolute keys, `..` components, escaping symlinks, cross-platform separators/UNC/NUL; define `key=""`/`key="."`/empty-suffix; a canonicalize-fail on a missing target → `load_not_found` (NOT `load_io`). **Read cap (FR-016/SC-009):** enforce `max_bytes` (sane default) → `load_io` on exceed.
- [X] T005 Implement `MemoryLoader` from a `key→text` map; miss → `PromptLoadError` `load_not_found` (FR-003/FR-006).
- [X] T006 [P] Rust tests: FileSystemLoader hit + miss (`load_not_found`); **traversal guard** (`../secret`, absolute, escaping symlink, `key=""`, `key="."`, empty suffix → `PromptLoadError`, no outside read, SC-008); **read cap** (exceed `max_bytes` → `load_io`, SC-009); MemoryLoader hit + miss; closure-as-loader; load-error ≠ parse-error at the CLASS level (compose with `from_yaml`, SC-010); scrubbed error message (no path/secret leak).
- [X] T007 [P] Rustdoc: interface contract, `load` returns raw text (not a Prompt), the `PromptLoadError`/`load_io`/`load_not_found` taxonomy, traversal guard + cap, sync nature, "compose with from_yaml; not fused" (FR-005/FR-011).

**Checkpoint**: Rust interface + built-ins + the NEW `PromptLoadError` taxonomy exist, traversal-guarded + capped, tested.

## Phase 3: User Story 1 — Filesystem behind a swappable interface (Priority: P1) 🎯 MVP

- [X] T008 [US1] Python: `PromptLoader` `runtime_checkable Protocol` + `FileSystemLoader(base, suffix=".yaml", max_bytes=<default>)` + callable coercion, in `packages/python/python/prompting_press/`. Sync `load(key)->str`; raises `PromptLoadError` via the T002a native-raise path; **traversal guard + read cap implemented natively (FR-017)** — not only in Rust.
- [X] T009 [US1] TypeScript: `PromptLoader` interface (`load(key): Promise<string>`) + `FileSystemLoader` (node `fs`, base+suffix+max_bytes, async) + function coercion, in `packages/typescript/src/`. Rejects with `PromptLoadError` (T002a); **traversal guard + read cap implemented natively (FR-017)**.
- [X] T010 [P] [US1] Python tests: hit/miss (`load_not_found`), traversal guard (`../`, absolute, symlink escape, `key=""`, empty suffix, SC-008), read cap (SC-009), callable coercion, compose `Prompt.from_yaml(loader.load(k))`, `except PromptLoadError` does NOT catch a malformed-YAML `LoadError` (SC-010).
- [X] T011 [P] [US1] TS tests: hit/miss (async), traversal guard (same cases), read cap, function coercion, a REJECTING loader surfaces `PromptLoadError` (not an unhandled rejection), compose `Prompt.fromYaml(await loader.load(k))`, class-level error distinction (SC-010).

**Checkpoint**: filesystem loading behind the interface works in all three bindings; swap-without-changing-call-sites demonstrated.

## Phase 4: User Story 2 — Memory loader for tests/embedding (Priority: P1)

- [X] T012 [US2] Python `MemoryLoader(prompts: dict[str,str])` + tests (hit/miss).
- [X] T013 [US2] TS `MemoryLoader(Record<string,string>)` + tests (hit/miss, async).
- [X] T014 [P] [US2] Demonstrate dependency-injection: same consuming code runs against FileSystemLoader and MemoryLoader unchanged (SC-002) — one test per binding.

## Phase 5: User Story 3 — Custom loader against the contract (Priority: P2)

- [X] T015 [P] [US3] Per binding: an example/test custom loader (implement the interface or pass a callable/function) used interchangeably with the built-ins, NO registration (SC-004); its failure surfaces as a load error distinct from parse errors.

## Phase 6: Constitution amendment (additive → v3.2.0, on 017's v3.0.0 baseline)

**⚠️ FR-018/019. 017 is the v3.0.0 baseline (already written); 019 CITES it and adds only its own edits.**

- [X] T016 Amend `.specify/memory/constitution.md` (assumes 017's v3.0.0 landed first): **CITE** the existing spec-017 v3.0.0 repositioning statement (do NOT re-declare it). Add ONLY 019's edits: **soften Principle III** to permit a caller-invoked, language-side loader seam (kernel + construction stay I/O-free) and **re-scope Scope-Discipline/C-08** so the Loader seam is an earned opt-in seam (earned by Bellwether). Bump version → **v3.2.0** (additive; 018 = v3.1.0). Update the sync-impact report noting BOTH the boundary softening AND the error-taxonomy expansion (below).
- [X] T017 Update `.specify/memory/roadmap.md`: the "**Never: I/O / storage adapters**" entry and the Scope-Discipline "Loader → eliminated" entry — reflect the earned opt-in loader seam; add spec 019 entry. Note heavier backends still deferred.
- [X] T018 Record in `.specify/memory/DECISIONS.md` (spec-015-style, enumerated): (a) Principle III softening + C-08 Loader re-scope; (b) **the error-taxonomy compatibility-surface expansion** — the NEW `PromptLoadError` type + `load_io`/`load_not_found` codes added to the closed `code` vocab + `ConsumerError`/error set, and the FFI-mapper/routing changes; state the bounded scope (opt-in leaf; kernel/construction I/O-free) and that it cites 017's v3.0.0 → additive v3.2.0.
- [X] T019 Regenerate the APM-rendered `CLAUDE.md`/`AGENTS.md` constitution copies to match (SC-007); `apm compile` is NOT runnable in this worktree — **deferred to reviewer** after merge (see sync-impact report note in constitution.md).

## Phase 7: Polish & Cross-Cutting

- [X] T020 [P] Boundary assertion: `git diff` shows NO change to `crates/prompting-press-core/` (SC-005); no cloud/third-party storage dependency added; both built-ins in the standard package (SC-006).
- [X] T021 [P] Docs: document the loader interface, both built-ins, the custom-loader contract (incl. error/traversal behavior, sync/async per language), and — honestly — that the loader's value is swappable/testable/centralized storage, NOT "you couldn't read a file before" (opt-in; single-file callers can still use read_text). Docs-are-product — current behavior only. (Documented via Rustdoc, Python docstrings, TS JSDoc, and API-ref regeneration.)
- [X] T022 Run full workspace + per-binding test suites; confirm green. Rust: 59 tests pass. Python/TS wheel suites need full build; not runnable in-worktree (noted). Rust unit + integration tests all green.

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
