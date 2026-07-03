# Phase 0 Research: Framework Integration Guides

All unknowns resolved. Sources: live SDK docs + published source (via research agents 2026-07-03), the extracted `@strands-agents/sdk` 1.7.0 `.d.ts`, and the in-repo docs-sample mechanism.

## R1 — LangChain / LangGraph message shape (Python + TypeScript)

**Decision**: Map a composition `[{role, text}]` to `[{"role": m.role, "content": m.text}]` (Python dicts / TS objects) and hand directly to `model.invoke(...)` / LangGraph state.

**Rationale**: LangChain natively coerces `{"role","content"}` dicts (and `(role, content)` tuples) to `SystemMessage`/`HumanMessage`/`AIMessage` at `.invoke()`, `MessagesState`, and `add_messages`. Prompting Press roles map 1:1 (`system`/`user`/`assistant`; LangChain also accepts `human`/`ai` as aliases). The mapping is a key rename (`text`→`content`).

**Note to document (FR-008)**: Do NOT route already-rendered text through `ChatPromptTemplate.from_messages` tuple/dict shorthand — it treats `content` as an f-string template and raises `KeyError` on literal `{...}` (e.g. JSON in rendered text). Feed the model/graph directly; PP already did the templating. (If typed message objects are wanted, map role→`SystemMessage`/`HumanMessage`/`AIMessage` explicitly — the public, stable surface.)

**Versions (authoring-time)**: Python `langchain-core` 1.4.8, `langgraph` 1.2.7; TS `@langchain/core` 1.2.1, `@langchain/langgraph` 1.4.7.

