# Feature Specification: Prompt `derive()` merge strategy

**Feature Branch**: `017-derive-merge-strategy`

**Created**: 2026-07-08

**Status**: Draft

**Input**: Issue #271 (feat: prompt inheritance). Resolved via design grilling to a
code-side merge strategy on the existing `derive` primitive — NOT a YAML `extends:` field,
NOT Jinja template inheritance, NOT a loader/registry.

## Clarifications

### Session 2026-07-08

- Q: How should the merge-strategy value be represented in the Python and TypeScript bindings? → A: A first-class, importable `MergeStrategy` enum/const in each binding (Python `MergeStrategy.MERGE`, TypeScript `MergeStrategy.Merge`), not a bare string literal — discoverable via autocomplete, one shared concept across all bindings, still validated (unknown → structured error).
- Q: What public surface should the merge strategy take on the Rust consumer `Prompt::derive`? → A: A `Default`-implementing options struct (`DeriveOptions { strategy: MergeStrategy }`, `#[derive(Default)]`, used with `..Default::default()`), rather than a bare enum param or a separate `derive_merged` method. This keeps existing `derive(overlay)` call sites non-breaking (default = `Replace`), is forward-extensible for a future value or additional options without a signature change, and is the idiomatic Rust defaultable-config shape. The shared `MergeStrategy` enum is the value inside the struct.

### Session 2026-07-08 (addendum — post-adversarial-review)

- Q: The strategy value `Shallow` collided with spec-008's "shallow-replace" term (where "shallow" describes the REPLACE behavior). Rename? → A: Rename `Shallow` → **`Merge`**; enum is `MergeStrategy { Replace, Merge }`. `merge`/`replace` is the industry-standard pair (RFC 7386 JSON Merge Patch, Kubernetes, Terraform `merge()`, Ansible `hash_behaviour`). The carrier parameter is named **`strategy`** (not `merge`) to avoid a field/value stutter.
- Q: The plan claimed the Node binding delegates `derive` to the Rust consumer — is that true? → A: No (verified). Only Python delegates; Node has a private `shallow_merge_json` in JSON space. Resolution: implement the union **once** in the consumer as a shared `serde_json::Value`-space helper that both the typed Rust path and Node call (FR-018) — genuine single-source parity (Principle I / decision D1).
- Q: Is `Merge` re-validation sufficient for soundness when a union replaces a variable's declaration a base variant body uses? → A: The agreement check is name-only. Name **removal** under `Merge` is caught (construction fails); a **type/trust swap** is accepted (validator's job, not the kernel's). Documented as expected behavior (FR-019).

## Iterations

### Iteration 2026-07-08: fold adversarial-review findings

**Change**: Corrected the false "Node delegates derive" premise (shared single-source `merge_definitions` helper, FR-018); renamed strategy `Shallow`→`Merge` with a `strategy` param (industry-standard merge/replace); upgraded the amendment to record the spec-008 FR-017(b) redefinition (FR-016.2), not just a Principle VI clarification; added the name-only soundness decision + tests (FR-019); added a metadata.guard inheritance doc note (SEC-002).
**Scope**: Feature-wide (pre-implementation refinement; not a pivot).
**Artifacts updated**: spec.md, plan.md, tasks.md, data-model.md, contracts/derive-merge.md, research.md.
**Tasks added**: T010a, T010b, T010c (soundness / map-coverage / error-scrubbing tests); shared-helper + Node-refactor folded into T005/T013.
**Tasks removed**: none.
**Tasks marked complete**: none (0 of 26 built).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Derive a child prompt that inherits a base's variables and adds its own (Priority: P1)

A consumer holds a base prompt object that declares one or more shared input variables
(e.g. a shared `extraction` variable used by a family of analyst prompts). They want to
produce several sibling prompts that each **keep** the base's declared variables **and add
their own** — without hand-copying the base's variable declarations into every child and
without losing the base's variables when they supply the child's.

**Why this priority**: This is the entire motivating use case (Bellwether `trend_value`:
one base `extraction` variable, children `bull`/`bear`/`valuation` each adding e.g.
`sentiment`). It is the capability the feature exists to deliver. Without it there is no
feature.

