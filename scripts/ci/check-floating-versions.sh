#!/usr/bin/env bash
# T030a — Floating-version lint (SEC-003).
#
# Rejects floating version specifiers (^, ~, "latest", "*") in the manifests
# that govern the codegen/tool chain. The current tree is clean — this script
# acts as a regression guard.
#
# Scoped manifests (anything else, e.g. lockfiles, is EXCLUDED):
#   mise.toml
#   packages/*/package.json          (devDependencies — the codegen/napi tools)
#   packages/*/pyproject.toml        (build-system requires + dependency-groups)
#   crates/*/Cargo.toml              (crate manifests)
#   Cargo.toml                       (workspace manifest)
#
# IMPORTANT nuances:
#   - `maturin>=1.14,<2.0` in pyproject.toml is a BOUNDED range, NOT a floating
#     specifier (SEC-003 targets the shorthand floats). Do NOT flag `>=x,<y`.
#   - Lockfiles (pnpm-lock.yaml, uv.lock) are intentionally EXCLUDED.
#   - The check is for literal `^`, `~`, `"latest"`, and `"*"` as version values
#     (e.g. `"^1.0.0"`, `~1`, `= "latest"`, version = "*").
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

# Manifests to scan — explicit list, not a glob that could pull in lockfiles.
MANIFESTS=(
  "mise.toml"
  "Cargo.toml"
)

# Collect package.json files — exclude node_modules and pnpm-lock.yaml.
# Only scan the project-root package.json, not transitive dependency manifests.
while IFS= read -r f; do
  MANIFESTS+=("${f}")
done < <(find packages -maxdepth 2 -name "package.json" ! -path "*/node_modules/*" 2>/dev/null)

# Collect pyproject.toml files
while IFS= read -r f; do
  MANIFESTS+=("${f}")
done < <(find packages -name "pyproject.toml" 2>/dev/null)

# Collect Cargo.toml files (crates and workspace root already added above)
while IFS= read -r f; do
  MANIFESTS+=("${f}")
done < <(find crates -name "Cargo.toml" 2>/dev/null)

FAILED=()

for manifest in "${MANIFESTS[@]}"; do
  [[ -f "${manifest}" ]] || continue

  # Pattern: lines containing a floating version specifier.
  # Patterns to detect:
  #   "^..."     — npm caret range in JSON
  #   "~..."     — npm tilde range in JSON
  #   "latest"   — the literal string latest as a version value
  #   "*"        — wildcard version in JSON
  #   = "*"      — wildcard in TOML
  #   = "~"...   — tilde in TOML (e.g. version = "~1.0")
  #   = "^"...   — caret in TOML (e.g. version = "^1.0")
  #
  # Explicitly NOT flagged:
  #   >=x,<y     — bounded range (acceptable per SEC-003 comment in pyproject.toml)
  #   ">=..."    — floor bound only, but present in pyproject build-system requires
  #
  # We use grep with -P (PCRE) for precise matching.
  if grep -Pn '"[\^~][^"]*"' "${manifest}" 2>/dev/null; then
    FAILED+=("${manifest}: caret or tilde range in JSON string")
  fi
  if grep -Pn '"latest"' "${manifest}" 2>/dev/null; then
    FAILED+=("${manifest}: literal 'latest' version")
  fi
  if grep -Pn '"[*]"' "${manifest}" 2>/dev/null; then
    FAILED+=("${manifest}: wildcard '*' version in JSON")
  fi
  # TOML: version = "*"  or  version = "~x"  or  version = "^x"
  if grep -Pn '=\s*"[*]"' "${manifest}" 2>/dev/null; then
    FAILED+=("${manifest}: wildcard '*' version in TOML")
  fi
  if grep -Pn '=\s*"[\^~][^"]*"' "${manifest}" 2>/dev/null; then
    FAILED+=("${manifest}: caret or tilde range in TOML string")
  fi
done

if [[ ${#FAILED[@]} -gt 0 ]]; then
  echo ""
  echo "ERROR: Floating-version lint FAILED (SEC-003)."
  echo "The following manifests contain floating version specifiers:"
  for msg in "${FAILED[@]}"; do
    echo "  - ${msg}"
  done
  echo ""
  echo "Pin all versions explicitly. Floating specifiers (^, ~, 'latest', '*')"
  echo "are not allowed in codegen/tool manifests per SEC-003."
  echo ""
  exit 1
fi

echo "Floating-version lint PASSED — all manifests use pinned versions."
for m in "${MANIFESTS[@]}"; do
  [[ -f "${m}" ]] && echo "  OK: ${m}"
done
