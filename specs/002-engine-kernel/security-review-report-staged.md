---
document_type: security-review
review_type: staged
assessment_date: 2026-06-26
codebase_analyzed: prompting-press / spec-002-engine-kernel (branch 002-engine-kernel, git diff main...HEAD)
findings_total: 4
findings_critical: 0
findings_high: 0
findings_moderate: 0
findings_low: 1
findings_informational: 3
overall_risk: low
owasp_categories:
  - "A03:2021 Injection (template/SSTI surface) — assessed in code; bound values are data, never re-parsed; guard text uses plain str::replace, not a re-render"
  - "A06:2021 Vulnerable and Outdated Components — minijinja 2.21.0 + sha2 0.10.9 pinned; cargo-deny advisory gate present and GREEN"
  - "A09:2021 Security Logging and Monitoring Failures — error detail may embed bound-value content; kernel never logs (confirmed by source scan)"
  - "A04:2021 Insecure Design — provenance tags / guard are advisory metadata, NON-enforcing; honest-boundary invariant now in rustdoc"
cwe_ids:
  - "CWE-1336: Improper Neutralization of Special Elements Used in a Template Engine (SSTI) — assessed in implementation; NOT present (values bound as minijinja::Value context, guard via str::replace)"
  - "CWE-1104: Use of Unmaintained Third-Party Components — cargo-deny unmaintained='all' + advisory gate green"
  - "CWE-209: Generation of Error Message Containing Sensitive Information — detail strings may carry bound-value content; kernel never emits/logs them (confirmed)"
  - "CWE-400: Uncontrolled Resource Consumption — no fuel/render limits; out of scope by construction (repo-canonical templates + in-process values)"
field_summaries:
  scope: "FFI-free, validation-blind, NO-I/O Rust template-rendering kernel (prompting-press-core), reviewed AS BUILT against git diff main...HEAD. Kernel source: engine.rs, agreement.rs, provenance.rs, hashing.rs, error.rs, lib.rs (generated/* excluded). Renders repo-canonical MiniJinja templates over already-validated minijinja::Value inputs; emits two SHA-256 content hashes + var-provenance metadata + an opt-in advisory guard string. No authn/authz, sessions, I/O, network, LLM calls, request-body assembly, token counting, or secrets handling — absent by constitutional construction (Principle III / C-03), not merely unimplemented. Confirmed by source scan: no fs/net/env-read/reqwest/tokio/log/tracing/println in kernel src."
  authentication: "N/A — the kernel performs no authentication. It is a set of pure library functions (render, required_roots, get_source, provenance_view) over pushed-in data. No credentials, tokens, or identity handling exist in the code."
  authorization: "N/A — no access-control decisions. Variant selection is caller-owned (C-05); resolve_variant only validates that a requested variant name exists, returning KernelError::UnknownVariant otherwise. No privilege logic."
  data_protection: "No data at rest, no transport, no secrets storage. Bound values are a minijinja::Value held in-memory for one synchronous render and never persisted, logged, or sent to a sink (confirmed: RenderResult is plain data, no telemetry). SHA-256 hashes are content-addressing identifiers (template_hash/render_hash), not security tokens — correct usage confirmed in hashing.rs."
  input_validation: "Kernel is deliberately validation-blind (FR-004): type validation is the consumer layer's job. Kernel-level input hardening is template-feature restriction — minijinja built with default-features=false WITHOUT macros/multi_template (Cargo.toml confirmed), so include/import/from-import/extends/macro/block are unrecognized tags that fail at parse time (add_template_owned parses eagerly in render and required_roots). Strict-undefined (UndefinedBehavior::Strict) turns a missing variable into a loud error, not a silent empty render. Bound values are passed as a Value context and never re-parsed as template source — the SSTI-defeating invariant, confirmed in engine.rs render()."
  injection: "Template-injection is the kernel's primary real risk class and the implementation holds. (1) Bound VALUES are passed to template.render(values) as a minijinja::Value context; there is NO path that feeds a value's string content back through add_template/parse, so a value containing {{ ... }} renders as literal text. (2) Guard expansion (build_guard_text) uses PLAIN str::replace of the {fields} placeholder — confirmed it does NOT re-enter MiniJinja; provenance.rs imports no minijinja symbol at all. A caller-controlled guard override template therefore cannot create a recursive-injection path. (3) macros/multi_template disabled at the crate-feature level removes the Jinja constructs most associated with SSTI/sandbox escape at parse time."
  cryptography: "SHA-256 via sha2 0.10.9 (RustCrypto), lowercase-hex, over UTF-8 string bytes (hashing.rs sha256_hex). Used purely for content identity in provenance/traces — never for authentication, adversary-resistant integrity, or secrecy. Collision/second-preimage concerns are not applicable to this content-addressing use. NIST test vectors (empty, 'abc') asserted in unit tests. No misuse."
  dependencies: "Two new pure-Rust deps, exact-pinned in Cargo.lock: minijinja 2.21.0 (default-features=false) and sha2 0.10.9. Neither pulls pyo3/napi — the ci:check-ffi gate (cargo tree -p <crate> -i pyo3/-i napi over prompting-press-core AND prompting-press) keeps the FFI boundary green. The SEC-001 plan recommendation is REALIZED: deny.toml is a real advisories config (version=2, unmaintained='all', unsound='all', yanked='deny', unknown registry/git denied) and the ci:check-advisories gate runs cargo-deny against Cargo.lock + the RustSec DB. Ran it during this review: 'advisories ok' — GREEN."
  error_handling: "KernelError variants carry free-form detail strings sourced from MiniJinja parse/render errors. Native KernelError is returned (not normalized — that is the consumer's job, C-06). The kernel never logs, prints, or emits these (confirmed: no println/eprintln/log/tracing in kernel src). The SEC-004 mitigation is realized in code: error.rs carries an explicit module-level rustdoc note that Parse/Render detail may embed bound-value content and the spec-003 normalization layer is responsible for scrubbing before logging."
  secrets_management: "N/A — the kernel handles no secrets, credentials, API keys, or connection strings, and does no environment access (the only env macro is the compile-time env!(\"CARGO_PKG_VERSION\"), not a runtime getenv). Bound values could contain caller secrets, but the kernel never persists, transmits, or logs them; the only path where value content could transit a boundary is KernelError detail returned to the in-process caller (SEC-004), which the kernel never itself emits."
