# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Loader guide — a missing key raises PromptLoadError (load_not_found),
distinct from LoadError (the parse/shape error raised on malformed YAML).

``except PromptLoadError`` does NOT catch a malformed-YAML ``LoadError``.
"""

from prompting_press import LoadError, Prompt, PromptLoadError
from prompting_press.loader import LOAD_NOT_FOUND, MemoryLoader


def main() -> None:
    loader = MemoryLoader({})

    # A missing key raises PromptLoadError — not a parse error.
    try:
        loader.load("missing")
        assert False, "should have raised"
    except PromptLoadError as exc:
        assert exc.errors[0].code == LOAD_NOT_FOUND

    # PromptLoadError is distinct from LoadError.
    # Parsing bad YAML raises LoadError — a different type on a different path.
    try:
        Prompt.from_yaml("not: valid: yaml: [")
        assert False, "should have raised"
    except LoadError:
        pass


if __name__ == "__main__":
    main()
