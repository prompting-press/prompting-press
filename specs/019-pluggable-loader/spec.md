# Feature Specification: Pluggable prompt loader

**Feature Branch**: `019-pluggable-loader`

**Created**: 2026-07-08

**Status**: Draft

**Input**: Issue #268 (feat: pluggable PromptLoader). Resolved via design grilling to a
**standalone, per-language pluggable loader interface** with two built-in implementations
(filesystem + in-memory), shipped in the standard package. The loader is a pure I/O leaf that
returns raw text; it is NOT fused into prompt construction, and there is NO name-keyed
container/registry in this feature (that is deferred to its own spec). Heavier storage backends
(cloud/object stores) are deferred to opt-in extras.

## Clarifications

### Session 2026-07-08 (proposed defaults — user away; revisit if needed)

- Q: What is the loader interface member + error type? → A: `load(key) -> str` (sync Py/Rust; async
  `Promise<string>` TS). Failure raises/returns a **`PromptLoadError`** (see the addendum — a NEW type,
  not the existing `LoadError`), normalized into the common `[{field, code, message}]` family (C-06).
  Distinct surface from construct-from-text (parse/validation) errors.
- Q: How does `FileSystemLoader` map a key to a file, and how is traversal handled? → A: `base`
  directory + a `suffix` (default `.yaml`); `load(key)` reads `{base}/{key}{suffix}`. Keys are
  treated as relative paths under `base`; a key that escapes `base` (e.g. via `..`) MUST be
  rejected with a `PromptLoadError` (path-traversal guard — a security default), not read.
- Q: Rust interface shape? → A: an object-safe `trait PromptLoader { fn load(&self, key: &str) ->
  Result<String, PromptLoadError>; }` (usable as `dyn PromptLoader`) plus a **blanket impl** for
  `Fn(&str) -> Result<String, PromptLoadError>` so a closure works as a loader without a struct.

### Session 2026-07-08 (addendum — post-adversarial-review)

- Q: The proposed `LoadError` name + `not_found`/`io` codes collided with the shipped taxonomy
  (`LoadError` IS the parse/shape error; `ConsumerError::Load` is a bare-string variant in a CLOSED
  enum; the `code` vocab is a CLOSED compatibility surface). Resolution? → A: The loader's I/O error
  is a **NEW `PromptLoadError`** class (paralleling `PromptRenderError`/`PromptValidationError`) with
  **NEW codes `load_io`/`load_not_found`**. This is a **compatibility-surface expansion** recorded
  explicitly in the amendment (FR-018), spec-015-style — NOT a reuse of `LoadError`/`ConsumerError::Load`.
- Q: Traversal-guard mechanics (the security review found `{base}/{key}{suffix}` is string-concat)? →
  A: validate the FINAL resolved path incl. suffix against canonicalized `base`; reject absolute/`..`/
  escaping-symlink keys; define `key=""`/`key="."`/empty-suffix; canonicalize-fail-on-missing →
  `load_not_found` (FR-002b). Add a `max_bytes` read cap (FR-016). Guard + cap tested per language (FR-017).
- Q: v3.0.0 coordination across the three specs? → A: 017 is the v3.0.0 baseline (already written); 019
  **cites** it and lands as an **additive amendment → v3.2.0** (018 = v3.1.0), adding only its own edits
  (Principle III softening + C-08 Loader re-scope + the error-taxonomy expansion). No re-declaration.

## Iterations

### Iteration 2026-07-08: fold adversarial-review findings

**Change**: Renamed the loader's I/O error to a NEW `PromptLoadError` + codes `load_io`/`load_not_found`
(the existing `LoadError` is the parse error — collision); recorded the error-taxonomy
compatibility-surface expansion in the amendment (spec-015-style); hardened the traversal guard
(final-path incl. suffix, absolute/`..`/symlink/empty-key, canonicalize-fail→not_found) and added a
`max_bytes` read cap (security MEDIUMs); made the guard + cap per-language (FR-017); added the
native-loader error-raise path (FR-008a); reconciled the amendment to cite 017's v3.0.0 baseline as an
additive v3.2.0.
**Scope**: Feature-wide (pre-implementation; amendment scope grew — new error taxonomy + security hardening).
**Artifacts updated**: spec.md, plan.md, research.md, data-model.md, contracts/loader.md, tasks.md.
**FRs added**: FR-002b (guard mechanics), FR-016 (read cap), FR-017 (per-language), FR-008a (native raise); FR-018/019 amendment reshaped.
**SCs added**: SC-009 (cap/not_found-vs-io), SC-010 (distinct class + recorded expansion).
**Tasks marked complete**: none (0 of 22 built).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load prompt source from the filesystem behind a swappable interface (Priority: P1)

