# Feature Specification: Tested Documentation Samples & Consumer Sample Apps

**Feature Branch**: `014-tested-doc-samples`

**Created**: 2026-06-29

**Status**: Draft

**Input**: User description: "014 — Every code sample in the docs site must have a passing
automated test, gated as part of docs publishing, so examples can't rot. VERSION-AGNOSTIC:
each docs version's samples are tested against THAT version of prompting-press." **Folded in
(2026-06-29)**: the former backlog item #30 (end-to-end consumer sample apps) — build real,
runnable apps that consume the library's full feature surface, with their own unit tests, proving
the library works end-to-end. Per the fold decision: **sample apps live in a top-level `samples/`
directory**; **doc-sample tests live in the docs directory, as clear tests next to the code that
is ingested into the docs pages**.

## Clarifications

### Session 2026-06-29

- Q1: Architecture for getting samples tested — source-canonical for ALL samples vs. hybrid? → A: **Source-canonical for ALL samples**, including migrating the ~108 existing hand-written MDX blocks. Every doc code sample becomes a tested source file (Rust doctest / Python test / TS example) INJECTED into the MDX at build time — the same generate-from-source pattern as the shape page and the spec-011 API references. One mechanism, no exceptions; drift-proof; per-version pinning falls out of the frozen source tree (Q3).
- Q2: Promote inline `// =>` / `# =>` expected-output comments to real assertions? → A: **Auto-promote ALL** of them to assertions (`assert_eq!` / `==` / `expect().toBe`). SHA-256 hash fields (`template_hash` / `render_hash`) are the exception — verified by format/length (64-hex), never exact value, since they are content-addressed. Maximum rot-protection: a shown output that changes fails the build.
- Q3: Per-version library-pin mechanism for version-agnostic testing? → A: **Git-branch-implicit (lockfiles in the frozen tree)**. Under source-canonical (Q1), a frozen docs version snapshots the source-tree state — including `Cargo.lock` / `package-lock.json` / the Python lock — which already pin the library version. Running `cargo test` / `pytest` / vitest within that snapshot tests against the matching library version automatically. No extra version manifest field; consistent with git-owns-versioning (Principle V) and coordinates with spec 012's frozen versioned trees.
- Q4: Fold the backlog consumer-sample-apps item (#30) into this spec, or keep separate? → A: **Fold into this spec.** Doc-sample tests and consumer sample apps are the same concern — "every shown/claimed usage is a tested artifact in the repo." One spec, one publishing gate. The two differ only in scope: a doc sample is a snippet ingested into a page; a sample app is a complete consumer program. Both are tested in CI.
- Q5: Where do the two artifact kinds live? → A: **Two distinct homes.** (a) **Consumer sample apps → a top-level `samples/` directory** (`samples/{rust,python,typescript}/<app>/`), each a complete, independently-buildable consumer project with its own unit tests. (b) **Doc-sample tests → inside the docs directory, beside the code ingested into the pages** (e.g. `docs/site/samples/{rust,python,typescript}/…`), so a reader of the docs source sees the runnable test next to the snippet the page injects. The doc-sample tests are NOT the sample apps; they are the per-snippet tested sources from Q1.
- Q6: How are consumer sample apps pinned to a library version pre-publish (the packages are not on crates.io/PyPI/npm yet)? → A: **Local/workspace dependencies now; flip to published-version deps at launch.** Pre-publish, each sample app depends on the in-repo library via a path/workspace dependency (Cargo path dep, `uv` editable/path, pnpm `workspace:`), so the apps build and their tests run in CI against the working tree immediately. At v1 publish, a follow-up flips the sample-app manifests to the published version constraints (the apps double as the published-package smoke test). The flip is the only post-publish step; everything else is testable now.
- Q7: What is each consumer sample app — a realistic program, a feature-tour harness, or both? → A: **Both — a realistic CLI AND an explicit feature-coverage test suite.** Each app is a small but realistic CLI program (loads a prompt from YAML, validates typed vars, renders default + a variant, composes a 2-message prompt, runs `check()`, shows the guard + provenance hashes, handles an error path; the "hand to an LLM" step is a printed/stubbed placeholder, never executed). Alongside its behavioral tests, the app carries an explicit feature-coverage test suite that walks every feature in the full surface list (FR-014) and asserts on each, so SC-009 is provable by inventory, not inspection.
- Q8: How broad is the sample-app suite at v1? → A: **Exactly one app per language (3 total)** — Rust, Python, TypeScript — each exercising the full feature surface. No multi-app sprawl; no per-feature mini-apps. (Matches FR-013.)

## Overview

The docs site (~108 fenced code blocks across getting-started, guides, reference, and templates
pages; Rust ~33, Python ~37, TypeScript ~38) contains runnable code samples with inline
expected-output comments (`// => "..."`, `# => ...`). Today those samples are not automatically
verified — they can silently drift from the real library surface with each refactor or new spec.
Separately, there is no end-to-end proof that an external consumer can install the library and
exercise its full feature surface.

This spec closes both gaps with one mechanism — *every shown or claimed usage is a tested artifact
in the repo* — across two artifact kinds:

1. **Doc-sample tests** (Q1/Q5): every doc code sample becomes a real, tested source file living in
   the docs directory next to the page that ingests it, injected into the MDX at build time
   (source-canonical), with its expected output promoted to a real assertion (Q2). Tests run as a
   gated step in docs publishing. Per-version pinning falls out of the frozen docs tree (Q3).

2. **Consumer sample apps** (Q4/Q5/Q6): complete, runnable consumer programs under a top-level
   `samples/` directory — one per language, each exercising the library's full public feature
   surface (construct → validate → render → variants → composition → check → guard → provenance
   hashes → error handling) — each with its own unit tests. They depend on the in-repo library via
   local/workspace deps now and flip to published-version deps at launch, doubling as the published
   package smoke test.

The deliverable is a developer-facing trust guarantee: every snippet shown in the docs compiles,
runs, and produces the documented output against the library version that docs version ships with;
and a real consumer app per language proves the library works end-to-end.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — A broken sample is caught before it reaches the docs (Priority: P1)

A developer changes the library's public API (e.g. renames a method, changes a constructor
signature). One or more doc code samples use the old surface. The docs-publish CI gate runs the
sample tests and fails before the stale doc is published.

