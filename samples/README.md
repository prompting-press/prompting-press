# Consumer sample apps

Complete, runnable consumer programs — one CLI per language — that exercise the
**full public feature surface** of Prompting Press end-to-end, proving the library
works for an external consumer (spec 014, WU-C). Each app is independently
buildable and carries its own tests.

| Language | App | Depends on the library via | Test command |
|----------|-----|----------------------------|--------------|
| Rust | `rust/greeter-cli` | Cargo `path` dep on `../../../crates/prompting-press` | `cargo test -p prompting-press-greeter-cli` |
| Python | `python/greeter-cli` | `uv` editable/path dep on `packages/python` | `uv run pytest` (from the app dir) |
| TypeScript | `typescript/greeter-cli` | pnpm `workspace:*` dep on `packages/typescript` | `pnpm -C samples/typescript/greeter-cli test` |

Each app walks the same feature sequence: **construct → validate → render default →
render a named variant → provenance hashes → compose a 2-message prompt → `check()` →
the advisory guard → an error path → a printed hand-off stub** (the library calls no
provider — the "send to an LLM" step is always a printed placeholder, FR-018).

These apps are **not published** — they consume the library across the package
boundary and double as the published-package smoke test at launch.

## Launch-flip (post-publish, FR-019)

Pre-publish, each app depends on the in-repo library via a **local/workspace/path**
dependency so the apps build and their tests run in CI against the working tree now.
At v1 publish, the single post-publish step is to flip each app's manifest from the
local dependency to the **published-version constraint**:

- **Rust** — `samples/rust/greeter-cli/Cargo.toml`: `prompting-press = { path = "../../../crates/prompting-press" }` → `prompting-press = "0.1"`.
- **Python** — `samples/python/greeter-cli/pyproject.toml`: the editable/path dependency on `packages/python` → `prompting-press >= 0.1` (from PyPI).
- **TypeScript** — `samples/typescript/greeter-cli/package.json`: `"prompting-press": "workspace:*"` → `"prompting-press": "^0.1"` (from npm).

After the flip the apps build against the *published* packages, so a green
`samples:test` also proves the published artifacts are consumable. This flip is the
only step deferred until launch; everything else is testable pre-publish.