**Alternatives rejected**: `ChatPromptTemplate` (double-templating hazard, and PP already rendered); a maintained adapter package (1-line mapping doesn't justify a dependency + version matrix).

## R2 — Strands agent shape (Python + TypeScript)

**Decision**: Partition the composition. Hoist all `system`-role texts (in order, `\n\n`-joined; `None`/absent if no system message) into the separate system-prompt argument; map the rest to `{"role", "content": [{"text": ...}]}` messages. Construct `Agent(system_prompt=..., messages=...)`.

**Rationale**: Strands has no in-list `system` role — `Role` is exactly `user|assistant`; the system prompt is a separate `Agent` argument (`system_prompt: str | list[SystemContentBlock]` in Python; `systemPrompt?: SystemPrompt = string | SystemContentBlock[]` in TS). `content` must be a list of content blocks; a plain-text block is `{"text": ...}` (`TextBlock`/`TextBlockData`). Python `system_prompt=str` and TS `systemPrompt: string` both accept a bare string — PP's rendered string maps with zero transformation.

**Note to document (FR-008)**: Strands flattens system position — a mid-conversation system message is hoisted to the single system prompt; Strands cannot preserve mid-stream system placement. Flag as a framework limitation, not a PP capability.

**Out of scope (FR-009)**: `guardContent` (Bedrock-Guardrails vocabulary: `qualifiers ∈ {grounding_source, query, guard_content}`), `toolResult`, `toolUse`, `cachePoint`, `reasoningContent`, `citationsContent` — all provider request-body block types. Recipes emit `{"text": ...}` only.

**Versions**: Python `strands-agents` 1.45.0; TS `@strands-agents/sdk` 1.7.0 (repo `strands-agents/harness-sdk`, official).

**Alternatives rejected**: emitting content blocks other than text (Principle III / out of scope); a `.toStrands()` method (would need a Principle-III MAJOR amendment + is Python/TS-only, no shared-core parity).

## R3 — CrewAI shape (Python only)

**Decision**: Assign rendered strings directly to `Agent(role=, goal=, backstory=)` and `Task(description=, expected_output=)`. No `[{role,text}]` concept — CrewAI is field-based.

**Rationale**: These are plain `str` Pydantic fields; the rendered string is final. CrewAI abstracts the LLM call and the system/user split internally — the integrator never touches a message array.

**Note to document (FR-008)**: Do NOT also pass `crew.kickoff(inputs={...})` for variables PP already rendered — CrewAI's own `{placeholder}` interpolation would look for text that no longer exists. Hand CrewAI final strings; don't drive its `system_template`/`prompt_template` from PP.

**Versions**: Python `crewai` 1.15.1.

**TypeScript**: No official CrewAI TS SDK. The npm `crewai` (author `jaafarskafi1`, 2024) is an unofficial third-party reimplementation and `@crewai/core` is empty. **CrewAI ships a Python page only** (FR-004/FR-021); do not fabricate a TS sample.

## R4 — Dependency weight (informs sample-only-dev-dep decision, clarify Q1)

**Finding**: `langchain-core` 544KB/9 deps (light; `langgraph` re-exports its message types — no separate types package). `strands-agents` 566KB but pulls `boto3`+`botocore`+3×OpenTelemetry+`mcp`+`watchdog`. `crewai` 1MB/31 deps (`chromadb`, `lancedb`, `openai`, `pdfplumber`, `tokenizers`…).

**Decision**: Install all as **sample-only dev deps** in the doc-sample projects (`docs/site/samples/python/pyproject.toml`, `docs/site/samples/typescript/package.json`), isolated from every shipped `prompting-press-*`. Accept the CI install weight for real-type-checking fidelity (clarify Q1 = Option A). This weight is exactly why an *adapter package* per framework was rejected — the library must not carry these transitively.

## R5 — Tested-sample mechanism (spec 014, confirmed in-repo)

**Decision**: Each integration sample is a COMPLETE standalone program under `docs/site/samples/{python,typescript}/examples/`, embedded verbatim via Astro `?raw`, executed by the `docs:test-samples` moon gate (Python: pytest runs each; TS: `tsc` typecheck + node test). Assertions are in-program.

**Constraint**: No live LLM calls in CI. Samples construct framework objects (`Agent(...)`, message lists) and assert on the resulting shape (roles, system-prompt hoist, field values) — they do not invoke a model. This proves the mapping without network/keys.

**Rationale**: This is the existing, proven pattern (36 Python / 32 TS / 33 Rust programs today). The file the reader sees IS the tested artifact — no rot (FR-005), matches real SDK types (FR-005a).

**Path correction**: sample root is `docs/site/samples/`, NOT repo-root `samples/` (the latter is spec-014's separate *consumer-app* project). Spec prose said `samples/{python,typescript}/examples/`; the real, load-bearing path is `docs/site/samples/...`.

## R6 — Unversioned deep-link redirect (FR-015/016/017)

**Decision**: Extend `build-versions.mjs`'s `emitRootRedirect()` to also emit a redirect stub at each unversioned deep path that exists under the latest version, pointing to `/v{latest}/<path>/`. Reuse the existing meta-refresh + canonical + `location.replace()` stub form. The set of paths comes from the latest version's page slugs (already collected for the manifest). Root `/` redirect unchanged (FR-016). Paths with no latest-version equivalent get no stub → natural 404 (FR-017).

**Rationale**: GitHub Pages is a static host with no server-side rewrites; the only mechanism is pre-emitted HTML stubs (same technique already used for `/`). Driving the stub set from the latest version's actual slugs guarantees no stub points at a nonexistent page.

**Alternatives rejected**: a SPA/catch-all 404 handler (Pages `404.html` can't do a clean redirect-with-path reliably and hurts SEO); server rewrites (not available on Pages).

## R7 — Positioning content (FR-010/011/012)

**Decision**: FAQ gains two entries — "Why not just Jinja/minijinja?" and "How does this fit ChatPromptTemplate / the system-user split?"; homepage gains a use-cases section. Framing: PP uses minijinja for templating (so syntax is familiar); the differentiator is **structured storage + typed inputs + a build-time (static, no-data) agreement check vs. Jinja's runtime `StrictUndefined` + variants + byte-identical cross-language rendering**. System/user fit: PP emits neutral role-tagged text; each Integrations page shows the shaping.

**Rationale**: Directly answers the launch-thread questions (Joseph/Jinja, David/ChatPromptTemplate+split). The Jinja distinction is honest: Jinja *can* catch undefined vars, but only at runtime on the executed path — PP's edge is the static/CI check against a declared typed model.

**Constraint (FR-013/014)**: No principle numbers or the word "provenance" in user-facing prose; the per-render fingerprint is described in plain terms ("reproduce exactly what you sent").