---

# Security Review (staged / code-stage) — Spec 002: Engine kernel (`prompting-press-core`)

## Executive summary

**Overall risk: LOW. The implementation confirms the plan-stage LOW assessment, with no new code-stage findings.** This is the SpecKit step-13 confirmation gate: the question is not "what could the design do wrong" (the plan review answered that) but "does the committed code actually hold the line the plan drew." It does.

Every load-bearing security property the plan claimed was verified directly in the source on branch `002-engine-kernel`:

1. **Anti-SSTI core holds (CWE-1336, assessed → not present).** In `engine.rs::render`, bound values reach the template via `template.render(values)` where `values: minijinja::Value` — a *data context*, never re-parsed as template source. No code path feeds value content back through `add_template_owned`/parse. A value containing `{{ ... }}` renders as literal text.

2. **Guard text uses plain `str::replace`, not a re-render (the explicitly-flagged injection vector).** `provenance.rs::build_guard_text` expands the `{fields}` placeholder with a single `template.replace(FIELDS_PLACEHOLDER, &joined)`. `provenance.rs` imports *no* MiniJinja symbol at all (verified by grep) — a caller-controlled guard override template can therefore never re-enter the engine, closing the recursive-injection path the plan and the inline rustdoc both call out.

3. **Excluded features are off at the crate-feature level.** `Cargo.toml` builds `minijinja` with `default-features = false` and a feature set of `["builtins", "deserialization", "serde", "std_collections", "adjacent_loop_items"]` — `macros` and `multi_template` are absent (grep count: 0). `include`/`import`/`from`/`extends`/`macro`/`block` are unrecognized tags that fail at parse time in both `render` and `required_roots` (both call `add_template_owned`, which parses eagerly).

4. **No I/O, no logging, no secrets, no env read.** A targeted scan of the kernel source (excluding generated code) for `std::fs`/`std::net`/`std::env`/`reqwest`/`tokio`/`hyper`/`println!`/`eprintln!`/`dbg!`/`std::io`/`log::`/`tracing`/`std::process`/`File::`/`TcpStream` returned **zero real matches** — the single hit was the doc-comment phrase "no tracing coupling." The only `env` macro is the compile-time `env!("CARGO_PKG_VERSION")` in `lib.rs::version`, which is not a runtime environment read. This means the SEC-004 leak (error detail echoing bound-value content) is *latent in the returned type but never realized by the kernel itself* — it neither logs nor emits detail strings.

5. **SHA-256 is content-addressing, used correctly.** `hashing.rs::sha256_hex` is lowercase-hex SHA-256 over UTF-8 string bytes, used only for `template_hash`/`render_hash`. Never an auth/secrecy/adversary-resistance token. No `vars_hash` exists (so no structured-input canonicalization surface). NIST vectors asserted in tests.