A consumer stores prompt definition files in an in-repo directory and wants to load a prompt's
source text by a logical key (not a hardcoded physical path), through an interface they can later
swap for a different storage backend without changing their call sites.

**Why this priority**: This is the primary use case and the reason the feature exists — a real
consumer (Bellwether) loads several prompts from a directory and wants storage to be a pluggable
axis. The filesystem loader behind the interface is the minimum viable delivery.

**Independent Test**: Point a filesystem loader at a directory of prompt files, request a prompt
by logical key, and assert it returns that file's raw text. Then substitute a different loader
implementing the same interface and confirm the consuming code that calls the interface is
unchanged.

**Acceptance Scenarios**:

1. **Given** a filesystem loader configured with a base directory and file convention, **When** a
   consumer requests a prompt by logical key, **Then** the loader returns the raw text of the
   corresponding file.
2. **Given** consuming code written against the loader interface, **When** the loader
   implementation is swapped (e.g. filesystem → in-memory), **Then** the consuming call sites do
   not change.
3. **Given** the returned raw text, **When** the consumer passes it to the existing
   construct-from-text path, **Then** a validated prompt object is produced (loader and
   construction remain separate, composable steps).

---

### User Story 2 - Load prompt source from memory for tests and embedding (Priority: P1)

A consumer wants to supply prompt source from an in-memory mapping (no filesystem access) — for
unit tests, or for embedding prompts in a binary — using the same interface as production code.

**Why this priority**: Testability is a core value of the pluggable interface (dependency-inject a
memory loader; production uses filesystem; the code under test never touches disk). An in-memory
loader is the second built-in and is essential to the interface's value proposition, so it is P1
alongside the filesystem loader.

**Independent Test**: Construct an in-memory loader from a key→text mapping, request a key, and
assert it returns the mapped text; request a missing key and assert a structured not-found error.

**Acceptance Scenarios**:

1. **Given** an in-memory loader built from a key→text mapping, **When** a consumer requests a
   present key, **Then** the loader returns the mapped raw text.
2. **Given** the same loader, **When** a consumer requests a key not in the mapping, **Then** the
   loader raises/returns a structured not-found error (never an empty string or null).

---

### User Story 3 - Implement a custom storage backend against a clear contract (Priority: P2)

A consumer (or a future extension package) wants to implement a custom loader for a storage system
the library does not ship (e.g. a cloud object store, a database, a remote cache), guided by a
clear, documented contract for the load operation and its error behavior.

**Why this priority**: The extension ecosystem is the strategic point of the interface (the built-in
loaders are just the first two implementations). It is P2 rather than P1 because the built-ins
deliver the immediate value; the documented custom-loader contract enables the ecosystem to grow.

**Independent Test**: Following the documented contract, implement a trivial custom loader (e.g.
one backed by a provided mapping with a transform), use it wherever a loader is expected, and
confirm it works interchangeably with the built-ins, including its failure behavior.

**Acceptance Scenarios**:

1. **Given** the documented loader contract, **When** a consumer implements a custom loader that
   satisfies it, **Then** that loader is usable anywhere a built-in loader is used, with no special
   registration.
2. **Given** a custom loader that fails to find a key, **When** it signals failure per the
   documented contract, **Then** the failure surfaces to the consumer as a load error distinct from
   any downstream parse/validation error.

---

### Edge Cases

- **Missing key**: every loader (built-in and custom) signals a missing key as a structured
  **load error** — never an empty string, null, or a partial result. This prevents an empty string
  silently flowing into construction and surfacing as a confusing parse error far from the cause.
- **Load error vs. parse/validation error are distinct surfaces**: a loader failure (I/O, missing
  key) is a different, documented error class from a construct-from-text failure (malformed or
  invalid content). A consumer can tell which layer failed.
