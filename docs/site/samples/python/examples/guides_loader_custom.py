# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Loader guide — custom loaders: a plain callable or a class implementing the PromptLoader protocol.

Any callable ``(key: str) -> str`` satisfies the loader contract — no class needed.
"""

from prompting_press import Prompt, PromptLoadError, make_prompt_load_error
from prompting_press.loader import LOAD_NOT_FOUND

GREET_YAML = """\
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
"""


def _source_loader(key: str) -> str:
    """A plain function works as a loader — just return the raw text or raise."""
    if key == "greet":
        return GREET_YAML
    raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`")


def main() -> None:
    # A plain callable — no struct or class required.
    raw = _source_loader("greet")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "greet"

    # A missing key raises PromptLoadError.
    try:
        _source_loader("missing")
        assert False, "should have raised"
    except PromptLoadError:
        pass


if __name__ == "__main__":
    main()
