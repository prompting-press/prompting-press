---
document_type: security-review
review_type: tasks
assessment_date: 2026-07-08
codebase_analyzed: prompting-press (spec 018 — provenance attributes helper)
total_files_analyzed: 6
total_findings: 2
overall_risk: LOW
critical_count: 0
high_count: 0
medium_count: 0
low_count: 1
informational_count: 1
owasp_categories: [A09]
cwe_ids: [CWE-200, CWE-201]
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

# Security Review (Tasks): 018 — provenance attributes helper

## Executive Summary

**Overall risk: LOW.** A pure projection of four fields already on `RenderResult` into a flat map;
no I/O, no new input path, no telemetry dependency, no emission. The security-relevant decisions are
**exclusions the spec already makes correctly** (no rendered text, no guard text, no metadata in the
map). The two findings are preventive: keep those exclusions enforced by test, and note a
data-provenance nuance about hashes in telemetry. Nothing blocks implementation.

## Tasks Reviewed

T001–T020 in `specs/018-provenance-attributes/tasks.md` against spec.md (FR-006/FR-007 exclusions),
plan.md, data-model.md, contracts, constitution Principle V, docs/memory D2/D3.

## Vulnerability Findings

### SEC-001 (LOW) — Enforce the "no rendered content in the map" exclusion by test (CWE-200)
- **OWASP**: A09:2025 (information exposure)
- **Location**: `provenance_attributes()` (T003/T004/T005/T006); spec FR-007
- **CVSS**: 2.5 (LOW)
- **Detail**: The value of this helper being safe rests entirely on the map containing ONLY the four
  content-identity fields and NEVER the rendered `text`, `guard` text, prompt/variant `metadata`, or
  `output_model`. Auto-attaching rendered prompt text (which may embed untrusted interpolated values)
  or an opaque metadata bag to a telemetry span is the data-exposure + span-cardinality foot-gun the
  spec explicitly avoids. This must be a **hard, tested invariant**, not just prose.
- **Remediation**: T012 already asserts the exclusion — keep it and make it explicit per binding
  (assert exactly the 4 keys; assert `text`/`guard`/metadata/`output_model` are absent). Ensure a
  future field added to `RenderResult` does NOT silently flow into the map (the helper enumerates 4
  fixed keys, so it won't — document that the helper is an explicit allowlist, not a reflection).

### SEC-002 (INFORMATIONAL) — Content hashes in telemetry are content-identifiers, not secrets, but note the property (CWE-201)
- **OWASP**: A09:2025 (informational)
- **Location**: `gen_ai.prompt.template_hash` / `render_hash` in the emitted map
- **Detail**: `render_hash = SHA256(rendered text)`. It is a one-way digest, not the content — safe
  to emit. Worth a one-line doc note that the hash is a content *identifier* (useful to correlate a
  trace to a render) and does not expose the rendered text; and that callers who put the hash in a
  low-trust sink accept only that correlation capability. No action beyond documentation.
- **Remediation**: One line in T019 docs.

## Confirmed Secure Patterns

- **Explicit 4-key allowlist** — the helper emits fixed keys, not a reflection of all result fields,
  so new fields don't leak (FR-002/FR-007).
- **No emission / no sink / no dependency** — provenance stays data on the return value; nothing is
  pushed anywhere; Principle V preserved (formatting-only softening). Rejects issue #270's OtelSink.
- **No metadata flattening** — the opaque bag is excluded, so the library never interprets it and
  span cardinality stays bounded.
- **No I/O, no new input path** — nothing to inject into; the four values are library-generated
  (name/variant from the definition, hashes from the kernel).

## Task Sequencing Assessment

- Shared keys/helper (Phase 2) precede binding methods (Phase 3) and the exclusion test (Phase 4). ✅
- No hidden security work; T012 is the security-relevant guard and it's present. SEC-001 strengthens
  it (per-binding + allowlist note); SEC-002 is a doc line. No new phases.

## Action Plan & Next Steps

1. Fold SEC-001 into T012 (per-binding exact-4-keys + absence assertions; allowlist-not-reflection
   note); SEC-002 into T019 (hash-is-identifier doc line). Neither blocks.
2. No CRITICAL/HIGH/MEDIUM → no `security-review.followup` needed.
3. No durable security memory to capture (project-specific preventive notes; the exclusion pattern is
   already implied by Principle V + the opaque-metadata doctrine).

## Memory Hub INDEX.md Row

```text
| specs/018-provenance-attributes/security-review-report.md | tasks | 2026-07-08 | LOW | C:0 H:0 M:0 L:1 | A09 |
```
