# T036 — Advisory Gate Report

## Tool and version chosen

**cargo-deny 0.19.9** (current stable as of 2026-06-26, confirmed via `cargo search cargo-deny`).

Rationale: cargo-deny is the modern standard for Rust advisory/license/bans/sources scanning in a
single tool. It is hash-pinnable via mise's cargo backend (`locked = true`) exactly like
`cargo-typify`, fits the existing pattern, and needs only `Cargo.lock` — no full build required.
`cargo-audit` was the alternative but is narrower (advisories only, no license/bans), less actively
maintained in comparison, and cargo-deny is already what the Rust ecosystem uses at scale (polars,
embassy, etc.).

## mise.toml entry

```toml
"cargo:cargo-deny" = { version = "0.19.9", locked = true }
```

Added under the new `# --- Security toolchain (T036 / SEC-001) ---` comment block after
`cargo-typify`.

## deny.toml

Created at repo root (`/Users/sjors/personal/dev/prompting-press/deny.toml`).

Key config decisions (cargo-deny v2 schema):

- `[advisories]`: `unmaintained = "all"`, `unsound = "all"`, `yanked = "deny"`, `ignore = []`.
  In v2 the `vulnerability`/`notice`/`severity-threshold` fields are removed; all CVE-tagged
  advisories are denied by default unless in `ignore`.
- `[licenses]`: explicit allow-list of common permissive licenses (MIT, Apache-2.0, BSD-*,
  ISC, etc.), `confidence-threshold = 0.8`. Deliberately permissive — policy gate deferred post-v1.
- `[bans]`: `multiple-versions = "warn"`, empty `deny` list. Not a hard gate in v1.
- `[sources]`: `unknown-registry = "deny"`, `unknown-git = "deny"` — only crates.io allowed.

Schema notes encountered during implementation:
- v2 removed `vulnerability`, `unsound` (as lint level), `notice`, `severity-threshold` from
  `[advisories]`. These emit hard errors if present.
- v2 removed `unlicensed` and `copyleft` from `[licenses]`.
- `unmaintained`/`unsound` now take a scope enum: `"all"/"workspace"/"transitive"/"none"`.

## moon task

Added to `ci/moon.yml`:

```yaml
check-advisories:
  description: 'T036 — Scan workspace deps for known CVEs via cargo-deny + RustSec advisory DB (SEC-001). Gate owner also owns minijinja stable-API re-check on each bump (roadmap Q3).'
  command: 'bash scripts/ci/check-advisories.sh'
  options:
    runFromWorkspaceRoot: true
    cache: false
```

## Gate script

`scripts/ci/check-advisories.sh` — follows house style (set -euo pipefail, SCRIPT_DIR/REPO_ROOT
pattern, explicit success/failure messaging). Runs:

```
cargo deny --manifest-path "${REPO_ROOT}/Cargo.toml" check advisories
```

Advisories-only invocation — does not run license/bans checks (those are policy gates for later).

## CI wiring

Added to `.github/workflows/ci.yml`, `gates` job, after `T029: Codegen freshness`:

```yaml
- name: 'T036: Advisory scan'
  run: mise exec -- moon run ci:check-advisories
```

No additional dep-install step needed: cargo-deny is installed by `mise-action` (already present
for all other cargo tools). The gate reads `Cargo.lock` — no build required.

## Roadmap-Q3 note location

The obligation is documented in two places:
1. `deny.toml` header comment (lines 9–17): the ROADMAP-Q3 NOTE block with full procedure.
2. `scripts/ci/check-advisories.sh` header comment (lines 11–17): same note, shorter form.
3. `ci/moon.yml` task description: "Gate owner also owns minijinja stable-API re-check on each
   bump (roadmap Q3)."

## Verbatim verification output

### Advisory gate (direct)

```
$ mise exec -- cargo deny check advisories
advisories ok
```

### Advisory gate via moon

```
$ mise exec -- moon run ci:check-advisories
▮▮▮▮ ci:check-advisories (efb946a8)
Advisory gate: running cargo deny check advisories...
  Config: /Users/sjors/personal/dev/prompting-press/deny.toml
  Lockfile: /Users/sjors/personal/dev/prompting-press/Cargo.lock

advisories ok

Advisory gate PASSED — no known vulnerabilities in workspace dependencies.
▮▮▮▮ ci:check-advisories (6s 389ms, efb946a8)

Tasks: 1 completed
 Time: 8s 564ms
```

### Floating-version gate re-check

```
$ mise exec -- moon run ci:check-floating-versions
▮▮▮▮ ci:check-floating-versions (abcf06ec)
Floating-version lint PASSED — all manifests use pinned versions.
  OK: mise.toml
  OK: Cargo.toml
  OK: packages/typescript/package.json
  OK: packages/python/pyproject.toml
  OK: crates/prompting-press/Cargo.toml
  OK: crates/prompting-press-core/Cargo.toml
  OK: crates/prompting-press-py/Cargo.toml
  OK: crates/prompting-press-node/Cargo.toml
▮▮▮▮ ci:check-floating-versions (17s 308ms, abcf06ec)

Tasks: 1 completed
 Time: 18s 212ms
```

## Changed files

- `mise.toml` — added `"cargo:cargo-deny" = { version = "0.19.9", locked = true }`
- `deny.toml` — new file (repo root)
- `scripts/ci/check-advisories.sh` — new gate script
- `ci/moon.yml` — added `check-advisories` task
- `.github/workflows/ci.yml` — added `T036: Advisory scan` step to `gates` job
