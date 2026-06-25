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
#
# COMMENT STRIPPING (prevents false positives):
#   TOML files (.toml) carry explanatory comments that legitimately reference the
#   forbidden patterns (e.g. "# no floating "latest"/"^"/"~"/"*" per SEC-003").
#   These comments are stripped before scanning. JSON has no comment syntax.
#
# PORTABILITY NOTE:
#   Uses grep -E (ERE), not grep -P (PCRE). -P is unreliable on BSD/ugrep
#   environments when invoked from a non-interactive bash script context.
#   All required patterns are expressible as ERE; -E works on GNU grep (CI)
#   and ugrep (macOS local) alike.
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

  # Strip comments before scanning to avoid false positives from inline
  # documentation that references the forbidden patterns (SEC-003 explanations,
  # etc.). TOML comment syntax: '#' to end-of-line. JSON has no comments.
  #
  # The sed expression removes:
  #   - Full-line comments      (^#...)
  #   - Trailing inline comments (value  # comment)
  # It is safe here because no version value in any in-scope manifest contains
  # a literal '#' character.
  case "${manifest}" in
    *.toml)
      scan_content="$(sed -E 's/(^|[[:space:]])#.*$//' "${manifest}")"
      ;;
    *)
      scan_content="$(cat "${manifest}")"
      ;;
  esac

  # Write to a temp file so grep reads from a file descriptor, not a pipeline.
  # This sidesteps a portability trap: `printf '%s\n' "$var" | grep -E ...`
  # can behave differently from `grep -E ... <(printf '%s\n' "$var")` under
  # some bash+ugrep combinations (grep -P in particular exits 1 silently in
  # the pipeline context on macOS/ugrep even when a match exists). Writing to
  # a temp file and grepping the file is unambiguous on all targets.
  tmp_scan="$(mktemp)"
  # shellcheck disable=SC2064
  trap "rm -f '${tmp_scan}'" EXIT
  printf '%s\n' "${scan_content}" > "${tmp_scan}"

  # --- Pattern checks (grep -E, portable ERE) ---
  #
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
  #   >=x,<y     — bounded range (acceptable per SEC-003; maturin in pyproject.toml)
  #   ">=..."    — floor bound only (also acceptable)

  if grep -En '"[\^~][^"]*"' "${tmp_scan}" 2>/dev/null; then
    FAILED+=("${manifest}: caret or tilde range in JSON/TOML string")
  fi
  if grep -En '"latest"' "${tmp_scan}" 2>/dev/null; then
    FAILED+=("${manifest}: literal 'latest' version")
  fi
  if grep -En '"[*]"' "${tmp_scan}" 2>/dev/null; then
    FAILED+=("${manifest}: wildcard '*' version in JSON")
  fi
  # TOML: version = "*"
  if grep -En '=\s*"[*]"' "${tmp_scan}" 2>/dev/null; then
    FAILED+=("${manifest}: wildcard '*' version in TOML")
  fi
  # TOML: version = "^x" or version = "~x"
  # (also caught by the first pattern for JSON, but explicit for TOML context)
  if grep -En '=\s*"[\^~][^"]*"' "${tmp_scan}" 2>/dev/null; then
    FAILED+=("${manifest}: caret or tilde range in TOML assignment")
  fi

  rm -f "${tmp_scan}"
  trap - EXIT
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
