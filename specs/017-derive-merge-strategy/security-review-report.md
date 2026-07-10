---
document_type: security-review
review_type: tasks
assessment_date: 2026-07-08
codebase_analyzed: prompting-press (spec 017 — derive() merge strategy)
total_files_analyzed: 7
total_findings: 2
overall_risk: LOW
critical_count: 0
high_count: 0
medium_count: 0
low_count: 1
informational_count: 1
owasp_categories: [A09]
cwe_ids: [CWE-209]
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

# Security Review (Tasks): 017 — derive() merge strategy

## Executive Summary

**Overall risk: LOW.** This feature adds no I/O, no network, no new dependency, no authentication/
authorization surface, and no new external input path. It is an in-memory field-merge on prompt
definitions the caller already holds, routed through the **existing** validating constructor. The
threat surface is essentially unchanged from today's `derive`. Two low-severity, forward-looking
items relate to the project's existing error-scrubbing doctrine (D2/D3) and to metadata inheritance
visibility — both preventive, neither blocking.

No security task is missing that would need to precede implementation; secure foundations (the
re-validation path, error scrubbing) already exist and are reused, not rebuilt.

## Tasks Reviewed

T001–T026 in `specs/017-derive-merge-strategy/tasks.md`, against plan.md, spec.md, research.md,
data-model.md, contracts/derive-merge.md, memory-synthesis.md, constitution v2.0.0, DECISIONS.md,
docs/memory/INDEX.md (D1/D2/D3/A1).

## Vulnerability Findings

### SEC-001 (LOW) — Merge error paths must preserve render-error scrubbing (CWE-209)
- **OWASP**: A09:2025 (Security Logging & Monitoring Failures / information exposure via errors)
- **Location**: `crates/prompting-press/src/prompt.rs` (T005/T006 merge → `Self::new(merged)` error path)
- **CVSS**: 2.0 (LOW)
- **Detail**: A `Shallow` merge that produces an invalid definition surfaces a construction error.
  Per decision **D2/D3** (docs/memory/decisions), parse/agreement errors are pre-binding template
  syntax (safe to surface), but any error carrying **bound values** must stay scrubbed by default.
  Merge itself binds no values (it operates on the definition, not on render-time values), so this
  is preventive: the tasks MUST NOT introduce a new error path that echoes overlay *content*
  (e.g. a full variable-value dump) into the default (scrubbed) error.
- **Remediation**: In T005/T006, reuse the existing `ConsumerError` construction-error path
  verbatim; do not add a merge-specific error that interpolates overlay values. Add a negative
  test (extend T010) asserting a failed `Shallow` merge yields the same scrubbed error class as a
  failed plain `derive` — no overlay value content in the default message.

### SEC-002 (INFORMATIONAL) — `metadata` inheritance visibility under `Shallow` (CWE-209 adjacent)
- **OWASP**: A09:2025 (informational)
- **Location**: merge semantics for the opaque `metadata` map (data-model.md; T024 docs)
- **Detail**: Under `Shallow`, a child inherits the base's opaque `metadata` entries (which may
  include a `guard` key or model hints). This is not a vulnerability — metadata is library-opaque
  and echoed, never acted upon — but a consumer could unknowingly propagate a base's metadata
  (e.g. an internal label) into a derived prompt's provenance/metadata. Purely a
  least-surprise/documentation matter.
- **Remediation**: Document metadata inheritance under `Shallow` in T024 (mirrors critique E1). No
  code change.

## Confirmed Secure Patterns

- **No I/O, no new dependency** (SC-006, plan Technical Context) — no supply-chain or I/O attack
  surface added; A06/A08 not applicable.
- **Kernel untouched, validation-blind** (Principles I/III) — no new trust boundary; the merge runs
  in the consumer layer and the merged whole is re-validated by the existing constructor.
- **`validation_required` coverage re-checked against the merged set** (FR-009, T011/T013) — the
  merge cannot be used to smuggle an unvalidated `validation_required` variable past coverage
  (Py/TS raise; Rust compile-time). This is a positive security property of the design.
- **No untrusted-input parsing added** — overlays are caller-constructed in-process objects, not
  deserialized from an external source by this feature.

## Task Sequencing Assessment

- Secure foundations first: the shared merge core + re-validation (Phase 2) precede all binding
  work (Phases 3–4). ✅
- No security work hidden in later phases; the only security-relevant tasks are the scrubbing
  negative-test (fold into T010) and the metadata doc note (T024). ✅
- No parallel task bypasses a security prerequisite. ✅

## Action Plan & Next Steps

1. Fold SEC-001 (scrubbing negative test) into **T010**; SEC-002 (metadata inheritance doc) into
   **T024**. Neither is blocking; no `security-review.followup` remediation-task spec needed
   (no Critical/High findings).
2. No durable security memory to capture — findings are project-specific preventive notes already
   covered by the existing D2/D3 doctrine; no new systemic pattern.

## Memory Hub INDEX.md Row

```text
| specs/017-derive-merge-strategy/security-review-report.md | tasks | 2026-07-08 | LOW | C:0 H:0 M:0 L:1 | A09 |
```
