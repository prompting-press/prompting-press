# Phase 0 Research: provenance attributes helper

No open NEEDS CLARIFICATION. Decisions from grill + the 2026-07-08 clarify defaults.

## R1 — Where the keys + map-building live (parity)

- **Decision:** Define the four `prompting_press.prompt.*` key strings as `pub const`s in the consumer crate
  (`prompting-press`) plus a small **free function** `provenance_attributes_of(name,variant,template_hash,render_hash)
  -> BTreeMap<String,String>` (NOT an inherent `impl RenderResult` — the consumer re-exports the
  kernel type, so that is E0116) + an optional `ProvenanceExt` trait for Rust method ergonomics. Each binding's `RenderResult` method (`provenance_attributes` /
  `provenanceAttributes`) produces the SAME map (calls the consumer helper where it can; mirrors the
  same constants otherwise).
- **Rationale:** Each binding surfaces its own `RenderResult` (Python pyclass `render.rs`, napi
  struct `render.rs`, TS type) marshaled 1:1 from the kernel. Centralizing the key strings + map shape once (in a free function over the 4 fields) keeps every binding byte-identical (Principle I) and gives the conformance case a single
  source of truth. No kernel behavior change (FR-010) — the kernel struct is untouched; the helper
  lives in the consumer/binding layer.
- **Alternatives rejected:** (a) hardcode the key strings independently in each binding — drift risk,
  three places to change; (b) add the method to the kernel `RenderResult` — unnecessary kernel change
  and the kernel stays presentation-agnostic.

## R2 — Key strings: LIBRARY-OWNED namespace (corrected post-adversarial-review)

- **Decision:** `prompting_press.prompt.name`, `prompting_press.prompt.variant`, `prompting_press.prompt.template_hash`,
  `prompting_press.prompt.render_hash`. Documented: `name`/`variant` align with the emerging OTel GenAI
  convention; `template_hash`/`render_hash` are prompting-press provenance extensions in that
  namespace (not official). No false OTel-standard claim.
- **Rationale:** ecosystem-recognizable namespace without misrepresentation. Fixed, no config knob
  (C-08); consumers wanting other keys read the 4 public fields directly (FR-008).

## R3 — Return shape (clarify default)

- **Decision:** flat string→string map — Python `dict[str,str]`, TS `Record<string,string>`, Rust
  `BTreeMap<String,String>` (deterministic key order, D1 parity). A method (projection), not a
  property.
- **Rationale:** directly passable to a span's bulk set-attributes call; deterministic ordering makes
  the conformance parity assertion exact.

## R4 — No telemetry dependency (boundary)

- **Decision:** emit the `prompting_press.prompt.*` keys as plain strings; never link an OTel/telemetry SDK
  (FR-006). No callback fired during render (FR-005). Rejected the issue's `ProvenanceSink`/`OtelSink`.
- **Rationale:** Principle V ("no telemetry sink, no OTel coupling", written verbatim in the kernel
  RenderResult rustdoc). The softening amendment permits *formatting only*, not emission/deps.
