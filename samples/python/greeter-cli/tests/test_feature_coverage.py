# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Feature-coverage suite (spec 014, FR-014a / SC-009): one assertion per feature in
the full public surface, so end-to-end coverage is provable by inventory, not
inspection. Also the behavioral test for the sample app (FR-013).

If a consumed library API changes incompatibly, this suite fails citing the app —
the consumer-facing smoke test (SC-010).

Mirror of ``samples/rust/greeter-cli/tests/feature_coverage.rs``.
"""

from __future__ import annotations

import re

import pytest
from pydantic import BaseModel, field_validator

from prompting_press import (
    Composition,
    GuardConfig,
    Prompt,
    PromptRenderError,
    PromptValidationError,
)

# ── 64-char lowercase hex pattern (provenance hash shape, FR-007/SC-005) ─────
HEX64 = re.compile(r"\A[0-9a-f]{64}\Z")

# ── Prompt YAML — exact same docs as the Rust reference app ──────────────────

GREET_YAML = """\
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages."
variables:
  name: {type: string, trusted: true}
  count: {type: integer, trusted: true}
variants:
  formal:
    body: "Good day, {{ name }}. You have {{ count }} messages awaiting your attention."
"""

ASK_YAML = """\
name: ask
role: user
body: "Tell me about {{ topic }}."
variables:
  topic: {type: string, trusted: false}
