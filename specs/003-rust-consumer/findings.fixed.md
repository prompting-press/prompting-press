# Findings Fixed Log — Spec 003 (review cycle, step 11b)

> 2026-06-26. 5 review lenses (code/tests/errors/types/comments) + simplify. No CRITICAL defects;
> the crate was independently confirmed sound (FFI-free, wraps the kernel, SEC-004 scrub real,
> determinism traced, comments accurate). Actionable findings batched + fixed.

## Fixed
- **CR-1 (IMPORTANT, code)** — `check()` mishandled a variant literally named `default`: it was
  analyzed twice and the declared arm never analyzed (silent miss in the headline lint). Fixed:
  `variants_to_check` removes the reserved name; added `FindingKind::ReservedVariantName` that flags
  such a prompt (the arm is unreachable/shadowed by the root body). + test.
- **ER-1 (IMPORTANT, errors)** — `insert_and_get` used `expect("just inserted")` (structurally
  unreachable but violates "no panic in src"). Rewritten with the `btree_map::Entry` API — returns the
  borrow, no re-get, no panic.
- **TS-1/2/3 (IMPORTANT, tests)** — added coverage: `get_source` (happy + UnknownPrompt + unknown
  variant), named-variant render (success + unknown→Kernel/UNKNOWN_VARIANT), and `ExcludedFeature`
  normalization (+ SEC-004 no-leak). 36 → 44 tests.
- **simplify (LOW)** — `check.rs` Err arm: `analysis_error_reason(&err)` computed once (was called
  twice), `detail` uses inline-capture `{reason}` matching the file's style.

## Accepted + documented (no code change)
- **TY-1** — `FieldError::code`/`AnalysisError.reason` are `String` (closed const vocab), NOT a Rust
  enum: deliberate cross-binding shape (code crosses PyO3/napi as a string; a Rust-only enum would
  diverge the binding shapes — C-06/Principle VII). Rationale now in error.rs docs.
- **TY-4** — `render`/`append` bound `V::Context: Default` (no-arg `validate()`) walls out
  context-carrying garde validators: intentional v1 scope; `render_with`/`append_with(ctx)` named as
  the future seam in the docs (not built — C-08 scope discipline).
- **ER-2** — `Value::from_serialize` infallibility (custom-Serialize failure → downstream
  strict-undefined): one-line comments at the call sites; unreachable for garde+std structs.

## Deferred (SUGGESTION, low value)
resolve-time unknown-prompt test, combined-findings-on-one-prompt test, constructed-object
render-parity, `passed()`/`is_empty()` synonym, `Display` arm factoring — recorded, not blocking.

Final: 44 tests green, clippy -D warnings + fmt clean.
