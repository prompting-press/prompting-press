---
document_type: security-review
review_type: tasks
assessment_date: 2026-07-08
codebase_analyzed: prompting-press (spec 019 — pluggable prompt loader)
total_files_analyzed: 8
total_findings: 4
overall_risk: MODERATE
critical_count: 0
high_count: 0
medium_count: 2
low_count: 1
informational_count: 1
owasp_categories: [A01, A03, A09]
cwe_ids: [CWE-22, CWE-59, CWE-209, CWE-400]
field_summaries:
  document_type: "Always 'security-review'. Allows indexers to skip non-review documents."
  review_type: "Which command generated this document: audit, branch, staged, plan, tasks, or followup."
  assessment_date: "ISO 8601 date the review was performed (YYYY-MM-DD)."
  overall_risk: "Highest severity tier with active findings (CRITICAL, HIGH, MODERATE, LOW, INFORMATIONAL)."
  critical_count: "Number of Critical findings (CVSS 9.0-10.0)."
  high_count: "Number of High findings (CVSS 7.0-8.9)."
  medium_count: "Number of Medium findings (CVSS 4.0-6.9)."
  low_count: "Number of Low findings (CVSS 0.1-3.9)."
  informational_count: "Number of Informational findings."
  owasp_categories: "OWASP Top 10 2025 categories (A01-A10) that have at least one finding."
  cwe_ids: "CWE identifiers referenced in this document."
  finding_id: "Unique finding identifier (SEC-NNN) for cross-referencing and task linkage."
  location: "File path and line number of the vulnerable code (path/to/file.ext:line)."
  owasp_category: "OWASP Top 10 2025 category for this finding (AXX:2025-Name)."
  cwe: "Common Weakness Enumeration identifier with short name (CWE-NNN: Name)."
  cvss_score: "CVSS v3.1 base score (0.0-10.0). 9.0+=Critical, 7.0-8.9=High, 4.0-6.9=Medium, 0.1-3.9=Low."
  spec_kit_task: "Spec-Kit task ID for backlog tracking and remediation follow-up (TASK-SEC-NNN)."
---

# Security Review (Tasks): 019 — pluggable prompt loader

## Executive Summary

**Overall risk: MODERATE.** This is the first feature to deliberately introduce **filesystem I/O**
into the library (the reason it carries a MAJOR amendment). The threat surface is real but bounded:
the loader reads files by a caller-supplied **key**, so path-traversal (CWE-22) and symlink escape
(CWE-59) are the primary concerns, plus error-message leakage (CWE-209) and unbounded-read DoS
(CWE-400). The spec already anticipates the biggest one (FR-002a/SC-008 traversal guard, task T004),
which is why this isn't HIGH — but the guard's **robustness** (symlinks, TOCTOU, cross-platform) is
under-specified for a security-relevant control, and two adjacent hardening items are missing from
the task list. All are resolvable pre-implementation by strengthening existing tasks.

## Tasks Reviewed

T001–T022 in `specs/019-pluggable-loader/tasks.md`, against spec.md (FR-002a, SC-008), plan.md,
research.md (R3), data-model.md (INV-4), contracts/loader.md, memory-synthesis.md, constitution
Principle III + C-08 + "Never: I/O", and docs/memory decisions D2/D3 (error scrubbing) + A1.

## Vulnerability Findings

