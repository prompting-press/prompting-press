"""Loader guide — FileSystemLoader: map a key to a file in a base directory.

Uses the ``assistant.yaml`` fixture that lives next to this program.
"""

from pathlib import Path

from prompting_press import Prompt
from prompting_press.loader import FileSystemLoader

_HERE = Path(__file__).parent


def main() -> None:
    # Construct from an existing directory (canonicalized at construction time).
    loader = FileSystemLoader(_HERE)

    # "assistant" maps to {dir}/assistant.yaml (default suffix ".yaml").
    raw = loader.load("assistant")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "assistant"


if __name__ == "__main__":
    main()