**Independent Test**: Construct a base prompt with `variables: {extraction}`. Call
`derive` with an overlay adding `variables: {sentiment}` under the `Merge` strategy.
Assert the derived prompt's declared variables are exactly `{extraction, sentiment}`, the
base prompt is unchanged, and rendering the child with both variables succeeds.

**Acceptance Scenarios**:

1. **Given** a base prompt declaring `variables: {extraction}`, **When** the consumer
   derives with overlay `variables: {sentiment}` and strategy `Merge`, **Then** the
   derived prompt declares `{extraction, sentiment}` and the base prompt is untouched.
2. **Given** the same base, **When** the consumer derives with overlay `variables:
   {extraction: <new decl>}` and strategy `Merge`, **Then** the derived prompt's
   `extraction` is the overlay's declaration (child wins on key collision), whole-entry
   (no field-level merge inside the `extraction` entry).
3. **Given** a base whose body references `{{ extraction }}`, **When** the consumer
   derives with overlay `body: "... {{ extraction }} ... {{ sentiment }}"` and
   `variables: {sentiment}` under `Merge`, **Then** construction succeeds because the
   agreement check runs over the merged variable set `{extraction, sentiment}`.

---

### User Story 2 - Preserve existing replace semantics by default (Priority: P1)

A consumer already using `derive` today (overlay fields replace wholesale) must see **no
behavior change** unless they explicitly opt into the new strategy.

**Why this priority**: `derive` is a shipped, load-bearing primitive (the sole prompt
mutator). A silent semantic change to the default would break every existing caller. The
default must remain the current replace behavior. Equal priority to US1 because shipping
US1 without this guarantee is unacceptable.

**Independent Test**: Call `derive` with an overlay replacing `variables` and **no**
`strategy` argument. Assert the result is identical to today's behavior (overlay's
`variables` map wholly replaces the base's).

**Acceptance Scenarios**:

1. **Given** a base declaring `variables: {extraction}`, **When** the consumer derives
   with overlay `variables: {sentiment}` and **no** strategy argument (or `strategy=Replace`),
   **Then** the derived prompt declares exactly `{sentiment}` (base's `extraction` is
   dropped) — identical to current behavior.
2. **Given** any existing call site that does not pass `strategy`, **When** the code is run
   against the new version, **Then** its observable result is unchanged.

---

### User Story 3 - Add-a-variable ergonomics without manual spread (Priority: P2)

A consumer wants the "inherit + add" operation to be a single expressive call, not a
manual spread of the base's current variables into the overlay.

**Why this priority**: The capability is already achievable today via manual spread
(`derive({variables: {...base.variables, sentiment}})`). The new strategy is an
ergonomics + clarity improvement, not a new raw capability — hence P2. It removes a
foot-gun (forgetting to spread silently drops inherited variables).

**Independent Test**: Compare the `Merge`-strategy call against the manual-spread
`Replace` call; assert they produce equal derived definitions for the add-a-key case.

**Acceptance Scenarios**:

1. **Given** a base with `variables: {extraction}`, **When** the consumer derives with
   `variables: {sentiment}` + `Merge`, **Then** the result equals deriving with
   `variables: {extraction, sentiment}` + `Replace` (the strategy did the spread).

---

### Edge Cases

- **Empty overlay map under `Merge`**: overlay supplies `variables: {}` → union with base
  yields the base's variables unchanged (empty child contributes nothing). Not a way to
  clear variables. (Clearing a map wholesale is `Replace` with an empty map.)
- **Overlay omits a map field entirely under `Merge`**: an absent field is untouched
  (same as `Replace` — absence never merges nor clears). `Merge` only changes behavior
  for map fields the overlay **does** supply.
- **Key collision under `Merge`**: the child's whole entry wins; there is no field-level
  merge inside the entry (that would be the excluded `deep` strategy).