### SEC-001 (MEDIUM) — Path-traversal guard robustness under-specified (CWE-22, CWE-59)
- **OWASP**: A01:2025 (Broken Access Control) / A03 (Injection-adjacent path handling)
- **Location**: `FileSystemLoader::load` (T004), spec FR-002a / SC-008
- **CVSS**: 5.8 (MEDIUM)
- **Detail**: T004 says "reject keys escaping `base` via canonicalize + prefix-check." That is the
  right idea but insufficient as written for a security control: (a) **symlinks** inside `base` can
  point outside it — canonicalizing the *final* path defeats a naive `base.join(key)` string-prefix
  check only if you canonicalize `base` too AND resolve symlinks (`std::fs::canonicalize` does, but
  it requires the file to exist — a missing file can't be canonicalized, so the guard must handle the
  not-found path without bypassing the check); (b) **TOCTOU** — canonicalize-then-open has a race if
  a symlink is swapped between check and read; (c) **cross-platform** — Windows (`\`, drive-relative,
  UNC, `NUL`), and keys with embedded NUL. A `..` after normalization, or an absolute key, must also
  be rejected.
- **Remediation**: Strengthen T004 into an explicit, tested guard: reject absolute keys and keys
  containing `..` components up front; resolve the final path and assert it is within a canonicalized
  `base`; document the symlink policy (recommend: reject symlinks that escape `base`, or open with
  `O_NOFOLLOW`-style semantics where available). Add these as **negative test cases** to T006 (Rust)
  and mirror in T010/T011 (Py/TS): `../secret`, `/etc/passwd`, `foo/../../etc/passwd`, a symlink
  inside base → outside, an absolute path, and (Windows) a backslash-traversal string.

### SEC-002 (MEDIUM) — No bound on read size; DoS via huge/streamed file (CWE-400)
- **OWASP**: A09:2025 / resource exhaustion
- **Location**: `FileSystemLoader::load` (T004); the loader returns the whole file as a `String`
- **CVSS**: 4.3 (MEDIUM)
- **Detail**: `load(key)` reads the entire file into memory. If `base` can contain (or a symlink can
  point at) a very large or unbounded pseudo-file (`/dev/zero`, `/dev/random`, a huge log), a single
  `load` can exhaust memory. Spec 009 (adversarial fuzzing) hardened the *render/parse* path against
  huge inputs but the loader is a NEW input path not covered there.
- **Remediation**: Decide + document a policy: either an optional `max_bytes` on `FileSystemLoader`
  (default a sane cap, e.g. a few MB, returning `LoadError` on exceed) or an explicit "unbounded,
  caller's responsibility" statement. Recommend a default cap (prompts are small). Add a task under
  Phase 2 and a test. The device-file case is largely mitigated by the SEC-001 symlink policy but the
  cap is defense-in-depth.

### SEC-003 (LOW) — LoadError must not leak file contents / secrets or full absolute paths (CWE-209)
- **OWASP**: A09:2025 (information exposure via errors)
- **Location**: `LoadError` construction (T002); default error message
- **CVSS**: 3.1 (LOW)
- **Detail**: Per D2/D3, default errors must be scrubbed. A `LoadError` should surface the **logical
  key** and an error class (`not_found`/`io`) but MUST NOT embed file **contents**, and should be
  cautious about echoing the **full absolute filesystem path** (info disclosure about deployment
  layout) or any secret-looking value in the key. T002 mentions scrubbing but doesn't enumerate the
  path-disclosure angle.
- **Remediation**: In T002, specify that `LoadError` default messages carry the logical key + code,
  not file contents, and prefer the relative key over the absolute resolved path in the default
  (full path only under the existing opt-in unsafe-detail flag, spec 013, if at all). Add a test
  (extend T006): a load failure on a path containing a secret-looking segment does not echo it
  verbatim beyond the key the caller already holds.

### SEC-004 (INFORMATIONAL) — Custom-loader trust boundary + async TS unhandled-rejection
- **OWASP**: A09:2025 (informational)
- **Location**: custom-loader contract (T015); TS async loader (T009/T011)
- **Detail**: (a) A custom loader is arbitrary caller code invoked by the caller — the library does
  not sandbox it (correct, but worth a one-line doc note that a custom loader's I/O/security is the
  implementer's responsibility). (b) The TS async `load(): Promise<string>` must document rejection
  behavior so a rejected promise surfaces as a `LoadError`, not an unhandled rejection.
- **Remediation**: One-line notes in T021 docs; ensure T011 tests a rejecting TS loader.

## Confirmed Secure Patterns

- **Traversal guard is in the spec, not an afterthought** (FR-002a/SC-008/T004) — the primary risk is
  acknowledged with a dedicated requirement + success criterion + test. SEC-001 hardens it, not adds
  it from scratch.
- **Kernel + construction stay I/O-free** (SC-005, FR-011) — the new I/O is confined to the loader
  leaf; the attack surface does not reach the kernel or the validating constructor.
- **Errors normalized + distinct** (FR-007/FR-008) — load errors are a separate, structured surface;
  reuses the existing scrubbed error family (D2/D3).
- **No new remote/cloud dependency** (SC-006) — heavier backends deferred; no supply-chain expansion.
- **MemoryLoader has no filesystem surface** — the recommended test/embedding loader is inherently
  free of CWE-22/59/400.

## Task Sequencing Assessment

- Secure foundation first: the interface + traversal-guarded FileSystemLoader + LoadError (Phase 2)
  precede the per-language facades (Phases 3–5). ✅
- **Gap:** the traversal guard (T004) and its negative tests (T006) exist but need the SEC-001
  hardening (symlink/TOCTOU/cross-platform) and SEC-002 size cap **added before implementation**, and
  SEC-003 path-disclosure scrubbing folded into T002. These are strengthenings of existing tasks, not
  new phases.
- No security work is hidden behind the amendment; the amendment itself (Phase 6) is governance.

## Action Plan & Next Steps

1. Fold into the tasks (pre-implementation): **SEC-001** → strengthen T004 + T006/T010/T011 negative
   cases (symlink escape, absolute, `..`, cross-platform, TOCTOU note); **SEC-002** → add a
   `max_bytes` default-cap task + test in Phase 2; **SEC-003** → enumerate path/secret scrubbing in
   T002 + a test; **SEC-004** → doc notes (T021) + TS rejection test (T011).
2. No CRITICAL/HIGH → no `security-review.followup` remediation spec required, but SEC-001/SEC-002 are
   MEDIUM and SHOULD be applied before the FileSystemLoader is implemented.
3. Durable memory: the **path-traversal-guard + read-cap pattern for any filesystem loader** is a
   reusable secure-by-design rule worth capturing once memory tooling is available (applies to any
   future storage backend under the new opt-in loader seam).

## Memory Hub INDEX.md Row

```text
| specs/019-pluggable-loader/security-review-report.md | tasks | 2026-07-08 | MODERATE | C:0 H:0 M:2 L:1 | A01,A03,A09 |
```
