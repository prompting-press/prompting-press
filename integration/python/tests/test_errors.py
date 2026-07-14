# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Integration: PromptingPressError hierarchy and .errors shape.

Covers: PromptingPressError is the base; PromptValidationError, PromptRenderError,
LoadError, PromptLoadError are all subtypes; .errors is [{field, code, message}].
"""

from __future__ import annotations

import pytest
from pydantic import BaseModel, field_validator

from prompting_press import (
    FieldError,
    LoadError,
    Prompt,
    PromptLoadError,
    PromptRenderError,
    PromptValidationError,
    PromptingPressError,
    make_prompt_load_error,
)
from prompting_press.loader import LOAD_NOT_FOUND


class Greeting(BaseModel):
    name: str
    count: int

    @field_validator("count")
    @classmethod
    def _non_negative(cls, v: int) -> int:
        if v < 0:
            raise ValueError("count must be non-negative")
        return v


GREET_DEF = {
    "name": "greet",
    "role": "user",
    "body": "Hi {{ name }}, you have {{ count }} messages",
    "variables": {
        "name": {"type": "string", "trusted": True},
        "count": {"type": "integer", "trusted": True},
    },
}


# ─── hierarchy ────────────────────────────────────────────────────────────────


def test_prompt_validation_error_is_prompting_press_error() -> None:
    assert issubclass(PromptValidationError, PromptingPressError)


def test_prompt_render_error_is_prompting_press_error() -> None:
    assert issubclass(PromptRenderError, PromptingPressError)


def test_load_error_is_prompting_press_error() -> None:
    assert issubclass(LoadError, PromptingPressError)


def test_prompt_load_error_is_prompting_press_error() -> None:
    assert issubclass(PromptLoadError, PromptingPressError)


# ─── .errors is [{field, code, message}] ─────────────────────────────────────


def test_prompt_validation_error_has_structured_errors() -> None:
    p = Prompt(GREET_DEF)
    with pytest.raises(PromptValidationError) as excinfo:
        p.render(Greeting, data={"name": "Ada", "count": -1})
    rows = excinfo.value.errors
    assert rows, "must carry at least one row"
    for row in rows:
        assert isinstance(row, FieldError)
        assert isinstance(row.field, str)
        assert isinstance(row.code, str)
        assert isinstance(row.message, str)


def test_prompt_render_error_has_structured_errors() -> None:
    p = Prompt(
        {
            "name": "simple",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": {"name": {"type": "string", "trusted": True}},
        }
    )

    class Named(BaseModel):
        name: str

    with pytest.raises(PromptRenderError) as excinfo:
        p.render(Named(name="Ada"), variant="nonexistent")
    rows = excinfo.value.errors
    assert rows
    for row in rows:
        assert isinstance(row, FieldError)
        assert isinstance(row.code, str)


def test_load_error_has_structured_errors() -> None:
    with pytest.raises(LoadError) as excinfo:
        Prompt.from_yaml("name: [unterminated")
    rows = excinfo.value.errors
    assert rows
    for row in rows:
        assert isinstance(row, FieldError)
        assert isinstance(row.code, str)


def test_prompt_load_error_has_structured_errors() -> None:
    err = make_prompt_load_error(LOAD_NOT_FOUND, "key not found: greet")
    rows = err.errors
    assert rows
    assert rows[0].code == LOAD_NOT_FOUND
    assert isinstance(rows[0].message, str)


# ─── PromptLoadError is distinct from LoadError ───────────────────────────────


def test_prompt_load_error_is_not_load_error() -> None:
    assert not issubclass(PromptLoadError, LoadError)
    assert not issubclass(LoadError, PromptLoadError)


# ─── catching the base catches all subtypes ───────────────────────────────────


def test_base_exception_catches_validation_error() -> None:
    p = Prompt(GREET_DEF)
    with pytest.raises(PromptingPressError):
        p.render(Greeting, data={"name": "Ada", "count": -1})


def test_base_exception_catches_load_error() -> None:
    with pytest.raises(PromptingPressError):
        Prompt.from_yaml("name: [unterminated")