**Why this priority**: this is the core anti-rot guarantee — the gate must exist and must block
publication of a broken sample. Without this, the feature delivers nothing. It is the MVP slice:
one language's gate catching one broken sample is already valuable and independently deliverable.

**Independent Test**: introduce a deliberate breakage in one doc sample (e.g. call a method with
the wrong signature), run the sample-test gate, and confirm CI fails with an error that points to
the broken sample file and line.

**Acceptance Scenarios**:

1. **Given** a doc sample that calls `greet.render(vars)` and the library changes `render` to
   require a second argument, **When** the sample-test gate runs, **Then** CI fails with a
   compiler/runtime error citing the example file and **does not** publish the docs.
2. **Given** all doc samples correctly reflect the current API, **When** the sample-test gate runs,
   **Then** CI passes and the docs-publish step proceeds.
3. **Given** a sample test gate failure, **When** a developer reads the error output, **Then** the
   error message identifies the sample file (e.g. `docs/site/samples/rust/getting_started.rs:L14`)
   and the specific failure, not an opaque harness error.

---

### User Story 2 — All doc code samples across all three languages are covered (Priority: P2)

Every fenced code block that contains runnable code in the docs site — across Rust, Python, and
TypeScript, across getting-started, guides, and reference pages — is backed by a tested source
file. No page has untested samples that could drift silently.

**Why this priority**: P1 establishes the gate; P2 extends coverage to the full surface. Partial
coverage (e.g. only Rust) already blocks rot in that language; full coverage is the complete
guarantee. Stacked on P1.

