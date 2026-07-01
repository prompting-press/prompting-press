"""Greeter CLI — a realistic Prompting Press consumer sample app (spec 014, WU-C).

Walks the FULL public feature surface end-to-end (FR-014):
construct → validate → render default + a named variant → compose a 2-message
prompt → check() → the advisory guard → provenance hashes → an error path.
The "hand to an LLM" step is a printed stub — the library never calls a provider
(FR-018).

Run it: python -m greeter_cli.main
"""

from __future__ import annotations

from pydantic import BaseModel, field_validator

from prompting_press import (
    Composition,
    GuardConfig,
    Prompt,
    PromptRenderError,
)

# ── Prompt documents (a real consumer would read these from files) ────────────

#: A greeting prompt with a ``formal`` variant, both sharing the same variables.
GREET_YAML: str = """\
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages."
variables:
  name:
    type: string
    trusted: true
  count:
    type: integer
    trusted: true
variants:
  formal:
    body: "Good day, {{ name }}. You have {{ count }} messages awaiting your attention."
"""

#: A prompt with an UNTRUSTED variable, used to demonstrate the guard + check().
ASK_YAML: str = """\
name: ask
role: user
body: "Tell me about {{ topic }}."
variables:
  topic:
    type: string
    trusted: false
"""

# ── Typed vars (validated by Pydantic before any templating) ─────────────────


class GreetVars(BaseModel):
    """Vars for the greet prompt — both fields are trusted."""

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
    """Vars for the ask prompt — topic is untrusted."""

    topic: str

    @field_validator("topic")
    @classmethod
    def _topic_non_empty(cls, value: str) -> str:
        if not value:
            raise ValueError("topic must not be empty")
        return value


class SysVars(BaseModel):
    """Vars for the system preamble prompt."""

    instruction: str

    @field_validator("instruction")
    @classmethod
    def _instruction_non_empty(cls, value: str) -> str:
        if not value:
            raise ValueError("instruction must not be empty")
        return value


def run() -> None:
    """Run the full feature walk.

    The demonstrated error path is caught and reported inline, not propagated.
    """
    print("=== Prompting Press — Python consumer sample ===\n")

    # 1. CONSTRUCT (from_yaml / from_json / from_toml / Prompt(dict))
    #    Each factory form validates the template↔variables agreement immediately.
    greet = Prompt.from_yaml(GREET_YAML)
    print(f"[construct] loaded prompt {greet.name!r}")
    print(f"[construct] variants: {list(greet.variants.keys())}")

    # Also exercise from_json and Prompt(dict) to mirror the Rust test:
    greet_json = Prompt.from_json(
        '{"name":"g","role":"user","body":"Hi {{ n }}","variables":{"n":{"type":"string","trusted":true}}}'
    )
    greet_toml = Prompt.from_toml(
        'name = "g"\nrole = "user"\nbody = "Hi {{ n }}"\n[variables.n]\ntype = "string"\ntrusted = true\n'
    )
    assert greet_json.body == greet_toml.body, "from_json and from_toml must agree"

    greet_dict = Prompt(
        {
            "name": "greet_dict",
            "role": "user",
            "body": "Hi {{ name }}, you have {{ count }} messages.",
            "variables": {
                "name": {"type": "string", "trusted": True},
                "count": {"type": "integer", "trusted": True},
            },
        }
    )
    assert greet_dict.name == "greet_dict"

    # 2. VALIDATE + RENDER the default arm.
    #    Pydantic validation runs before any templating (FR-002).
    default = greet.render(GreetVars, data={"name": "Ada", "count": 3})
    print(f"\n[render:default] {default.text}")

    # 3. RENDER a named variant — a different body from the same vars.
    formal = greet.render(GreetVars, data={"name": "Ada", "count": 3}, variant="formal")
    print(f"[render:formal]  {formal.text}")

    # 4. PROVENANCE — content-addressed hashes on the result.
    print(
        f"\n[provenance] variant={default.variant}"
        f" template_hash={default.template_hash[:8]}…"
        f" render_hash={default.render_hash[:8]}…"
    )

    # 5. COMPOSE a 2-message prompt (system preamble + the greeting).
    sys_prompt = Prompt.from_yaml(
        'name: sys\nrole: system\nbody: "{{ instruction }}"\n'
        "variables:\n  instruction:\n    type: string\n    trusted: true\n"
    )
    comp = Composition()
    comp.append(sys_prompt, SysVars(instruction="Be concise."))
    comp.append(greet, GreetVars(name="Ada", count=3))
    messages = comp.resolve()
    print(f"\n[compose] {len(messages)} messages:")
    for m in messages:
        print(f"  {m.role}: {m.text}")

    # 6. CHECK — the advisory lint. `ask` declares an untrusted var with no guard
    #    metadata, so check() surfaces one finding. `greet` is clean.
    ask = Prompt.from_yaml(ASK_YAML)
    report = ask.check()
    print(
        f"\n[check] ask.check() passed={report.passed()} findings={len(report.findings)}"
    )
    for f in report.findings:
        print(f"  {f.kind}: {f.detail}")

    # 7. GUARD — enable it: the untrusted value is delimited in the body and an
    #    advisory is returned. The library never sends this anywhere.
    guarded = ask.render(
        AskVars(topic="rivers"),
        guard=GuardConfig(enabled=True),
    )
    print(f"\n[guard] text  = {guarded.text}")
    print(f"[guard] guard = {guarded.guard or '<none>'}")

    # 8. ERROR PATH — an unknown variant fails loudly with a structured error.
    try:
        greet.render(GreetVars, data={"name": "Ada", "count": 3}, variant="nonexistent")
        raise AssertionError("expected the unknown-variant render to fail")
    except PromptRenderError as exc:
        code = exc.errors[0].code
        print(f"\n[error] unknown variant rejected: code={code}")

    # 9. HAND-OFF STUB — a real app would send `messages` to a provider here.
    #    The library does no I/O and calls no model; this is a printed placeholder.
    print(
        f"\n[handoff] (stub) would POST {len(messages)} messages to the configured LLM provider."
    )

    print("\n=== done ===")


def main() -> None:
    """Entry point."""
    run()


if __name__ == "__main__":
    main()
