# Findings Fixed Log

> Resolved during Phase-3 QA (after /speckit.verify), 2026-06-26.

## Summary

- **Findings resolved**: 1 (F-01, MEDIUM)
- **Findings deferred**: 0
- **Final status**: CLEAN

## F-01 (MEDIUM, FR-029) — render-regression guard was unbacked

**Source**: `/speckit.verify` (verify-report.md).
**Finding**: The render-regression fixtures (`tests/fixtures/render/{interpolation,conditional-loop}.json`)
and the `common::load_regression_case` loader existed, but no test RENDERED them and asserted output ==
expected — `scaffold.rs` only checked deserialization, and `conditional-loop.json` was consumed by
nothing. FR-029's explicit engine-regression guard was therefore unbacked (behavior was incidentally
covered via render.rs def-fixtures, but the dedicated guard did nothing).

**Fix**: Added `crates/prompting-press-core/tests/render_regression.rs` —
`render_fixtures_match_pinned_output` iterates both render fixtures, builds a default-arm
`PromptDefinition` from each fixture's `template`, renders via `render(&def, None, values, &no_guard())`,
and asserts `result.text == case.expected` byte-for-byte (+ variant == "default"). Now a real
regression guard; genuinely exercises the previously-near-dead loader. No fixture `expected` values
needed correcting (both already matched real render output — assertions are meaningful, not tautological).

**Verification**: 42 tests green (new `render_regression` suite included), clippy -D warnings clean,
fmt clean. Only `crates/prompting-press-core/tests/` touched; no kernel src, no spec-doc change.

## Review cycle (step 11b) — 2026-06-26

Five review lenses (code/tests/errors/types/comments) + simplify. No CRITICAL code/error/type defects;
the kernel was independently confirmed sound (no I/O, FFI-free, deterministic). Actionable findings
batched and fixed:

- **CM-1 (IMPORTANT, comments)**: 6 doc-comments said "constitution C-0N" — but C-0N are *roadmap*
  decisions (the constitution defines Principles I–VII, zero C-0N tokens). Regressed spec-001's
  ratified analysis-I2 fix. Reworded all 6 to "roadmap decision C-0N" across lib/error/hashing/
  agreement + generated/README.
- **TY-1/TY-2/TY-3 (types)**: added `PartialEq, Eq` to `RenderResult` and `KernelError`; added
  `PartialEq, Eq, Default` to `GuardConfig` (Default = the opt-out path). Enables structural test
  assertions + removes opt-out boilerplate.
- **Tests (TS-C1 CRITICAL + TS-C2/I1/I2/I5/I7/S-3)**: added 8 tests covering the previously-uncovered
  `KernelError::Render` variant (loop-over-int → InvalidOperation → Render; mapping confirmed correct,
  no bug), reserved `Some("default")` resolution (FR-011), empty-body + unicode edge cases, named-variant
  agreement analysis, all-trusted-guard→None, and Display for all 5 error variants. 42 → 50 tests.
- **TS-I3**: stale finding — V1.5 already asserts `variant == "default"`; not changed (the tests
  reviewer misread it).

**Accepted-as-is** (SUGGESTION-level, reviewers concurred no v1 change): per-render Environment
re-parse, `UndefinedVariable.name` best-effort (documented), GuardConfig flat-struct dead-state
(FFI-ergonomics), heuristic error-path alloc.

**Simplify pass**: one LOW-value parse-helper dedup proposed, declined — net-neutral, would duplicate
the borrow-split `get_template` anyway, and the inline parse-boundary comments are clearer in place
(C-08 scope discipline). No changes warranted.

Final: 50 tests green, clippy -D warnings + fmt clean.