**Independent Test**: run the coverage audit tool (`moon run docs:check-sample-coverage` or
equivalent) against the docs site and confirm it reports 0 uncovered fenced runnable code blocks
across all three languages.

**Acceptance Scenarios**:

1. **Given** the full docs site MDX tree, **When** the coverage audit runs, **Then** it reports
   100% of fenced runnable code blocks (those with a language tag other than pure config/YAML/TOML
   definition blocks) are backed by a corresponding example source file.
2. **Given** a new docs page is added with a Rust code block but no corresponding test file,
   **When** the coverage audit runs, **Then** it reports the missing test file by name and the gate
   fails.
3. **Given** the three language test suites, **When** each runs in isolation (`cargo test --doc`,
   `pytest`, `node:test` / vitest on TS examples), **Then** all pass without requiring
   the other two languages' environments to be present.

---

### User Story 3 — Expected-output comments in samples are verified as assertions (Priority: P3)

Doc samples include inline expected-output annotations (e.g. `result.text; // => "Hi Ada, you
have 3 messages."`, `# => "default"`). These are currently illustrative. This user story makes
them real assertions: a mismatch between the computed value and the annotated expected output
fails the test.

**Why this priority**: P1/P2 ensure samples compile and run; P3 ensures they produce the
documented result. A sample that runs but produces the wrong output is still a broken doc.
Stacked on P2 — assertions are only meaningful once coverage is full. SHA-256 hash fields
(`template_hash`, `render_hash`) are exempt from exact-match assertion (they are verified by
format/length only).

**Independent Test**: change the expected-output annotation in a sample (`// => "Hi Ada, you have
3 messages."` → `// => "Hello Ada"`) without changing the code, run the gate, and confirm it fails
with an assertion mismatch pointing to that annotation.

**Acceptance Scenarios**:

1. **Given** a Rust sample `result.text; // => "Hi Ada, you have 3 messages."`, **When** the
   test runs, **Then** `assert_eq!(result.text, "Hi Ada, you have 3 messages.")` (or equivalent)
   is verified.
2. **Given** a Python sample `result.text  # => "Hi Ada, you have 3 messages."`, **When** the
   test runs, **Then** the equality is asserted (via doctest `==` form or pytest `assert ==`).
3. **Given** a TypeScript sample `result.text; // => "Hi Ada, you have 3 messages."`, **When** the
   test runs, **Then** `expect(result.text).toBe("Hi Ada, you have 3 messages.")` is verified.
4. **Given** a sample that asserts a hash field (`result.template_hash; // => "9f2c…"`), **When**
   the test runs, **Then** the test checks `templateHash.length === 64` and `/^[0-9a-f]+$/.test(...)`,
   **not** an exact-value match (the exact hash is an implementation detail that changes with the
   template source).

---

### User Story 4 — A real consumer app per language proves the library end-to-end (Priority: P2)

A developer (or a prospective user evaluating the library) opens `samples/`, picks their language,
and finds a complete, runnable app that depends on the library exactly as an external consumer
would and exercises its full public feature surface. The app has its own unit tests; the
sample-app gate builds and tests all three in CI.

**Why this priority**: doc snippets prove individual call shapes; a sample app proves the features
compose into a working program — the integration guarantee a reader actually wants before adopting.
Independently deliverable: one language's working, tested app is already valuable. Stacked
conceptually beside US1/US2 (same "tested artifact" mechanism, different granularity), but it does
not block the doc-sample gate and can land in parallel.

**Independent Test**: from a clean checkout, run the sample-app gate (`moon run samples:test` or
equivalent); confirm each language's app builds against the in-repo library and its unit tests pass.
Then break a library API the app uses and confirm the app's build/test fails.

**Acceptance Scenarios**:

1. **Given** the `samples/rust/` app, **When** `cargo test` runs in it, **Then** it builds against
   the in-repo `prompting-press` crate (path dependency) and its unit tests pass, exercising
   construct/validate/render/variant/compose/check/guard/hashes/error paths.
