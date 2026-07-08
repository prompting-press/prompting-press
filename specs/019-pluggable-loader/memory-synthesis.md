# Memory Synthesis

## Current Scope

Spec 019 — a language-native `PromptLoader` (`load(key)->text`) + `FileSystemLoader` (base+suffix,
traversal-guarded) + `MemoryLoader`, per binding. Pure I/O leaf composed with the existing text
factories; no fusion, no container, no cross-FFI loader. Carries the MAJOR amendment (Principle III
softening + C-08 Loader re-scope), citing spec-017's v3.0.0 repositioning.

## Relevant Decisions

- **A1 — Loader does serde shape-validation, not full JSON-Schema** (Reason: 019 formalizes the
  I/O layer as distinct from the construct-from-text validation layer; the loader produces text, the
  factory validates shape. Status: active. Source: docs/memory/architecture/2026-06-28-loader-vs-schema-validation-layers.md)
- **Spec-017 v3.0.0 repositioning statement** (Reason: 019 cites it as the anchor for reintroducing
  the Loader seam; earned by second consumer Bellwether. Status: pending/landing with 017.)

## Active Architecture Constraints

- **Kernel + construction stay I/O-free** — the softening is scoped to a caller-invoked, language-side
  loader leaf; `prompting-press-core` untouched (SC-005); `from_yaml` etc. do no I/O (FR-011).
- **Existing text factories** (`crates/prompting-press/src/prompt.rs:108+`: from_yaml/json/toml) are
  the composition target — loader output flows into them; loader is NOT fused in.
- **Existing error family** `ConsumerError::Load` + `[{field,code,message}]` (error.rs:97-109) is
  what `LoadError` normalizes into (C-06).

## Accepted Deviations

- **MAJOR boundary amendment (ratified this spec):** Principle III "no I/O" softened for an opt-in
  language-side loader; C-08 "Loader eliminated" re-scoped to "earned opt-in seam". Justified by the
  repositioning + Bellwether. Kernel/construction I/O-free preserved. Status: accepted-deviation →
  becomes constitution v3.0.0.

## Relevant Security Constraints

- **Path traversal (SC-008/FR-002a):** FileSystemLoader must reject keys escaping `base` (`..`,
  absolute) — canonicalize + prefix-check. Prevents arbitrary file read when keys are attacker-influenced.
- **Error scrubbing (D2/D3 lineage):** a `LoadError` must not leak file contents / secret-looking
  values into the default message; only the key/path context, scrubbed consistently with the existing
  boundary.

## Related Historical Lessons

- **C-08 earn-the-seam:** the Loader is exactly the interface R1 eliminated; reintroduced only because
  a real second consumer (Bellwether) now needs swappable storage — the C-08 bar, met.
- **Per-language idiom (C-06):** sync Rust/Py, async TS; native protocol/trait/interface — a loader
  does not cross FFI.

## Conflict Warnings

- **Hard (resolved by ratified amendment):** without the amendment, a `FileSystemLoader` reading files
  DIRECTLY violates Principle III "no I/O / no storage layer" and the "Never: I/O/storage adapters"
  roadmap entry. Resolved by the MAJOR amendment (Principle III softening + C-08 re-scope + roadmap
  update), scoped so kernel/construction stay I/O-free. Not a silent violation.

## Retrieval Notes

- Read: INDEX.md, A1, D2/D3 (scrubbing); prompt.rs (factories), error.rs (error family). Governance:
  constitution Principle III + Scope Discipline/C-08 + "Never" list; roadmap. memory-md MCP
  unavailable → direct reads. Budget within limits.
