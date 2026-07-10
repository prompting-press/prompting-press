# Grill notes — issues #268 / #270 / #271 (2026-07-08)

Session decision: **constitution will be amended (repositioning B)** — Prompting Press
deliberately moves from "never a framework" toward batteries-included, opt-in seams.
Driver: repositioning + real consumer (Bellwether/claudebroker). Three SEPARATE specs.

## BACKLOG (user-raised 2026-07-08) — consolidated docs + examples pass
After ALL THREE specs (017 derive-merge, 018 provenance-attributes, 019 loader) are
IMPLEMENTED, do a single cross-cutting pass:
- **Docs site**: update guides / API-ref / getting-started to cover the merge strategy, the
  provenance-attributes helper, and the pluggable loader (+ the v3.0.0 repositioning framing).
  Docs-are-product rule: current behavior only; lands with/after the impl.
- **Examples / samples**: update to use the loader (FileSystem/Memory), `derive(merge=shallow)`
  for Bellwether-style shared-base prompts, and `provenance_attributes()` in the tracing sample.
NOT a per-spec Phase-7 task (each spec already has its own local docs touch) — this is the
after-the-trio integration pass. Formalize as a roadmap Deferred entry or a spec 020 once the
three land.

## #268 — Pluggable PromptLoader — RESOLVED SHAPE
- **Option A** (standalone loader), NOT the container. Container (`PromptLibrary`,
  name-keyed get + check_all) DEFERRED to its own future spec (it's the deferred
  "query-capable registry"; revives the spec-008 registry drop → ratify separately).
- Interface: `load(key) -> str` returns **raw text** (format-agnostic leaf; no parsing).
- Sync in Python/Rust; **async `Promise<string>` in TS** (C-06 native idiom).
- Ship **FileSystemLoader + MemoryLoader only**. fsspec (Py) / object_store (Rust) /
  S3/GCS (TS) DEFERRED to opt-in extras.
- Interface + FileSystemLoader ship in the **standard package**; heavier backends opt-in.
- Do **NOT** fuse into `Prompt.load(loader, format)` — keep loader (pure I/O leaf) and
  the spec-008 text factories (`from_yaml/from_json`) separate & composable:
  `Prompt.from_yaml(loader.load(key))`.
- Loader value = swappable/testable/centralized storage (Strategy-for-I/O), NOT "you
  couldn't read a file before". Opt-in; trivial single-file case can still use read_text.
  Document this honestly.
- Per-language plugin ecosystems (3 parallel by construction); pure Rust core untouched
  (Principle II/I): the loader is language-side only, kernel never learns about it.
- OPEN: error contract + custom-loader extension contract (key-not-found, LoadError
  normalization) — asking next.

## #270 — ProvenanceSink — RESOLVED: NOT a sink. Shape 2 helper.
- REJECTED the callback sink (Shape 1) + built-in OtelSink — violates Principle V
  ("no telemetry sink, no OTel coupling", written verbatim in RenderResult rustdoc)
  and re-adds a C-08-eliminated seam + hot-path hook + OTel optional-dep.
- SHIP: a projection method on RenderResult: `provenance_attributes()` (Py dict) /
  `provenanceAttributes()` (TS Record) / `provenance_attributes()` (Rust Vec<(String,String)>
  or BTreeMap). Returns FLAT string map with hardcoded OTel GenAI semantic-convention keys:
  gen_ai.prompt.{name,variant,template_hash,render_hash}.
- **4 content-identity fields ONLY** (Option A). NOT text, NOT guard (leak/size footgun),
  NOT metadata (opaque bag — flattening = interpretation, violates opaque-metadata doctrine;
  + span-cardinality footgun), NOT output_model/version (version doesn't exist in schema).
- Method not property (it's a projection). Hardcoded keys, no config knob (C-08). Caller
  who wants other keys reads the 4 public fields directly.
- ZERO kernel change (all 4 fields already on RenderResult). Pure binding-side getter.
- Amendment: SMALL — softens Principle V to "library MAY format provenance into a flat
  attribute map (still data on the return value); still no sink, no push, no telemetry dep."
## #271 — inheritance — RESOLVED: merge strategy on derive (NOT schema extends:)
- REJECTED YAML `extends:` field entirely — user does NOT want YAML-declarative
  inheritance; code-side is fine. So NO schema change, NO kernel change, NO loader/
  registry resolution (Q's Option 2/3/4 all moot). Jinja {% extends %} dead (Principle IV).
- The shipped primitive today is `derive(overlay)` (spec-008; named `derive`, NOT `with`)
  = shallow REPLACE per top-level field + full re-validation of merged whole, immutable.
  (Rust `Prompt::derive(PromptOverlay)`, Py `derive(overlay, validators=)`, TS `with(overlay, validators?)`.)
- SHIP (Shape 1): add keyword-only **`merge` strategy param to derive**:
  `derive(overlay, merge="replace"|"shallow")`, **default "replace"** (zero behavior
  change for existing callers even though 0.x lets us break).
  - `replace` = today's semantics (overlay map wins wholesale).
  - `shallow` = map-typed fields (variables, variants, metadata) UNION top-level keys,
    **child-wins** on collision; scalar fields (name/role/body/output_model) always replace.
- **`deep` and `none` EXCLUDED** (documented rationale): deep can't express deletion
  (spec-008 §9b sentinel problem) + would force library to interpret opaque `metadata`
  (violates opaque-metadata doctrine); `none` (add-only) has no motivating use case.
  Reserve the enum axis — a real consumer (C-08) can add `deep` later as a new value,
  no new method.
- Delivers Bellwether's "inherit base extraction var + add my own" via
  `base.derive({variables:{sentiment:{...}}}, merge="shallow")` → {extraction, sentiment}.
- 0.x: breaking semantic changes acceptable, but default=replace avoids gratuitous break.
- OPEN (Q13): does `variants` merge too or only `variables`? validator carry-forward
  under merge? name-collision on scalar during shallow (n/a — scalars always replace).
