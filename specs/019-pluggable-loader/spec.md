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

- **FR-001**: The library MUST define a **loader interface** whose single operation takes a logical
  key and returns the prompt source as **raw text**. The interface MUST be language-native in each
  binding (an implementable protocol/trait/interface), so consumers can provide their own
  implementations.
- **FR-002**: The library MUST ship a **filesystem loader** built-in that maps a logical key to a
  file under a configured base location (with a configurable file-name convention) and returns that
  file's raw text.
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
  construct-from-text (parse/validation) error, so a consumer can distinguish an I/O/lookup failure
  from a content failure.
- **FR-008**: The built-in loaders MUST raise/return the library's **normalized load-error type** on
  failure. The custom-loader contract MUST **document** raising that type for normalized handling,
  while allowing a custom loader's own error to propagate as-is (the library MUST NOT silently
  catch-and-wrap arbitrary third-party errors).
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

### Constitution amendment requirements

- **FR-016**: This feature MUST make its constitutional edit by **relaxing the boundary**: it
  re-scopes Scope-Discipline (the previously eliminated **Loader** seam is reintroduced as an
  earned, opt-in seam) and softens the "no I/O / no storage layer" clause of Principle III to permit
  a **caller-invoked, language-side loader seam** — while preserving that the **kernel** performs no
  I/O and stays validation-blind (Principles I/II unchanged), and that **construction** remains
  I/O-free (FR-011).
- **FR-017**: This feature MUST **cite** the v3.0.0 repositioning statement introduced by spec 017
  (minimal core PLUS earned, opt-in seams) as the shared anchor for reintroducing the loader seam,
  with the real second consumer (Bellwether) as the earning trigger under the Scope-Discipline
  "second concrete implementation/consumer" rule.
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
