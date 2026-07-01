#!/usr/bin/env bash
# Third-party license attribution GENERATOR (Apache-2.0 release compliance).
#
# Regenerates the bundled THIRD-PARTY-LICENSES.md files for the two native
# artifacts that statically link the Rust dependency graph:
#
#   packages/python/THIRD-PARTY-LICENSES.md      <- prompting-press-py    graph
#   packages/typescript/THIRD-PARTY-LICENSES.md  <- prompting-press-node  graph
#
# These files reproduce the upstream copyright + license notices that MIT / BSD /
# ISC / Apache-2.0 require to be preserved in BINARY distributions (the wheel and
# the .node addon bundle the compiled Rust code). This script WRITES the files;
# the ci:check-third-party-licenses gate runs it and asserts `git diff` is clean.
#
# Tool: cargo-about (pinned in mise.toml under "cargo:cargo-about").
# Config: about.toml at the repo root; template: ci/about.hbs.
#         about.toml's `accepted` list MUST match deny.toml's [licenses].allow.
#
# MAINTAINER NOTES:
#   - cargo-about harvests license TEXT from each crate's local source (the
#     Cargo registry cache); it optionally queries clearlydefined.io to fill
#     gaps. Our graph resolves fully from local sources, so the clearlydefined
#     WARN lines are harmless and the output is deterministic offline.
#   - Regenerate after ANY change to Cargo.lock, about.toml, or ci/about.hbs,
#     then commit the updated THIRD-PARTY-LICENSES.md files.
#   - A NEW bundled crate under a license absent from about.toml's `accepted`
#     will surface here (and fail ci:check-licenses first); triage per deny.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

CONFIG="${REPO_ROOT}/about.toml"
TEMPLATE="${REPO_ROOT}/ci/about.hbs"

# Resolve the cargo-about BINARY (absolute path). This tool is uniquely painful to
# get onto PATH in CI:
#   - `cargo about` (subcommand form) needs cargo-about on PATH, but mise installs
#     it into its own tool dir (NOT ~/.cargo/bin), and moon does not propagate
#     mise's tool PATH into task subprocesses → bare `cargo about` fails.
#   - `mise which` only QUERIES; and the runner's mise cache can hold a poisoned
#     "installed-but-empty" cargo-about entry (a leftover from the earlier
#     features=["cli"] source-build that produced no binary), so `mise install`
#     no-ops against it and `mise which` returns a path to nothing.
# So: try mise/PATH first, and if that yields no WORKING binary, self-heal by
# binstalling the prebuilt release binary into ~/.cargo/bin (fast, no source
# build; ~/.cargo/bin is on PATH like cargo-deny). Version-pinned to match
# mise.toml. This resolves under moon, mise exec, and standalone.
CARGO_ABOUT_VERSION="0.9.0"
resolve_cargo_about() {
  local c
  c="$(mise which cargo-about 2>/dev/null || true)"
  [ -n "${c}" ] && [ -x "${c}" ] && { echo "${c}"; return 0; }
  c="$(command -v cargo-about 2>/dev/null || true)"
  [ -n "${c}" ] && { echo "${c}"; return 0; }
  return 1
}
CARGO_ABOUT="$(resolve_cargo_about || true)"
if [ -z "${CARGO_ABOUT}" ] || ! "${CARGO_ABOUT}" --version >/dev/null 2>&1; then
  echo "cargo-about not resolvable; binstalling prebuilt ${CARGO_ABOUT_VERSION} into ~/.cargo/bin..." >&2
  if command -v cargo-binstall >/dev/null 2>&1; then
    cargo-binstall -y "cargo-about@${CARGO_ABOUT_VERSION}" >&2
  elif command -v mise >/dev/null 2>&1; then
    mise exec -- cargo-binstall -y "cargo-about@${CARGO_ABOUT_VERSION}" >&2
  fi
  CARGO_ABOUT="$(command -v cargo-about 2>/dev/null || resolve_cargo_about || true)"
fi
if [ -z "${CARGO_ABOUT}" ] || ! "${CARGO_ABOUT}" --version >/dev/null 2>&1; then
  echo "ERROR: cargo-about not found and could not be installed." >&2
  echo "Install it: cargo-binstall cargo-about@${CARGO_ABOUT_VERSION}  (or mise install 'cargo:cargo-about')" >&2
  exit 1
fi

# artifact-crate:output-path pairs — the two bundled bindings.
generate() {
  local crate="$1" out="$2"
  echo "  ${crate} -> ${out}"
  # --offline: crawl ONLY local crate sources for license info — no clearlydefined.io
  # lookups. This makes the output DETERMINISTIC (network state can't change it), so
  # the ci:check-third-party-licenses freshness diff is stable across machines/CI.
  # Requires crate sources in the cargo cache; CI runs `cargo fetch --locked` first.
  "${CARGO_ABOUT}" generate \
    --offline \
    -c "${CONFIG}" \
    "${TEMPLATE}" \
    --manifest-path "${REPO_ROOT}/crates/${crate}/Cargo.toml" \
    -o "${REPO_ROOT}/${out}"
  # Normalize the trailing newlines to EXACTLY ONE, matching what the repo's
  # end-of-file-fixer pre-commit hook enforces. cargo-about's Handlebars output
  # ends with a blank line ("...\n\n"); the hook collapses that to a single "\n".
  # If gen leaves the doubled newline, the committed (hook-normalized) file and a
  # fresh regen differ by one blank line and the ci:check-third-party-licenses
  # freshness diff fails forever. Strip all trailing newlines, then re-add one.
  # perl -0777 slurps the whole file; s/\n+\z/\n/ replaces the final run of
  # newlines with a single one (portable, no in-place-sed newline quirks).
  local abs="${REPO_ROOT}/${out}"
  perl -0777 -i -pe 's/\n+\z/\n/' "${abs}"
}

echo "Generating third-party license attribution (cargo-about)..."
generate "prompting-press-py"   "packages/python/THIRD-PARTY-LICENSES.md"
generate "prompting-press-node" "packages/typescript/THIRD-PARTY-LICENSES.md"

echo ""
echo "Third-party license files regenerated."
