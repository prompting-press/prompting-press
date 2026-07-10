# Phase 0 Research: derive() merge strategy

All unknowns resolved (most during the grill + clarify sessions). No open NEEDS CLARIFICATION.

## R1 — Where the merge logic lives (CORRECTED after adversarial review)

- **Decision:** Implement the union algorithm **once** in the consumer as a shared
  `merge_definitions(base, overlay, strategy)` helper operating in `serde_json::Value` space.
  Both the typed Rust `derive`/`derive_with` and the Node binding's construction path call it.
- **Correction:** the earlier draft claimed BOTH Py and Node delegate `derive` to the consumer.
  **Verified false for Node.** Python DOES delegate (`prompting-press-py/src/prompt.rs:454` calls
  `self.inner.derive`). **Node does NOT** — `prompting-press-node/src/prompt.rs:245-262` has a
  private `shallow_merge_json` (lines 335-349) → `Prompt::from_json`, deliberately avoiding a
  `Deserialize` dep on `PromptOverlay` (doc lines 233-236). So "delegated by Py/TS" was wrong.
- **Rationale for the shared helper:** Principle I/C-01 requires ONE union algorithm. A JSON-space
  union (Node) and a typed-map union (Rust `derive`) could **diverge** on exactly what decision D1
  documents — per-binding date/decimal serialization (`Z`/`1E-17` vs `.000Z`). A single helper both
  paths call is byte-identical by construction, not by two implementations kept in sync.
- **Alternatives rejected:** (a) two implementations + parity tests — the drift Principle I exists
  to prevent, and D1 shows drift is real here; (b) add `Deserialize` to `PromptOverlay` so Node
  builds a typed overlay — reverses Node's deliberate design and perturbs the existing replace
  byte-path (SC-002 risk).

## R2 — Rust public surface

- **Decision:** `MergeStrategy` enum + a `#[derive(Default)]` options struct
  `DeriveOptions { strategy: MergeStrategy }`, consumed by a derive entry that defaults via
  `..Default::default()`. `MergeStrategy::default() == Replace`.
- **Rationale:** Clarify decision. Rust has no default/keyword args; a `Default`-implementing
  options struct is the idiomatic defaultable-config shape and keeps existing `derive(overlay)`
  call sites non-breaking + forward-extensible (a future `deep` value or extra option needs no
  signature change). Satisfies the C-11 Rust threshold (no bare optional forced; struct is the
  named-config idiom).
- **Signature approach (plan-time, within clarify envelope):** keep today's
  `derive(&self, overlay: PromptOverlay) -> Result<Prompt>` behavior as the default path, and add
  the options-carrying entry. Exact method naming (e.g. keep `derive` taking `DeriveOptions`, or a
  paired entry) is a mechanical detail settled in tasks; the constraint is: existing
  `derive(overlay)` semantics remain reachable and unchanged (SC-002).
- **Alternatives rejected:** bare required enum param (breaks all call sites — Rust can't default a
  positional arg); separate `derive_merged` method (diverges from single-method Py/TS shape,
  method proliferation on future `deep`).

## R3 — Python surface

- **Decision:** Export a `MergeStrategy` enum; add a **keyword-only** `strategy` argument:
  `derive(overlay, *, validators=None, strategy=MergeStrategy.REPLACE)`.
- **Rationale:** The existing PyO3 signature is already `#[pyo3(signature = (shape, *, validators = None))]`
  style (keyword-only tail; `crates/prompting-press-py/src/prompt.rs:122`). Adding `strategy` as
  another keyword-only arg is idiomatic, C-11-compliant, and non-breaking (default Replace). An
  importable enum (clarify choice) is discoverable and Pydantic/marshaling-validated.
- **Alternatives rejected:** positional param (C-11 violation); bare string literal (clarify chose
  a first-class enum for discoverability + one shared concept).

## R4 — TypeScript surface (the one real shape change)

