# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Loader guide — MemoryLoader: load raw text by key, then construct a Prompt.

The kernel stays I/O-free; the loader is a separate, caller-invoked I/O leaf.
"""

from prompting_press import Prompt
from prompting_press.loader import MemoryLoader

GREET_YAML = """\
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
"""


def main() -> None:
    loader = MemoryLoader({"greet": GREET_YAML})

    # load() returns raw text — parsing is a separate step.
    raw = loader.load("greet")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "greet"


if __name__ == "__main__":
    main()
