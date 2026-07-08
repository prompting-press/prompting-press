"""Loader guide — loading JSON and TOML.

The loader is format-agnostic (returns raw text), so only the FileSystemLoader
``suffix`` and the ``from_*`` parser change. Uses the ``assistant.json`` /
``assistant.toml`` fixtures next to this program.
"""

from pathlib import Path

from prompting_press import Prompt
from prompting_press.loader import FileSystemLoader

_HERE = Path(__file__).parent


def main() -> None:
    # JSON: suffix ".json" -> loads {dir}/assistant.json, parsed with from_json.
    json_loader = FileSystemLoader(_HERE, suffix=".json")
    from_json = Prompt.from_json(json_loader.load("assistant"))
    assert from_json.name == "assistant"

    # TOML: suffix ".toml" -> loads {dir}/assistant.toml, parsed with from_toml.
    toml_loader = FileSystemLoader(_HERE, suffix=".toml")
    from_toml = Prompt.from_toml(toml_loader.load("assistant"))
    assert from_toml.name == "assistant"

    # Empty suffix -> the extension lives in the key instead (same file either way).
    bare_loader = FileSystemLoader(_HERE, suffix="")
    assert Prompt.from_json(bare_loader.load("assistant.json")).name == "assistant"


def test_loader_formats() -> None:
    main()


if __name__ == "__main__":
    main()
