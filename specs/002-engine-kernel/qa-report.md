# QA Report — Spec 002 (Engine kernel)

**Date**: 2026-06-26 · **Mode**: CLI QA (library kernel — no web UI/server to drive) · **Verdict**: ✅ ALL PASSED

The "application" is the `prompting-press-core` Rust library kernel. It has no runnable app, server, or
UI — its acceptance criteria (the spec's V-scenarios and SCs) are exercised behaviorally by the
`cargo test` suite and the CI gates. QA = running the full suite + confirming each acceptance scenario
maps to passing test evidence. (No browser/Playwright mode applies.)

## Test Suite Results (evidence base)

`mise exec -- cargo test -p prompting-press-core` — **50 passed, 0 failed, 0 ignored**, across 11 suites:

| Suite | Tests | Result |
|-------|-------|--------|
| lib unit (incl. error, engine, env) | 10 | ✅ |
| agreement | 9 | ✅ |
| agreement_purity | 1 | ✅ |
| excluded_features | 5 | ✅ |
| hashing | 4 | ✅ |
| provenance | 6 | ✅ |
| render | 7 | ✅ |
| render_errors | 3 | ✅ |
| render_regression | 1 | ✅ |
| scaffold | 3 | ✅ |
| doctest (lib.rs) | 1 | ✅ |

No pre-existing failures; no new failures.

## Acceptance Scenario → Test Coverage Matrix

| TC | User story / scenario | Backing test | Result |
|----|----------------------|--------------|--------|
| TC-001 | US1 V1.1 render single body | render.rs | ✅ |
| TC-002 | US1 V1.2 determinism (render twice, equal hashes) — SC-001 | hashing.rs | ✅ |
| TC-003 | US1 V1.3 named-variant selection | render.rs | ✅ |
| TC-004 | US1 V1.4 unknown variant → error | render_errors.rs | ✅ |
| TC-005 | US1 V1.5 multi-variant None → root body as `default` | render.rs (asserts variant=="default") | ✅ |
| TC-006 | US1 V1.6 conditional + loop | render.rs / render_regression.rs | ✅ |
| TC-007 | US1 V1.7 strict-undefined → loud error — SC-009 | render_errors.rs | ✅ |
| TC-008 | US1 V1.8 get_source hash cross-check | hashing.rs | ✅ |
| TC-009 | reserved `Some("default")` resolution (FR-011) | render.rs | ✅ |
| TC-010 | render-time non-iterable → `KernelError::Render` (FR-028, Edge Case) | render_errors.rs | ✅ |
| TC-011 | empty body → "" + valid hashes (Edge Case) | render.rs + agreement.rs | ✅ |
| TC-012 | unicode/multibyte body (Edge Case) | render.rs | ✅ |
| TC-013 | US2 V2.1–V2.4 required-roots exclusions (loop/set/nested/globals) — SC-002 | agreement.rs | ✅ |
| TC-014 | US2 V2.6 undeclared var detectable — SC-003 | agreement.rs | ✅ |
| TC-015 | US2 V2.5 analysis purity — SC-006 | agreement_purity.rs | ✅ |
| TC-016 | US2 named-variant agreement analysis | agreement.rs | ✅ |
| TC-017 | US3 V3.1 provenance view | provenance.rs | ✅ |
| TC-018 | US3 V3.2/V3.3 guard opt-out/opt-in body byte-identical — SC-005 | provenance.rs | ✅ |
| TC-019 | US3 V3.4 guard override template | provenance.rs | ✅ |
| TC-020 | US3 V3.5 untrusted value unmutated — FR-025 | provenance.rs | ✅ |
| TC-021 | US3 all-trusted guard → None | provenance.rs | ✅ |
| TC-022 | V4.1/V4.2 excluded features rejected (6 constructs) — SC-008 | excluded_features.rs | ✅ |
| TC-023 | V4.3 excluded feature → analysis errs, not empty (FR-016a) | excluded_features.rs / agreement.rs | ✅ |
| TC-024 | FR-029 render-regression guard (renders + asserts == expected) | render_regression.rs | ✅ |

## CI Gate Results (acceptance for the build/quality SCs)

| Gate | Result |
|------|--------|
| `moon run :build` (workspace) | ✅ |
| `schemas:codegen-check` (codegen freshness) | ✅ |
| `ci:check-ffi` (SC-007 — FFI isolation) | ✅ |
| `ci:check-floating-versions` | ✅ |
| `ci:check-advisories` (cargo-deny) | ✅ |
| `clippy -p prompting-press-core --all-targets -- -D warnings` | ✅ |
| `cargo fmt --check` | ✅ |

## Metrics

- Acceptance scenarios: 24 mapped, 24 passed, 0 failed/partial/skipped.
- Test suite: 50 passed / 50 total.
- CI gates: 7/7 green.
- Coverage: every SC-001..009 and every quickstart V-scenario has ≥1 passing backing test (per the
  verify T035 reconciliation + the review-cycle additions).

## Verdict

✅ **ALL PASSED** — every acceptance criterion is met with passing test evidence; all CI gates green;
no failures. No app/server to drive (library kernel), so CLI/test-suite validation is the complete and
appropriate QA surface. Safe to proceed to code-review + security-review (steps 12/13).
