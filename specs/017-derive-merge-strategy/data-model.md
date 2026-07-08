# Phase 1 Data Model: derive() merge strategy

No prompt-definition schema change (Principle VII). The only new "data" is an API-surface value
type. Entities below describe the surface types and the merge algorithm's data behavior.

## Entity: MergeStrategy

A small closed enumeration selecting how `derive` combines overlay fields with the base.

| Member | Meaning |
|---|---|
| `Replace` (default) | Each overlay-present top-level field replaces that whole field (today's behavior). |
| `Merge` | Map-typed fields union at top-level keys (child-wins, whole-entry); scalar fields replace. |

- **Excluded (reserved axis, C-08):** `Deep`, `None` — not implemented; addable later as new
  members without a new method or breaking signature.
- **Per-language representation** (clarify): Rust `enum MergeStrategy { Replace, Merge }` with
  `impl Default` → `Replace`; Python importable `MergeStrategy` enum (`REPLACE`/`MERGE`);
  TypeScript exported const/enum (`Replace`/`Merge`).
- **Default:** `Replace` in all three (non-breaking).
- **Invalid value:** unrepresentable in Rust (type); structured error in Python/TS (FR-011).

## Entity: DeriveOptions (Rust only)

`#[derive(Default)]` options struct carrying the derive configuration tail.

| Field | Type | Default |
|---|---|---|
| `merge` | `MergeStrategy` | `MergeStrategy::Replace` |

- Used with struct-update `..Default::default()`; forward-extensible (future options append here).
- Python/TS do not need this struct — Python uses keyword-only args; TS uses an inline options
  object literal `{ validators?, strategy? }`.

## Entity: PromptOverlay (existing — unchanged shape)

The existing partial set of top-level prompt-definition fields passed to `derive`
(`name`, `role`, `body`, `variables`, `variants`, `output_model`, `metadata`; all optional).
Unchanged by this feature; the `strategy` selector governs how its **map** fields combine.

- **Map-typed fields** (subject to union under `Merge`): `variables`, `variants`, `metadata`.
- **Scalar fields** (always replace when present): `name`, `role`, `body`, `output_model`.

## Merge algorithm (data behavior)

Given `base` definition and `overlay`, producing `merged`, then `Prompt::new(merged)`:

```
for each top-level field F:
    if overlay does not supply F:
        merged.F = base.F                      # untouched (both strategies)
    else if F is scalar (name/role/body/output_model):
        merged.F = overlay.F                    # replace (both strategies)
    else (F is a map: variables/variants/metadata):
        if strategy == Replace:
            merged.F = overlay.F                # wholesale replace (today)
        else (strategy == Merge):
            merged.F = { ...base.F, ...overlay.F }   # top-level key union, child-wins,
                                                     # whole-entry (NO recursion)
merged  ->  Prompt::new(merged)                 # re-validate whole: agreement, parse,
                                                # reserved-name; coverage vs merged var set
```

### Invariants

- **INV-1 (immutability):** `base` is never mutated; `derive` returns a new `Prompt` (or error).
- **INV-2 (default parity):** with `Replace`, `merged` is byte-identical to today's `derive`
  output for the same overlay (SC-002).
- **INV-3 (soundness):** `merged` always passes through the full validating constructor; a merge
  that yields an agreement violation or (Py/TS) an uncovered `validation_required` variable fails
  at construction (SC-004).
- **INV-4 (no recursion):** on a map-key collision under `Merge`, the overlay's entry replaces
  the base's entry wholesale; entry internals are never merged (that would be `Deep`, excluded).
- **INV-5 (empty map):** an overlay map supplied as empty contributes no keys under `Merge`
  (base map unchanged); it is not a way to clear a map (that is `Replace` with an empty map).
