# Implementation Plan: Framework Integration Guides

**Branch**: `016-framework-integration-guides` | **Date**: 2026-07-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/016-framework-integration-guides/spec.md`

## Summary

Ship a new docs "Integrations" chapter (LangChain/LangGraph, Strands, CrewAI), an expanded FAQ (why-vs-Jinja; system/user fit), a homepage use-cases section, and a build-time redirect for unversioned deep links — with **zero library/engine code changes**. Every code sample is a complete standalone program under `docs/site/samples/{python,typescript}/examples/`, embedded verbatim via Astro `?raw` and executed/type-checked by the existing `docs:test-samples` moon gate (spec 014 mechanism). The framework SDKs are added only to the doc-samples projects' dependency sets (sample-only dev deps), never to any shipped `prompting-press-*` package. The system/messages split and content-block wrapping live entirely in these user-side sample programs — the library keeps emitting neutral `[{role, text}]`.

## Technical Context

**Language/Version**: Docs content — MDX (Astro 7 + Starlight). Samples — Python ≥3.12, TypeScript (Node, `tsc`); no Rust integration samples (frameworks are Py/TS only). Build scripts — Node ESM (`docs/site/scripts/*.mjs`).

**Primary Dependencies**: Astro/Starlight (docs); doc-sample dev deps to ADD — Python: `langchain-core`, `langgraph`, `strands-agents`, `crewai`; TypeScript: `@langchain/core`, `@langchain/langgraph`, `@strands-agents/sdk`. `prompting-press` remains the samples' path/editable dep.

**Storage**: N/A (no I/O; static site).

**Testing**: `docs:test-samples` moon gate — Python via pytest (`tests/test_examples.py` executes each example), TypeScript via `tsc` typecheck + node test, Rust via `run-examples.sh` (unaffected). Docs build via `build-versions.mjs`. No live LLM calls: integration samples construct framework objects and assert on the mapping shape only.

**Target Platform**: GitHub Pages static host (no server-side routing — redirects are emitted HTML stubs).

**Project Type**: Documentation site + tested doc-sample programs. No application/service.

**Performance Goals**: N/A (docs). Constraint: CI sample-suite install time grows with the added SDKs (accepted per clarification Q1).

**Constraints**: Docs-are-product / no future tense; multi-version build must stay green (`next` + frozen `v0.1`/`v0.2`); no framework dep in shipped packages; no live network in CI samples.

**Scale/Scope**: 4 new docs pages (Integrations intro + 3 frameworks) + FAQ edits + homepage edit + sidebar edit; ~5 new sample programs per supported (framework × language) cell (LangChain Py+TS, Strands Py+TS, CrewAI Py); 1 build-script redirect change; 1 one-time frozen-snapshot backfill.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Verdict | Notes |
|---|---|---|
| **I. Shared Core, Structural Parity** | ✅ Pass | No engine change; rendering untouched. Samples consume existing byte-identical output. |
| **II. FFI Isolation** | ✅ Pass | No change to `-py`/`-node` crates. Framework SDKs enter only the doc-sample projects, isolated from all `prompting-press-*` (FR-005a/007). Verifiable by manifest inspection. |
| **III. Minimal Boundary (NON-NEGOTIABLE)** | ✅ Pass | The system/messages split + content-block wrapping — the things Principle III forbids the *library* from doing — are performed in **user-side sample programs**, not shipped code. Library still emits neutral `[{role, text}]`. No I/O, no LLM call, no request-body assembly enters any package. This is the load-bearing gate and it holds. |
| **IV. Typed Input Is the Differentiator** | ✅ Pass | Role enum unchanged (`system\|user\|assistant`); agreement check untouched. Samples use existing typed Vars. |
| **V. Repo Is Canonical; Git Owns Versioning** | ✅ Pass | No version axis added. The deep-link redirect is a build-time convenience over the existing per-version layout, not a managed version pin. |
| **VI. Per-Language Idiom** | ✅ Pass | Recipes are idiomatic per language (Python dicts, TS objects); no forced-uniform API — indeed, no API at all. |
| **VII. JSON Schema Single Source of Truth** | ✅ Pass | No schema change (role enum stays). Codegen untouched. |
| **Scope Discipline (R1)** | ✅ Pass | No new pluggable seam, no adapter package, no `.toX()` method. Explicitly rejected in spec Out-of-Scope. Mirrors the deliberate no-storage-adapters stance. |
| **Docs-are-product (project rule)** | ✅ Pass | FR-020 mandates current-behavior-only prose; samples are tested, so no rot. |

**One authorized deviation** (tracked in Complexity Tracking): backfilling frozen `v0.1`/`v0.2` snapshots (FR-018) is a one-time exception to the frozen-snapshot doctrine, user-authorized during clarify (greenfield, pre-1.0). Not a constitution principle violation — the frozen-snapshot rule is an operational doctrine, not a numbered principle — but recorded explicitly as non-precedential.

**Result: PASS.** No constitution amendment required. No unresolved NEEDS CLARIFICATION.

## Project Structure

### Documentation (this feature)

```text
specs/016-framework-integration-guides/
├── plan.md              # This file
├── spec.md              # Feature spec (with Clarifications)
├── memory-synthesis.md  # before_plan memory hook output
├── research.md          # Phase 0 — SDK shapes + redirect approach (this command)
├── data-model.md        # Phase 1 — page/sample/redirect entities (this command)
├── quickstart.md        # Phase 1 — how to validate the feature (this command)
├── contracts/           # Phase 1 — recipe input/output contracts (this command)
├── checklists/
│   └── requirements.md  # Spec quality checklist (passing)
└── tasks.md             # Phase 2 — /speckit-tasks (NOT this command)
```

### Source Code (repository root)

```text
docs/site/
├── astro.config.mjs                      # EDIT: add "Integrations" sidebar section (4 entries)
├── scripts/
│   └── build-versions.mjs                # EDIT: extend emitRootRedirect → also emit deep-path redirect stubs (FR-015/016/017)
├── src/
│   ├── content/docs/
│   │   ├── index.mdx                      # EDIT: add use-cases section (FR-012)
│   │   ├── faq.mdx                         # EDIT: add why-vs-Jinja + system/user-fit entries (FR-010/011)
│   │   └── integrations/                   # NEW chapter
│   │       ├── index.mdx                   # intro + positioning (FR-001)
│   │       ├── langchain.mdx               # Python + TS (FR-002/008)
│   │       ├── strands.mdx                 # Python + TS, partition recipe (FR-003/008)
│   │       └── crewai.mdx                  # Python only (FR-004/008)
│   └── versions/
│       ├── v0.1/…                          # EDIT (backfill): add integrations/ pages + sidebar parity (FR-018)
│       └── v0.2/…                          # EDIT (backfill): add integrations/ pages + sidebar parity (FR-018)
└── samples/                                # spec-014 tested doc-sample projects (the REAL sample root)
    ├── python/
    │   ├── pyproject.toml                  # EDIT: add langchain-core, langgraph, strands-agents, crewai (sample-only dev deps)
    │   └── examples/
    │       ├── integrations_langchain_*.py # NEW standalone programs (assert on mapping; no LLM call)
    │       ├── integrations_strands_*.py
    │       └── integrations_crewai_*.py
    └── typescript/
        ├── package.json                    # EDIT: add @langchain/core, @langchain/langgraph, @strands-agents/sdk
        ├── tsconfig.json                   # (verify example globbing picks up new files)
        └── examples/
            ├── integrations_langchain_*.ts # NEW standalone programs
            └── integrations_strands_*.ts
```

**Structure Decision**: Reuse the spec-014 doc-sample mechanism exactly — standalone programs under `docs/site/samples/{python,typescript}/examples/`, executed by the `docs:test-samples` gate, embedded verbatim in MDX via `?raw`. This is the single most important structural correction from the spec draft: the sample root is **`docs/site/samples/`**, not repo-root `samples/` (that is the separate spec-014 *consumer-app* project). Framework SDKs go in the two doc-sample projects' manifests only. No Rust integration samples (frameworks are Python/TypeScript). The redirect is emitted by the same build step that already writes the root redirect.

## Complexity Tracking

| Deviation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Backfill Integrations pages into frozen `v0.1`/`v0.2` (FR-018) | A single global Starlight sidebar entry would dead-link on frozen versions lacking the pages; user chose consistent nav across all versions | Per-version conditional sidebar rendering (Option C) needs sidebar machinery the site lacks; leaving old-version links inert (Option A) was declined by the user. Bounded as one-time greenfield exception, non-precedential. |
| Heavy sample-only dev deps (boto3 via strands, chromadb/lancedb via crewai) | Type-checking/executing samples against **real** SDK types (clarify Q1) is the only way "tested recipe" means the mapping actually matches the SDK | Local stub types (Option B) don't catch real SDK drift; compile-only (Option C) doesn't verify the shape. Weight is a CI-time cost, isolated from shipped packages. |
