# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Integration: render happy path, variants, and error cases.

Covers: render via (Schema, data={}), render via instance, named variant, missing
variant → PromptRenderError, undefined variable loud error.
"""

from __future__ import annotations

import re

import pytest
from pydantic import BaseModel, field_validator

from prompting_press import (
    Prompt,
    PromptRenderError,
    PromptValidationError,
    RenderResult,
)

HEX64 = re.compile(r"\A[0-9a-f]{64}\Z")


class Greeting(BaseModel):
    name: str
    count: int

    @field_validator("count")
    @classmethod
    def _non_negative(cls, v: int) -> int:
        if v < 0:
            raise ValueError("count must be non-negative")
        return v


class Named(BaseModel):
    name: str


class TopicVars(BaseModel):
    topic: str


GREET_DEF = {
    "name": "greet",
    "role": "user",
    "body": "Hi {{ name }}, you have {{ count }} messages",
    "variables": {
        "name": {"type": "string", "trusted": True},
        "count": {"type": "integer", "trusted": True},
    },
}

SIMPLE_DEF = {
    "name": "simple",
    "role": "user",
    "body": "Hi {{ name }}",
    "variables": {"name": {"type": "string", "trusted": True}},
}

UNTRUSTED_DEF = {
    "name": "ask",
    "role": "user",
    "body": "Tell me about {{ topic }}.",
    "variables": {"topic": {"type": "string", "trusted": False}},
}


# ─── happy path — schema + data= ──────────────────────────────────────────────


def test_render_schema_and_data_returns_render_result() -> None:
    p = Prompt(GREET_DEF)
    result = p.render(Greeting, data={"name": "Ada", "count": 3})
    assert isinstance(result, RenderResult)
    assert result.text == "Hi Ada, you have 3 messages"
    assert result.name == "greet"
    assert result.variant == "default"
    assert HEX64.match(result.template_hash)
    assert HEX64.match(result.render_hash)
    assert result.guard is None


# ─── happy path — pre-constructed instance ────────────────────────────────────


def test_render_instance_path() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Bo"))
    assert result.text == "Hi Bo"
    assert result.variant == "default"
    assert HEX64.match(result.template_hash)


# ─── named variant ────────────────────────────────────────────────────────────


def test_render_named_variant() -> None:
    p = Prompt(
        {
            "name": "salute",
            "role": "user",
            "body": "Hi {{ name }}",
            "variants": {"formal": {"body": "Good day, {{ name }}."}},
            "variables": {"name": {"type": "string", "trusted": True}},
        }
    )
    result = p.render(Named(name="Di"), variant="formal")
    assert result.text == "Good day, Di."
    assert result.variant == "formal"


# ─── missing variant → PromptRenderError ─────────────────────────────────────


def test_render_missing_variant_raises_prompt_render_error() -> None:
    p = Prompt(SIMPLE_DEF)
    with pytest.raises(PromptRenderError) as excinfo:
        p.render(Named(name="Ada"), variant="nonexistent")
    assert any(r.code == "unknown_variant" for r in excinfo.value.errors)


# ─── validation failure before kernel ────────────────────────────────────────


def test_render_validation_failure_raises_prompt_validation_error() -> None:
    p = Prompt(GREET_DEF)
    with pytest.raises(PromptValidationError) as excinfo:
        p.render(Greeting, data={"name": "Ada", "count": -1})
    rows = excinfo.value.errors
    assert any(r.field == "count" for r in rows)
    assert all(r.code == "validation" for r in rows if r.field == "count")


# ─── get_source returns raw template ─────────────────────────────────────────


def test_get_source_returns_uninterpolated_template() -> None:
    p = Prompt(SIMPLE_DEF)
    src = p.get_source()
    assert src == "Hi {{ name }}"
    assert "{{" in src


def test_get_source_named_variant() -> None:
    p = Prompt(
        {
            "name": "s",
            "role": "user",
            "body": "root {{ x }}",
            "variants": {"v": {"body": "variant {{ x }}"}},
            "variables": {"x": {"type": "string", "trusted": True}},
        }
    )
    assert p.get_source(variant="v") == "variant {{ x }}"
    assert p.get_source() == "root {{ x }}"


def test_get_source_unknown_variant_raises() -> None:
    p = Prompt(SIMPLE_DEF)
    with pytest.raises(PromptRenderError) as excinfo:
        p.get_source(variant="nope")
    assert any(r.code == "unknown_variant" for r in excinfo.value.errors)


# ─── guard does not affect guard-off render ───────────────────────────────────


def test_render_guard_off_body_is_unmodified() -> None:
    p = Prompt(UNTRUSTED_DEF)
    result = p.render(TopicVars(topic="rivers"))
    assert result.text == "Tell me about rivers."
    assert result.guard is None
