# Memory synthesis — spec 019 (pluggable loader)

Compact planning context. Source of truth; ≤900 words. This is the MAJOR-amendment feature.

## What already ships (verified)

- Construction from text is the spec-008 taxonomy: primary constructor takes the shape;
  `from_yaml(text)` / `from_json(text)` parse foreign TEXT into the validating constructor. All
  I/O-free — the caller supplies text.
- Conformance decision **A1** (spec 006): the binding loaders do serde SHAPE validation, not full
  JSON-Schema. Relevant: loader (I/O) vs. construct-from-text (parse/validate) are already distinct
  layers; 019 formalizes the I/O layer as a pluggable seam.
- No loader/store exists today — C-08 eliminated it as a v1 seam; the "Never" list has "I/O /
  storage adapters." 019 reintroduces it as an EARNED opt-in seam (Bellwether is the trigger).

## What spec 019 adds

- A language-native **PromptLoader interface**: `load(key) -> raw text`, sync in Py/Rust, async
  (Promise) in TS (C-06). Format-agnostic leaf; returns text, never a Prompt, never parses.
- Two built-ins in the STANDARD package: **FileSystemLoader** (base dir + suffix convention) +
  **MemoryLoader** (key→text map). Heavier backends (fsspec/object_store/S3/GCS) DEFERRED to extras.
- Custom loaders: implement the interface, NO registration. Documented contract.
- Errors: missing key → structured **LoadError** (never null/empty). Built-ins raise it; custom
  documented-to but propagate their own as-is. LoadError is a DISTINCT surface from parse/validation.

## Explicit non-goals (load-bearing grill decisions)

- **NOT fused into construction** — no `Prompt.load(loader, ...)`; compose
  `Prompt.from_yaml(loader.load(key))`. Construction stays I/O-free.
- **NO name-keyed container/registry** in 019 (load-and-resolve-by-name / cache / check_all) —
  deferred to its own spec (it's the deferred "query-capable registry"; Q5b soundness hole for
  check_all under lazy loading is why it's separate).
- **NO cross-FFI loader** — per-language ecosystems by construction; a Python loader ≠ a TS loader.

## Governing constraints

- **Principles I/II unchanged:** kernel never learns about loaders, no FFI dep, validation-blind.
- **Principle III (softened here):** permit a caller-invoked, LANGUAGE-SIDE loader seam; kernel +
  construction stay I/O-free. This is THE major boundary relaxation.
- **C-08 (re-scoped):** Loader reintroduced as earned opt-in seam (second consumer = Bellwether).
- **C-06:** native idiom; sync/async per language; LoadError normalized into the common error shape.

## Amendment (MAJOR)

- 019 **cites** spec-017's v3.0.0 repositioning statement; its own edit = re-scope C-08 (Loader) +
  soften Principle III. Update the roadmap's Scope-Discipline + "Never: I/O/storage adapters" entries.
- Record in DECISIONS.md; propagate to constitution body/version + roadmap + rendered CLAUDE/AGENTS.

## Motivating consumer

Bellwether loads 4+ prompts (bull/bear/valuation/judge) from a directory; wants storage as a
swappable axis and disk-free tests (MemoryLoader). Value = centralize key→location, swap backend,
test without disk — Strategy-for-I/O, NOT "couldn't read a file before" (documented honestly).

## Open questions → clarify

1. Interface member name (`load`), FileSystemLoader options (base + suffix; options object per C-11).
2. LoadError placement in the normalized `[{field, code, message}]` taxonomy — is it that shape or a
   simpler I/O error? (Lean: normalized error family with an io/not-found code.)
3. TS async signature — `load(key): Promise<string>` confirmed; confirm FileSystemLoader uses async fs.
4. Rust: object-safe `dyn PromptLoader` + blanket impl for `Fn(&str) -> Result<String, LoadError>`?
   (matches issue sketch; confirm at plan.)