- **Scalar field under `Merge`**: `name`/`role`/`body`/`output_model` have no map to
  union; an overlay-present scalar replaces (identical to `Replace`). Strategy only affects
  the three map fields.
- **`Merge` merge introduces an agreement violation**: e.g. child body references a
  variable neither base nor overlay declares → construction fails with the same structured
  agreement error as today (validation runs over the merged whole).
- **`Merge` merge adds a `validation_required` variable the effective validator does not
  cover** (Python/TS): construction raises/throws the coverage error against the merged
  variable set. In Rust the analogous mismatch is a **compile-time** error at the
  `render::<V>` call site (no runtime coverage throw — Principle VI asymmetry).
- **Unknown strategy value**: a strategy value outside the supported set is rejected with a
  structured error (Python/TS) or is unrepresentable (Rust typed enum).
- **`metadata`/`variants` union under `Merge`**: opaque `metadata` and `variants` maps
  union at top-level keys, child-wins, identically to `variables` — the library does not
  interpret their contents (opaque-metadata doctrine preserved).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `derive` operation MUST accept an optional merge-strategy selector with
  two supported values, expressed as a first-class **`MergeStrategy`** type in every binding
  (members `Replace` = current behavior, and `Merge` = new). The selector MUST be carried by a
  parameter named **`strategy`** (not `merge`, to avoid a field/value stutter) and exposed
  per-language in native idiom, never as a positional mode boolean (C-11):
  - **Python** — an importable `MergeStrategy` enum passed via a **keyword-only** argument
    (`derive(overlay, *, strategy=MergeStrategy.MERGE)`); the default is `MergeStrategy.REPLACE`.
  - **TypeScript** — an exported `MergeStrategy` const/enum supplied inside the derive
    **options object** (`derive(overlay, { strategy: MergeStrategy.Merge })`); default
    `Replace`. (Moving the optional tail into an options object is a breaking TS signature
    change at 0.x — `validators` moves inside the object too; see research R4.)
  - **Rust** — a `MergeStrategy` enum carried inside a **`Default`-implementing options
    struct** (`DeriveOptions { strategy: MergeStrategy }`, used with `..Default::default()`),
    whose default is `MergeStrategy::Replace`, consumed by a `derive_with(overlay, options)`
    entry. The existing `derive(overlay)` keeps its exact signature and `Replace` semantics
    (non-breaking); `derive_with` is additive and forward-extensible without a signature change
    (idiomatic Rust defaultable config; satisfies the C-11 Rust threshold).
  `MergeStrategy` uses the industry-standard **merge/replace** pair (RFC 7386 JSON Merge Patch,
  Kubernetes `merge`/`replace`, Terraform `merge()`, Ansible `hash_behaviour`). A value outside
  the supported set is unrepresentable by the type in Rust and rejected via the structured-error
  path in Python/TS (see FR-011).
- **FR-002**: The default strategy, when the caller does not specify one, MUST be `Replace`
  — byte-for-byte the current `derive` behavior. No existing call site changes observable
  behavior.
- **FR-003**: Under `Replace`, each overlay-present top-level field MUST replace that field
  wholesale (unchanged from today); absent overlay fields MUST be left untouched.
- **FR-004**: Under `Merge`, the three **map-typed** fields (`variables`, `variants`,
  `metadata`) MUST union at their top-level keys with **child-wins** on key collision, taking
  the overlay's **whole entry** for a colliding key (no recursion into the entry's contents).
- **FR-005**: Under `Merge`, the **scalar** fields (`name`, `role`, `body`,
  `output_model`) MUST replace when overlay-present (there is nothing to union); an absent
  scalar overlay field MUST leave the base's value untouched.
- **FR-006**: Under `Merge`, an overlay field the caller does **not** supply MUST leave the
  base's corresponding field untouched (absence never unions nor clears). An overlay map
  supplied as empty MUST union to the base unchanged (contributes no keys).
- **FR-007**: For both strategies, the operation MUST re-validate the **entire merged
  definition** through the existing validating construction path (reserved-variant-name
  rejection, per-arm template parse / excluded-feature rejection, agreement check that
  referenced roots — across the root body AND every variant arm — are a subset of the merged
  declared variables). Merge MUST NOT bypass or weaken any construction invariant.
