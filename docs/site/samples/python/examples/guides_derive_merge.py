# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Derive guide — MergeStrategy.MERGE: union the base's declared variables with the
overlay's, so a child prompt inherits ``company`` + ``max_words`` and adds its own ``tone``
without hand-spreading the base's variables. The base is untouched.

Standalone — the docs page displays this file verbatim; run it directly to check.
"""

from pathlib import Path

from prompting_press import MergeStrategy, Prompt

# The caller reads the definition; the library does no file I/O itself.
# Resolve the file next to this program (a real app uses its own path).
_HERE = Path(__file__).parent


def main() -> None:
    base = Prompt.from_yaml((_HERE / "assistant.yaml").read_text())

    # MERGE unions the map-typed fields (variables/variants/metadata) at their top-level
    # keys — child-wins on collision. The base's `company` + `max_words` survive; the
    # overlay only needs to declare what it adds.
    child = base.derive(
        {
            "body": "You are a {{ tone }} assistant for {{ company }}. "
            "Keep replies under {{ max_words }} words.",
            "variables": {"tone": {"type": "string", "trusted": True}},
        },
        strategy=MergeStrategy.MERGE,
    )

    # child inherited the base's two variables and gained its own — three in total.
    assert set(child.variables) == {"company", "max_words", "tone"}
    # base is untouched: no `tone` leaked back onto it.
    assert "tone" not in base.variables


if __name__ == "__main__":
    main()
