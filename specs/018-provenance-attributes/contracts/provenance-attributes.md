# Contract: provenance attributes helper (per binding)

Identical semantics (Principle I); native idiom (C-06). Fixed `gen_ai.prompt.*` keys; 4 entries.

## Rust (`prompting-press` consumer)

```rust
// shared key constants (single source):
pub const GEN_AI_PROMPT_NAME: &str = "gen_ai.prompt.name";
pub const GEN_AI_PROMPT_VARIANT: &str = "gen_ai.prompt.variant";
pub const GEN_AI_PROMPT_TEMPLATE_HASH: &str = "gen_ai.prompt.template_hash";
pub const GEN_AI_PROMPT_RENDER_HASH: &str = "gen_ai.prompt.render_hash";

impl RenderResult {
    /// Flat content-identity provenance for telemetry span attributes. Pure projection.
    pub fn provenance_attributes(&self) -> std::collections::BTreeMap<String, String>;
}
```

## Python (`prompting-press` wheel)

```python
class RenderResult:
    def provenance_attributes(self) -> dict[str, str]: ...
    # span.set_attributes(result.provenance_attributes())
```

## TypeScript (`prompting-press` npm)

```ts
interface RenderResult {
  provenanceAttributes(): Record<string, string>;
  // span.setAttributes(result.provenanceAttributes())
}
```

## Cross-binding contract

- Returns exactly 4 entries under the fixed `gen_ai.prompt.*` keys; values = `name`, `variant`,
  `template_hash`, `render_hash`.
- Never includes `text`, `guard`, metadata, or `output_model`.
- Pure: no I/O, no callback, no emission, no telemetry dependency.
- Identical map for identical renders across bindings (D1; Rust deterministic via `BTreeMap`).
- The 4 provenance fields remain publicly readable for custom-key use (helper is additive).
