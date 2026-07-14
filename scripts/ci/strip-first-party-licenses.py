#!/usr/bin/env python3

# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Strip first-party crates from a cargo-about THIRD-PARTY-LICENSES.md.

cargo-about lists EVERY crate in the bundled dependency graph, including this
workspace's own crates (prompting-press, -core, -py, -node). Those are
first-party code already covered by the repo root LICENSE + NOTICE (Apache-2.0)
— listing them under "Third-Party Licenses" is wrong, and they carry the only
version-bearing lines in the file, so they make it churn on every release.

cargo-about has no native "exclude a PUBLISHED workspace crate from the output"
option (`private.ignore` only drops UNpublished members; ours publish to
crates.io), so we post-process: remove each first-party bullet line and
decrement the matching license's count in the overview header, dropping a
license from the overview entirely if its count reaches zero.

A first-party crate is identified by its repository URL (the bullet renders as
`- **<name> <version>** — <<repo-url>>`). The repo URL is passed in so the rule
is explicit and not hard-coded to crate names.

Usage: strip-first-party-licenses.py <file> <first-party-repo-url>
Idempotent: running twice is a no-op (the bullets are already gone).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def strip(text: str, repo_url: str) -> str:
    lines = text.splitlines(keepends=True)

    # 1. Walk the body, tracking the current "## <license>" section. Drop every
    #    bullet whose repository URL is the first-party repo, tallying how many
    #    were removed per license-section name.
    removed_per_license: dict[str, int] = {}
    current_license: str | None = None
    bullet_suffix = f"— <{repo_url}>"
    kept: list[str] = []
    for line in lines:
        header = re.match(r"^## (.+?)\s*$", line)
        if header:
            current_license = header.group(1)
            kept.append(line)
            continue
        if line.lstrip().startswith("- **") and line.rstrip().endswith(bullet_suffix):
            if current_license is not None:
                removed_per_license[current_license] = (
                    removed_per_license.get(current_license, 0) + 1
                )
            continue  # drop this first-party bullet
        kept.append(line)

    if not removed_per_license:
        return text  # nothing first-party found — idempotent no-op

    body = "".join(kept)

    # 2. Fix the overview header: "Licenses in this artifact: <Name> (<n>), ...".
    #    Decrement each affected license by the count removed; drop it if zero.
    overview_re = re.compile(r"^(Licenses in this artifact: )(.+?)\.\s*$", re.MULTILINE)

    def fix_overview(m: re.Match[str]) -> str:
        prefix, listing = m.group(1), m.group(2)
        out: list[str] = []
        for entry in listing.split(", "):
            em = re.match(r"^(.*) \((\d+)\)$", entry)
            if not em:
                out.append(entry)
                continue
            name, count = em.group(1), int(em.group(2))
            count -= removed_per_license.get(name, 0)
            if count > 0:
                out.append(f"{name} ({count})")
        return f"{prefix}{', '.join(out)}."

    return overview_re.sub(fix_overview, body)


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    path, repo_url = Path(sys.argv[1]), sys.argv[2]
    original = path.read_text(encoding="utf-8")
    stripped = strip(original, repo_url)
    path.write_text(stripped, encoding="utf-8")
    removed = original.count(f"— <{repo_url}>") - stripped.count(f"— <{repo_url}>")
    print(f"  {path}: removed {removed} first-party bullet(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
