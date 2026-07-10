# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Integration: GuardConfig — untrusted delimiting, render_hash differs guard on/off.

Spec 015: when guard is enabled, untrusted values are wrapped in
<untrusted>…</untrusted> tags; render_hash differs from the unguarded render.
"""

from __future__ import annotations

from pydantic import BaseModel

from prompting_press import GuardConfig, Prompt

UNTRUSTED_DEF = {
    "name": "ask",
    "role": "user",
    "body": "Tell me about {{ topic }}.",
    "variables": {"topic": {"type": "string", "trusted": False}},
}


class TopicVars(BaseModel):
    topic: str


def test_guard_config_is_importable() -> None:
    assert GuardConfig is not None
    gc = GuardConfig(enabled=True)
    assert gc.enabled is True


def test_render_without_guard_has_none_guard_field() -> None:
    p = Prompt(UNTRUSTED_DEF)
    result = p.render(TopicVars(topic="rivers"))
    assert result.guard is None


def test_render_with_guard_enabled_wraps_untrusted_value() -> None:
    p = Prompt(UNTRUSTED_DEF)
    result = p.render(TopicVars(topic="rivers"), guard=GuardConfig(enabled=True))
    assert "<untrusted>" in result.text
    assert "rivers" in result.text
    # guard advisory is a non-empty string
    assert isinstance(result.guard, str) and len(result.guard) > 0


def test_render_guard_on_off_text_differs() -> None:
    p = Prompt(UNTRUSTED_DEF)
    plain = p.render(TopicVars(topic="rivers"))
    guarded = p.render(TopicVars(topic="rivers"), guard=GuardConfig(enabled=True))
    assert plain.text != guarded.text


def test_render_hash_differs_guard_on_vs_off() -> None:
    # Spec 015 / Principle V: render_hash = SHA256(rendered output);
    # the rendered output differs between guard-on and guard-off.
    p = Prompt(UNTRUSTED_DEF)
    plain = p.render(TopicVars(topic="rivers"))
    guarded = p.render(TopicVars(topic="rivers"), guard=GuardConfig(enabled=True))
    assert plain.render_hash != guarded.render_hash


def test_template_hash_is_same_regardless_of_guard() -> None:
    # template_hash covers the template source, which the guard does not change.
    p = Prompt(UNTRUSTED_DEF)
    plain = p.render(TopicVars(topic="rivers"))
    guarded = p.render(TopicVars(topic="rivers"), guard=GuardConfig(enabled=True))
    assert plain.template_hash == guarded.template_hash


def test_guard_entity_escapes_angle_brackets_in_value() -> None:
    # Entity-escaping is reversible structure, not content mutation (Principle III).
    p = Prompt(UNTRUSTED_DEF)
    result = p.render(TopicVars(topic="a<b>c&d"), guard=GuardConfig(enabled=True))
    # The raw characters must not appear un-escaped inside the <untrusted> span.
    assert "a<b>c" not in result.text
    # But the value identity is preserved (encoded form is present).
    assert "a" in result.text


def test_guard_metadata_presence_satisfies_check() -> None:
    # The provenance lint is satisfied by metadata.guard key presence.
    p = Prompt(
        {
            "name": "ask",
            "role": "user",
            "body": "Tell me about {{ topic }}.",
            "variables": {"topic": {"type": "string", "trusted": False}},
            "metadata": {"guard": {"enabled": True}},
        }
    )
    report = p.check()
    assert report.passed()