- **Custom loader throwing a non-standard error**: the contract documents raising the library's
  load-error type for normalized handling, but a custom loader's own error propagates as-is (the
  library does not silently catch-and-wrap third-party errors it cannot anticipate).
- **Asynchronous backends**: the loader contract follows each language's native I/O idiom — the
  operation is synchronous where that is idiomatic and asynchronous where that is idiomatic (so
  cloud/remote backends are expressible in the language whose ecosystem expects async I/O).
- **Format-agnostic**: the loader returns raw text and does not detect or parse the definition
  format; choosing YAML vs. JSON (etc.) belongs to the construct-from-text step, not the loader.
- **Loader is not fused into construction**: there is no single call that both loads and constructs;
  the consumer composes the loader's output into the existing construct-from-text step. (Any
  ergonomic sugar over the two steps is out of scope for this feature.)
- **No name-keyed container in this feature**: there is no library object that holds a loader and
  resolves prompts by name / caches parsed prompts / lints them all. That capability is deferred to
  its own spec; this feature ships only the loader leaf.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The library MUST define a **loader interface** whose single operation `load(key)`
  takes a logical key and returns the prompt source as **raw text**. The interface MUST be
  language-native in each binding (an implementable protocol/trait/interface), so consumers can
  provide their own implementations. In Rust it MUST be object-safe (usable as `dyn PromptLoader`)
  with a **blanket impl** for `Fn(&str) -> Result<String, PromptLoadError>` so a closure works as a loader
  without defining a struct; Python/TS provide equivalent callable/function coercion.
- **FR-002**: The library MUST ship a **filesystem loader** built-in that maps a logical key to a
  file under a configured **base** directory with a configurable **suffix** (default `.yaml`),
  reading `{base}/{key}{suffix}` and returning that file's raw text.
- **FR-002a** (security): The filesystem loader MUST treat keys as relative paths under `base` and
  MUST **reject a key that escapes `base`** (e.g. via `..` traversal or an absolute path) with a
  `PromptLoadError`, rather than reading outside the configured base directory.
- **FR-003**: The library MUST ship an **in-memory loader** built-in constructed from a key→text
  mapping, returning the mapped text.
- **FR-004**: Both built-in loaders MUST ship in the **standard package** (no extra install
  required). Heavier/third-party storage backends MUST NOT be added in this feature; they are
  deferred to opt-in extras.
- **FR-005**: The loader operation MUST return **raw text only** — it MUST NOT parse, validate, or
  detect the definition format, and MUST NOT return a constructed prompt object. Loading and
  construction remain separate, composable steps.
- **FR-006**: A missing key MUST produce a **structured load error** in every loader (built-in and,
  per contract, custom) — never an empty string, null, or partial result.
- **FR-007**: A **load error** MUST be a distinct, documented error surface from a
  construct-from-text (parse/validation) error — distinct **at the class/type level**, not merely by
  an inspected code string — so a consumer can `except PromptLoadError` / `catch (PromptLoadError)`
  without also catching a malformed-content error. (The existing `LoadError` class is the **parse/
  shape** error; the loader's I/O error MUST NOT reuse it — see FR-008.)
- **FR-008**: The loader's failure surface is a **NEW** error type **`PromptLoadError`** (paralleling
  the existing `PromptRenderError`/`PromptValidationError` family), NOT the existing `LoadError`
  (which remains the parse/shape error). It normalizes into the common `[{field, code, message}]`
  family (C-06) under **NEW codes `load_io` / `load_not_found`** added to the closed `code` vocab.
  Because the `code` vocabulary and the `ConsumerError` variant set are documented **closed
  compatibility surfaces**, introducing the new type + codes is a **compatibility-surface expansion**
  that MUST be recorded explicitly in the amendment (FR-016) — enumerated spec-015-style — and carried
  through both FFI mappers + the per-binding routing. Built-in loaders raise/return `PromptLoadError`;
  the custom-loader contract documents raising it, while a custom loader's own error propagates as-is
  (the library MUST NOT silently catch-and-wrap arbitrary third-party errors).
- **FR-008a** (native error-raise): Because the binding exception types are minted in Rust
  (`create_exception!`, which carries no typed field), each binding MUST expose a way for a
  **native-language** loader (a pure-Python / pure-TS `FileSystemLoader` or custom loader) to raise
  `PromptLoadError` **with a populated `[{field, code, message}]` payload** (e.g. a
  constructor/factory the native loader calls), so "loaders are native per language" holds without
  the error being un-constructable from that language.