- **FR-008**: The operation MUST remain **immutable** — it returns a new prompt object and
  leaves the base prompt object entirely unchanged, for both strategies.
- **FR-009**: `validation_required` coverage MUST be evaluated against the **merged** variable
  set. In the dynamic bindings (Python/TypeScript) an uncovered `validation_required` variable
  in the merged set MUST raise/throw at construction. In Rust, coverage MUST remain a
  compile-time guarantee via the generic validator type; Rust MUST NOT introduce a runtime
  coverage throw (Principle VI asymmetry, preserved).
- **FR-010**: Validator carry-forward under both strategies MUST follow the existing rule:
  an overlay that supplies validators uses them; otherwise the base prompt's validators are
  carried forward to the derived prompt (Python/TS). (Rust carries no runtime validator.)
- **FR-011**: A merge-strategy value outside the supported set MUST be rejected via each
  binding's normal structured-error path (Python/TS), or be unrepresentable by the type
  system (Rust).
- **FR-012**: The capability MUST be present in all three bindings (Rust consumer, Python,
  TypeScript) with equivalent semantics, expressed in each language's native idiom (C-06).
- **FR-013**: The change MUST NOT modify the prompt-definition JSON Schema, MUST NOT modify
  the `prompting-press-core` kernel, and MUST NOT introduce any I/O. It is confined to the
  consumer/binding `derive` surface (Principle III; the kernel stays validation-blind and
  I/O-free).
- **FR-014**: The `deep` and `none` merge strategies MUST NOT be implemented in this feature.
  The strategy selector MUST be designed so a future value can be added without a new method
  or a breaking signature change (reserve the axis; C-08 — earned by a future consumer).
- **FR-018** (single-source merge): The map-union algorithm MUST be implemented **once** in the
  Rust consumer crate as a shared helper operating in `serde_json::Value` space (e.g.
  `merge_definitions(base, overlay, strategy)`). BOTH the typed Rust `derive`/`derive_with`
  path AND the Node binding's construction path MUST call that single helper. The Node binding
  MUST NOT retain a second, independent union implementation (it currently merges in JSON space
  via a private `shallow_merge_json` and MUST be refactored to call the shared helper). This
  guarantees byte-identical results across bindings by construction (Principle I) and honors
  decision D1 (per-binding date/decimal serialization would otherwise let a JSON-space union and
  a typed-map union diverge). Python already delegates to the consumer `derive`.
- **FR-019** (soundness boundary — explicit): The agreement check is **name-only** (referenced
  root names ⊆ declared variable names; Principle IV, `nested=false`). Consequently:
  - A `Merge` that **removes** a variable a base variant body still references (the name is in
    neither base nor overlay after the union) MUST fail construction via the agreement check.
  - A `Merge` that **replaces** an existing variable's declaration (changing its `type` or
    `trusted` flag) that a base arm references is **accepted** — type/trust correctness is the
    validator's responsibility, not the kernel's, consistent with the existing name-only
    boundary. This is documented, expected behavior, not a defect.

### Constitution amendment requirements

