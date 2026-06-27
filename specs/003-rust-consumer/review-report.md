# Review Report — Spec 003 (Rust consumer)

**Date**: 2026-06-26 · **Scope**: `main...HEAD` consumer diff · **Lenses**: code, tests, errors, types, comments (+ simplify after fixes)

All 5 lenses ran clean-context (no tool glitches this round; correct agent-type names). Overall: **no
CRITICAL defects.** The crate is sound, idiomatic, FFI-free, wraps the kernel without duplication,
SEC-004 scrub verified, comments accurate (no `constitution C-NN`, no stale `UntrustedOutsideGuard`,
no token-hook ghost). One real soundness edge in the headline lint + a handful of coverage/idiom items.

## Triage → action

| ID | Lens | Sev | Finding | Disposition |
|----|------|-----|---------|-------------|
| CR-1 | code | IMPORTANT (82) | `check()` `variants_to_check` pushes `"default"` then extends with all `def.variants` keys → a variant literally named `"default"` is analyzed TWICE (dup findings) AND its declared arm body is never analyzed (resolve_variant maps `Some("default")`→root body) — a SILENT miss in the headline lint. Schema forbids it but typify strips `propertyNames` and the loaders run no schema gate. | **FIX** — exclude reserved `"default"` from the named set (one line); optionally flag a `default`-named variant. |
| ER-1 | errors | IMPORTANT | `insert_and_get` uses `expect("just inserted")` — structurally unreachable but violates "no panic in src". | **FIX** — use the Entry API to return the borrow without re-get/expect. |
| TS-1 | tests | IMPORTANT | `get_source` (FR-010) has ZERO tests (happy path + UnknownPrompt + unknown-variant). | **FIX** — add 3 small tests. |
| TS-2 | tests | IMPORTANT | Named-variant render path untested end-to-end (all render tests pass `None`); unknown-variant→`ConsumerError::Kernel(unknown_variant)` not exercised at the consumer layer. | **FIX** — add a named-variant render test + an unknown-variant test. |
| TS-3 | tests | IMPORTANT | `ExcludedFeature` is the only `KernelError` arm with no normalization unit test. | **FIX** — 3-line test. |
| TY-1/TY-2 | types | IMPORTANT | `FieldError::code` (and `AnalysisError.reason`) are `String`-from-consts, not enums — consumers can't match exhaustively. | **ACCEPT + document**: deliberate cross-binding shape (code crosses PyO3/napi as a string; a Rust-only enum would diverge the bindings). Record the rationale in the error.rs doc. |
| TY-4 | types | SUGGESTION | `render`/`append` bound `V::Context: Default` walls out context-carrying garde validators. | **ACCEPT + document** as a deferred seam (`render_with`/`append_with`); v1 simplicity per C-08. |
| ER-2 | errors | SUGGESTION | `Value::from_serialize` swallows a custom-Serialize failure as `UNDEFINED` → misleading downstream error. | **ACCEPT** (unreachable for garde+std structs); a one-line code comment is enough. |
| TS-S* | tests | SUGGESTION | resolve-time unknown-prompt, combined-findings, constructed-object render-parity untested. | **DEFER** (low value; behavior is additive/covered structurally). |
| C-1/C-6 (comments), TY-6 | comments/types | SUGGESTION | minor doc terseness; `passed()`/`is_empty()` synonyms. | **DEFER** (cosmetic). |

## Strengths (independently confirmed across lenses)
- No logic duplication — render/agreement/variant/hash are all kernel calls; `check()` is pure set-arithmetic (C-01).
- Determinism traced end-to-end: BTreeMap registry + BTreeSet funneling; no HashMap order leaks into output (Principle I).
- SEC-004 scrub real (Parse/Render bind `detail:_`, fixed message; planted-secret tests pass).
- `From<KernelError>` exhaustive over the closed 5-variant enum, no wildcard → future variant = compile error.
- Comments accurate: zero `constitution C-NN`, zero stale `UntrustedOutsideGuard`, token-hook drop correctly documented as a drop; doctests pass.
- No caller-reachable panic except the (now-being-fixed) `insert_and_get` expect.

## Next
Actionable fixes (CR-1, ER-1, TS-1/2/3) + 2 doc notes (TY-1, TY-4) routed to one fix pass. Accept/defer
items recorded above. Then simplify-lens + re-verify the touched paths.
