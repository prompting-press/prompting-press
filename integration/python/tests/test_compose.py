# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration: Composition of multiple prompts → messages with roles.

Covers: append / from_messages paths, order + roles, variant arg, empty composition,
no .chain() (FR-013).
"""

from __future__ import annotations

import pytest
from pydantic import BaseModel, field_validator

from prompting_press import (
    Composition,
    Message,
    Prompt,
    PromptRenderError,
    PromptValidationError,
)


class Named(BaseModel):
    name: str

    @field_validator("name")
    @classmethod
    def _non_empty(cls, v: str) -> str:
        if not v:
            raise ValueError("name must be non-empty")
        return v


class Empty(BaseModel):
    pass


SYS_PROMPT = Prompt(
    {"name": "sys", "role": "system", "body": "You are helpful.", "variables": {}}
)

GREET_PROMPT = Prompt(
    {
        "name": "greet",
        "role": "user",
        "body": "Hi {{ name }}",
        "variables": {"name": {"type": "string", "trusted": True}},
    }
)

FAREWELL_PROMPT = Prompt(
    {
        "name": "farewell",
        "role": "user",
        "body": "Bye {{ name }}",
        "variables": {"name": {"type": "string", "trusted": True}},
    }
)

VARIANT_PROMPT = Prompt(
    {
        "name": "salute",
        "role": "user",
        "body": "Hi {{ name }}",
        "variants": {"formal": {"body": "Good day, {{ name }}"}},
        "variables": {"name": {"type": "string", "trusted": True}},
    }
)


# ─── append path ──────────────────────────────────────────────────────────────


def test_append_resolves_in_order_with_correct_roles() -> None:
    comp = Composition()
    assert comp.append(SYS_PROMPT, Empty()) is None  # non-fluent
    assert comp.append(GREET_PROMPT, Named(name="Ada")) is None
    assert len(comp) == 2
    messages = comp.resolve()
    assert [type(m) for m in messages] == [Message, Message]
    assert messages[0].role == "system"
    assert messages[0].text == "You are helpful."
    assert messages[1].role == "user"
    assert messages[1].text == "Hi Ada"


# ─── from_messages path ───────────────────────────────────────────────────────


def test_from_messages_resolves_in_order() -> None:
    comp = Composition.from_messages(
        [(SYS_PROMPT, Empty()), (GREET_PROMPT, Named(name="Bo"))]
    )
    messages = comp.resolve()
    assert [m.role for m in messages] == ["system", "user"]
    assert [m.text for m in messages] == ["You are helpful.", "Hi Bo"]


def test_both_construction_paths_produce_identical_messages() -> None:
    entries = [(SYS_PROMPT, Empty()), (GREET_PROMPT, Named(name="Cy"))]
    via_append = Composition()
    for p, v in entries:
        via_append.append(p, v)
    via_factory = Composition.from_messages(entries)
    assert [(m.role, m.text) for m in via_append.resolve()] == [
        (m.role, m.text) for m in via_factory.resolve()
    ]


# ─── variants ─────────────────────────────────────────────────────────────────


def test_three_tuple_selects_named_variant() -> None:
    comp = Composition.from_messages([(VARIANT_PROMPT, Named(name="Di"), "formal")])
    assert comp.resolve()[0].text == "Good day, Di"


def test_two_tuple_defaults_to_default_arm() -> None:
    comp = Composition.from_messages([(VARIANT_PROMPT, Named(name="Eli"))])
    assert comp.resolve()[0].text == "Hi Eli"


def test_unknown_variant_fails_at_resolve() -> None:
    comp = Composition()
    comp.append(VARIANT_PROMPT, Named(name="Fa"), variant="nonexistent")
    with pytest.raises(PromptRenderError):
        comp.resolve()


# ─── invalid entry stores nothing ────────────────────────────────────────────


def test_invalid_vars_at_append_raises_and_stores_nothing() -> None:
    comp = Composition()
    comp.append(GREET_PROMPT, Named(name="ok"))
    assert len(comp) == 1
    with pytest.raises(PromptValidationError):
        comp.append(GREET_PROMPT, Named.model_construct(name=""))
    assert len(comp) == 1
    assert comp.resolve()[0].text == "Hi ok"


# ─── empty composition ────────────────────────────────────────────────────────


def test_empty_composition_resolves_to_empty_list() -> None:
    assert Composition().resolve() == []


# ─── no .chain() (FR-013) ─────────────────────────────────────────────────────


def test_no_chain_on_class_or_instance() -> None:
    assert not hasattr(Composition, "chain")
    assert not hasattr(Composition(), "chain")


# ─── mixed system + multiple user entries ─────────────────────────────────────


def test_mixed_system_and_two_user_entries() -> None:
    comp = Composition.from_messages(
        [
            (SYS_PROMPT, Empty()),
            (GREET_PROMPT, Named(name="Ada")),
            (FAREWELL_PROMPT, Named(name="Bo")),
        ]
    )
    messages = comp.resolve()
    assert [m.role for m in messages] == ["system", "user", "user"]
    assert [m.text for m in messages] == ["You are helpful.", "Hi Ada", "Bye Bo"]


# ─── Message is read-only ─────────────────────────────────────────────────────


def test_message_role_and_text_are_read_only() -> None:
    msg = Composition.from_messages([(GREET_PROMPT, Named(name="Gu"))]).resolve()[0]
    assert msg.role == "user"
    assert msg.text == "Hi Gu"
    with pytest.raises((AttributeError, TypeError)):
        msg.text = "tampered"  # type: ignore[misc]