- **Decision:** Move `derive` to an **options object** for its optional tail and add `merge`:
  `derive(overlay: Partial<PromptDefinition>, options?: { validators?: ValidatorMap; strategy?: MergeStrategy })`.
  Export a `MergeStrategy` const/enum. Default `Replace`.
- **Rationale:** Today `derive(overlay, validators?)` (`packages/typescript/src/index.ts:692`) has
  ONE positional optional. Adding `strategy` makes TWO optionals — C-11 (strict for TS) requires the
  optional tail to become a named options object. This is a **breaking TS signature change**
  (`validators` moves into the options object), acceptable at 0.x (spec Assumptions) and the
  correct C-11 shape. Aligns with the spec-005 precedent (`render`/`getSource`/`Composition` moved
  to options objects, DECISIONS 2026-06-28).
- **Alternatives rejected:** second positional optional `derive(overlay, validators?, merge?)` —
  C-11 violation (order-fragile positional optional tail); string-literal merge — clarify chose an
  enum/const.
- **Open (tasks-level, non-blocking):** TS `enum` vs `const` object with a union type for
  `MergeStrategy`. Lean `const` object + `as const` union (tree-shakeable, no runtime enum
  overhead), but either satisfies the contract.

## R5 — Merge semantics (fixed at grill/clarify; recorded for implementers)

- **Decision:** `Merge` = for each of `variables`/`variants`/`metadata`, `{...base, ...overlay}`
  (top-level key union; overlay's whole entry wins on collision; NO recursion into an entry).
  Scalars (`name`/`role`/`body`/`output_model`) replace when overlay-present. Absent overlay
  field → untouched (both strategies). Empty overlay map under `Merge` → base unchanged.
- **Rationale:** Whole-entry union is `deep`-free (recursion is the excluded `deep`); matches JS
  spread / Python `{**a, **b}` / Rust map `extend`. `deep`/`none` excluded (delete-expressibility +
  opaque-metadata interpretation; C-08 reserve the axis).
- **Re-validation:** both strategies route the merged definition through `Prompt::new(merged)`
  (`crates/prompting-press/src/prompt.rs:334`) — agreement, parse, reserved-name — unchanged.

## R6 — Validator coverage under merge

- **Decision:** Coverage of `validation_required` variables is evaluated against the **merged**
  variable set with the effective validator (existing R6 carry-forward: overlay validators else
  base's). Python/TS raise/throw at construction if uncovered; Rust guarantees at compile time via
  the generic `V` — NO Rust runtime coverage throw.
- **Rationale:** Principle VI asymmetry, preserved (DECISIONS 2026-06-28). Merge changes only WHICH
  variable set coverage is checked against (the merged one), not the enforcement mechanism.

## R7 — Cross-binding parity verification

- **Decision:** Add a `conformance/` case exercising `Merge` merge; assert equal merged
  definition (canonical serialized form) + equal render/hashes across the three bindings.
- **Rationale:** D1 (canonical serialized form marshaling) — compare serialized form, not native
  objects. Parity is structural (Principle I) but the corpus guards the FFI marshaling of the new
  `MergeStrategy` value + the merged result.

## R8 — Shared single-source merge helper (added post-adversarial-review)

- **Decision:** One `merge_definitions(base: Value, overlay: Value, strategy: MergeStrategy) -> Value`
  helper in the consumer crate, in `serde_json::Value` space. `derive`/`derive_with` serialize the
  typed definition → call the helper → `Prompt::new`/`from_json`; Node calls the same helper on its
  already-JSON values → `from_json`. One algorithm, both callers.
- **Rationale:** `serde_json::Value` is the common denominator both paths already reach (Node is
  JSON-native; the typed path round-trips through JSON for construction anyway). Guarantees
  byte-identical union across bindings (Principle I), honoring D1. Construction-time only (no
  hot-path cost).
- **Alternatives rejected:** typed-`HashMap` union in `derive` + separate JSON union in Node (two
  algorithms; D1 divergence risk).