- **FR-015**: This feature MUST record the one-time **v3.0.0 repositioning statement** in the
  constitution: Prompting Press relaxes its "minimal core that never grows into a framework"
  thesis to "a minimal, validation-blind core PLUS earned, opt-in seams," motivated by a real
  second consumer (Bellwether). This statement is the shared anchor that specs 018 (#270) and
  019 (#268) cite.
- **FR-016**: This feature MUST record its concrete constitutional edits, which are TWO, not
  one:
  1. A **Principle VI clarification** that `derive` gains a merge-strategy axis while the
     compile-time-vs-runtime coverage asymmetry is preserved.
  2. A **redefinition of spec-008 FR-017(b)** — the shipped FR-017 mandates that the only way to
     vary a prompt is a copy-with-overlay that "**shallow-replaces** each supplied top-level
     field (**no deep merge**)". Introducing `MergeStrategy::Merge` (top-level key union) is a
     **backward-incompatible redefinition** of that FR (analogous to how spec-015 redefined
     spec-002's guard body-invariant). `DECISIONS.md` MUST record this FR-017(b) redefinition
     explicitly — the "no deep merge / wholesale-replace-only" wording is superseded by
     "overlay MAY union under `Merge`" — not merely a Principle VI clause.
  Both edits, the amendment rationale, and the version bump MUST be recorded in `DECISIONS.md`
  per the Governance amendment policy.
- **FR-017**: The amendment MUST propagate to dependent artifacts as the Governance policy
  requires (constitution body + version line → **v3.0.0**; `DECISIONS.md`; the roadmap's
  spec-008/spec-017 entries; the APM-rendered `CLAUDE.md` / `AGENTS.md` copies regenerated). No
  structural change to plan/spec/tasks templates is expected.

### Key Entities *(include if feature involves data)*

- **Merge strategy**: a small, closed enumeration governing how overlay fields combine with
  base fields during `derive`. Two members in this feature: `Replace` (wholesale field
  replacement; the default and current behavior) and `Merge` (top-level key union with
  child-wins for the three map fields; wholesale replace for scalars). Extensible by design.
- **Prompt overlay**: the existing partial set of top-level prompt-definition fields supplied
  to `derive`. Unchanged by this feature except that the merge strategy now governs how its
  map fields combine with the base.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A consumer can produce a child prompt that declares the union of a base's
  variables and the child's added variables in a **single** `derive` call, with no manual
  copying of the base's variable declarations.
- **SC-002**: 100% of existing `derive` call sites (those not passing a strategy) produce
  identical results before and after the change — verified by the existing `derive` test
  suites passing unchanged plus explicit default-is-replace tests.
- **SC-003**: Deriving with `Merge` and an overlay that adds one variable yields a derived
  prompt whose declared variable set equals the base's set plus the added key, in all three
  bindings, verified by parity tests.
- **SC-004**: A `Merge` derive that produces an unsound merged prompt (agreement violation,
  or an uncovered `validation_required` variable in Python/TS) fails at construction with a
  structured error — no silent acceptance and no deferred/empty render.
- **SC-005**: The base prompt object is observably unchanged after any `derive` call under
  either strategy (immutability), verified per binding.
- **SC-006**: The `prompting-press-core` crate and the prompt-definition JSON Schema are
  unchanged by this feature (no diff), confirming the consumer/binding-only, no-I/O,
  no-schema boundary.
- **SC-007**: The constitution carries the v3.0.0 repositioning statement and the Principle VI
  clarification, with the amendment recorded in `DECISIONS.md` and the rendered agent-context
  copies in sync.

## Assumptions

- The existing immutable `derive` primitive (shipped in spec 008, re-validating the merged
  whole through the single validating constructor) is the surface being extended; this feature
  does not redesign construction or validation, only how overlay fields combine.
- Code-side derivation is sufficient for the motivating consumer (Bellwether). Declarative
  YAML/JSON inheritance (an `extends:` field) is explicitly out of scope and not desired.
- Breaking changes are permissible at the current 0.x version, but the default strategy is
  kept as `Replace` to avoid gratuitously breaking existing callers.
- The three map fields (`variables`, `variants`, `metadata`) are the complete set of
  map-typed top-level fields on a prompt definition; scalar fields are `name`, `role`, `body`,
  `output_model`. (Verified against the current JSON Schema.)
- `variants` and `metadata` are library-opaque; union-by-top-level-key does not require the
  library to interpret their contents, so it does not violate the opaque-metadata doctrine.
- The per-language surface was settled at clarify (see Clarifications, Session 2026-07-08):
  a shared `MergeStrategy` enum/const in every binding; Python keyword-only `strategy=`; TS via
  the existing options object; Rust via a `Default`-implementing `DeriveOptions` struct
  (non-breaking, forward-extensible). Remaining fine-grained naming (exact struct/field
  identifiers, TS enum-vs-const-object) is confirmed at plan time within C-06/C-11.