2. **Given** the `samples/python/` and `samples/typescript/` apps, **When** their native test
   commands run, **Then** each builds against the in-repo binding (path/workspace dependency) and
   its unit tests pass over the same feature surface.
3. **Given** a library public-API change that breaks a feature a sample app uses, **When** the
   sample-app gate runs, **Then** it fails citing the app file — proving the apps are real
   consumers, not stale copies.
4. **Given** the pre-publish state, **When** a reviewer inspects the sample-app manifests, **Then**
   they use local/workspace dependencies (no crates.io/PyPI/npm version pin yet); the launch flip
   to published versions is the single documented post-publish step (FR-019).

---

### Edge Cases

- **Config/definition-only blocks**: fenced blocks tagged `yaml`, `json`, `toml` that are prompt
  definitions or config snippets (not executable code) are NOT required to have a backing test —
  they are data, not code. The coverage audit MUST distinguish runnable code blocks from
  definition-only blocks.
- **Tabs components**: many samples appear inside `<Tabs syncKey="..."><TabItem>` MDX structures.
  The injection and coverage tooling MUST handle samples nested inside Tabs without
  mis-attributing them to another page or skipping them.
- **Incomplete fragments**: doc pages may show intentional fragment snippets (e.g. a struct
  declaration without a `main`) that are designed to be read in context, not run standalone. Such
  fragments SHOULD be wrapped into a compile-check-only test (type-checks, no execute) rather than
  skipped entirely.
- **SHA-256 hash assertions**: `template_hash` and `render_hash` values are content-addressed over
  the template source; they change if the sample's template text changes. Expected-output
  assertions for hash fields MUST use format/length checks, not exact-match.
- **Version-agnostic pinning across docs versions**: a frozen v1.1 docs tree (spec 012) runs its
  samples against the v1.1 library, not the latest — pinned implicitly by the lockfiles in that
  frozen source tree (Q3 resolved; see FR-006).
- **Sample apps must not call an LLM or do real I/O the library forbids**: the apps consume the
  library (which performs no I/O / no LLM calls — Principle III). A sample app MAY show how a
  rendered prompt would be handed to a provider, but that call MUST be stubbed/illustrative and not
  executed in the test (no network in CI). The apps exercise the *library* surface, not a live model.