6. **Supply-chain gate is real and green.** `deny.toml` is a genuine cargo-deny v2 advisories config (not a no-op): `unmaintained = "all"`, `unsound = "all"`, `yanked = "deny"`, unknown registry/git denied, `ignore = []`. Dependencies are exact-pinned in `Cargo.lock` (`minijinja 2.21.0`, `sha2 0.10.9`). I ran `mise exec -- moon run ci:check-advisories` during this review: **`advisories ok` — PASSED.** The FFI-isolation gate (`ci:check-ffi`) covers both `prompting-press-core` and `prompting-press`.

The two plan-stage findings that called for code/doc action — **SEC-002** (honest-boundary invariant for the non-enforcing guard) and **SEC-004** (error-detail leak note) — are **realized in the implementation**: the "does NOT sanitize / NO runtime enforcement" invariant appears as normative rustdoc in `lib.rs` (the `## What the kernel does NOT do` section), on `GuardConfig`, and on `ProvenanceView`; the SEC-004 leak note appears as a module-level rustdoc in `error.rs`. Both are downgraded accordingly below.

Finding counts (staged): **0 critical, 0 high, 0 moderate, 1 low, 3 informational.** None blocks anything. The single LOW (SEC-001) is satisfied-as-implemented and recorded only so the recurring obligation (re-confirm on each MiniJinja bump) is not lost.

## Diff reviewed (kernel files)

Reviewed `git diff main...HEAD`, focused on the kernel source. Generated code (`src/generated/**`, `src/generated.rs`) excluded per scope.

- `crates/prompting-press-core/src/engine.rs` (+307) — `build_environment`, `resolve_variant`, `get_source`, `RenderResult`, `render`, `map_minijinja_error`, `looks_like_excluded_feature`.
- `crates/prompting-press-core/src/agreement.rs` (+168) — `Agreement`, `required_roots`, `env_globals`.
- `crates/prompting-press-core/src/provenance.rs` (+156) — `ProvenanceView`, `GuardConfig`, `provenance_view`, `build_guard_text`, `DEFAULT_GUARD_TEMPLATE`.
- `crates/prompting-press-core/src/hashing.rs` (+60) — `sha256_hex`.
- `crates/prompting-press-core/src/error.rs` (+160) — `KernelError` + `Display`/`Error` impls.
- `crates/prompting-press-core/src/lib.rs` (+154) — crate docs, module wiring, public re-exports, `version()`.

Supporting supply-chain / boundary artifacts also confirmed: `deny.toml` (+95), `scripts/ci/check-advisories.sh` (+42), `ci/moon.yml` (+7), `crates/prompting-press-core/Cargo.toml`, root `Cargo.toml`, `Cargo.lock`, `scripts/ci/check-ffi-isolation.sh`.

## Vulnerability findings

### SEC-001 — Dependency-advisory gate present and green; recurring MiniJinja-bump obligation must stay bound to it (LOW)

- **OWASP 2025:** A06:2021 Vulnerable and Outdated Components.
- **CWE:** CWE-1104 (Use of Unmaintained Third-Party Components) — forward-looking; no vulnerable component identified.
- **Severity:** LOW (downgraded from a plan-stage process gap to a maintenance note, because the recommended control now exists and is green).
- **Location:** `deny.toml`, `scripts/ci/check-advisories.sh`, `ci/moon.yml` (`ci:check-advisories`), `Cargo.lock`.
- **Evidence:** The plan-stage SEC-001 recommendation (add a `cargo audit`/`cargo deny` advisory gate) is implemented. `deny.toml` is a real cargo-deny v2 advisories config: `[advisories] version = 2`, `unmaintained = "all"`, `unsound = "all"`, `yanked = "deny"`, `ignore = []`, and `[sources] unknown-registry = "deny"` / `unknown-git = "deny"` with only crates.io allowed — not a permissive no-op (only the `[licenses]`/`[bans]` sections are deliberately permissive on day one, and the gate script runs `check advisories` *only*, so those do not weaken the security gate). Versions are exact-pinned (`minijinja 2.21.0`, `sha2 0.10.9` in `Cargo.lock`). Running `moon run ci:check-advisories` produced `advisories ok`. The residual is purely operational: cargo-deny fetches the RustSec DB at runtime, so the gate is only as fresh as CI runs it, and the roadmap-Q3 "re-confirm `Template::undeclared_variables` stability on each MiniJinja bump" obligation is documented as a comment in both `deny.toml` and the gate script but is not itself machine-enforced.
- **Remediation:** None required for this spec. Maintenance: keep `ci:check-advisories` on every CI run (it already is, per `ci/moon.yml`), and honor the documented MiniJinja-bump procedure (re-run the gate + check the changelog for `undeclared_variables` deprecation) when bumping. Optionally pin the advisory-DB revision for fully reproducible offline runs. Tracking-only.