- **FR-009**: The loader operation MUST follow each language's native I/O idiom regarding
  synchronous vs. asynchronous execution (synchronous where idiomatic; asynchronous where idiomatic,
  so remote/cloud backends are expressible) (C-06).
- **FR-010**: A custom loader satisfying the interface MUST be usable anywhere a built-in loader is
  used, with **no registration step** and no library-side allow-list.
- **FR-011**: The loader MUST NOT be fused into prompt construction: the library MUST NOT add a
  construction path that performs I/O via a loader. Construction remains I/O-free and consumes text
  the caller supplies (Principle III). Any convenience that composes load + construct is out of
  scope for this feature.
- **FR-012**: This feature MUST NOT add a name-keyed prompt **container/registry** (an object that
  owns a loader, resolves prompts by name, caches, or lints a collection). That capability is
  deferred to a separate spec.
- **FR-013**: The loader interface and built-ins MUST be **language-side only**: the
  `prompting-press-core` kernel MUST NOT learn about loaders, MUST NOT depend on any I/O, and MUST
  remain validation-blind (Principles I, II, III). The interface is defined independently per binding
  (three parallel per-language ecosystems by construction), not as a cross-FFI concept.
- **FR-014**: The loader contract MUST be **documented** for custom-loader authors: the operation
  signature, the raw-text return, the missing-key/error behavior (FR-006/007/008), and the
  sync/async idiom (FR-009), so third parties can implement conforming loaders.
- **FR-015**: The capability MUST be delivered in all three bindings (Rust consumer, Python,
  TypeScript) with equivalent capability, expressed in each language's native idiom (C-06); a custom
  loader written for one language is not expected to run in another (the ecosystems are parallel).
- **FR-002b** (traversal-guard mechanics): The guard MUST validate the **final resolved path
  including the `suffix`** (`{base}/{key}{suffix}` is a path join + suffix, not a way to bypass the
  check), against a canonicalized `base`. It MUST reject: absolute keys, keys with `..` components,
  and symlinks that escape `base`. Behavior MUST be defined for `key=""`, `key="."`, and a
  caller-set empty `suffix` (these resolve to a structured error or a defined not-found, never an
  undefined read). Cross-platform separators / UNC / embedded-NUL keys MUST be rejected. A
  canonicalize failure on a missing target MUST surface as `load_not_found`, not `load_io`.
- **FR-016** (read-size cap): `FileSystemLoader` MUST support an optional maximum read size
  (`max_bytes`) with a sane default, returning `PromptLoadError` (`load_io`) when exceeded — defense
  against unbounded-read / device-file DoS (a NEW input path not covered by spec 009's fuzzing).
- **FR-017** (per-language guard): The traversal guard (FR-002a/FR-002b) and the read cap (FR-016)
  MUST be implemented and tested in **each** language's `FileSystemLoader` (loaders are native per
  language), not only in Rust.

### Constitution amendment requirements

- **FR-018**: This feature MUST make its constitutional edit by **relaxing the boundary**: it
  re-scopes Scope-Discipline (the previously eliminated **Loader** seam is reintroduced as an
  earned, opt-in seam) and softens the "no I/O / no storage layer" clause of Principle III to permit
  a **caller-invoked, language-side loader seam** — while preserving that the **kernel** performs no
  I/O and stays validation-blind (Principles I/II unchanged), and that **construction** remains
  I/O-free (FR-011). It MUST ALSO record the **error-taxonomy compatibility-surface expansion**
  (the new `PromptLoadError` type + `load_io`/`load_not_found` codes added to the closed `code`
  vocab and error set) — enumerated explicitly, spec-015-style — since the `code` vocabulary and
  `ConsumerError` variant set are documented compatibility surfaces.
- **FR-019**: This feature MUST **cite** the v3.0.0 repositioning statement introduced by spec 017
  (minimal core PLUS earned, opt-in seams) as the shared anchor — NOT re-declare it. Spec 017 is the
  v3.0.0 baseline; 019 is an **additive amendment on top of it → v3.2.0** (018 is v3.1.0), adding
  only 019's own edits (Principle III softening, C-08 Loader re-scope, the error-taxonomy expansion).
  The real second consumer (Bellwether) is the earning trigger under the Scope-Discipline
  "second concrete consumer" rule.
