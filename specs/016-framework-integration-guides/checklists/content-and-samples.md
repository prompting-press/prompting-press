# Requirements Quality Checklist: Docs Content & Tested Samples

**Purpose**: PR-review-gate validation that the *requirements* for spec 016's docs content and tested samples are complete, clear, consistent, and measurable — before implementation. This tests the writing, not the code.
**Created**: 2026-07-03
**Feature**: [spec.md](../spec.md) · [plan.md](../plan.md) · [contracts/recipe-contracts.md](../contracts/recipe-contracts.md)
**Focus**: Docs content quality & accuracy; Tested-sample rigor
**Depth**: Standard (PR-review gate) · **Audience**: Reviewer

## Docs Content Completeness

- [ ] CHK001 - Are requirements defined for every framework × language cell the feature claims to cover (LangChain Py+TS, Strands Py+TS, CrewAI Py)? [Completeness, Spec §FR-002/003/004]
- [ ] CHK002 - Is the CrewAI TypeScript exclusion stated as an explicit requirement with its reason (no official SDK), rather than a silent omission? [Completeness, Spec §FR-004/FR-021]
- [ ] CHK003 - Are the required positioning artifacts each specified as distinct deliverables (FAQ "why-vs-Jinja", FAQ "system/user fit", homepage use-cases, Integrations intro)? [Completeness, Spec §FR-010/011/012]
- [ ] CHK004 - Does the homepage use-cases requirement enumerate all six value points, or does it leave "use cases" open to interpretation? [Completeness, Spec §FR-012]
- [ ] CHK005 - Is each framework page's required pitfall explicitly named (LangChain brace re-templating; Strands system-flattening; CrewAI kickoff double-fill), rather than a generic "document caveats"? [Completeness, Spec §FR-008]

## Docs Content Clarity & Accuracy

- [ ] CHK006 - Are the target SDK shapes the docs must reflect stated concretely enough to verify accuracy (LangChain `{role,content}`; Strands `systemPrompt` + `{role,content:[{text}]}`; CrewAI `Agent`/`Task` fields)? [Clarity, Research §R1–R3]
- [ ] CHK007 - Is the "why-vs-Jinja" requirement precise about the *honest* distinction (static/build-time check vs. Jinja's runtime `StrictUndefined`), so the page can't be written as vague marketing? [Clarity, Spec §FR-010]
- [ ] CHK008 - Is "neutral role-tagged text" defined unambiguously (role ∈ {system,user,assistant}; `text` string) so the system/user-fit FAQ entry has a concrete referent? [Clarity, Spec §FR-011, Data-model]
- [ ] CHK009 - Is the per-render fingerprint value point specified in user-facing terms with an explicit prohibition on the word "provenance", rather than left to author discretion? [Clarity/Consistency, Spec §FR-014]
- [ ] CHK010 - Is the prohibition on principle-number references stated as a checkable rule (not a vibe), e.g. via the SC-008 grep guard? [Measurability, Spec §FR-013/SC-008]
- [ ] CHK011 - Is "no future-tense / coming-soon" specified as a requirement over *this feature's* prose specifically, so it's reviewable per page? [Clarity, Spec §FR-020]

## Docs Content Consistency

- [ ] CHK012 - Are the out-of-scope framework content types (Strands `guardContent`/`toolResult`/etc.) consistently excluded across the spec, research, and contracts (no page allowed to imply support)? [Consistency, Spec §FR-009, Research §R2, Contracts §C2]
- [ ] CHK013 - Do the role-vocabulary statements agree across spec, plan, and data-model (unchanged `system|user|assistant`; `human`/`ai` only as LangChain aliases; no `tool`/`function`/`developer`)? [Consistency, Spec Assumptions, Plan Constitution Check]
- [ ] CHK014 - Is the sample root path stated consistently (`docs/site/samples/...`) after the plan's correction, with no lingering repo-root `samples/` references that would misdirect implementation? [Consistency/Conflict, Plan Structure Decision vs. Spec §FR-005]

## Tested-Sample Rigor (Acceptance-Criteria Quality)

- [ ] CHK015 - Does each recipe contract (C1–C3) state an objectively checkable assertion, not just "maps correctly"? [Measurability, Contracts §C1–C3]
- [ ] CHK016 - Is the "no live LLM call / no network / no keys" constraint specified as a hard requirement every sample must meet (not just a suggestion)? [Completeness, Research §R5, Contracts cross-cutting]
- [ ] CHK017 - Is "embedded code == tested file" (verbatim `?raw` import, no inline untested snippet) stated as a verifiable requirement with a check, rather than assumed? [Measurability, Spec §FR-005/SC-003]
- [ ] CHK018 - Is "real SDK types, not stubs" specified precisely enough to verify (sample-only dev deps installed; drift causes a type/exec failure), including *which* deps go *where*? [Clarity/Measurability, Spec §FR-005a/SC-009, Data-model SampleProjectManifest]
- [ ] CHK019 - Is the no-shipped-package-dependency requirement expressed as a checkable condition (grep of `packages/**`/`crates/**` manifests returns nothing) rather than a principle statement? [Measurability, Spec §FR-007/SC-004]
- [ ] CHK020 - For the Strands partition contract (C2), are the edge cases (multiple system messages; non-leading system message → hoist + flatten) captured as requirements the sample/prose must address? [Coverage/Edge Case, Spec Edge Cases, Contracts §C2]
- [ ] CHK021 - For LangChain (C1), is the brace-round-trip edge (literal `{...}` in rendered text survives) an explicit assertion requirement, tying the pitfall prose to a tested behavior? [Coverage/Edge Case, Contracts §C1]

## Dependencies, Assumptions & Ambiguities

- [ ] CHK022 - Are the authoring-time SDK versions recorded as an assumption with the acknowledgement that samples track compatible versions (so "accurate at authoring time" is a stated, bounded claim)? [Assumption, Spec Assumptions, Research §R1–R3]
- [ ] CHK023 - Is the frozen-snapshot backfill documented as an explicit, bounded, non-precedential exception (so a reviewer can confirm it isn't silent scope creep)? [Assumption/Consistency, Spec §FR-018, Plan Complexity Tracking]
- [ ] CHK024 - Are there any remaining ambiguous terms in the docs requirements ("short", "concise", "familiar") that lack a checkable definition and could produce inconsistent pages? [Ambiguity]
- [ ] CHK025 - Is it unambiguous whether the CrewAI page must show the framework's own `{placeholder}`/`system_template` mechanics or merely warn against double-filling (scope boundary of the CrewAI prose)? [Ambiguity, Spec §FR-008, Research §R3]

## Notes

- Items are requirement-quality questions ("is X specified/clear/consistent?"), not implementation tests. Answer during PR review of spec/plan/contracts; any "no" is a spec fix before `/speckit.analyze` or implementation.
- Traceability: every item cites a spec §, contract, research §, or a `[Gap]`/`[Ambiguity]`/`[Assumption]`/`[Conflict]` marker (≥80% requirement met).
