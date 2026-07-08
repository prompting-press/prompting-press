"""Integration: derive() — Replace default and MergeStrategy.MERGE.

Spec 017: derive(overlay) shallow-replaces (SC-002 default); derive(...,
strategy=MergeStrategy.MERGE) unions variables/variants/metadata (child-wins
on collision). Base is always immutable.
"""

from __future__ import annotations

import pytest
from pydantic import BaseModel

from prompting_press import (
    MergeStrategy,
    Prompt,
    PromptRenderError,
    PromptValidationError,
)

BASE_DEF = {
    "name": "analyst",
    "role": "user",
    "body": "{{ extraction }}",
    "variables": {
        "extraction": {"type": "string", "trusted": True},
    },
}

NAMED_DEF = {
    "name": "simple",
    "role": "user",
    "body": "Hi {{ name }}",
    "variables": {"name": {"type": "string", "trusted": True}},
}


class Named(BaseModel):
    name: str


class ExtVars(BaseModel):
    extraction: str


class MergedVars(BaseModel):
    extraction: str
    sentiment: str


# ─── MergeStrategy importable ─────────────────────────────────────────────────


def test_merge_strategy_has_replace_and_merge() -> None:
    assert hasattr(MergeStrategy, "REPLACE")
    assert hasattr(MergeStrategy, "MERGE")


# ─── default (REPLACE) — SC-002 ───────────────────────────────────────────────


def test_derive_default_replaces_body() -> None:
    p = Prompt(NAMED_DEF)
    derived = p.derive({"body": "Hey {{ name }}"})
    assert derived.body == "Hey {{ name }}"
    assert p.body == "Hi {{ name }}", "original must be untouched"


def test_derive_explicit_replace_equals_default() -> None:
    p = Prompt(BASE_DEF)
    overlay = {"body": "Hello {{ extraction }}!"}
    via_default = p.derive(overlay)
    via_replace = p.derive(overlay, strategy=MergeStrategy.REPLACE)
    assert via_default.body == via_replace.body == "Hello {{ extraction }}!"


def test_derive_can_rename_prompt() -> None:
    p = Prompt(NAMED_DEF)
    derived = p.derive({"name": "simple-renamed"})
    assert derived.name == "simple-renamed"
    assert p.name == "simple"


def test_derive_undeclared_variable_raises() -> None:
    p = Prompt(NAMED_DEF)
    with pytest.raises((PromptValidationError, PromptRenderError)):
        p.derive({"body": "{{ ghost }}"})


# ─── MERGE — unions variables ─────────────────────────────────────────────────


def test_derive_merge_unions_variables() -> None:
    base = Prompt(BASE_DEF)
    derived = base.derive(
        {
            "body": "{{ extraction }} | {{ sentiment }}",
            "variables": {"sentiment": {"type": "string", "trusted": True}},
        },
        strategy=MergeStrategy.MERGE,
    )
    keys = set(derived.variables.keys())
    assert "extraction" in keys
    assert "sentiment" in keys
    assert len(keys) == 2


def test_derive_merge_base_unchanged() -> None:
    base = Prompt(BASE_DEF)
    base.derive(
        {
            "body": "{{ extraction }} | {{ sentiment }}",
            "variables": {"sentiment": {"type": "string", "trusted": True}},
        },
        strategy=MergeStrategy.MERGE,
    )
    assert "sentiment" not in base.variables


def test_derive_merge_unions_variants() -> None:
    base = Prompt(
        {
            "name": "base",
            "role": "user",
            "body": "{{ name }}",
            "variables": {"name": {"type": "string", "trusted": True}},
            "variants": {"v1": {"body": "v1: {{ name }}"}},
        }
    )
    derived = base.derive(
        {"variants": {"v2": {"body": "v2: {{ name }}"}}},
        strategy=MergeStrategy.MERGE,
    )
    keys = set(derived.variants.keys())
    assert "v1" in keys
    assert "v2" in keys


def test_derive_merge_unions_metadata_child_wins_on_collision() -> None:
    base = Prompt(
        {
            "name": "base",
            "role": "user",
            "body": "{{ name }}",
            "variables": {"name": {"type": "string", "trusted": True}},
            "metadata": {"base_key": "base_val", "shared": "from_base"},
        }
    )
    derived = base.derive(
        {"metadata": {"overlay_key": "overlay_val", "shared": "from_overlay"}},
        strategy=MergeStrategy.MERGE,
    )
    meta = derived.metadata
    assert meta.get("base_key") == "base_val"
    assert meta.get("overlay_key") == "overlay_val"
    assert meta.get("shared") == "from_overlay"


# ─── immutability across all paths (SC-005) ───────────────────────────────────


def test_derive_any_strategy_leaves_base_immutable() -> None:
    base = Prompt(BASE_DEF)
    original_body = base.body
    original_vars = set(base.variables.keys())

    # Both strategies must leave base untouched
    base.derive({"body": "new {{ extraction }}"})
    base.derive(
        {
            "body": "{{ extraction }} | {{ sentiment }}",
            "variables": {"sentiment": {"type": "string", "trusted": True}},
        },
        strategy=MergeStrategy.MERGE,
    )

    assert base.body == original_body
    assert set(base.variables.keys()) == original_vars
