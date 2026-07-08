# Contract: provenance attributes helper (per binding)

Identical semantics (Principle I); native idiom (C-06). Fixed `prompting_press.prompt.*` keys; 4 entries.

## Rust (`prompting-press` consumer)

```rust
// shared key constants (single source):
pub const PROMPT_ATTR_NAME: &str = "prompting_press.prompt.name";
pub const PROMPT_ATTR_VARIANT: &str = "prompting_press.prompt.variant";
pub const PROMPT_ATTR_TEMPLATE_HASH: &str = "prompting_press.prompt.template_hash";
pub const PROMPT_ATTR_RENDER_HASH: &str = "prompting_press.prompt.render_hash";

// Shared map-builder — a FREE FUNCTION over the four field values (NOT an inherent method:
// the consumer re-exports the kernel's RenderResult, so `impl RenderResult` here is E0116).
// This is the single source the Python/TS bindings also call.
pub fn provenance_attributes_of(
    name: &str, variant: &str, template_hash: &str, render_hash: &str,
) -> std::collections::BTreeMap<String, String>;

// Optional Rust ergonomics: an extension trait so callers can write result.provenance_attributes().
pub trait ProvenanceExt {
    fn provenance_attributes(&self) -> std::collections::BTreeMap<String, String>;
}
impl ProvenanceExt for RenderResult {
    fn provenance_attributes(&self) -> std::collections::BTreeMap<String, String> {
        provenance_attributes_of(&self.name, &self.variant, &self.template_hash, &self.render_hash)
    }
}
// NOTE: `use prompting_press::ProvenanceExt;` is required to call the method (trait-in-scope).
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

- Returns exactly 4 entries under the fixed `prompting_press.prompt.*` keys; values = `name`, `variant`,
  `template_hash`, `render_hash`.
- Never includes `text`, `guard`, metadata, or `output_model`.
- Pure: no I/O, no callback, no emission, no telemetry dependency.
- Identical map for identical renders across bindings (D1; Rust deterministic via `BTreeMap`).
- The 4 provenance fields remain publicly readable for custom-key use (helper is additive).
