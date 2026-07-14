# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Integration: Prompt construction — from_yaml / from_json / from_toml / Prompt(shape).

Exercises FR-001 to FR-008: all four construction entry points, valid and invalid inputs,
LoadError on parse/shape failures, and the `trusted` boolean field (spec 015).
"""

from __future__ import annotations

import pytest
from pydantic import BaseModel

from prompting_press import (
    LoadError,
    Prompt,
    PromptingPressError,
    PromptRenderError,
    PromptValidationError,
)
from prompting_press.generated import PromptDefinition

# Minimal known-good prompt — referenced throughout this harness.
MINIMAL_YAML = """\
name: base
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
"""

MINIMAL_DICT = {
    "name": "base",
    "role": "user",
    "body": "Hi {{ name }}",
    "variables": {"name": {"type": "string", "trusted": True}},
}

MINIMAL_JSON = (
    '{"name":"base","role":"user","body":"Hi {{ name }}",'
    '"variables":{"name":{"type":"string","trusted":true}}}'
)

MINIMAL_TOML = """\
name = "base"
role = "user"
body = "Hi {{ name }}"

[variables.name]
type = "string"
trusted = true
"""


class Named(BaseModel):
    name: str


# ─── from_yaml ────────────────────────────────────────────────────────────────


def test_from_yaml_constructs() -> None:
    p = Prompt.from_yaml(MINIMAL_YAML)
    assert p.name == "base"
    assert p.role == "user"
    assert p.body == "Hi {{ name }}"


def test_from_yaml_malformed_raises_load_error() -> None:
    with pytest.raises(LoadError):
        Prompt.from_yaml("name: [unterminated")


def test_from_yaml_missing_required_field_raises_load_error() -> None:
    # `body` is required
    with pytest.raises(LoadError):
        Prompt.from_yaml("name: hi\nrole: user\n")


# ─── from_json ────────────────────────────────────────────────────────────────


def test_from_json_constructs() -> None:
    p = Prompt.from_json(MINIMAL_JSON)
    assert p.name == "base"
    assert p.body == "Hi {{ name }}"


def test_from_json_malformed_raises_load_error() -> None:
    with pytest.raises(LoadError):
        Prompt.from_json("{ not valid json ")


def test_from_json_missing_required_field_raises_load_error() -> None:
    with pytest.raises(LoadError):
        Prompt.from_json('{"name":"hi","role":"user"}')


# ─── from_toml ────────────────────────────────────────────────────────────────


def test_from_toml_constructs() -> None:
    p = Prompt.from_toml(MINIMAL_TOML)
    assert p.name == "base"
    assert p.body == "Hi {{ name }}"


def test_from_toml_malformed_raises_load_error() -> None:
    with pytest.raises(LoadError):
        Prompt.from_toml("name = [unterminated")


# ─── Prompt(dict) / Prompt(PromptDefinition) ──────────────────────────────────


def test_prompt_from_dict_constructs() -> None:
    p = Prompt(MINIMAL_DICT)
    assert p.name == "base"
    assert p.role == "user"
    assert isinstance(p.variables, dict)
    assert "name" in p.variables


def test_prompt_from_pydantic_model_constructs() -> None:
    shape = PromptDefinition.model_validate(MINIMAL_DICT)
    p = Prompt(shape)
    assert p.name == "base"
    assert p.body == "Hi {{ name }}"


def test_prompt_dict_missing_body_raises() -> None:
    with pytest.raises((LoadError, PromptingPressError)):
        Prompt({"name": "hi", "role": "user"})


# ─── Agreement check at construction ─────────────────────────────────────────


def test_undeclared_variable_raises_at_construction() -> None:
    # `ghost` referenced in body but not in variables
    bad = {
        "name": "bad",
        "role": "user",
        "body": "{{ ghost }}",
        "variables": {"name": {"type": "string", "trusted": True}},
    }
    with pytest.raises(PromptingPressError) as excinfo:
        Prompt(bad)
    assert excinfo.value.errors


def test_reserved_variant_name_raises_at_construction() -> None:
    bad = {
        "name": "bad",
        "role": "user",
        "body": "Hi",
        "variables": {},
        "variants": {"default": {"body": "shadowed"}},
    }
    with pytest.raises((PromptValidationError, PromptRenderError)):
        Prompt(bad)


def test_template_syntax_error_raises_at_construction() -> None:
    bad = {"name": "bad", "role": "user", "body": "{{ unclosed", "variables": {}}
    with pytest.raises((PromptValidationError, PromptRenderError)):
        Prompt(bad)


# ─── trusted field (spec 015) ─────────────────────────────────────────────────


def test_trusted_boolean_accepted() -> None:
    p = Prompt(MINIMAL_DICT)
    assert p.name == "base"


def test_origin_field_rejected_by_serde() -> None:
    # pre-spec-015 `origin` enum field → must fail with deny_unknown_fields
    with pytest.raises(LoadError):
        Prompt.from_json(
            '{"name":"bad","role":"user","body":"Hi {{ x }}",'
            '"variables":{"x":{"type":"string","origin":"trusted"}}}'
        )


# ─── Properties are read-only ─────────────────────────────────────────────────


def test_prompt_properties_read_only() -> None:
    p = Prompt(MINIMAL_DICT)
    with pytest.raises(AttributeError):
        p.name = "tampered"  # type: ignore[misc]
    with pytest.raises(AttributeError):
        p.body = "tampered"  # type: ignore[misc]
