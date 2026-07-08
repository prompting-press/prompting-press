#!/usr/bin/env bash
# Lint + format gate — Python (ruff).
#
# Runs `ruff check` (lint) and `ruff format --check` (formatting) over the
# hand-written + generated Python sources. Cheap: pure static analysis, no build,
# no Rust extension. Mirrors the Rust (ci:check-fmt) and Node (biome via
# prompting-press-typescript:lint) lint gates.
#
# Tool: ruff, pinned via `uv run --with ruff==0.15.12` — the SAME exact version
#       the codegen dep-group pins (packages/python/pyproject.toml), so the lint
#       gate and the codegen formatter never disagree. Exact pin → invisible to
#       the floating-version gate (SEC-003).
#
# Scope: packages/python (the importable package + tests). The generated Pydantic
# models under python/prompting_press/generated/** are excluded by pyproject.toml
# [tool.ruff] exclude, which applies even when a path is passed explicitly
# (force-exclude = true). ruff reads pyproject.toml from the nearest parent, so
# running against packages/python picks up the config automatically.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

TARGET="packages/python"

echo "Lint gate (Python): ruff check + ruff format --check on ${TARGET} (generated/** excluded)..."
echo ""

uv run --with ruff==0.15.12 -- ruff check "${TARGET}"
uv run --with ruff==0.15.12 -- ruff format --check "${TARGET}"

echo ""
echo "Lint gate (Python) PASSED."
