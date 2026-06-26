#!/usr/bin/env bash
# T036 — Dependency-advisory CI gate (SEC-001).
#
# Runs `cargo deny check advisories` against the workspace Cargo.lock to detect
# known CVEs (RustSec advisory database). Also checks yanked crate versions.
#
# PRIMARY PURPOSE: catch known vulnerabilities in workspace dependencies before
# they land in a shipped release.
#
# Tool: cargo-deny (pinned in mise.toml under "cargo:cargo-deny").
# Config: deny.toml at the repository root.
#
# ROADMAP-Q3 NOTE — minijinja bump obligation:
#   Whenever minijinja is bumped in Cargo.toml, the bumping author MUST re-confirm
#   that Template::undeclared_variables (nested=false) remains a stable, public API
#   in the new release (roadmap Open Question Q3). This gate's owner also owns that
#   re-check. See deny.toml for the full procedure.
#
# MAINTAINER NOTES:
#   - cargo-deny fetches the RustSec advisory DB at runtime; the gate requires
#     outbound HTTPS to github.com (advisory-db). In air-gapped environments,
#     pre-fetch the DB and set CARGO_DENY_ADVISORIES_DB_PATH.
#   - To suppress a specific advisory (after security review), add an [advisories]
#     ignore entry in deny.toml with a comment explaining why.
#   - This gate does NOT require `cargo build`; it reads only Cargo.lock + deny.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

echo "Advisory gate: running cargo deny check advisories..."
echo "  Config: ${REPO_ROOT}/deny.toml"
echo "  Lockfile: ${REPO_ROOT}/Cargo.lock"
echo ""

# Run advisories check only (not licenses/bans — those have their own gates when
# the policy is defined). --manifest-path anchors to the workspace root.
cargo deny --manifest-path "${REPO_ROOT}/Cargo.toml" check advisories

echo ""
echo "Advisory gate PASSED — no known vulnerabilities in workspace dependencies."
