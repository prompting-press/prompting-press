# Phase 1 Data Model: Framework Integration Guides

This feature adds no runtime data types to the library. The "entities" here are the
docs/sample artifacts and the shapes the recipes map between. Recorded so tasks and
review have a precise target.

## Artifact entities

### IntegrationPage
- **What**: one MDX page per framework under `docs/site/src/content/docs/integrations/`.
- **Instances**: `index.mdx` (intro/positioning), `langchain.mdx`, `strands.mdx`, `crewai.mdx`.
- **Fields**: frontmatter `title`, `description`; prose (positioning, footgun callout, out-of-scope note); one or more `?raw` sample imports per supported language.
- **Rules**: internal links root-absolute (so the remark plugin version-prefixes them); no future-tense prose (FR-020); no principle numbers / no "provenance" (FR-013/014); every embedded snippet is a real sample import, not inline prose (FR-005).

### RecipeSample
- **What**: a complete standalone program under `docs/site/samples/{python,typescript}/examples/`.
- **Naming**: `integrations_<framework>_<scenario>.<ext>` (e.g. `integrations_langchain_messages.py`, `integrations_strands_partition.ts`).
- **Fields**: imports (prompting-press + framework SDK); build of a Prompt/Composition from existing public API; the mapping; in-program assertions on the mapped shape.
- **Rules**: uses only existing PP public API (FR-006); no live LLM call (R5) — construct + assert only; must pass `docs:test-samples`; matches real SDK types (FR-005a).
- **Coverage cells**: LangChain {py, ts}, Strands {py, ts}, CrewAI {py}. (No Rust; no CrewAI ts — FR-004/021.)

### SampleProjectManifest
- **What**: the two doc-sample project manifests gaining framework dev deps.
- **Instances**: `docs/site/samples/python/pyproject.toml` (+`langchain-core`, `langgraph`, `strands-agents`, `crewai`); `docs/site/samples/typescript/package.json` (+`@langchain/core`, `@langchain/langgraph`, `@strands-agents/sdk`).
- **Rules**: framework deps scoped to these sample projects only; zero addition to any `packages/*` or `crates/*` manifest (FR-007, SC-004).

### SidebarConfig
- **What**: the Starlight `sidebar` array in `docs/site/astro.config.mjs`.
- **Change**: add an "Integrations" group with 4 links (`/integrations/`, `/integrations/langchain/`, `/integrations/strands/`, `/integrations/crewai/`).
- **Rules**: global across all version builds → requires the frozen-version backfill (see FrozenSnapshotBackfill) so no dead links (FR-018).

### FrozenSnapshotBackfill
- **What**: copies of the Integrations pages placed into `docs/site/src/versions/v0.1/` and `v0.2/` so the global sidebar link resolves there.
- **Rules**: one-time authorized exception (clarify Q2); content may be the same pages (version-appropriate); must survive each frozen build with correct version-prefixed links (FR-019). Non-precedential.

### DeepLinkRedirectRule
- **What**: build-time-emitted HTML redirect stubs for unversioned deep paths.
- **Owner**: `docs/site/scripts/build-versions.mjs` (extends `emitRootRedirect`).
- **Rule**: for each page slug present under the latest version, emit `dist/<slug>/index.html` → `/v{latest}/<slug>/`; root `/` unchanged; no stub for slugs absent under latest (natural 404). (FR-015/016/017.)

## Mapping shapes (what recipes convert between)

### Source: Composition result (existing, unchanged)
```
[ { role: "system"|"user"|"assistant", text: string }, ... ]   # ordered
```

### Target: LangChain messages
```
[ { role: "system"|"user"|"assistant", content: string }, ... ]   # key rename; invoke-ready
```

### Target: Strands
```
system:  string | None            # system-role texts, in order, "\n\n"-joined
convo:   [ { role: "user"|"assistant", content: [ { text: string } ] }, ... ]
# Agent(system_prompt=system, messages=convo)  /  new Agent({ systemPrompt, messages })
```

### Target: CrewAI (field assignment; no message array)
```
Agent(role=<rendered>, goal=<rendered>, backstory=<rendered>)
Task(description=<rendered>, expected_output=<rendered>)
```

## State / lifecycle
None. All artifacts are static build inputs; no runtime state, transitions, or persistence.
