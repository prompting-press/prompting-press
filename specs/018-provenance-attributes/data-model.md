# Phase 1 Data Model: provenance attributes helper

No prompt-definition schema change. The only new surface is a projection method + fixed key set.

## Entity: Provenance attribute map

Flat `string → string` mapping produced on demand from a `RenderResult`. Exactly four entries:

| Key (fixed) | Source field | Notes |
|---|---|---|
| `prompting_press.prompt.name` | `RenderResult.name` | OTel-GenAI-aligned |
| `prompting_press.prompt.variant` | `RenderResult.variant` | OTel-GenAI-aligned; always populated (`default` if none) |
| `prompting_press.prompt.template_hash` | `RenderResult.template_hash` | prompting-press provenance extension |
| `prompting_press.prompt.render_hash` | `RenderResult.render_hash` | prompting-press provenance extension |

- **Excluded:** `text`, `guard`, prompt/variant `metadata`, `output_model` (FR-007).
- **Per-language return:** Python `dict[str,str]`, TS `Record<string,string>`, Rust
  `BTreeMap<String,String>` (deterministic order).
- **Not configurable:** fixed keys, fixed 4 fields, no toggle (FR-011 / C-08).

## Entity: RenderResult (existing — unchanged)

Kernel `RenderResult` (`crates/prompting-press-core/src/engine.rs:117`) already carries `text`,
`name`, `variant`, `template_hash`, `render_hash`, `guard`. Surfaced 1:1 in each binding. This
feature ADDS a projection method; it changes no field and no kernel behavior.

## Note
- The map is an explicit 4-key ALLOWLIST, not a reflection of all RenderResult fields — a future
  field cannot silently leak into telemetry.

## Behavior

```
provenance_attributes(result):
    return {
      "prompting_press.prompt.name":          result.name,
      "prompting_press.prompt.variant":       result.variant,
      "prompting_press.prompt.template_hash": result.template_hash,
      "prompting_press.prompt.render_hash":   result.render_hash,
    }   # pure: no I/O, no callback, no mutation, no emission
```

### Invariants

- **INV-1 (pure projection):** reads only existing fields; no side effects (FR-004).
- **INV-2 (exactly four):** the map has exactly the four entries above, never more (FR-002/FR-007).
- **INV-3 (always populated):** on a successful render all four values are non-empty (`variant` is
  `default` when unselected); the map always has four entries.
- **INV-4 (no dependency):** no telemetry SDK linked; keys are plain strings (FR-006).
- **INV-5 (parity):** identical render → identical map across all three bindings (D1; deterministic
  order via BTreeMap in Rust).