- **FR-018**: The amendment MUST be recorded in `DECISIONS.md` with rationale and version bump per
  the Governance policy, and MUST propagate to the constitution body + version line, the roadmap
  (the Scope-Discipline/Loader and the "Never: I/O / storage adapters" entries updated), and the
  rendered agent-context copies (`CLAUDE.md` / `AGENTS.md`).

### Key Entities *(include if feature involves data)*

- **Loader interface**: the language-native contract with one operation — given a logical key,
  return the prompt source as raw text, or signal a structured load error. The extension point for
  custom storage backends.
- **Filesystem loader**: a built-in loader resolving a logical key to a file under a configured base
  location and file-name convention.
- **In-memory loader**: a built-in loader resolving a logical key against a provided key→text
  mapping; the primary testing/embedding implementation.
- **Load error**: the library's normalized error type for load failures (missing key, I/O failure),
  distinct from construct-from-text errors.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A consumer can load a prompt's source by logical key from the filesystem and from an
  in-memory mapping, using the same interface, in all three bindings.
- **SC-002**: A consumer can swap the loader implementation (filesystem ↔ in-memory ↔ custom)
  without changing the call sites that consume prompts — verified by a test that runs the same
  consuming code against two loaders.
- **SC-003**: Requesting a missing key yields a structured load error (not an empty string/null) in
  every built-in loader, and that error is distinguishable from a construct-from-text error —
  verified per binding.
- **SC-004**: A custom loader implemented solely against the documented contract is usable
  interchangeably with the built-ins, with no registration — verified by an example/test custom
  loader in each binding.
- **SC-005**: The `prompting-press-core` crate is unchanged by this feature (no diff), and no I/O is
  added to the kernel or to prompt construction — confirming the language-side-only, kernel-pure
  boundary.
- **SC-006**: Both built-in loaders are available from the standard package with no extra install;
  no cloud/third-party storage dependency is added in this feature — verified by manifest inspection.
- **SC-007**: The constitution reflects the relaxed boundary (Loader reintroduced as an earned
  opt-in seam; Principle III softened for a caller-invoked language-side loader while the kernel and
  construction stay I/O-free), cites the spec-017 repositioning statement, and the amendment is
  recorded in `DECISIONS.md` and the roadmap with rendered copies in sync.
- **SC-008**: The filesystem loader rejects a path-traversal key (e.g. `../secret`), an absolute key,
  and an escaping symlink with a `PromptLoadError` and never reads a file outside its configured
  `base` directory — verified by tests in **each** binding (FR-017).
- **SC-009**: The filesystem loader returns `PromptLoadError` (`load_io`) when a file exceeds the
  configured `max_bytes` cap, and a missing key returns `load_not_found` (not `load_io`) — verified.
- **SC-010**: `PromptLoadError` is a distinct class from the parse-error `LoadError` (`except
  PromptLoadError` does not catch a malformed-YAML error), and the amendment records the
  `PromptLoadError` + `load_io`/`load_not_found` compatibility-surface expansion explicitly.

## Assumptions

- The existing construct-from-text path (construct a validated prompt from YAML/JSON text) is the
  composition target for loader output; this feature does not change construction.
- Code-side, per-language loaders are the accepted model (three parallel ecosystems); a single
  cross-language loader format is explicitly not a goal (it would break FFI isolation).
- The name-keyed container/registry (load-and-resolve-by-name, caching, check-all) is intentionally
  deferred to its own spec; this feature is the loader leaf only.
- The loader's value is centralization + swappability + testability (a Strategy-for-I/O), not a
  new capability that was impossible before (a consumer can still read a single local file directly
  without a loader). This is documented so the loader is not mistaken for magic and its opt-in
  nature is clear.
- Cloud/object-store and other heavy backends are deferred to opt-in extras once demand is
  established; only filesystem + in-memory ship now.
- Breaking changes are permissible at 0.x, but this feature is additive (new interface + built-ins);
  it changes no existing behavior.
- The exact per-language surface (interface member name, filesystem-loader configuration options,
  sync/async signature per binding, load-error type shape and its place in the normalized error
  taxonomy) is settled at plan time within the C-06 idiom and the error-normalization constraints
  recorded here.
