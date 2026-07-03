---
description: "Task list for Framework Integration Guides (spec 016)"
---

# Tasks: Framework Integration Guides

**Input**: Design documents from `specs/016-framework-integration-guides/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/recipe-contracts.md, quickstart.md

**Tests**: This feature's "tests" ARE the doc-sample programs — each is a standalone program with in-file assertions, executed by the `docs:test-samples` moon gate (spec-014 mechanism). There are therefore no separate test tasks; the sample authoring tasks include their assertions per the contracts (C1–C3). No live LLM calls (construct + assert only).

**Organization**: Grouped by user story (US1 integration recipes+pages, US2 positioning, US3 deep-link redirect). US1/US2/US3 are independent and can ship separately.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on an incomplete task)
- Paths are repo-relative. Sample root is **`docs/site/samples/`** (NOT repo-root `samples/`).

## Path Conventions (this feature)

- Docs pages: `docs/site/src/content/docs/**`
- Sidebar: `docs/site/astro.config.mjs`
- Samples: `docs/site/samples/{python,typescript}/examples/**` + project manifests
- Build/redirect: `docs/site/scripts/build-versions.mjs`
- Frozen backfill: `docs/site/src/versions/{v0.1,v0.2}/**`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Make the framework SDKs available to the doc-sample projects (sample-only dev deps, isolated from shipped packages).

- [ ] T001 [P] Add framework dev dependencies to `docs/site/samples/python/pyproject.toml`: `langchain-core`, `langgraph`, `strands-agents`, `crewai` (compatible with authoring-time versions in research.md R1–R3). Keep them in this project only.
- [ ] T002 [P] Add framework dev dependencies to `docs/site/samples/typescript/package.json`: `@langchain/core`, `@langchain/langgraph`, `@strands-agents/sdk`; confirm `docs/site/samples/typescript/tsconfig.json` globs `examples/**` so new files are type-checked.
- [ ] T003 Sync/install both sample projects and confirm the baseline `docs:test-samples` gate still passes BEFORE adding new samples (`moon run docs:test-samples-python` and `moon run docs:test-samples-typescript`) — establishes the SDKs install cleanly and no existing sample broke.

**Checkpoint**: SDKs installed in sample projects only; existing samples green; `rg "langchain|langgraph|strands|crewai" packages/*/pyproject.toml packages/*/package.json crates/*/Cargo.toml` returns nothing (FR-007 guard).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The Integrations chapter shell and sidebar wiring that every US1 page and the US2 intro depend on.

⚠️ Blocks US1 pages and the US2 integrations-intro cross-links.

- [ ] T004 Create the Integrations chapter directory `docs/site/src/content/docs/integrations/` and the intro page `integrations/index.mdx` with frontmatter (title "Integrations", description) and a stub body (positioning filled in US2 T017). Internal links must be root-absolute.
- [ ] T005 Add an "Integrations" group to the Starlight `sidebar` in `docs/site/astro.config.mjs` with entries: `/integrations/`, `/integrations/langchain/`, `/integrations/strands/`, `/integrations/crewai/` (order: intro, LangChain, Strands, CrewAI).

**Checkpoint**: `node docs/site/scripts/build-versions.mjs` builds `next` with the intro page reachable from the sidebar (frozen-version backfill handled in Phase 6 so the global sidebar entry does not dead-link — do a `next`-only sanity check here).

---

## Phase 3: User Story 1 — Wire a rendered prompt into my agent framework (Priority: P1) 🎯 MVP

**Goal**: A developer copies a tested helper + construction snippet and gets a working prompt-fed agent, per framework/language.

**Independent test**: Open each framework page, run its sample against the real SDK; `docs:test-samples` is green; embedded code == tested file.

### LangChain / LangGraph (FR-002, C1)

- [ ] T006 [P] [US1] Author `docs/site/samples/python/examples/integrations_langchain_messages.py` — build a Composition, map `[{role,text}]`→`[{"role","content"}]`, assert order/role preserved and a brace-containing text round-trips verbatim (use `langchain_core.messages.utils.convert_to_messages` or a fake model; no network). Include the role→`SystemMessage/HumanMessage/AIMessage` typed variant.
- [ ] T007 [P] [US1] Author `docs/site/samples/typescript/examples/integrations_langchain_messages.ts` — same mapping + assertions in TS against `@langchain/core`.
- [ ] T008 [US1] Create `docs/site/src/content/docs/integrations/langchain.mdx` — Python + TS tabs importing T006/T007 via `?raw`; prose covering the key-rename mapping, LangGraph `MessagesState`/`add_messages` note, and the `ChatPromptTemplate.from_messages` brace-re-templating footgun (FR-008).

### Strands (FR-003, C2)

- [ ] T009 [P] [US1] Author `docs/site/samples/python/examples/integrations_strands_partition.py` — partition helper returning `system` (`\n\n`-joined, `None` if none) + `convo` (`{role, content:[{text}]}`); assert on a `[system, system, user, assistant, user]` fixture (TWO system messages so the `\n\n`-join + order is actually exercised per G1: assert `system == "<s1>\n\n<s2>"`; 3 convo entries all user|assistant; each content `[{text}]`); construct `Agent(system_prompt=, messages=)` (no `.run()`; no network).
- [ ] T010 [P] [US1] Author `docs/site/samples/typescript/examples/integrations_strands_partition.ts` — same partition + assertions against `@strands-agents/sdk` (`systemPrompt`, `MessageData{role,content:[TextBlockData]}`), including the two-system-message `\n\n`-join assertion (G1); construct the Agent (no invoke).
- [ ] T011 [US1] Create `docs/site/src/content/docs/integrations/strands.mdx` — Python + TS tabs importing T009/T010; prose covering the no-in-list-system-role partition, the system-position-flattening limitation (FR-008), and the `guardContent`/`toolResult` out-of-scope note (FR-009).

### CrewAI (FR-004, C3)

- [ ] T012 [P] [US1] Author `docs/site/samples/python/examples/integrations_crewai_fields.py` — render each prompt and use the `render(...).text` field (the RENDERED string, NOT `Prompt.body` which is the raw template); assign to `Agent(role=,goal=,backstory=)` + `Task(description=,expected_output=)`; assert field values equal the rendered `.text` strings (constructor only; no `kickoff()`, no network).
- [ ] T013 [US1] Create `docs/site/src/content/docs/integrations/crewai.mdx` — Python-only page importing T012; prose covering field assignment, the `crew.kickoff(inputs=...)` double-fill footgun (FR-008), and an explicit note that no official CrewAI TypeScript SDK exists (FR-021) so this page is Python-only.

- [ ] T014 [US1] Run `moon run docs:test-samples-python` and `moon run docs:test-samples-typescript`; fix any sample until green (validates C1–C3 against real SDK types — SC-009).

**Checkpoint**: US1 is independently shippable — three framework pages, five tested samples, all green.

---

## Phase 4: User Story 2 — Why Prompting Press over raw Jinja / framework templating (Priority: P2)

**Goal**: FAQ + homepage answer the launch-thread positioning questions.

**Independent test**: FAQ has both entries; homepage has the use-cases section covering all six points; no principle numbers / no "provenance" (SC-007/008).

- [ ] T015 [P] [US2] Add FAQ entry "Why not just use Jinja / minijinja / a framework's own templating?" to `docs/site/src/content/docs/faq.mdx` — acknowledge minijinja under the hood; distinguish on structured storage, typed inputs, static (build-time, no-data) agreement check vs. Jinja runtime `StrictUndefined`, variants, byte-identical cross-language rendering (FR-010).
- [ ] T016 [P] [US2] Add FAQ entry "How does this fit ChatPromptTemplate / the system-user split?" to `docs/site/src/content/docs/faq.mdx` — PP emits neutral role-tagged text; link to the per-framework Integrations pages for shaping (FR-011).
- [ ] T017 [US2] Fill the Integrations intro `integrations/index.mdx` (from T004) with the "PP owns storage/typing/variants/the check; the framework owns the call" framing; link to the three framework pages.
- [ ] T018 [US2] Add a use-cases section to the homepage `docs/site/src/content/docs/index.mdx` covering all six FR-012 points: structured reviewable artifacts; storage-agnostic loading; variant switching without redeploy (migration / per-user / multilingual / tone — not just A/B); catch prompt/variable mismatches before shipping; one prompt identical across Py/TS/Rust (frontend+backend consistency); per-render content fingerprint (plain terms, NOT "provenance"). Link out to existing Variants / Lint-in-CI guides rather than duplicating.
- [ ] T019 [US2] Grep-guard (broadened per FR-013): over `docs/site/src/content/docs/{faq,index}.mdx` + `docs/site/src/content/docs/integrations/` + the new `docs/site/samples/**/examples/integrations_*` files, confirm NONE of these internal-artifact patterns appear: `principle [ivxIVX0-9]+`, `provenance`, `\bspec[- ]?016\b|016-framework`, `\b(FR|SC|CHK|US|T)-?[0-9]{2,3}\b`, `SpecKit|speckit|acceptance scenario|conformance corpus`, `\b[CDRA]-?[0-9]{1,2}\b` (roadmap/decision codes), `specs/|\.specify/`. Fix any leak. (SC-008)

**Checkpoint**: US2 independently shippable — positioning present and clean.

---

## Phase 5: User Story 3 — Unversioned deep-link redirect (Priority: P2)

**Goal**: `/getting-started/rust/` (and any unversioned deep path present under latest) redirects to `/v{latest}/...` instead of 404.

**Independent test**: quickstart §4 — deep stub points at `/v{latest}/<path>/`; root stub intact; no stub for nonexistent paths.

- [ ] T020 [US3] Extend `docs/site/scripts/build-versions.mjs`: generalize `emitRootRedirect()` (or add a sibling) to also emit a redirect stub at `dist/<slug>/index.html` → `/v{latest}/<slug>/` for every page slug present under the latest version, reusing the existing meta-refresh + canonical + `location.replace()` stub form. Root `/` behavior unchanged (FR-016); no stub for slugs absent under latest (FR-017).
- [ ] T021 [US3] Build (`node docs/site/scripts/build-versions.mjs`) and verify per quickstart §4: `dist/getting-started/rust/index.html` redirects to `/v{latest}/getting-started/rust/`; `dist/index.html` root redirect intact; a nonexistent path has no stub (SC-005).

**Checkpoint**: US3 independently shippable — shared/bookmarked deep links resolve.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Frozen-snapshot backfill (needed so the global sidebar entry from T005 doesn't dead-link on old versions) + full multi-version verification.

- [ ] T022 Backfill the Integrations pages into frozen `docs/site/src/versions/v0.1/` (add `integrations/` pages consistent with that version; ensure links version-prefix correctly) — one-time authorized exception (FR-018).
- [ ] T023 Backfill the Integrations pages into frozen `docs/site/src/versions/v0.2/` likewise (FR-018).
- [ ] T024 Full multi-version build + link check (quickstart §3): `node docs/site/scripts/build-versions.mjs` exits 0; `integrations/{,,langchain,strands,crewai}` present on `next`, `v0.1`, `v0.2`; sidebar "Integrations" links version-prefixed on each; no internal link 404s (FR-018/019, SC-006).
- [ ] T025 Final no-leak sweep across ALL new/edited docs AND the new integration sample files (FR-020 no future tense; FR-013 no internal-artifact leak — run the full broadened pattern set from T019 across every new/edited page and every `integrations_*` sample, including code comments; FR-005 every embedded snippet is a `?raw` import, no inline untested code); confirm `rg` guard for FR-007 (no framework dep in shipped `packages/**`/`crates/**` manifests); and confirm FR-006 (samples use only existing public API — structurally enforced because `docs:test-samples` type-check/exec would fail on any nonexistent method/field/option).

---

## Dependencies & Execution Order

- **Setup (P1: T001–T003)** → blocks everything (samples need SDKs).
- **Foundational (P2: T004–T005)** → blocks US1 pages (T008/T011/T013) and US2 intro (T017).
- **US1 (P3)**: samples T006/T007/T009/T010/T012 are all [P] (distinct files); each page task (T008/T011/T013) depends on its samples + T004; T014 gates on all samples.
- **US2 (P4)**: T015/T016 [P] (both edit faq.mdx — serialize if the tool locks the file; otherwise coordinate); T017 depends on T004; T018 independent; T019 after T015–T018.
- **US3 (P5)**: T020→T021, independent of US1/US2.
- **Polish (P6)**: T022/T023 depend on US1 pages existing (T008/T011/T013) and T005; T024 after T022/T023; T025 last.

**Independent delivery**: US1, US2, US3 can each ship alone. US3 (redirect) has zero dependency on the docs-content stories and could ship first as a quick win.

## Parallel Opportunities

- T001 ∥ T002 (Setup).
- All five sample-authoring tasks: T006 ∥ T007 ∥ T009 ∥ T010 ∥ T012 (distinct files).
- T015 ∥ T016 (same file — serialize) ; T018 ∥ (T015/T016) ; T020/T021 (US3) ∥ any US1/US2 work.

## MVP Scope

**US1 (Phase 3)** alone is the MVP: the three framework integration pages + five tested samples directly answer the launch-thread's headline ask. US2 (positioning) and US3 (redirect) are independent P2 increments.

## Implementation Strategy

1. Setup (T001–T003) → confirm baseline green + no shipped-package leak.
2. Foundational (T004–T005).
3. US1 (T006–T014) → MVP; ship if desired.
4. US2 (T015–T019) and US3 (T020–T021) in either order (independent).
5. Polish (T022–T025): backfill + full multi-version verification + final leak sweep.
