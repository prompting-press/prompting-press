# Implementation Plan: Tested Documentation Samples & Consumer Sample Apps

**Branch**: `014-tested-doc-samples` | **Date**: 2026-06-29 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/014-tested-doc-samples/spec.md`

## Summary

Make every shown or claimed usage of the library a tested artifact in the repo, across two kinds:

1. **Doc-sample tests** — each runnable fenced code block in the docs becomes a real, tested source
   file living *in the docs directory* (`docs/site/samples/{rust,python,typescript}/`), injected
   into the MDX at build time (source-canonical, the spec-011 generate-from-source pattern). Inline
   `// =>` / `# =>` expected-output annotations are promoted to real assertions (hash fields by
   format/length only). A coverage audit + a test gate run in the docs-publish workflow.

2. **Consumer sample apps** — one realistic CLI per language under top-level
   `samples/{rust,python,typescript}/<app>/`, each depending on the library through the package
   boundary (local/workspace deps now, published-version flip at launch), exercising the full
   public feature surface, with behavioral tests *and* an explicit feature-coverage suite. A
   sample-app gate builds + tests all three in CI.

The library itself is untouched (Principle III): this is entirely dev-time tooling + consumer-side
code. Per-version pinning falls out of spec 012's frozen docs trees (git-branch-implicit lockfiles).

## Technical Context

**Language/Version**: build tooling in Node (the docs prebuild, matching spec 011's
`docs/site/scripts/*.mjs`); doc-sample sources + sample apps in Rust (1.95.0 stable, pinned via
mise), Python (3.12 floor), TypeScript (Node 22.12 floor). No library source change.

**Primary Dependencies**: existing only — the spec-011 prebuild pipeline (`gen-shape-table.mjs` +
`gen-api-refs.mjs`, now on `main`), moon as orchestrator, the three published bindings consumed via
path/workspace deps. The injection/audit tooling adds dev-only Node scripts; the apps add dev-only
consumer projects. **No new published runtime dependency** (FR-005, FR-019, SC-006).

**Storage**: N/A.

**Testing**: per-language native idioms (FR-004): Rust `cargo test`(+`--doc`), Python `pytest`,
TypeScript `tsc --noEmit` + `node:test`/vitest. Two new moon gates: `docs:test-samples` (+
`docs:check-sample-coverage`) and `samples:test`.

**Target Platform**: the docs-publish CI workflow + the repo's CI; both gate on the new checks.

**Project Type**: docs tooling + consumer-side sample code (no library behavior).

**Performance Goals**: none (CI-time gates).

**Constraints**: source-canonical for ALL doc samples (FR-003); doc-sample sources live in the docs
dir, NOT the top-level `samples/` tree (FR-003a / FR-013 separation); apps depend on the library as
external consumers, no internals (FR-015); no network/LLM in any test (FR-018); version-agnostic via
git-branch-implicit lockfiles (FR-006); injection coordinates with the spec-011 prebuild stage
(FR-009); no published runtime dep added (SC-006).

**Scale/Scope**: ~108 fenced blocks migrated to injected source files (Rust ~33 / Py ~37 / TS ~38);
3 consumer apps (one per language); 2 new CI gates; 1 documented launch-flip step.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Shared core / structural parity)** — ✅ No kernel change. Doc samples and apps are
  consumers of the existing bindings; cross-language *parity* is NOT tested here (that is the
  conformance corpus, spec 006) — these tests assert documented/expected output per language.
- **Principle II (FFI isolation)** — ✅ No binding-crate change; apps consume the published package
  surface. No FFI dep added anywhere.
- **Principle III (Minimal boundary)** — ✅ The library gains no behavior, no I/O, no LLM call. The
  sample apps MAY *illustrate* handing a rendered prompt to a provider, but that step is
  stubbed/printed and never executed in tests (FR-018). Build-time/dev-time + consumer-side only.
