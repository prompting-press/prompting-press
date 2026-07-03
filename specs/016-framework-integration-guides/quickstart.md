# Quickstart: Validating Framework Integration Guides

How to prove this feature works end-to-end. No live LLM calls anywhere.

## Prerequisites

- Repo toolchain via `mise` (Node/pnpm, Python/uv, Rust) as used by CI.
- Framework SDKs installed into the doc-sample projects (added by this feature):
  - `docs/site/samples/python/` → `langchain-core`, `langgraph`, `strands-agents`, `crewai`
  - `docs/site/samples/typescript/` → `@langchain/core`, `@langchain/langgraph`, `@strands-agents/sdk`

## 1. Samples compile, type-check, and pass (FR-005/005a, C1–C3, SC-003/009)

```bash
# All doc-sample suites (the gate that also covers the new integration samples):
moon run docs:test-samples
# Or per language:
moon run docs:test-samples-python      # pytest runs each examples/*.py (incl. integrations_*)
moon run docs:test-samples-typescript  # tsc typecheck + node test over examples/*.ts
```
**Expected**: green. Each `integrations_*` sample constructs framework objects and asserts on the mapped shape (roles preserved, system hoisted, `{text}` blocks, field assignment). A shape drift in a real SDK fails here (that is the point of installing the real SDKs).

## 2. No framework dependency leaked into shipped packages (FR-007, SC-004)

```bash
# Should return NOTHING:
rg -n "langchain|langgraph|strands|crewai" packages/*/package.json packages/*/pyproject.toml crates/*/Cargo.toml
```
**Expected**: no matches. Framework deps live only under `docs/site/samples/**`.

## 3. Docs build across all versions with the new pages (FR-018/019, SC-006)

```bash
node docs/site/scripts/build-versions.mjs
```
**Expected**: exit 0. Then verify:
```bash
# Integrations pages built for next + frozen versions (backfilled):
for v in next v0.1 v0.2; do
  for p in integrations integrations/langchain integrations/strands integrations/crewai; do
    test -f "docs/site/dist/$v/$p/index.html" && echo "$v/$p ok" || echo "$v/$p MISSING"
  done
done
# Sidebar "Integrations" link is version-prefixed on each version:
rg -o 'href="[^"]*integrations[^"]*"' docs/site/dist/v0.2/index.html | head
```
**Expected**: all pages present on `next`, `v0.1`, `v0.2`; sidebar links prefixed (`/v0.2/integrations/...`, etc.), no dead links.

## 4. Unversioned deep-link redirect (FR-015/016/017, SC-005)

```bash
# A deep path that exists under latest gets a redirect stub to /v{latest}/...:
test -f docs/site/dist/getting-started/rust/index.html && \
  rg -o 'url=/v[0-9.]+/getting-started/rust/' docs/site/dist/getting-started/rust/index.html
# Root redirect still works:
rg -o 'url=/v[0-9.]+/' docs/site/dist/index.html
# A path with no latest-version page has NO stub (natural 404):
test ! -f docs/site/dist/nonexistent-page/index.html && echo "no bogus stub (correct)"
```
**Expected**: deep stub points at `/v{latest}/getting-started/rust/`; root stub intact; no stub for nonexistent paths.

## 5. Positioning content present (FR-010/011/012, SC-007/008)

```bash
# FAQ has both entries; homepage has use-cases; no principle numbers / no "provenance":
rg -i "jinja|chatprompttemplate|system.*prompt" docs/site/src/content/docs/faq.mdx
rg -i "use case|variant|migration|multilingual" docs/site/src/content/docs/index.mdx
rg -in "principle [ivx]+|provenance" docs/site/src/content/docs/{faq,index}.mdx docs/site/src/content/docs/integrations/ ; echo "^ should be EMPTY"
```
**Expected**: FAQ + homepage matches present; the principle/provenance grep is empty.

## Done-when
All five checks pass, mapping to SC-001…SC-009. The feature is docs + tested samples + a redirect; there is no service to run.