- **Sample apps are NOT doc-injected**: the `samples/` apps are whole programs; they are not
  injected into MDX pages (that is the doc-sample tests' job). A page MAY link to a sample app, but
  the app's source is not ingested block-by-block.

## Requirements *(mandatory)*

### Functional Requirements

#### Doc-sample tests (the docs-directory artifacts)

- **FR-001**: Every fenced code block in the docs site whose language tag identifies runnable code
  (i.e., `rust`, `python`/`py`, `typescript`/`ts`, `javascript`/`js`) MUST be backed by a
  tested source file in the repo.
- **FR-002**: The sample-test gate MUST run as part of the docs-publish CI workflow — docs MUST NOT
  be published if any sample test fails (gate is non-optional, same model as the conformance corpus
  gate from spec 006).
- **FR-003**: The source-canonical architecture MUST be used for **all** doc samples: real, tested
  example source files live in the repo and are injected into the MDX pages at docs build time — the
  MDX pages are never the source of truth for sample code. This is the same pattern as
  `gen-shape-table.mjs` + the spec-011 reference generator, extended to all doc samples. The ~108
  existing hand-written MDX code blocks MUST be migrated into tested source files (no hybrid /
  extract-from-MDX path is retained).
- **FR-003a**: The doc-sample source files MUST live **inside the docs directory, next to the code
  ingested into the pages** (Q5) — e.g. under `docs/site/samples/{rust,python,typescript}/` — so a
  reader of the docs source sees the runnable, tested source beside the snippet a page injects. They
  MUST NOT be co-located with the consumer sample apps in the top-level `samples/` tree (FR-013).
- **FR-004**: Per-language testing MUST use each language's native testing idiom:
  - **Rust**: `cargo test --doc` (rustdoc doctests in `///` doc comments) or `cargo test` on
    example files — whichever fits the sample's completeness level.
  - **Python**: `pytest` on the example files; docstring doctests via the `doctest` module are
    acceptable for simple inline assertions.
  - **TypeScript**: example files type-checked with `tsc --noEmit` and executed with `node:test`
    or vitest (TypeScript has no native doctest facility).
- **FR-005**: Doc-sample tests MUST be **build-time/dev-time only** — no new runtime dependency is
  introduced into any published library package (`prompting-press`, `prompting-press-py`,
  `prompting-press-node`). The test harness and example files are dev-only artifacts. (Principle
  II/III: FFI isolation and minimal boundary are not affected.)
- **FR-006**: The doc-sample feature MUST be **version-agnostic**: a frozen docs version (spec 012)
  MUST run its sample tests against the matching version of the library, not the latest. The pin is
  **git-branch-implicit**: a frozen docs version snapshots the source tree including its lockfiles
  (`Cargo.lock` / `package-lock.json` / the Python lock), which already pin the library version, so
  running the sample tests within that snapshot tests against the matching version automatically. No
  separate version-manifest field is introduced (consistent with Principle V / git-owns-versioning;
  coordinates with spec 012's frozen versioned trees).
- **FR-007**: ALL expected-output annotations in doc-sample files (`// => "..."`, `# => ...`) MUST
  be promoted to real assertions in the test (`assert_eq!` / `==` / `expect().toBe`) — a shown
  output that no longer matches fails the build. The sole exception: SHA-256 hash fields
  (`template_hash`, `render_hash`, `templateHash`, `renderHash`) MUST be verified by format and
  length only (64-char lowercase hex), never by exact value (they are content-addressed, not
  human-stable).
- **FR-008**: A coverage-audit step MUST report which fenced runnable code blocks in the MDX pages
  lack a corresponding tested source file. The audit MUST distinguish runnable code blocks from
  definition-only blocks (YAML/JSON/TOML prompt definitions and config snippets).
- **FR-009**: The injection step (source → MDX) MUST coordinate with `gen-shape-table.mjs` (spec
  011) to run in the same `prebuild` / `pregenerate` pipeline stage so there is one consistent
  build entry point. The injected content in MDX pages MUST be marked with a generator comment
  (e.g. `{/* AUTO-INJECTED from docs/site/samples/rust/... */}`) so it is not hand-edited.
- **FR-010**: Samples nested inside `<Tabs>` / `<TabItem>` MDX components MUST be covered by the
  injection and audit tooling; they MUST NOT be silently skipped due to MDX nesting.
- **FR-011**: The gate MUST produce error output that identifies the failing sample by its source
  file and line number (not just the MDX page), so a developer can fix the right file.
- **FR-012**: Fragment-only samples (intentional partial code snippets not runnable as a standalone
  program) SHOULD receive a compile-check-only test (type checking, no execution) rather than being
  excluded from coverage.

#### Consumer sample apps (the `samples/` artifacts)

- **FR-013**: A top-level `samples/` directory MUST contain one consumer sample app per language
  (`samples/rust/<app>/`, `samples/python/<app>/`, `samples/typescript/<app>/`), each a complete,
  independently-buildable consumer project (its own manifest, source, and tests). These are
  separate from the doc-sample sources under the docs directory (FR-003a).
- **FR-014**: Each sample app MUST be a small but realistic CLI program (Q7) AND exercise the
  library's full public feature surface end-to-end: prompt construction (inline builder AND
  from-YAML/JSON), validation, render (default + named variant), composition of a multi-message
  prompt, `check()`, the opt-in guard, the render-result provenance hashes, and error handling (a
  deliberately-triggered render/validation error). Any "hand the rendered prompt to a provider" step
  MUST be a printed/stubbed placeholder, never an executed network/LLM call (FR-018). Each exercised
  feature MUST be covered by an assertion in the app's own unit tests.
- **FR-014a**: Beyond its behavioral tests, each sample app MUST carry an explicit
  **feature-coverage test suite** (Q7) that walks every feature in the FR-014 surface list and
  asserts on each, so full-surface coverage (SC-009) is provable by an assertion inventory rather
  than by manual inspection.
- **FR-015**: Each sample app MUST depend on the library as an **external consumer would** — through
  the package boundary, not by reaching into library internals. Pre-publish (Q6) this is a
  local/workspace dependency (Cargo path dep / `uv` path / pnpm `workspace:`); the app code itself
  MUST import only the public API.
- **FR-016**: A sample-app gate (`moon run samples:test` or equivalent) MUST build and run the unit
  tests of all three apps in CI. A library public-API change that breaks a feature a sample app uses
  MUST fail this gate (proving the apps are live consumers, not stale snapshots).
- **FR-017**: Each per-language sample app's test suite MUST run independently without requiring the
  other languages' toolchains to be present.
- **FR-018**: The sample apps MUST NOT perform real network I/O or call an LLM in their tests
  (Principle III; CI has no network). Any "hand the rendered prompt to a provider" step MUST be
  illustrative/stubbed and excluded from execution.
- **FR-019**: Pre-publish, the sample-app manifests MUST NOT pin a crates.io/PyPI/npm version (the
  packages are unpublished). The feature MUST document the single launch-time flip from
  local/workspace deps to published-version constraints; after the flip the apps double as the
  published-package smoke test. No other part of this feature waits for publish.

### Key Entities

- **Doc-sample source file**: a real, tested code file (`.rs` doctest / example, `.py` example,
  `.ts` example) under the docs directory (e.g. `docs/site/samples/{rust,python,typescript}/`). It
  is the source of truth for a doc snippet; the MDX block is injected from it (FR-003/FR-003a).
- **Injection marker**: an MDX comment pair (`{/* INJECT: docs/site/samples/rust/greet.rs#render */}`
  … `{/* END INJECT */}`) that the build-time injector replaces with the fenced code block content
  from the referenced source file and anchor.
- **Coverage audit**: a script (moon task `docs:check-sample-coverage` or equivalent) that walks
  the MDX tree, identifies all fenced runnable code blocks, and reports which lack injection
  markers (i.e. are not backed by a tested source file).
- **Doc-sample-test gate**: a CI step (`moon run docs:test-samples` or equivalent) that runs all
  three language doc-sample suites and fails the build if any sample test fails.
- **Consumer sample app**: a complete, runnable consumer program under `samples/<lang>/<app>/` with
  its own manifest, source, and unit tests, depending on the library through the package boundary
  (FR-013/FR-015) and exercising its full feature surface (FR-014).
- **Sample-app gate**: a CI step (`moon run samples:test` or equivalent) that builds and tests all
  three sample apps and fails the build on any failure (FR-016).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of fenced runnable code blocks in the docs site MDX tree (tagged `rust`,
  `python`/`py`, `typescript`/`ts`, `javascript`/`js`) are backed by a tested source file. The
  coverage audit reports zero uncovered blocks.
- **SC-002**: A deliberately broken doc sample (wrong method name, wrong argument count) causes the
  doc-sample-test gate to fail CI and halts the docs-publish step.
- **SC-003**: Each per-language doc-sample suite runs independently without requiring the other
  languages' toolchains to be present (Rust suite: `cargo test`; Python suite: `pytest`; TS suite:
  `tsc` + vitest / `node:test`).
