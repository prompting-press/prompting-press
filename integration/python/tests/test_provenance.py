# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration: result.provenance_attributes() → dict with exactly 4 keys.

Principle V: provenance carries template_hash + render_hash per variant, plus name
and variant. provenance_attributes() returns a dict[str, str] with exactly 4
prompting_press.prompt.* keys whose values match result fields.
"""

from __future__ import annotations

import re

from pydantic import BaseModel

from prompting_press import GuardConfig, Prompt

HEX64 = re.compile(r"\A[0-9a-f]{64}\Z")

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


class Named(BaseModel):
    name: str


class TopicVars(BaseModel):
    topic: str


EXPECTED_KEYS = {
    "prompting_press.prompt.name",
    "prompting_press.prompt.variant",
    "prompting_press.prompt.template_hash",
    "prompting_press.prompt.render_hash",
}


def test_provenance_attributes_returns_dict() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert isinstance(attrs, dict)


def test_provenance_attributes_has_exactly_four_keys() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert set(attrs.keys()) == EXPECTED_KEYS


def test_provenance_attributes_values_are_strings() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert all(isinstance(v, str) for v in attrs.values())


def test_provenance_attributes_name_matches_result() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert attrs["prompting_press.prompt.name"] == result.name


def test_provenance_attributes_variant_matches_result() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert attrs["prompting_press.prompt.variant"] == result.variant


def test_provenance_attributes_template_hash_matches_result() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert attrs["prompting_press.prompt.template_hash"] == result.template_hash
    assert HEX64.match(attrs["prompting_press.prompt.template_hash"])


def test_provenance_attributes_render_hash_matches_result() -> None:
    p = Prompt(SIMPLE_DEF)
    result = p.render(Named(name="Ada"))
    attrs = result.provenance_attributes()
    assert attrs["prompting_press.prompt.render_hash"] == result.render_hash
    assert HEX64.match(attrs["prompting_press.prompt.render_hash"])


def test_provenance_attributes_guard_off_vs_on_render_hash_differ() -> None:
    # Spec 015: render_hash covers the rendered output, which differs guard-on vs off.
    p = Prompt(UNTRUSTED_DEF)
    plain = p.render(TopicVars(topic="rivers"))
    guarded = p.render(TopicVars(topic="rivers"), guard=GuardConfig(enabled=True))
    plain_attrs = plain.provenance_attributes()
    guarded_attrs = guarded.provenance_attributes()
    assert (
        plain_attrs["prompting_press.prompt.render_hash"]
        != guarded_attrs["prompting_press.prompt.render_hash"]
    )
    # template_hash is unaffected by the guard (template source unchanged)
    assert (
        plain_attrs["prompting_press.prompt.template_hash"]
        == guarded_attrs["prompting_press.prompt.template_hash"]
    )
