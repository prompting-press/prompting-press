# Memory Synthesis

## Current Scope

Docs-only + tested-sample feature (spec 016): a new "Integrations" chapter (LangChain/LangGraph, Strands, CrewAI), expanded FAQ + homepage use-cases, and an unversioned deep-link redirect fix. Affected modules: `docs/site/**` (content pages, sidebar config, build-versions redirect logic), `samples/{python,typescript}/examples/**` (new tested recipe files + sample-only framework dev-deps), and a one-time backfill into frozen `docs/site/src/versions/v0.1` and `v0.2`. **No `prompting-press-*` library/engine code changes.**

## Relevant Decisions

- D2 — Parse detail preserved, Render detail scrubbed (Status: active, Source: decisions/2026-06-29). Reason included: only tangential — recipe samples call `render()`; sample prose must not imply render errors leak bound values. Low impact (samples use benign data).
- No durable decision exists for docs/samples/integration surface — this is the first feature in that area. D1/D3/A1 concern engine marshaling & render-error detail, not applicable here.

## Active Architecture Constraints

- Principle III (Minimal Boundary, NON-NEGOTIABLE): library produces rendered text + provenance only; MUST NOT assemble a provider request body (system/messages split, content blocks). Reason included: this feature's core design decision — the system/messages split is done in *user-side recipe samples*, never in shipped packages. Source: `.specify/memory/constitution.md`.
- Principle II (FFI Isolation) + Scope Discipline: no new pluggable seam; no framework dependency in `prompting-press-*`. Reason included: FR-007/FR-005a keep framework SDKs as sample-only dev-deps. Source: constitution.
- Docs-are-product / no future tense (HARD RULE): docs describe current shipping behavior only; unshipped-feature docs land with impl. Reason included: FR-020. Source: project memory `docs-are-product-no-future-tense`.
- Frozen-snapshot doctrine: frozen version trees are normally immutable. Reason included: this feature takes an explicit one-time backfill exception (FR-018), bounded to greenfield pre-1.0.

## Accepted Deviations

- One-time backfill of Integrations pages into frozen `v0.1`/`v0.2` snapshots (FR-018). Status: Accepted-Deviation. Reason included: user-authorized greenfield exception to the frozen-snapshot rule; explicitly non-precedential.

## Relevant Security Constraints

- Samples must not surface unsafe render detail or secrets; use benign example vars only (aligns with D2/SEC-004). Low impact — no real credentials, no live LLM calls in CI.

## Related Historical Lessons

- Changelog sidebar link (recent): a page can be published but orphaned if the global sidebar isn't updated / frozen versions lack the page — directly motivates FR-018's backfill + sidebar-entry requirement.
- Internal-link prefixing (PR #250): plain markdown links need version-prefixing via the remark plugin; new Integrations pages' internal links must be root-absolute so they get prefixed per-version.
- Worktree isolation for parallel branch work (memory): implementation should stay in this worktree.

## Conflict Warnings

- Soft: FR-018 backfill contradicts the frozen-snapshot doctrine. Resolved: user explicitly authorized as a one-time greenfield exception; recorded as Accepted-Deviation above. No hard conflict.
- No Principle III conflict: the system/messages split lives in user-side samples, not shipped library code — the boundary holds.

## Retrieval Notes

- Index entries considered: 5 (A1, D1–D3, workflow). Durable memory read: INDEX.md only (small); decision bodies not expanded (not applicable to docs scope). Constitution consulted for Principles II/III + Scope Discipline. Project auto-memory (MEMORY.md) consulted for docs-as-product + changelog-orphan + worktree lessons. speckit_memory MCP unavailable in this environment → used budget-bounded direct INDEX read per retrieval-order fallback. Budget: well under 900 words.
