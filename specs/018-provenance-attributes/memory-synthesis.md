# Memory synthesis — spec 018 (provenance attributes helper)

Compact planning context. Source of truth; ≤900 words.

## What already ships (verified in code)

- `RenderResult` (`crates/prompting-press-core/src/engine.rs:117`) carries: `text`, `name`,
  `variant`, `template_hash` (`SHA256(resolved variant source)`), `render_hash`
  (`SHA256(rendered text)`), `guard: Option<String>`. Its rustdoc says verbatim: *"Plain data
  returned to the caller — no telemetry sink, no tracing coupling."* — that is Principle V in code.
- No `metadata`/`output_model`/`version` on `RenderResult` — they live on the definition and are
  library-opaque. Bringing them into an attribute map would be a kernel change + interpretation of
  an opaque bag. Excluded.

## What spec 018 adds

- A **projection method** on the render result → flat `string→string` map of the **four**
  content-identity provenance fields, keyed by the GenAI convention `gen_ai.prompt.{name, variant,
  template_hash, render_hash}`. Pure getter; no I/O, no callback, no mutation, no dependency.
- Rejected alternative: a callback `ProvenanceSink` + built-in `OtelSink` (issue #270's proposal)
  — violates Principle V ("no telemetry sink, no OTel coupling") + re-adds a C-08-eliminated seam.

## Governing constraints

- **Principle V (softened here):** provenance stays data on the return value; the library MAY now
  FORMAT it into an attribute map, but still exposes no sink, no push/emit, no telemetry dep. The
  two hashes and their semantics are unchanged.
- **Principle III / C-03:** no kernel change (fields already exist), no I/O.
- **Principle VI / C-06:** native idiom — dict (Py) / Record (TS) / map or Vec<(String,String)>
  (Rust); a method (projection), not a stored property.
- **C-08 / Scope Discipline:** no config knob (alternate keys, field selection) — fixed convention
  keys; consumers wanting variation read the four public fields directly.
- **Opaque-metadata doctrine:** metadata/variants/output_model excluded (flattening = interpreting
  the opaque bag + span-cardinality foot-gun). Rendered text + guard text excluded (leak/size).

## Amendment

- 018 **cites** spec-017's v3.0.0 repositioning statement; makes its own edit = soften Principle V.
- Record in DECISIONS.md; propagate to constitution body/version + rendered CLAUDE.md/AGENTS.md.

## Motivating consumer

Every consumer today hand-writes `span.set_attribute("gen_ai.prompt.name", result.name)` ×4 with
typo-prone key strings. Helper collapses it to `span.set_attributes(result.provenance_attributes())`.

## Open questions → clarify

1. Exact final GenAI key strings (confirm against current semantic-convention spec at plan time).
2. Rust return type: `BTreeMap<String,String>` (ordered, deterministic) vs `Vec<(String,String)>`.
   (Lean BTreeMap — deterministic iteration, map-like.)
3. Method name parity: `provenance_attributes()` (Py/Rust) / `provenanceAttributes()` (TS).
