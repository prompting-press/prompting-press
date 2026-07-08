# Contract: `derive` merge-strategy surface (per binding)

The public call shape for the merge strategy in each binding. Semantics are identical
(structural parity, Principle I); idiom differs (C-06). Default `Replace` = today's behavior.

## Rust (`prompting-press` consumer)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeStrategy {
    #[default]
    Replace,
    Merge,
}

#[derive(Debug, Clone, Default)]
pub struct DeriveOptions {
    pub strategy: MergeStrategy,
}

impl<V: /* existing validate bound */> Prompt<V> {
    // Existing behavior preserved: derive(overlay) continues to mean Replace.
    // Options-carrying entry adds the strategy without breaking existing call sites.
    pub fn derive(&self, overlay: PromptOverlay) -> Result<Self, ConsumerError>;              // == Replace
    pub fn derive_with(&self, overlay: PromptOverlay, options: DeriveOptions)
        -> Result<Self, ConsumerError>;                                                        // strategy-aware
    // (Exact method naming settled in tasks; constraint: derive(overlay) semantics unchanged.)
}
```

- Unknown strategy: unrepresentable (type). No runtime `validation_required` coverage throw
  (compile-time via `V`).
- **Single-source merge (FR-018):** the union algorithm lives in ONE consumer helper
  (`merge_definitions(base, overlay, strategy)` operating in `serde_json::Value` space). Both
  `derive`/`derive_with` and the Node binding call it. There is no second union implementation.

## Python (`prompting-press` wheel)

```python
class MergeStrategy(enum.Enum):
    REPLACE = "replace"
    MERGE = "merge"

class Prompt:
    def derive(
        self,
        overlay: dict,
        *,
        validators: ValidatorMap | None = None,
        strategy: MergeStrategy = MergeStrategy.REPLACE,
    ) -> "Prompt": ...
```

- `strategy` is **keyword-only** (C-11). Unknown value → structured `PromptValidationError`-family
  error. Uncovered `validation_required` in the merged set → raises at construction.

## TypeScript (`prompting-press` npm)

```ts
export const MergeStrategy = { Replace: "replace", Merge: "merge" } as const;
export type MergeStrategy = (typeof MergeStrategy)[keyof typeof MergeStrategy];

class Prompt {
  // BREAKING (0.x): optional tail moves into an options object (C-11).
  derive(
    overlay: Partial<PromptDefinition>,
    options?: { validators?: ValidatorMap; strategy?: MergeStrategy },
  ): Prompt;
}
```

- `strategy` rides the options object (C-11). Unknown value → structured thrown error. Uncovered
  `validation_required` in the merged set → throws at construction.

## Cross-binding contract (all three)

- **Default** (`Replace` / omitted): output byte-identical to today's `derive` for the same
  overlay.
- **`Merge`**: `variables`/`variants`/`metadata` union top-level keys (child-wins, whole-entry);
  scalars replace; merged whole re-validated.
- **Immutable**: base prompt unchanged; a new prompt (or structured error) is returned.
- **Parity**: identical overlay + strategy → identical merged definition (canonical serialized
  form, D1) + identical render/`template_hash`/`render_hash` across bindings — guaranteed by the
  single shared `merge_definitions` helper (FR-018), not by two implementations kept in sync.
- **Node binding**: its construction path calls the shared consumer helper with the selected
  `MergeStrategy` (replacing its former private `shallow_merge_json`), keeping the no-`Deserialize`
  property (the helper takes JSON values).
