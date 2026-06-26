# Review Report — Spec 002 (Engine kernel)

**Date**: 2026-06-26 · **Scope**: `main...HEAD` kernel diff · **Lenses**: code, tests, errors, types, comments (6th, simplify, run after fixes)

5 review lenses ran (2 initially failed on wrong agent-type names, relaunched). Overall: **no CRITICAL
code-quality, error-handling, or type-safety defects.** The kernel is sound, idiomatic,
constitution-compliant (no I/O, no FFI, deterministic — all independently re-verified). Findings are
coverage gaps, a doc-attribution regression, and ergonomic derives.

## Triage → action

| ID | Lens | Sev | Finding | Disposition |
|----|------|-----|---------|-------------|
| CM-1 | comments | IMPORTANT | `constitution C-NN` misattribution (6×, 4 files) — C-0N are *roadmap* decisions, not constitution clauses (Principles I–VII). Regresses spec-001 analysis I2. | **FIX** |
| TS-C1 | tests | CRITICAL | `KernelError::Render` variant (render-time non-iterable) has zero coverage — spec Edge Case + FR-028. | **FIX** |
| TS-C2 | tests | IMPORTANT | `Some("default")` reserved-name resolution (FR-011) never tested directly (only `None` path). | **FIX** |
| TS-I1 | tests | IMPORTANT | Empty-body edge case (spec Edge Cases) untested end-to-end. | **FIX** |
| TS-I2 | tests | IMPORTANT | Unicode/multibyte content (spec Edge Cases) untested. | **FIX** |
| TS-I3 | tests | IMPORTANT | V1.5 multi-variant `None` asserts text-negative but not `variant == "default"`. | **FIX** (tighten) |
| TS-I5 | tests | IMPORTANT | `required_roots` never called with a named variant. | **FIX** (add 1 case) |
| TY-1 | types | IMPORTANT | `RenderResult` lacks `PartialEq`/`Eq` (asymmetric w/ Agreement/ProvenanceView; it's the content-addressed type tests most want to compare). | **FIX** |
| TY-2 | types | IMPORTANT | `GuardConfig` lacks `PartialEq`/`Eq`/`Default` (Default removes boilerplate for the dominant opt-out path). | **FIX** |
| TY-3 | types | SUGGESTION | `KernelError` lacks `PartialEq`/`Eq` (would enable structural test asserts vs Display). | **FIX** (cheap, enables better tests) |
| TS-I6/I7/I8, S-1..4 | tests | SUGGESTION | get_source(default) path, all-trusted guard→None, determinism structural-hash cross-check, diagnostic-not-assertion, display covers 2/5 variants. | **FIX the cheap high-value ones** (display all 5; all-trusted guard→None; get_source default) |
| CQ-1 | code | SUGGESTION | Per-render `Environment` re-parse. | ACCEPT (v1 simplicity; reviewer: no change) |
| CQ-2 / ER-1 | code/errors | SUGGESTION | `UndefinedVariable.name` best-effort (Display may read as if the sentence is the var name). | ACCEPT + note for spec-003 consumer (documented honestly already); optionally clarify Display wording |
| ER-2 | errors | SUGGESTION | `get_template` after `add_template_owned` maps the impossible `TemplateNotFound`→Render. | ACCEPT (structurally unreachable; v1 fine) |
| TY-S / ER-3 | types/errors | SUGGESTION | `GuardConfig` dead-state; heuristic per-call alloc (error path). | ACCEPT (FFI-ergonomics tradeoff; immaterial) |

## Strengths (independently confirmed across lenses)

- Boundary discipline: no `fs`/`net`/`env`/`io`/log in any logic file; `env!` is compile-time. FFI-free.
- Determinism enforced structurally — `HashMap` inputs laundered into `BTreeSet` outputs; no order leak.
- FR-016a parse-first short-circuit + FR-001a strict-undefined are structurally enforced, not just tested.
- `KernelError` closed-enum (C-08) deliberate + documented; `looks_like_excluded_feature` heuristic tight both ways; `env_globals` derives the allowlist live (no hardcode).
- SC-002 soundness exclusions + SC-008 excluded-feature rejection thoroughly tested.

## Next

Actionable findings (CM-1, the test gaps, the derives) routed to a single fix-findings pass, then
`simplify` review + re-verify. Accept-as-is items recorded above with rationale.
