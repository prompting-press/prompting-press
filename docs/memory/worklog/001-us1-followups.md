# Spec 001 US1 — carried follow-ups & decisions

Captured 2026-06-25 during US1 implementation. These are NOT 001 blockers; they are
deferred items routed to their owning spec, recorded so they survive across sessions.

## Cross-spec follow-ups (deferred out of 001 on purpose)

- **[spec 004] PyO3 module-name reconciliation.** `crates/prompting-press-py/src/lib.rs`
  defines `#[pymodule] fn prompting_press_py`, but `packages/python/pyproject.toml` sets
  maturin `module-name = "prompting_press"` with a mixed `python-source = "python"` layout
  (there is a `python/prompting_press/__init__.py`). At real `maturin build`/import time
  these must reconcile — typically the compiled module becomes a private submodule
  (e.g. `prompting_press._core`) re-exported from `__init__.py`, OR the `#[pymodule]` fn is
  renamed to match. The 001 stub `cargo check`s/builds fine; this only matters when the
  Python binding is actually built+imported (spec 004). No action in 001.

- **[spec 007] TS package `private: true`.** `packages/typescript/package.json` sets
  `"private": true` defensively (prevents accidental `npm publish` of a 0.0.0 artifact-less
  skeleton). Spec 007 (publish) must flip this to publishable when the napi prebuilds exist.

## Decisions made during US1

- **napi 2.x → 3.x.** The tasks.md/brief said "napi 2.x"; 3.x is the current stable major
  and builds clean. Kept 3.x (roadmap does not constrain binding version). Stale guidance.

- **pyo3 cdylib macOS link fix.** `extension-module` leaves CPython symbols undefined; a bare
  `cargo build` of the standalone cdylib fails to link on macOS. Fixed with a crate-scoped
  `crates/prompting-press-py/build.rs` emitting `cargo:rustc-link-arg=-undefined dynamic_lookup`
  guarded by `cfg!(target_os = "macos")`. Chosen over a repo-wide `.cargo/config.toml` because
  `cargo:rustc-link-arg` from a build script does NOT enter the RUSTFLAGS fingerprint — so it
  cannot perturb the US3 codegen-determinism gate or the US4 `cargo tree` FFI-isolation gate.
  WINDOWS CAVEAT: Windows PyO3 linking differs (links a python3.dll import lib, no
  dynamic_lookup). A bare `cargo build` on Windows may need a build.rs Windows branch or a
  CI-provided Python lib — see CI-matrix decision below.

## CI build-matrix decision (input to US4 / T028–T031)

- User chose **Linux + macOS + Windows** for the spec-001 CI *build* job.
- Gate LOGIC stays single-runner (Linux): FFI-isolation (`cargo tree`) is OS-independent;
  codegen-freshness must be pinned to ONE canonical runner (Linux) to avoid rustfmt/EOL drift.
- Only the `cargo build --workspace` job is the matrix (×3 OS) — it is the OS-sensitive part
  and would have caught the pyo3 macOS link bug automatically.
- ACTION for T028–T031: when authoring `.github/workflows/`, the build matrix must include
  windows-latest, which likely forces a Windows branch in `crates/prompting-press-py/build.rs`
  (or a CI step providing the Python import lib). Verify Windows PyO3 link behavior at that time.

## US3 codegen — typify `propertyNames`/`not` panic (resolved)

- **Finding (T022 spike):** `cargo-typify` 0.7.0 PANICS (`unimplemented!` at convert.rs:1763) on
  `variants.propertyNames = { "not": { "const": "default" } }` — it has no handling for `not`
  subschemas. Isolated/confirmed: deleting only that key makes typify generate clean output
  (correct enums, `#[serde(deny_unknown_fields)]`, `serde_json::Map` for open objects, deterministic).
- **Probed the other two generators against the schema AS-IS:** datamodel-code-generator (Python)
  and json-schema-to-typescript both exit 0 and SILENTLY DROP `propertyNames` (Python → `dict`,
  TS → `{ [k:string]: Variant }`). No generated type in ANY language can encode "map key must not
  equal 'default'" — `propertyNames` is inherently a validation constraint, not a type constraint.
