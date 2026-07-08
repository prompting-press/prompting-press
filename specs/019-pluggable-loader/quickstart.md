# Quickstart / Validation: pluggable prompt loader

Runnable scenarios; full tests in tasks.md. See [contracts](./contracts/loader.md).

## Prerequisites
- Workspace builds green; no new standard-package dependency.

## Scenario 1 — Load from filesystem behind the interface (US1)
`FileSystemLoader(base="prompts/")`; `loader.load("bull_v1")` → raw text of `prompts/bull_v1.yaml`;
compose `Prompt.from_yaml(loader.load("bull_v1"))`. Swap to another loader impl → call sites unchanged.

## Scenario 2 — Memory loader for tests (US2)
`MemoryLoader({"bull_v1": "role: user\nbody: ..."})`; present key → text; missing key → `LoadError`.

## Scenario 3 — Custom loader against the contract (US3)
Implement the interface (or pass a closure/callable/function) → usable anywhere a built-in is, no
registration. Its failure surfaces as a load error distinct from parse errors.

## Scenario 4 — Missing key is structured (FR-006)
Every loader: missing key → `LoadError` (never empty string/null). Assert distinct from a
construct-from-text error (FR-007).

## Scenario 5 — Path-traversal guard (SC-008, security)
`FileSystemLoader(base="prompts/").load("../secret")` → `LoadError`; no file outside `base` is read.

## Scenario 6 — Not fused / no container (FR-011/FR-012)
Confirm there is no `Prompt.load(loader,...)` and no name-keyed container object; loading and
construction are separate composable steps.

## Boundary checks (SC-005/006)
- `prompting-press-core` unchanged (no diff); construction still I/O-free.
- No cloud/third-party storage dependency added; both built-ins in the standard package.

## Amendment check (SC-007)
- Constitution: Loader reintroduced as earned opt-in seam; Principle III softened for a
  caller-invoked language-side loader (kernel + construction I/O-free); cites spec-017 repositioning;
  roadmap "Never: I/O" + Scope-Discipline updated; DECISIONS.md recorded; rendered copies in sync.