- **SC-004**: Each docs version's doc-sample tests run against the matching version of the library —
  the v1.1 frozen docs tree tests against the v1.1 library, not against a later release.
- **SC-005**: Expected-output annotations (`// => "..."`, `# => ...`) for all three languages are
  promoted to real assertions; the assertions pass on all three doc-sample suites. Hash fields
  (`template_hash`, `render_hash`) are verified by format/length (64-char hex), not exact value.
- **SC-006**: No new runtime dependency is added to any published library package; all example
  files and test harness code are confined to the docs/`samples` trees and dev-only tooling
  (verifiable by inspecting published package manifests).
- **SC-007**: Injected MDX blocks carry a generator comment; the coverage audit confirms no
  hand-edited sample blocks exist in the MDX tree without a corresponding injection marker.
- **SC-008**: `samples/` contains one buildable, tested consumer app per language; the sample-app
  gate builds and passes all three from a clean checkout.
- **SC-009**: Each sample app's feature-coverage suite (FR-014a) asserts on every feature in the
  full surface list (FR-014); the assertion inventory confirms no listed feature is unexercised.
- **SC-010**: Breaking a library public API a sample app uses causes the sample-app gate to fail,
  citing the app source file (proving live-consumer status).
- **SC-011**: Pre-publish, no sample-app manifest references a published package version; the
  launch-flip step is documented and is the only post-publish action for this feature.

