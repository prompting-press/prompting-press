# Feature Specification: Provenance attributes helper

**Feature Branch**: `018-provenance-attributes`

**Created**: 2026-07-08

**Status**: Draft

**Input**: Issue #270 (feat: ProvenanceSink). Resolved via design grilling to a
**projection helper** on the render result — NOT a callback sink, NOT built-in OTel coupling.
The library formats provenance it already returns into a flat, telemetry-ready attribute map;
it never emits, pushes, or depends on any telemetry system.

## Clarifications

### Session 2026-07-08 (proposed defaults — user away; revisit if needed)

- Q: Which exact attribute-key strings should the helper emit? → A: (SUPERSEDED by the addendum
  below — the OTel-alignment premise was wrong.) A **library-owned** `prompting_press.prompt.*`
  namespace for all four keys.
- Q: What return shape per binding? → A: A flat string→string map — Python `dict[str, str]`,
  TypeScript `Record<string, string>`, Rust **`BTreeMap<String, String>`** (deterministic key
  order, D1 parity discipline; directly passable to a span's bulk set-attributes call).

### Session 2026-07-08 (addendum — post-adversarial-review)

- Q: Are the `gen_ai.prompt.*` keys actually the right namespace? → A: No (verified). Only
  `gen_ai.prompt.name` exists in the OTel GenAI convention; **`gen_ai.prompt.variant` is not a
  convention key**, and bare `gen_ai.prompt` is **deprecated**. So 3 of 4 keys were misrepresented.
  Resolution: use a **library-owned `prompting_press.prompt.*` namespace** for all four keys; DROP
  the "aligns with the OTel convention" claim; doc-note that a consumer MAY remap onto their tracer's
  convention. Honest, consistent, and future-proof (OTel can't collide with our namespace).
- Q: Can the Rust helper be an inherent method on `RenderResult` in the consumer crate? → A: No
  (verified E0116). The consumer **re-exports** the kernel's `RenderResult`
  (`crates/prompting-press/src/lib.rs:178`), so an inherent `impl` there violates the orphan rule.
  Resolution: a shared **free function** builds the map from the four field values; an optional
  **extension trait** gives Rust `result.provenance_attributes()`. Python/TS expose the method on
  their own `RenderResult`, calling the shared field-based builder. No kernel change (SC-006 holds).
- Q: v3.0.0 coordination? → A: 017 is the v3.0.0 baseline (already written); 018 **cites** its
  repositioning statement and adds ONLY the Principle V softening as an **additive amendment →
  v3.1.0** (019 = v3.2.0). No re-declaration. Merge after 017.

## Iterations

### Iteration 2026-07-08: fold adversarial-review findings

**Change**: Switched keys from the (mostly non-existent) `gen_ai.prompt.*` to a library-owned
`prompting_press.prompt.*` namespace (dropped the false OTel-alignment claim); fixed the Rust surface
to a shared free function + optional extension trait (an inherent method on the re-exported kernel
`RenderResult` is E0116); strengthened the 4-key allowlist-not-reflection exclusion; added the
render_hash-is-identifier note; reconciled the amendment to cite 017's v3.0.0 baseline as an additive
v3.1.0.
**Scope**: Feature-wide (pre-implementation; Rust surface redesign + key-namespace change).
**Artifacts updated**: spec.md, plan.md, research.md, data-model.md, contracts/provenance-attributes.md, tasks.md.
**Tasks marked complete**: none (0 of 20 built).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Attach render provenance to a telemetry span in one call (Priority: P1)

A consumer renders a prompt and wants to record its content-identity provenance (prompt name,
selected variant, and the two content hashes) onto their observability span/trace, using
stable library-owned attribute keys, without hand-writing the key strings and
field mapping after every render.

**Why this priority**: This is the entire motivating use case — today every consumer hand-rolls
the same 4-line "copy provenance fields onto my span with the right key names" block, and the
key names are easy to typo. Removing that boilerplate is the feature. Without it there is no
feature.

**Independent Test**: Render a prompt, call the provenance-attributes helper on the result, and
assert it returns a flat map whose keys are the fixed `prompting_press.prompt.*` provenance keys and whose values
equal the result's `name`, `variant`, `template_hash`, and `render_hash`. Pass that map to a
span's set-attributes call and confirm the four attributes are present.

**Acceptance Scenarios**:

1. **Given** a successful render result, **When** the consumer calls the provenance-attributes
   helper, **Then** it returns a flat string-keyed map containing exactly four entries — the
   prompt name, resolved variant, template hash, and render hash — under stable, documented
   library-owned `prompting_press.prompt.*` keys.
2. **Given** that map, **When** the consumer passes it to their telemetry span's bulk
   set-attributes API, **Then** all four attributes are recorded with no additional mapping
   code.
3. **Given** two renders of the same prompt/variant/values, **When** the helper is called on
   each result, **Then** the returned maps are equal (the values are deterministic content
   identity).

---

### User Story 2 - No telemetry coupling or hidden emission (Priority: P1)

A consumer who does not use OpenTelemetry (or uses a different telemetry system, or none) must
be able to use the library — and this helper — without pulling in any telemetry dependency, and
must be assured the library never emits, pushes, or logs provenance on its own.

**Why this priority**: The library's boundary guarantee (no telemetry sink, no I/O, no external
coupling) is load-bearing and a headline promise. A helper that imported a telemetry SDK, or
fired a callback inside render, would violate it. Equal priority to US1: shipping US1 in a way
that breaks this guarantee is unacceptable.

**Independent Test**: Inspect the package's dependency manifest before and after the feature and
confirm no telemetry/observability dependency is added (not even optional). Confirm the helper is
a pure return-value projection with no side effects and no callback invoked during render.

**Acceptance Scenarios**:

1. **Given** the library installed with default dependencies, **When** a consumer uses the
   provenance-attributes helper, **Then** no telemetry/observability package is required or
   imported by the library.
2. **Given** any render call, **When** it completes, **Then** the library invokes no
   caller-supplied callback and performs no emission — provenance remains purely data on the
   returned result until the caller chooses to read it.
3. **Given** the helper, **When** it is called, **Then** it only reads fields already present on
   the result and returns a new map; it performs no I/O and mutates nothing.

---

### User Story 3 - Consumers who want other keys or fields keep full access (Priority: P2)

A consumer who wants attribute keys different from the built-in convention, or who wants to log
additional fields, can still do so by reading the result's public fields directly.

**Why this priority**: The helper is an opinionated convenience for the common case (fixed library-owned
keys, the four provenance fields). It must not become the *only* path — the raw
fields stay public. P2 because it is a non-regression guarantee rather than new capability.

**Independent Test**: Confirm the result's `name`, `variant`, `template_hash`, `render_hash`
remain publicly readable and that a consumer can build their own attribute map with custom keys
from them.

**Acceptance Scenarios**:

1. **Given** a render result, **When** a consumer wants custom attribute keys, **Then** they can
   read the four provenance fields directly and build their own map — the helper is additive, not
   a replacement for field access.

---

### Edge Cases

- **Guard-enabled render**: `template_hash`/`render_hash` reflect whichever body the guard mode
  produced (per the guard body-invariant). The helper reports those hashes verbatim; it does not
  add a guard indicator attribute. (Which guard mode produced a hash is recorded by the caller,
  per the constitution — not by this helper.)
- **Default variant**: when no variant was selected, the `variant` value is the reserved default
  name (`default`) — the helper always emits a variant value, never empty/absent.
- **Rendered text and guard text are never included**: the map excludes the rendered body and any
  guard text — auto-logging rendered content to telemetry is a data-exposure and span-size
  foot-gun and is deliberately out of scope.
- **Prompt/variant metadata and output-model are never included**: the opaque metadata bag,
  variant metadata, and the output-model reference are excluded — flattening an opaque bag into
  flat attributes would require the library to interpret it (violating the opaque-metadata
  doctrine) and risks unbounded span cardinality.
- **Empty/degenerate values**: provenance fields are always populated on a successful render
  (name, variant, and both hashes are non-empty by construction), so the map always has four
  populated entries.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The render result MUST expose a helper that returns the render's content-identity
  provenance as a **flat, string-keyed map of string values**, suitable for direct use as
  telemetry span attributes.
- **FR-002**: The map MUST contain exactly four entries: the prompt **name**, the resolved
  **variant**, the **template hash**, and the **render hash** — and no other entries.
- **FR-003**: The map keys MUST be the fixed **library-owned** strings `prompting_press.prompt.name`,
  `prompting_press.prompt.variant`, `prompting_press.prompt.template_hash`,
  `prompting_press.prompt.render_hash` — stable and documented. The documentation MUST NOT claim OTel
  GenAI-convention alignment (only `gen_ai.prompt.name` exists in that convention; `variant` is not a
  convention key; bare `gen_ai.prompt` is deprecated). It MAY note that a consumer can remap these
  onto their tracer's own convention.
- **FR-004**: The helper MUST be a **pure projection** of fields already present on the result:
  it MUST perform no I/O, invoke no callback, mutate nothing, and produce no side effects.
- **FR-005**: The library MUST NOT invoke any caller-supplied callback during render, and MUST
  NOT emit, push, or log provenance to any destination on its own. Provenance remains data on the
  returned result (no telemetry sink).
- **FR-006**: The library MUST NOT add any telemetry/observability dependency — not a hard
  dependency and not an optional extra — as part of this feature. The convention key strings are
  emitted as plain strings; the telemetry SDK is never linked.
- **FR-007**: The map MUST NOT include the rendered body text, any guard text, the prompt or
  variant metadata bag, or the output-model reference.
- **FR-008**: The four provenance fields MUST remain publicly readable on the result, so a
  consumer can construct a custom-keyed attribute map without the helper. The helper is additive.
- **FR-009**: The helper MUST be present in all three bindings with equivalent semantics, returning
  a flat string→string map in native idiom: Python `dict[str, str]`, TypeScript
  `Record<string, string>`, Rust **`BTreeMap<String, String>`** (deterministic key order for
  cross-binding parity, D1) (C-06). **Rust surface (orphan rule):** the consumer re-exports the
  kernel's `RenderResult`, so an inherent method on it in the consumer crate is illegal (E0116). The
  shared map-building logic MUST therefore be a **free function over the four field values**; Rust
  MAY additionally expose an **extension trait** so callers write `result.provenance_attributes()`.
  Python and TypeScript expose the method on their own `RenderResult` type, calling the shared
  field-based builder (so the four key strings + assembly live in one place — Principle I).
- **FR-010**: The helper MUST NOT require modifying the `prompting-press-core` kernel: the four
  provenance fields are already present on the render result, so this feature is confined to the
  consumer/binding layer (Principle III; kernel unchanged, no I/O).
- **FR-011**: The helper MUST NOT be configurable via a key-mapping option, alternate key sets, or
  a field-selection toggle in this feature. The convention keys are fixed; consumers wanting
  variation use the public fields directly (C-08 — no speculative configuration seam).

### Constitution amendment requirements

- **FR-012**: This feature MUST make its constitutional edit by **softening Principle V**: the
  library MAY **format** its provenance into a flat attribute map (still data on the returned
  value); it still exposes **no telemetry sink, no push/emit, and no telemetry dependency**. The
  edit MUST preserve the rest of Principle V (provenance is data on the return value; the two
  content hashes and their semantics are unchanged).
- **FR-013**: This feature MUST **cite** the v3.0.0 repositioning statement introduced by spec 017
  (minimal core PLUS earned, opt-in seams) as the shared anchor for this relaxation, rather than
  re-introducing the repositioning rationale independently.
- **FR-014**: The amendment MUST be recorded in `DECISIONS.md` with rationale and version bump per
  the Governance policy, and MUST propagate to the constitution body + version line and the
  rendered agent-context copies (`CLAUDE.md` / `AGENTS.md`). Version: **v3.1.0** — an **additive**
  amendment on top of spec-017's v3.0.0 baseline (019 is v3.2.0); 018 MUST NOT re-declare v3.0.0.
  Merge after 017.

### Key Entities *(include if feature involves data)*

- **Provenance attribute map**: a flat, string→string mapping of the render's content-identity
  provenance, keyed by the fixed library-owned `prompting_press.prompt.*` names. Four entries: name, variant,
  template hash, render hash. Derived on demand from the render result; not stored state.
- **Render result**: the existing return value of a render, already carrying the four provenance
  fields (plus rendered text and optional guard text, which are excluded from the map). Unchanged
  by this feature except for the added projection helper.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A consumer can attach the four provenance fields to a telemetry span in a **single**
  helper call plus one bulk set-attributes call, with zero hand-written key strings.
- **SC-002**: The library's dependency manifest gains **no** telemetry/observability dependency
  (hard or optional) as a result of this feature — verified by manifest diff.
- **SC-003**: The provenance-attributes helper returns identical maps for identical
  prompt/variant/values renders, in all three bindings — verified by parity tests.
- **SC-004**: The returned map contains exactly the four provenance entries and never the rendered
  body, guard text, metadata, or output-model — verified by explicit tests in each binding.
- **SC-005**: No caller callback is invoked and no emission occurs during render — verified by a
  test asserting render has no observable side effect beyond returning the result.
- **SC-006**: The `prompting-press-core` crate is unchanged by this feature (no diff), confirming
  the consumer/binding-only, no-kernel-change boundary.
- **SC-007**: The constitution reflects the softened Principle V (formatting allowed; no sink/push/
  dependency), cites the spec-017 repositioning statement, and the amendment is recorded in
  `DECISIONS.md` with rendered copies in sync.

## Assumptions

- The render result already carries the four provenance fields (`name`, `variant`,
  `template_hash`, `render_hash`) in all three bindings; this feature only projects them. (Verified
  against the current core `RenderResult`.)
- The library-owned `prompting_press.prompt.*` keys are the chosen stable key set (the OTel GenAI convention was found NOT to fit — only `gen_ai.prompt.name` exists there); the
  exact final key strings are confirmed at plan time against the current convention, but the
  four-field scope and the "hardcoded, documented, no config knob" decision are fixed here.
- Telemetry-agnostic formatting is sufficient value; the earlier proposal of a callback sink +
  built-in OTel sink is explicitly rejected (it would violate the no-telemetry-sink boundary and
  re-introduce an eliminated pluggable seam).
- Provenance semantics (content-addressed hashes, guard-mode dependence of `render_hash`) are
  owned by prior specs and unchanged; this feature neither adds nor reinterprets provenance data.
- Breaking changes are permissible at 0.x, but this feature is purely additive (a new method) and
  changes no existing behavior.