- **Principle IV** — N/A (no agreement/template-analysis change).
- **Principle V (Repo canonical; git owns versioning)** — ✅ Reinforced. Per-version pinning is
  git-branch-implicit (the frozen docs tree's lockfiles, FR-006) — no managed version axis is
  introduced in the library or a new manifest. The launch dep-flip (FR-019) is a git-tracked edit.
- **Principle VI (per-language idiom)** — ✅ Each doc-sample suite + each app uses its native test
  idiom; no forced-uniform harness.
- **Principle VII (schema is source of truth)** — ✅ Unaffected; YAML/JSON definition blocks remain
  definition-only (excluded from runnable-coverage, edge case).

**Scope Discipline (R1)**: no new pluggable interface. The injection/audit/gate scripts are concrete
build tooling (the spec-011 pattern extended), and the apps are concrete consumers. **PASS.**

**Boundary-defense triggers**: none — no I/O, LLM, request-body assembly, token counting, output
parsing, managed version axis, or new pluggable seam is added. The "hand to provider" illustration
is explicitly non-executed (FR-018).

**Result: PASS.** Re-check post-design: still PASS (the design adds only dev-tooling + consumer
projects).

## Project Structure

### Documentation (this feature)

```text
specs/014-tested-doc-samples/
├── plan.md              # this file
├── research.md          # Phase 0 — injection-marker format, coverage-audit MDX parse, app dep
│                        #   wiring per language, assertion-promotion grammar, 011/012 seam
├── data-model.md        # Phase 1 — the artifact kinds + the injection-marker + manifest entities
├── quickstart.md        # Phase 1 — prove: break a sample → gate fails; coverage audit = 0 gaps;
│                        #   apps build+test on local deps; assertions catch a changed output
├── contracts/
│   ├── injection-marker.md      # the MDX INJECT/END-INJECT marker + anchor grammar
│   ├── coverage-audit.md        # what counts as a runnable block; report shape; exit codes
│   └── sample-app.md            # the per-language app layout + dep mode + feature-surface list
├── checklists/requirements.md   # spec quality gate (clarifications resolved)
└── tasks.md             # Phase 2 (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

```text
docs/site/samples/                      # FR-003a — doc-sample sources, BESIDE the ingested code
├── rust/         *.rs   (doctest / example files, one per ingested snippet or snippet group)
├── python/       *.py   (pytest / doctest files)
└── typescript/   *.ts   (tsc + node:test/vitest files)

docs/site/scripts/                      # extends the spec-011 prebuild pipeline
├── inject-samples.mjs          # source → MDX injection (runs in prebuild, after gen-api-refs)
├── check-sample-coverage.mjs   # the coverage audit (runnable blocks ↔ injection markers)
└── lib/                        # shared parse helpers (reuse spec-011 lib where possible)

samples/                                 # FR-013 — top-level consumer sample APPS
├── rust/<app>/        Cargo.toml (path dep on ../../crates/prompting-press), src/, tests/
├── python/<app>/      pyproject (uv path dep on packages/python), app/, tests/
└── typescript/<app>/  package.json (workspace: dep on packages/typescript), src/, test/

moon.yml / docs/site/moon.yml / samples/*/moon.yml   # the two new gates:
  docs:test-samples + docs:check-sample-coverage      (doc-sample gate, in docs-publish)
  samples:test                                        (sample-app gate, in CI)

.github/workflows/{ci.yml, docs.yml}     # wire the gates; docs.yml already mise-bootstrapped (011)
```

**Structure Decision**: two physically separate trees enforce FR-003a/FR-013 by construction —
**`docs/site/samples/`** holds the per-snippet tested sources injected into pages; **top-level
`samples/`** holds the whole consumer apps. The injection harness is a sibling of spec-011's
`gen-api-refs.mjs` in the *same* prebuild stage (FR-009), so there is one build entry point and the
freshness model is identical (regenerate → assert no diff). The apps are ordinary consumer projects
wired by path/workspace deps so CI exercises them now; the only launch-time change is the FR-019
dependency-source flip.

## Phase 0 — Research (open items to resolve in research.md)

1. **Injection-marker grammar** (contract): the MDX comment-pair form
   (`{/* INJECT: docs/site/samples/rust/greet.rs#anchor */}` … `{/* END INJECT */}`) and the
   in-source anchor convention (named region markers in the `.rs`/`.py`/`.ts` files) — must survive
   MDX/Astro parsing and `<Tabs><TabItem>` nesting (FR-010), and be idempotent (re-inject → no diff).
2. **Coverage-audit classification** (contract): how to distinguish a runnable block (`rust`,
   `python`/`py`, `typescript`/`ts`, `javascript`/`js`) from a definition-only block
   (`yaml`/`json`/`toml`) and from a fragment-only snippet (FR-012, compile-check-only). Reuse the
   spec-011 MDX walk if one exists.
3. **Assertion-promotion grammar**: how `// => "..."` / `# => ...` map to `assert_eq!` / `==` /
   `expect().toBe`, and the hash-field exemption (64-char lowercase hex, format/length only, FR-007).
   Where the promotion happens (in the source file authored as a test, vs. a transform step).