## Assumptions

- **Spec 011 (generate-from-source)**: a gen-shape-table–style injection pipeline already exists
  (spec 011, `prebuild` / `pregenerate` step in `docs/site/scripts/`). This spec extends that
  pipeline for sample injection; it does not create a parallel build path.
- **Spec 012 (versioned docs)**: a frozen docs version is a git-tagged snapshot of the
  `docs/site/` tree plus its source dependencies. The per-version library-pin is git-branch-implicit
  via the snapshot's lockfiles (Q3 / FR-006).
- **~108 fenced code blocks in scope**: approximate count based on the docs site MDX tree at
  spec-010-delivery time (Rust ~33, Python ~37, TypeScript ~38). The exact count will shift as the
  site evolves; the coverage audit is the authoritative check, not a static number.
- **Definition-only blocks are excluded**: fenced blocks tagged `yaml`, `json`, `toml` containing
  prompt definitions (not executable code) are not runnable and are explicitly excluded from the
  coverage requirement.
- **No new published runtime deps**: the doc-sample-test harness and the sample apps are entirely
  dev-only / consumer-side; no changes to the published library manifests.
- **Moon is the build orchestrator**: moon tasks (`docs:test-samples`, `docs:check-sample-coverage`,
  `samples:test`) are the entry points, consistent with the rest of the project's build system.
- **Sample apps are pre-publish-buildable**: local/workspace deps let the apps build + test now;
  the published-version flip (FR-019) is the only launch-time step.

## Dependencies

- **Depends on**: 010 (the docs site / MDX tree the injection and audit tooling targets) and the
  library bindings (specs 002/004/005) the sample apps consume.
- **Coordinates with**: 011 (generate-from-source pattern — shares the `prebuild` pipeline and
  the injection-marker convention) and 012 (versioned docs — the per-version library-pin mechanism
  required by FR-006/SC-004).
- **Should land before**: 007 (v1 release publish) — so the publish gate already enforces both
  doc-sample correctness and end-to-end consumer-app health on day one; the only post-publish action
  is the FR-019 dependency flip.
- **Supersedes**: backlog item #30 (end-to-end consumer-app validation), folded in here per Q4.

## Out of Scope

- **Re-implementing or replacing `gen-shape-table.mjs`** (spec 011 deliverable) — this spec
  extends the same pipeline, not replaces it.
- **Testing non-code content** (prose, tables, diagrams, YAML/JSON/TOML prompt definition blocks).
- **A docs-specific linter for prose quality** (grammar, style).
- **Adding runtime behavior to the library** — this spec is build-time/dev-time + consumer-side
  only; no library behavior changes (Principle III).
- **Cross-language render-parity testing via doc samples or apps** — that is the conformance
  corpus's job (spec 006); these tests assert documented/expected output, not cross-language identity.
- **Calling a live LLM / real provider from a sample app** — illustrative only; never executed in
  CI (Principle III, FR-018).
- **The post-publish dependency flip's execution** (FR-019) — this spec builds + documents the
  mechanism and runs everything against local deps; flipping to published versions happens at launch.
