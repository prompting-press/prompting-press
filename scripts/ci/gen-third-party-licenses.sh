#!/usr/bin/env bash
# Third-party license attribution GENERATOR (Apache-2.0 release compliance).
#
# Generates the bundled THIRD-PARTY-LICENSES.md files for the two native
# artifacts that statically link the Rust dependency graph:
#
#   packages/python/THIRD-PARTY-LICENSES.md      <- prompting-press-py    graph
#   packages/typescript/THIRD-PARTY-LICENSES.md  <- prompting-press-node  graph
#
# These files reproduce the upstream copyright + license notices that MIT / BSD /
# ISC / Apache-2.0 require to be preserved in BINARY distributions (the wheel and
# the .node addon bundle the compiled Rust code). This script WRITES the files at
# package-build time in CI (invoked by build-wheels, build-sdist, and publish-npm
# in release.yml before the respective packaging/publish step). The files are NOT
# committed to the repository (issue #16).
#
# Tool: cargo-about (pinned in mise.toml under "github:EmbarkStudios/cargo-about").
# Config: about.toml at the repo root; template: ci/about.hbs.
#         about.toml's `accepted` list MUST match deny.toml's [licenses].allow.
#
# MAINTAINER NOTES:
#   - cargo-about harvests license TEXT from each crate's local source (the
#     Cargo registry cache); it optionally queries clearlydefined.io to fill
#     gaps. Our graph resolves fully from local sources, so the clearlydefined
#     WARN lines are harmless and the output is deterministic offline.
#   - After a change to Cargo.lock, about.toml, or ci/about.hbs, the next CI
#     build will automatically pick up the change — no manual regeneration needed.
#   - A NEW bundled crate under a license absent from about.toml's `accepted`
#     will surface here (and fail ci:check-licenses first); triage per deny.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

CONFIG="${REPO_ROOT}/about.toml"
TEMPLATE="${REPO_ROOT}/ci/about.hbs"

# Resolve the cargo-about BINARY. cargo-about is installed via the `github:` backend
# (see mise.toml) — a prebuilt release binary that mise puts on PATH like any
# pinned tool, so `cargo-about` resolves directly. `mise which` gives the concrete
# path (the moon task runs under `mise exec`, so mise + the tool dirs are on PATH);
# fall back to a plain PATH lookup for non-mise environments.
CARGO_ABOUT="$(mise which cargo-about 2>/dev/null || command -v cargo-about || true)"
if [ -z "${CARGO_ABOUT}" ] || ! "${CARGO_ABOUT}" --version >/dev/null 2>&1; then
  echo "ERROR: cargo-about not found. Install it: mise install (it is pinned as" >&2
  echo "'github:EmbarkStudios/cargo-about' in mise.toml)." >&2
  exit 1
fi

# This workspace's own repository URL — the marker for first-party crates that
# strip-first-party-licenses.py removes from the attribution (see generate()).
FIRST_PARTY_REPO="https://github.com/prompting-press/prompting-press"

# artifact-crate:output-path pairs — the two bundled bindings.
generate() {
  local crate="$1" out="$2"
  echo "  ${crate} -> ${out}"
  # --locked: resolve the dependency graph strictly from Cargo.lock. WITHOUT it,
  # cargo-about re-resolves versions and picks the NEWEST semver-compatible release
  # (the crate specs are caret by default, e.g. napi = "3.9.4" allows 3.10.0), so a
  # runner whose registry cache has newer patch releases than the committed lock
  # produces a DIFFERENT attribution → the freshness diff fails only on CI. --locked
  # pins to the lock (napi 3.9.4 etc.) so output matches everywhere.
  # --offline: crawl ONLY local crate sources — no clearlydefined.io network lookups
  # (deterministic; requires the cargo cache, which CI populates via `cargo fetch`).
  "${CARGO_ABOUT}" generate \
    --locked \
    --offline \
    -c "${CONFIG}" \
    "${TEMPLATE}" \
    --manifest-path "${REPO_ROOT}/crates/${crate}/Cargo.toml" \
    -o "${REPO_ROOT}/${out}"
  # Normalize the trailing newlines to EXACTLY ONE, matching what the repo's
  # end-of-file-fixer pre-commit hook enforces. cargo-about's Handlebars output
  # ends with a blank line ("...\n\n"); the hook collapses that to a single "\n".
  # If gen leaves the doubled newline, the committed (hook-normalized) file and a
  # fresh regen differ by one blank line (a historical artefact from when the file
  # was committed). Strip all trailing newlines, then re-add one.
  # perl -0777 slurps the whole file; s/\n+\z/\n/ replaces the final run of
  # newlines with a single one (portable, no in-place-sed newline quirks).
  local abs="${REPO_ROOT}/${out}"
  perl -0777 -i -pe 's/\n+\z/\n/' "${abs}"
  # Drop this workspace's OWN crates from the attribution. cargo-about lists every
  # crate in the bundled graph, including prompting-press{,-core,-py,-node} — but
  # those are first-party code already covered by the repo root LICENSE + NOTICE
  # (Apache-2.0), and they carry the only version-bearing lines, which made the
  # file churn every release. cargo-about cannot exclude a PUBLISHED workspace
  # member (`private.ignore` only drops unpublished ones), so post-process.
  python3 "${SCRIPT_DIR}/strip-first-party-licenses.py" "${abs}" "${FIRST_PARTY_REPO}"
}

echo "Generating third-party license attribution (cargo-about)..."
generate "prompting-press-py"   "packages/python/THIRD-PARTY-LICENSES.md"
generate "prompting-press-node" "packages/typescript/THIRD-PARTY-LICENSES.md"

echo ""
echo "Third-party license files regenerated."
