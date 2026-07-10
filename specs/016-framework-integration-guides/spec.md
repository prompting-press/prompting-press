# Feature Specification: Framework Integration Guides

**Feature Branch**: `016-framework-integration-guides`

**Created**: 2026-07-03

**Status**: Draft

**Input**: User description: "Framework integration guides (LangChain/LangGraph, Strands, CrewAI) — a new docs 'Integrations' chapter plus expanded FAQ and homepage use-cases, showing how to use Prompting Press *with* agent frameworks via tested, CI-compiled helper recipes (not new library API), and a redirect so unversioned deep doc URLs no longer 404."

## Overview

Prompting Press launched and the first adopter questions all landed in the same place: *how do I use this with the framework I already have?* (LangChain/LangGraph, Strands, CrewAI), *how is this different from raw Jinja?*, and *how does it fit the system/user prompt split?* Separately, a shared deep link into the docs (`/getting-started/rust/`) returned a hard 404 because it lacked a version prefix.

This feature answers those questions **entirely in documentation and tested sample code** — no new library API, no framework dependency in any shipped `prompting-press-*` package. Prompting Press renders prompts; the framework makes the LLM call. The bridge between them is a handful of lines of user-side glue, and this feature ships those lines as real, CI-verified sample files with prose that explains where each framework's shape differs and where the sharp edges are.

It also fixes the unversioned-deep-link 404 so shared and bookmarked URLs resolve to the current version.

## Clarifications

### Session 2026-07-03

- Q: How are the framework samples type-checked, given the heavy SDKs (Strands→boto3/OTel; CrewAI→chromadb/lancedb)? → A: Install the real framework SDKs as **sample-only** dev dependencies (isolated from shipped `prompting-press-*` packages) and fully compile/type-check the samples against the true SDK types — highest fidelity; the CI install weight is accepted.
- Q: How should the global Integrations sidebar entry behave on frozen historical versions (`v0.1`/`v0.2`) that predate these pages? → A: **Backfill** the Integrations pages into the existing frozen `v0.1`/`v0.2` snapshots so the nav link resolves on every version. This is a one-time exception to the frozen-snapshot doctrine, justified because the project is still greenfield (pre-1.0, tiny released-version set).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Wire a rendered prompt into my agent framework (Priority: P1)

A developer already using LangChain/LangGraph, Strands, or CrewAI wants to source their prompts from Prompting Press instead of inline strings. They open the Integrations chapter, find their framework and language, and copy a short, tested helper that maps a Prompting Press render into exactly what their framework's constructor/invoke call expects.

**Why this priority**: This is the headline ask from the launch thread (Pedro, David) and the direct driver of adoption — "show me it works with what I use." Without it, a prospective user has to reverse-engineer the mapping themselves and may bounce.

**Independent Test**: Can be fully tested by opening each framework's integration page, copying the shown sample, and running it against the real framework SDK; the sample is also compiled/type-checked in CI so it cannot rot. Delivers value even if no other story ships.

**Acceptance Scenarios**:

1. **Given** a developer on the LangChain (Python) integration page, **When** they copy the `to_langchain` helper and pass a Prompting Press composition result, **Then** they get a list of `{"role", "content"}` dicts that a LangChain chat model accepts via `.invoke()` with no further transformation.
2. **Given** a developer on the Strands (Python or TypeScript) integration page, **When** they apply the shown partition helper to a composition that includes a system-role message, **Then** they receive a separate system prompt value plus a messages list, and the page explains that Strands has no in-list system role.
3. **Given** a developer on the CrewAI (Python) integration page, **When** they follow the sample, **Then** they see rendered strings assigned directly to `Agent` and `Task` string fields, with a note not to also pass the same variables to `crew.kickoff(inputs=...)`.
4. **Given** any framework/language page, **When** the docs site is built, **Then** the sample code embedded on the page is the exact file that CI compiled/type-checked (imported verbatim), not hand-copied prose.

---

### User Story 2 - Understand why Prompting Press over raw Jinja or a framework's own templating (Priority: P2)