4. **Per-language app dependency wiring** (contract): exact local/workspace dep syntax — Cargo
   `path = "../../crates/prompting-press"`, Python `uv` path/editable dep on `packages/python`,
   pnpm `workspace:*` on `packages/typescript` — plus the documented launch-flip to published
   versions (FR-019). Confirm each builds in CI without publishing.
5. **moon project topology**: docs/site is now a moon project (spec 012 added `docs/site/moon.yml`);
   decide whether `samples/` becomes one moon project or one-per-app, and how `docs:test-samples` /
   `samples:test` are declared (cacheable, locally runnable). Justify (mirror spec 012 R3 reasoning).
6. **Spec 011/012 seam**: the injection step runs in the same prebuild as `gen-api-refs.mjs`
   (FR-009, now on main); the per-version pin is the frozen-tree lockfiles (FR-006, spec 012). State
   that 014 builds on both (both now landed/landing) — no stubbing needed.
7. **CI wiring**: docs.yml is already mise-bootstrapped (the spec-011 fix) so the doc-sample gate has
   rust/uv/node available; confirm the sample-app gate's matrix (3 independent language legs, FR-017).

## Phase 1 — Design & Contracts (artifacts to generate)

- **data-model.md**: the entities (doc-sample source file, injection marker, coverage audit,
  doc-sample-test gate, consumer sample app, sample-app gate) and their relationships; no runtime
  state.
- **contracts/injection-marker.md**, **contracts/coverage-audit.md**, **contracts/sample-app.md** as
  above.
- **quickstart.md**: the end-to-end validation (break a doc sample → `docs:test-samples` fails citing
  the source file:line; coverage audit reports 0 gaps; change a `// =>` value → assertion fails;
  `samples:test` builds + tests all three apps on local deps; break a library API an app uses → gate
  fails).
- **Agent context update**: point AGENTS.md's plan reference at this plan.

## Work units (independently testable; map to the user stories)

1. **WU-A — doc-sample injection harness + coverage audit** (US1/US2): the `inject-samples.mjs` +
   `check-sample-coverage.mjs` scripts, the marker grammar, and migrating the ~108 blocks into
   `docs/site/samples/**`. Gate: a broken sample fails; coverage = 0 gaps.
2. **WU-B — assertion promotion** (US3): `// =>`/`# =>` → real assertions in the doc-sample sources;
   hash-field exemption. Gate: changing a shown output fails the build.
3. **WU-C — consumer sample apps** (US4): the three `samples/<lang>/<app>/` projects (realistic CLI +
   feature-coverage suite), local/workspace deps, the `samples:test` gate. Gate: apps build+test;
   breaking a consumed API fails the gate.
4. **WU-D — CI wiring + launch-flip doc** (cross-cutting): both gates wired into docs.yml/ci.yml;
   the FR-019 launch-flip documented. Gate: gates run in the right workflows; no publish enabled.

WU-A and WU-C are largely independent (different trees, different gates) and can proceed in parallel;
WU-B stacks on WU-A (assertions live in the injected sources); WU-D wires whatever A/B/C produce.

## Complexity Tracking

> No constitution violations. The two-tree split is the simplest way to enforce the FR-003a/FR-013
> separation the user mandated; the apps' local-dep mode is the only way to test consumer behavior
> pre-publish without violating the no-publish hold.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| _(none)_ | — | — |