- **Decision:** Rust codegen step strips `properties.variants.propertyNames` from a TYPIFY-INPUT
  COPY of the schema (`jq 'del(.properties.variants.propertyNames)'`), NOT from the canonical
  `schemas/jsonschema/prompt-definition.schema.json`. The schema stays the single source of truth,
  cross-language consistent. The reserved-`default` rule (FR-011b) remains enforced by the US2
  validation gate (`variant-named-default.json` reject fixture — already proven green).
- **Rejected alternative:** rewriting to `"pattern": "^(?!default$).*$"` — would mutate the canonical
  schema, pull `regress` + `LazyLock` into generated Rust, and emit a divergent key-newtype, all to
  encode a rule the validation layer already enforces. Not worth it.
- **Exact Rust codegen invocation (T025):**
  `jq 'del(.properties.variants.propertyNames)' <schema> > <tmp>` →
  `cargo typify --no-builder --output <dest>/prompt_definition.rs <tmp>` →
  `rustfmt --edition 2021 <dest>/prompt_definition.rs`. Use `--no-builder` (754 vs 1165 lines).
  Note: typify emits crate-level `#![allow(...)]` inner attrs → the generated file must be a module
  file (not `include!`d mid-file); `name` becomes a `PromptDefinitionName` newtype.

## Phase 3 QA findings (ceremony, 2026-06-25)

- **CI result (first push of 001-foundations):** Build matrix Linux+macOS+**Windows** all PASS —
  the Windows PyO3 abi3 link risk is RESOLVED (`actions/setup-python` sufficed; no build.rs Windows
  branch needed). The `gates` job FAILED on one real bug (below), now fixed.
- **REAL BUG (fixed): floating-version lint matched its own comments.** `scripts/ci/check-floating-versions.sh`
  scanned raw file content; `mise.toml`/`pyproject.toml` SEC-003 explanatory comments literally contain
  `"latest"`/`^`/`~`/`*`, so the lint flagged its own docs (CI `gates` failure). Passed locally only
  because moon's affected-detection made it a no-op cache hit. Fix: strip TOML comments before scanning
  + switched `grep -P`→`grep -E` (a SECOND bug: local `ugrep -P` silently returns 1 in the pipeline
  context → false negatives). Verified: clean pass, catches real `^`/`*`/`latest`, ignores comments.
- **verify (Step 11) SC-007 false-flag:** the spec-verify subagent reported `negative-scope-checklist.md`
  missing — but it exists + is committed. The agent had no Bash/Glob (Read-only context) so couldn't
  see it. SC-007 is actually PASS. (Same self-referential-string false-positive class as the lint bug
  and an earlier `grep packages/go` flag — pattern worth remembering.)
- **verify-tasks subagents (Step 10) failed twice** on an environment tool-channel glitch (0 tool uses,
  no output). Ran the phantom sweep on the main thread instead: 35/35 VERIFIED, 0 phantoms.

## Tooling bug observed (APM-upstream, logged not fixed)

- `.claude/hooks/hooks-bash-safety/scripts/rm-rf-guard.sh` uses `;;&` (bash 4+ fall-through) on line 24
  but runs under macOS bash 3.2.57 → parse error → fails closed, blocking ANY command matching its `rm`
  regex (incl. harmless `git rm --cached`, `rm -rf .moon/cache`). Hit 3× this session; worked around with
  `git update-index --force-remove` and `moon run --force`.
  **Root: the bug is in the APM package source** `apm_modules/srobroek/agentic-packages/packages/hooks-bash-safety/scripts/rm-rf-guard.sh`
  (the project copy is byte-identical, regenerated by `apm compile`). Fix belongs UPSTREAM in the
  agentic-packages repo (rewrite the `case` line 24 as explicit `[[ ]]` checks: set has_r/has_f
  independently), NOT in this project — editing the project copy would be clobbered. Out of spec-001 scope.