A developer evaluating the library asks "this uses minijinja under the hood — why not just use Jinja / nunjucks / `ChatPromptTemplate` directly?" They find a direct, honest answer in the FAQ and a use-cases summary on the homepage that frames what Prompting Press owns (prompt storage, typed inputs, variants, the static check, cross-language consistency, a per-render fingerprint) versus what the framework owns (the LLM call).

**Why this priority**: Two separate launch-thread questions (Joseph: "how is this different than jinja?"; David: how it fits `ChatPromptTemplate`/the system-user split). Answering them converts interest into adoption and pre-empts the most common objection. Depends on nothing in Story 1 but reinforces it.

**Independent Test**: Can be tested by reading the FAQ "Why not just Jinja?" and "How does this fit ChatPromptTemplate / the system-user split?" entries and the homepage use-cases section, and confirming each makes a concrete, defensible distinction (not marketing fluff). Deliverable on its own.

**Acceptance Scenarios**:

1. **Given** the FAQ page, **When** a reader looks for the Jinja comparison, **Then** they find an entry that acknowledges Prompting Press uses minijinja for templating and distinguishes on storage structure, typed inputs, the static (build-time) agreement check versus Jinja's runtime `StrictUndefined`, variants, and cross-language byte-identical rendering.
2. **Given** the FAQ page, **When** a reader looks for how it fits a framework's system/user split, **Then** they find an entry explaining that Prompting Press emits neutral role-tagged text and each framework's page shows how to shape it (system-in-list for LangChain, system-separate for Strands, field-based for CrewAI).
3. **Given** the homepage, **When** a reader scans below the intro, **Then** they see a concise use-cases section covering: prompts as structured reviewable artifacts, storage-agnostic loading, variant switching without redeploy (migration / per-user / multilingual / tone), a build-time check against typos, one prompt rendering identically across languages, and a per-render content fingerprint.

---

### User Story 3 - Follow a shared or bookmarked deep link without hitting a 404 (Priority: P2)

Someone clicks a shared link like `https://prompting-press.github.io/getting-started/rust/` (no version prefix). Instead of a hard 404, they are redirected to the current version's equivalent page (`/v{latest}/getting-started/rust/`).

**Why this priority**: A real reported bug (Pedro) that makes shared/bookmarked links from before the versioning switch — and any hand-typed unversioned URL — dead ends. Small, self-contained, and independent of the docs-content stories.

**Independent Test**: Can be tested by requesting an unversioned deep path on the built site and confirming it resolves (via redirect) to the latest-version equivalent rather than returning 404. Independent of Stories 1 and 2.

**Acceptance Scenarios**:

1. **Given** the built multi-version docs site, **When** a request arrives for an unversioned deep path that exists under the latest version (e.g. `/getting-started/rust/`), **Then** the site redirects to the latest version's equivalent (`/v{latest}/getting-started/rust/`).
2. **Given** the built site, **When** a request arrives for the site root (`/`), **Then** the existing redirect to `/v{latest}/` continues to work unchanged.
3. **Given** an unversioned path that does not correspond to any page under the latest version, **When** it is requested, **Then** the behavior is a normal not-found (no misleading redirect to a nonexistent versioned page).

---

### Edge Cases

- **Composition with multiple system messages (Strands):** the partition helper joins all system-role texts (in order) into the single system prompt Strands expects; the page documents this.
- **Composition with a system message that is not first (Strands):** system text is hoisted to the separate system prompt regardless of position — the page flags that Strands cannot preserve mid-conversation system placement (a framework limitation, not something Prompting Press can carry).
- **Rendered text containing literal braces (LangChain):** the page warns that routing already-rendered text through `ChatPromptTemplate.from_messages` tuple/dict shorthand re-templates `{...}` and raises on literal braces; the recipe feeds the model/graph directly instead.
- **Variables already rendered by Prompting Press (CrewAI):** the page warns not to also pass those variables to `crew.kickoff(inputs=...)`, since CrewAI's own `{placeholder}` interpolation would look for text that no longer exists.
- **CrewAI in TypeScript:** no official TypeScript SDK exists, so CrewAI ships a Python page only; the chapter does not fabricate a TypeScript CrewAI sample.
- **Framework-specific content types out of scope:** Strands `guardContent`, `toolResult`, `toolUse`, and similar provider request-body block types are explicitly out of scope; recipes map plain rendered text only.
- **New pages under multi-version build:** the Integrations pages must build for the working-tree (`next`) version and be frozen into subsequent released-version snapshots like any other content page.
- **Sidebar link on versions without the pages:** the global sidebar entry would otherwise dead-link on frozen versions that predate the pages; resolved by backfilling the Integrations pages into the frozen `v0.1`/`v0.2` snapshots (FR-018) — a one-time greenfield exception, not a general license to edit frozen history.