### SEC-002 — Non-enforcing guard / provenance tags: honest-boundary invariant is now in the code (INFORMATIONAL)

- **OWASP 2025:** A04:2021 Insecure Design (security-control assumption mismatch).
- **CWE:** CWE-655 (Improper Initialization of a security mechanism) — assessed; the documentation mitigation is in place.
- **Severity:** INFORMATIONAL (downgraded from plan-stage MODERATE: the plan's required action — a discoverable, normative "this does not sanitize" invariant — is implemented).
- **Location:** `crates/prompting-press-core/src/lib.rs` (crate-level `## What the kernel does NOT do … (normative — critique X1 / SEC-002)`), `crates/prompting-press-core/src/provenance.rs` (`GuardConfig` and `ProvenanceView` rustdoc).
- **Evidence:** The kernel does not enforce provenance tags — confirmed in code: `render` always calls `template.render(values)` regardless of any field's provenance, and `build_guard_text` only *names* fields, never touching values. That non-enforcement is now stated as a normative, discoverable contract in three places: the crate docs enumerate "The guard field does NOT sanitize," "Provenance tags are declarative metadata with NO runtime enforcement," and "`output_model` … never parsed"; `GuardConfig`'s rustdoc carries a bold "**This is NOT a sanitizer (critique X1 / SEC-002)**" block; `ProvenanceView`'s rustdoc carries "**These tags are declarative metadata, NOT runtime enforcement**." The pass-through is correct (values flow into `text` byte-for-byte; SC-005), and the API surface now signals the non-control nature at the type level. The residual is the same one the plan named: the *names* `untrusted`/`guard` still connote a sanitizer to a casual consumer, but that connotation is now actively contradicted in the doc surface a consumer reads.
- **Remediation:** No kernel change required. Carry the same invariant forward into the consumer specs (003/004/005) where binding-level Vars facades and `check`/lint surfaces wire tags to fields — the place a consumer is most likely to over-trust. (Optional, deferred: an `advisory_`-prefixed name; the doc invariant is the load-bearing fix and it is present.)

### SEC-003 — No render/analysis resource bounds (`fuel`/recursion limits) — out of scope by construction (INFORMATIONAL)

- **OWASP 2025:** A06:2021-adjacent (availability).
- **CWE:** CWE-400 (Uncontrolled Resource Consumption) — assessed, not applicable to the current threat model.
- **Severity:** INFORMATIONAL.
- **Location:** `crates/prompting-press-core/src/engine.rs::build_environment`; feature set in root `Cargo.toml`.
- **Evidence:** Confirmed in code that `build_environment` sets only `UndefinedBehavior::Strict` and adds no `fuel`/iteration cap, and that the `minijinja` feature set in `Cargo.toml` does not enable `fuel`. A pathological template (e.g. a huge bound-list loop, deep nesting) could in principle consume unbounded CPU/memory. For this kernel's threat model this is correctly out of scope: templates are repo-canonical / PR-gated (a billion-iteration loop is a code-review defect, not an external vector), `macros`/`multi_template` are disabled (no recursion-via-macro or include-expansion blowup), and the bound *values* — the only caller-supplied input — are sized by the in-process consuming application, which owns its own request limits. The kernel is not a multi-tenant service accepting attacker-controlled templates.
- **Remediation:** None for v1. **Trigger to revisit:** re-evaluate `fuel`/iteration limits only if a future spec lets templates OR loop-bound collection sizes become externally/attacker-controlled (e.g. a hosted authoring backend — on the roadmap "Never" list — or untrusted template upload). Recorded so the decision stays revisitable, not silently permanent.

### SEC-004 — Error detail may embed bound-value content; kernel never logs it, and the consumer-scrub note is in code (INFORMATIONAL)

- **OWASP 2025:** A09:2021 Security Logging and Monitoring Failures / A04 Insecure Design.
- **CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information).
- **Severity:** INFORMATIONAL (downgraded from plan-stage LOW: the kernel was confirmed never to log/emit these strings, and the consumer-scrub responsibility is now documented in the error contract in-code).
- **Location:** `crates/prompting-press-core/src/error.rs` (`KernelError::Parse`/`Render`/`ExcludedFeature` `detail` fields; module-level info-leakage rustdoc) and `engine.rs::map_minijinja_error`.
- **Evidence:** `map_minijinja_error` sources `detail` from `err.to_string()`, and a MiniJinja render-time error (e.g. a type error iterating a value) can include a representation of the offending — possibly caller-supplied, possibly sensitive — value. The genuinely confirmed mitigations: (a) the kernel **never logs, prints, or otherwise emits** these strings — the forbidden-primitive scan returned no `println`/`eprintln`/`log`/`tracing` in kernel src — so the only place detail crosses a boundary is the `KernelError` returned to the in-process caller, which is correct and useful; (b) `error.rs` carries an explicit module rustdoc: *"the `detail` strings on `KernelError::Parse` / `KernelError::Render` may embed bound-value content … Holding it in-process is fine; the spec-003 normalization layer is responsible for scrubbing it before logging."* The leakage is therefore latent-by-type but unrealized-by-kernel, with the consumer obligation documented at the source.
- **Remediation:** No kernel change required. Track as a requirement for the spec-003 error-normalization design: the consumer's `message` (the externally-surfaced field) must scrub/elide value fragments and confine raw `detail` to trusted debug logging. The kernel is the right place to *hold* the detail, the wrong place to assume it is safe to surface — and it correctly never surfaces it.