"""

# ── Typed vars (Pydantic — the per-language idiom; Principle VI) ─────────────


class GreetVars(BaseModel):
    name: str
    count: int

    @field_validator("name")
    @classmethod
    def _name_non_empty(cls, value: str) -> str:
        if not value:
            raise ValueError("name must not be empty")
        return value

    @field_validator("count")
    @classmethod
    def _count_non_negative(cls, value: int) -> int:
        if value < 0:
            raise ValueError("count must be non-negative")
        return value


class AskVars(BaseModel):
    topic: str

    @field_validator("topic")
    @classmethod
    def _topic_non_empty(cls, value: str) -> str:
        if not value:
            raise ValueError("topic must not be empty")
        return value


class SysVars(BaseModel):
    instruction: str

    @field_validator("instruction")
    @classmethod
    def _instruction_non_empty(cls, value: str) -> str:
        if not value:
            raise ValueError("instruction must not be empty")
        return value


# ── Fixtures ──────────────────────────────────────────────────────────────────


def greet() -> Prompt:
    return Prompt.from_yaml(GREET_YAML)


# ── construct ─────────────────────────────────────────────────────────────────


def test_feature_construct_from_yaml() -> None:
    p = greet()
    assert p.name == "greet"
    assert "formal" in p.variants


def test_feature_construct_from_json_and_toml_agree() -> None:
    """from_json and from_toml parse to the same prompt body."""
    json_text = '{"name":"g","role":"user","body":"Hi {{ n }}","variables":{"n":{"type":"string","trusted":true}}}'
    toml_text = 'name = "g"\nrole = "user"\nbody = "Hi {{ n }}"\n[variables.n]\ntype = "string"\ntrusted = true\n'
    assert Prompt.from_json(json_text).body == Prompt.from_toml(toml_text).body


def test_feature_construct_from_dict() -> None:
    """Prompt(dict) is equivalent to Prompt.from_yaml for the same shape."""
    p = Prompt(
        {
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}, you have {{ count }} messages.",
            "variables": {
                "name": {"type": "string", "trusted": True},
                "count": {"type": "integer", "trusted": True},
            },
            "variants": {
                "formal": {
                    "body": "Good day, {{ name }}. You have {{ count }} messages awaiting your attention."
                }
            },
        }
    )
    assert p.name == "greet"
    assert "formal" in p.variants


# ── validate (Pydantic runs before templating) ────────────────────────────────


def test_feature_validate_rejects_invalid_vars() -> None:
    """Empty name violates the validator → PromptValidationError; kernel never reached."""
    with pytest.raises(PromptValidationError):
        greet().render(GreetVars, data={"name": "", "count": 1})


# ── render default ────────────────────────────────────────────────────────────


def test_feature_render_default() -> None:
    r = greet().render(GreetVars, data={"name": "Ada", "count": 3})
    assert r.text == "Hi Ada, you have 3 messages."
    assert r.variant == "default"


def test_feature_render_accepts_model_instance() -> None:
    """Passing a pre-constructed Pydantic instance (no data= kwarg) also works."""
    r = greet().render(GreetVars(name="Ada", count=3))
    assert r.text == "Hi Ada, you have 3 messages."
    assert r.variant == "default"


# ── render variant ────────────────────────────────────────────────────────────


def test_feature_render_variant() -> None:
    r = greet().render(GreetVars, data={"name": "Ada", "count": 3}, variant="formal")
    assert r.variant == "formal"
    assert r.text.startswith("Good day, Ada.")


# ── provenance hashes (format-checked, not exact — content-addressed) ────────


def test_feature_provenance_hashes() -> None:
    r = greet().render(GreetVars, data={"name": "Ada", "count": 3})
    assert HEX64.match(r.template_hash), (
        f"template_hash is not 64-char lowercase hex: {r.template_hash!r}"
    )
    assert HEX64.match(r.render_hash), (
        f"render_hash is not 64-char lowercase hex: {r.render_hash!r}"
    )


# ── compose ───────────────────────────────────────────────────────────────────


def test_feature_compose_two_messages() -> None:
    sys_prompt = Prompt.from_yaml(
        'name: sys\nrole: system\nbody: "{{ instruction }}"\n'
        "variables:\n  instruction: {type: string, trusted: true}\n"
    )
    comp = Composition()
    assert len(comp) == 0
    comp.append(sys_prompt, SysVars(instruction="Be concise."))
    comp.append(greet(), GreetVars(name="Ada", count=3))
    assert len(comp) == 2
    msgs = comp.resolve()
    assert len(msgs) == 2
    assert msgs[0].role == "system"
    assert msgs[0].text == "Be concise."
    assert msgs[1].role == "user"


# ── check (advisory lint) ─────────────────────────────────────────────────────


def test_feature_check_surfaces_untrusted_without_guard() -> None:
    ask = Prompt.from_yaml(ASK_YAML)
    report = ask.check()
    assert not report.passed(), "ask has an untrusted var with no guard → a finding"
    assert report.findings[0].kind == "untrusted_without_guard"
    # greet has only trusted vars → clean.
    assert greet().check().passed()


# ── guard (delimiting + advisory) ─────────────────────────────────────────────


def test_feature_guard_wraps_untrusted_and_returns_advisory() -> None:
    ask = Prompt.from_yaml(ASK_YAML)
    r = ask.render(AskVars(topic="rivers"), guard=GuardConfig(enabled=True))
    assert "<untrusted>rivers</untrusted>" in r.text, (
        f"untrusted value must be delimited in the body, got: {r.text!r}"
    )
    assert r.guard is not None, "an advisory must be returned when the guard is enabled"
    # Guard OFF => no delimiters, no advisory.
    plain = ask.render(AskVars(topic="rivers"))
    assert "<untrusted>" not in plain.text
    assert plain.guard is None


def test_feature_guard_off_matches_no_guard() -> None:
    ask = Prompt.from_yaml(ASK_YAML)
    no_guard = ask.render(AskVars(topic="rivers"))
    disabled = ask.render(AskVars(topic="rivers"), guard=GuardConfig(enabled=False))
    assert no_guard.text == disabled.text
    assert no_guard.guard is None
    assert disabled.guard is None


# ── error path (unknown variant → structured PromptRenderError) ───────────────


def test_feature_error_unknown_variant() -> None:
    with pytest.raises(PromptRenderError) as exc_info:
        greet().render(GreetVars, data={"name": "Ada", "count": 3}, variant="nope")
    assert exc_info.value.errors[0].code == "unknown_variant"


# ── get_source (raw template source per variant) ──────────────────────────────


def test_feature_get_source() -> None:
    p = greet()
    assert p.get_source() == p.body
    assert p.get_source(variant="formal").startswith("Good day")


# ── the app's run() completes end-to-end ─────────────────────────────────────


def test_app_run_end_to_end(capsys: pytest.CaptureFixture[str]) -> None:
    """The sample app's run() must complete without raising.

    The demonstrated error path is caught internally — only unexpected errors
    should propagate (and fail this test).
    """
    from greeter_cli.main import run

    run()
    captured = capsys.readouterr()
    assert "=== done ===" in captured.out
    assert "[construct]" in captured.out
    assert "[render:default]" in captured.out
    assert "[render:formal]" in captured.out
    assert "[provenance]" in captured.out
    assert "[compose]" in captured.out
    assert "[check]" in captured.out
    assert "[guard]" in captured.out
    assert "[error]" in captured.out
    assert "[handoff]" in captured.out