## Requirements *(mandatory)*

### Functional Requirements

#### Integrations chapter & recipes

- **FR-001**: The docs site MUST have a new top-level "Integrations" chapter with an introduction page and one page per framework: LangChain/LangGraph, Strands, and CrewAI.
- **FR-002**: The LangChain/LangGraph page MUST cover both Python and TypeScript, each showing a tested helper that maps a Prompting Press composition result (`[{role, text}]`) to the framework's accepted message shape (`[{role, content}]`) and demonstrating handing it to a chat model / graph node.
- **FR-003**: The Strands page MUST cover both Python and TypeScript, each showing a tested helper that partitions a composition into (a) a single system prompt value (system-role texts joined in order; absent when there is no system message) and (b) a messages list in the framework's content-block shape, and demonstrating construction of an agent from both.
- **FR-004**: The CrewAI page MUST cover Python only and show rendered strings assigned to the relevant `Agent` and `Task` string fields.
- **FR-005**: Every code sample shown on an Integrations page MUST be a real sample file under `docs/site/samples/{python,typescript}/examples/` (the spec-014 tested doc-sample project root — NOT the repo-root `samples/` consumer-app project) that is compiled/type-checked/executed by CI and embedded into the page verbatim (not hand-transcribed prose).
- **FR-005a**: Samples MUST be compiled/type-checked against the **real** framework SDK types (not local stubs). The framework SDKs MUST be installed as **sample-only dev dependencies**, scoped to the samples/CI toolchain and isolated from every shipped `prompting-press-*` package's dependency set, so that a real SDK shape drift would surface as a sample type-check failure.
- **FR-006**: The Prompting Press portion of each sample MUST use only the existing public library surface; no sample requires a new method, field, or option on any `prompting-press-*` package.
- **FR-007**: No `prompting-press-*` package MUST gain a dependency on any agent framework as a result of this feature; framework SDK usage appears only in sample/demo code and its sample-only dev-dependency set (FR-005a), never in shipped library packages.
- **FR-008**: Each framework page MUST document that framework's known pitfall(s): LangChain — do not route rendered text through `ChatPromptTemplate.from_messages` tuple/dict shorthand (brace re-templating); Strands — system position is flattened into the separate system prompt; CrewAI — do not double-fill via `crew.kickoff(inputs=...)` for already-rendered variables.
- **FR-009**: The Integrations pages MUST NOT describe or imply any framework-specific request-body content types (e.g. Strands `guardContent`/`toolResult`) as supported outputs; recipes map plain rendered text only.

#### Positioning: why-vs-Jinja, framework fit, use cases