## Confirmed secure patterns

Verified to hold **in the committed implementation** (not merely planned). Recorded so a later reviewer need not re-litigate.

- **Values are data, never re-parsed as template syntax (anti-SSTI core).** `engine.rs::render` binds values via `template.render(values: minijinja::Value)`. No code path feeds a bound value's content back through `add_template_owned`/parse. CWE-1336: assessed, not present.
- **Guard expansion is `str::replace`, not a re-render.** `provenance.rs::build_guard_text` is `template.replace("{fields}", &joined)`; `provenance.rs` imports no MiniJinja symbol. A caller-controlled guard override cannot re-enter the engine — the recursive-injection path is closed in code, matching the inline invariant.
- **Excluded features off at the crate-feature level.** `Cargo.toml` `minijinja` feature set omits `macros`/`multi_template` (grep count 0); `render` and `required_roots` both parse eagerly via `add_template_owned`, so excluded tags fail at parse time. `agreement.rs` additionally parses *before* calling `undeclared_variables(false)`, foreclosing the documented empty-set-on-parse-error footgun (a broken/excluded template cannot masquerade as "requires no variables").
- **Strict-undefined backstop.** `build_environment` sets `UndefinedBehavior::Strict`; a missing variable is `KernelError::UndefinedVariable`, not a silent empty render — verified by the `builds_with_strict_undefined` unit test.
- **Globals allowlist derived from the live env, not hardcoded.** `agreement.rs::env_globals` reads `env.globals()`, so the subtraction set cannot drift from the actual engine config.
- **Hashing is content-addressing, not a token.** `hashing.rs::sha256_hex` — lowercase-hex SHA-256 over string bytes, NIST-vector-tested; no `vars_hash`; never used for auth/secrecy.
- **No I/O, no logging, no runtime env read.** Source scan clean; only `env!("CARGO_PKG_VERSION")` (compile-time). The no-I/O boundary is structurally true in the code, not merely asserted.
- **FFI isolation + advisory gate are CI-enforced.** `ci:check-ffi` covers `prompting-press-core` and `prompting-press`; `ci:check-advisories` runs cargo-deny and is green. Both new deps are pure-Rust and exact-pinned.
- **Purity / determinism.** `provenance_view`, `build_guard_text`, and `required_roots` take shared borrows and produce sorted `BTreeSet` outputs; no time/random/global state in the render or hash path.

## Proposed INDEX.md routing row

flash-mem / memory-hub not installed (markdown-only flow). Proposed row for the **Security** section of the memory index (create the heading if absent), kept distinct from the plan-stage row:

```markdown
## Security

- [Spec 002 staged (code-stage) security review](../../specs/002-engine-kernel/security-review-report-staged.md) —
  CODE-stage confirmation of the engine kernel against `git diff main...HEAD`. Overall risk LOW;
  no new findings — the implementation confirms the plan-stage LOW assessment. Verified in source:
  values bound as data (no SSTI), guard text via plain str::replace (no engine re-render), macros/
  multi_template off at the crate-feature level, no I/O/logging/env-read in kernel src, SHA-256 is
  content-addressing only. Plan follow-ups realized in code: SEC-002 non-enforcing-guard invariant
  now normative rustdoc (lib.rs / GuardConfig / ProvenanceView) → INFORMATIONAL; SEC-004 error-leak
  note now in error.rs and kernel never logs → INFORMATIONAL; SEC-001 cargo-deny advisory gate
  present and GREEN → LOW (maintenance: re-confirm on each MiniJinja bump); SEC-003 no fuel limits,
  out of scope by construction → INFORMATIONAL (revisit only if templates/value sizes become
  externally controlled).
```