- **FR-010**: The FAQ MUST include an entry answering "Why not just use Jinja / minijinja / a framework's own templating?" that acknowledges Prompting Press uses minijinja for templating and distinguishes on: structured prompt storage, a typed input model, a build-time (static, no-data) agreement check versus Jinja's runtime undefined-variable error, variants, and byte-identical cross-language rendering.
- **FR-011**: The FAQ MUST include an entry answering how Prompting Press fits a framework's system/user prompt split and `ChatPromptTemplate`, framing Prompting Press as the producer of neutral role-tagged text and pointing to the per-framework Integrations pages for the shaping.
- **FR-012**: The homepage MUST include a concise use-cases section covering: (a) prompts as structured, reviewable artifacts rather than scattered string literals; (b) storage-agnostic loading (the caller pushes prompt data in from a file, database, object store, or anywhere); (c) variant switching without redeploy, with migration, per-user preference, multilingual, and tone framing (not only A/B testing); (d) catching prompt/variable mismatches before shipping; (e) one prompt rendering identically across Python, TypeScript, and Rust (frontend + backend consistency); (f) a per-render content fingerprint to reproduce exactly what was sent.
- **FR-013**: User-facing docs prose MUST NOT reference or expose ANY internal specification/development artifact. This includes, non-exhaustively: constitution principle numbers or names (e.g. "Principle III", "Minimal Boundary"); spec/feature IDs (e.g. "spec 016", "016-framework-integration-guides"); requirement/criteria IDs (FR-###, SC-###, US#, CHK###, T### task IDs); SpecKit workflow/tooling terms (SpecKit, clarify/plan/tasks/analyze phases, "the spec", "the plan", "acceptance scenario"); internal roadmap/decision codes (C-##, R#, D#, A#) and file references into `specs/` or `.specify/`; and internal-only vocabulary that is not part of the public product surface (e.g. "provenance", "agreement check" as a coined term, "conformance corpus", "the kernel"). Capabilities MUST be described purely in end-user product terms as if the reader has never seen the internal specs. (This is a HARD rule: the docs are the shipped product and must read as first-class product documentation, not as a rendering of internal planning artifacts.)
- **FR-014**: FR-012(f) MUST describe the per-render fingerprint capability in plain user-facing terms (e.g. "a content hash of each rendered prompt so you can reproduce exactly what was sent") and MUST NOT use the internal term "provenance" (covered generally by FR-013; called out explicitly here because it is the most likely leak).

#### Unversioned deep-link redirect

- **FR-015**: The built docs site MUST redirect an unversioned deep path that corresponds to a page under the latest version to that latest version's equivalent path (e.g. `/getting-started/rust/` → `/v{latest}/getting-started/rust/`).
- **FR-016**: The existing site-root redirect to `/v{latest}/` MUST continue to function unchanged.
- **FR-017**: An unversioned path with no corresponding page under the latest version MUST resolve to a normal not-found result, not a redirect to a nonexistent versioned page.

#### Docs mechanics & multi-version integrity

- **FR-018**: The new Integrations pages MUST be added to the site sidebar/navigation and MUST resolve correctly (version-prefixed) in each version build. To avoid dead sidebar links on frozen historical versions that predate these pages, the Integrations pages MUST be **backfilled into the existing frozen `v0.1` and `v0.2` snapshots** so the entry resolves on every published version. This backfill is a one-time, explicitly-authorized exception to the otherwise-inviolable frozen-snapshot rule, justified by the project's pre-1.0 greenfield status and the small released-version set; it MUST NOT be treated as precedent for editing frozen snapshots in general.
- **FR-019**: All new pages and samples MUST survive the multi-version docs build (working-tree `next` build plus frozen released-version snapshots) with no build failure and no broken internal links.
- **FR-020**: Docs prose MUST describe only current, shipping behavior — no future-tense or "coming soon" language for unshipped capability.
- **FR-021**: Every SDK version or API shape asserted in a sample or page MUST match a currently published version of that framework at authoring time; where a language lacks an official SDK (CrewAI TypeScript), the docs MUST NOT present an unofficial third-party package as the integration target.

### Key Entities

- **Integration page**: A docs page for one framework, containing prose (positioning, pitfalls, out-of-scope notes) and one or more embedded, CI-tested sample files, per supported language.
- **Recipe / helper sample**: A real source file under `docs/site/samples/{python,typescript}/examples/` that maps a Prompting Press render result into a target framework's expected shape; the unit of "tested glue" this feature ships.
- **Composition result**: The existing Prompting Press output — an ordered list of role-tagged rendered messages (`[{role, text}]`, role ∈ {system, user, assistant}) that recipes consume. Unchanged by this feature.
- **Use-cases section**: Homepage content summarizing framework-agnostic value; links out to existing guides (Variants, Lint-in-CI, etc.) rather than duplicating them.
- **Redirect rule**: Build-time-emitted behavior mapping an unversioned deep path to its latest-version equivalent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can go from a framework's integration page to a working prompt-fed agent by copying one shown helper plus the construction snippet — no additional glue code beyond what the page shows.
- **SC-002**: All three frameworks that have an official SDK for a language are covered for that language: LangChain (Python + TypeScript), Strands (Python + TypeScript), CrewAI (Python).
- **SC-003**: 100% of code samples embedded on Integrations pages are the same files CI compiles/type-checks; there is no page-only, untested snippet.
- **SC-004**: Zero `prompting-press-*` shipped packages gain an agent-framework dependency (verifiable by inspecting package manifests before/after).
- **SC-005**: The previously-reported unversioned deep link resolves: a request to `/getting-started/rust/` reaches the latest version's page rather than returning 404.
- **SC-006**: The full multi-version docs build succeeds with the new pages present, and no internal link on any version — including the backfilled frozen `v0.1`/`v0.2` — 404s; the Integrations sidebar entry resolves on every published version.
- **SC-009**: Sample type-checks fail if a framework SDK's message/agent shape drifts away from what a recipe maps to (i.e. samples are checked against real SDK types, not stubs), verifiable by the sample-only dev-dependency install in CI.
- **SC-007**: A reader can find, in the FAQ, a direct answer to both "why not just Jinja?" and "how does this fit my framework's system/user split?", and on the homepage a use-cases summary covering all six value points in FR-012.
- **SC-008**: No user-facing docs prose introduced by this feature exposes any internal spec/development artifact (per FR-013): a review + grep sweep for principle references, spec/feature IDs, FR-/SC-/US/CHK/T-### identifiers, SpecKit phase/tooling terms, roadmap/decision codes, `specs/`/`.specify/` paths, and the term "provenance" returns nothing in the new/edited pages and samples.

## Assumptions

- The existing Prompting Press public API is sufficient to author every recipe; this feature adds no library code and no constitution amendment. (Confirmed during scoping: all three mappings reduce to a key rename, a partition, or field assignment over the existing composition output.)
- The role vocabulary stays `system | user | assistant`; it already maps cleanly to all target frameworks (LangChain treats `human`/`ai` as aliases of `user`/`assistant`; Strands has `user`/`assistant` with a separate system prompt; CrewAI is field-based). `tool`/`function`/`developer` roles are out of scope.
- Framework SDK targets at authoring time: LangChain Python (`langchain-core`, `langgraph`), LangChain TypeScript (`@langchain/core`, `@langchain/langgraph`), Strands Python (`strands-agents`), Strands TypeScript (`@strands-agents/sdk`), CrewAI Python (`crewai`). Samples pin/track compatible versions; the CrewAI JavaScript npm package is an unofficial third-party reimplementation and is deliberately not used.
- The docs site's existing sample-embedding mechanism (raw file import) and multi-version build pipeline are reused as-is; this feature adds pages and samples within those mechanisms rather than changing them, except for the redirect rule and the one-time frozen-snapshot backfill (FR-018).
- Framework SDKs are installed as sample-only dev dependencies (FR-005a); the heavy transitive weight (Strands→boto3/OpenTelemetry, CrewAI→chromadb/lancedb/tokenizers) is accepted as a CI-time cost in the samples toolchain and is kept out of every shipped `prompting-press-*` package's dependency set.
- Backfilling the Integrations pages into frozen `v0.1`/`v0.2` (FR-018) is a deliberate one-time exception granted because the project is pre-1.0 with only two released minor versions; it is not a standing pattern and later feature pages will appear only in `next` and subsequently-cut snapshots.
- The unversioned deep-link redirect is implemented at the static-site build layer (the same layer that already emits the root redirect), consistent with a static host that has no server-side routing.
- CrewAI's own `{placeholder}` interpolation and custom `system_template`/`prompt_template` mechanics are acknowledged in prose but not generated or driven by Prompting Press; recipes hand CrewAI final strings.

## Out of Scope

- Any new library API, method, or option (e.g. `.toLangchain()` / `.toStrands()` on a Prompting Press object) — explicitly rejected in favor of tested user-side recipes; would require a constitution amendment.
- Framework adapter packages that depend on a framework SDK (`prompting-press-langchain`, etc.).
- Storage adapters (file/DB/object-store loaders) — the push model is unchanged.
- Mapping or emitting framework-specific request-body content types (Strands `guardContent`, `toolResult`, tool-use blocks, cache points, reasoning/citation blocks).
- Expanding the role vocabulary beyond `system | user | assistant`.
- A standalone "Use cases" page (folded into homepage + FAQ instead).
- Runtime execution of the frameworks in CI beyond compile/type-check of the sample glue (no live LLM calls).
