# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Use Prompting Press prompts with the Strands Agents SDK (Python).

Strands has no ``system`` role in its message list: ``role`` is exactly
``user`` | ``assistant``, and the system prompt is a SEPARATE ``Agent``
argument. So the bridge PARTITIONS a Prompting Press composition:

  * system-role texts  -> a single ``system_prompt`` string (joined with "\\n\\n")
  * everything else     -> ``messages`` as ``{role, content: [{"text": ...}]}``

Note: Strands cannot preserve the POSITION of a system message inside the
conversation — every system-role text is hoisted into the one system prompt,
regardless of where it sat. That is a Strands limitation, not something
Prompting Press can carry across. Only plain text is mapped; Strands'
provider-specific content blocks (guardContent, toolResult, ...) are out of
scope here. Standalone.
"""

from prompting_press import Composition, Prompt
from pydantic import BaseModel
from strands import Agent


class TextVars(BaseModel):
    text: str


def to_strands(messages):
    """Partition a Prompting Press composition for Strands.

    Returns ``(system_prompt, convo)`` where ``system_prompt`` is the
    "\\n\\n"-joined system-role texts (or ``None`` if there were none) and
    ``convo`` is the non-system messages as Strands content-block messages.
    """
    system_texts = [m.text for m in messages if m.role == "system"]
    system_prompt = "\n\n".join(system_texts) if system_texts else None
    convo = [
        {"role": m.role, "content": [{"text": m.text}]}
        for m in messages
        if m.role != "system"
    ]
    return system_prompt, convo


def _prompt(name, role, trusted=True):
    return Prompt(
        {
            "name": name,
            "role": role,
            "body": "{{ text }}",
            "variables": {"text": {"type": "string", "trusted": trusted}},
        }
    )


# TWO system messages (so the "\n\n"-join + ordering is actually exercised),
# then a user/assistant/user exchange.
comp = Composition()
comp.append(_prompt("sys-1", "system"), TextVars(text="You are a support agent."))
comp.append(_prompt("sys-2", "system"), TextVars(text="Answer only in English."))
comp.append(
    _prompt("u-1", "user", trusted=False), TextVars(text="What's your return policy?")
)
comp.append(_prompt("a-1", "assistant"), TextVars(text="30 days, unopened."))
comp.append(_prompt("u-2", "user", trusted=False), TextVars(text="And opened items?"))

system_prompt, convo = to_strands(comp.resolve())

# Construct the agent from the two seams (no .run()/no model call — offline).
agent = Agent(system_prompt=system_prompt, messages=convo)
print(agent.system_prompt)  # "You are a support agent.\n\nAnswer only in English."

# --- assertions (this file is executed by CI) ---

# Both system texts hoisted, joined in order with a blank line between.
assert system_prompt == "You are a support agent.\n\nAnswer only in English."

# convo drops the system messages; only user/assistant remain, in order,
# each wrapped as a single {"text": ...} content block.
assert convo == [
    {"role": "user", "content": [{"text": "What's your return policy?"}]},
    {"role": "assistant", "content": [{"text": "30 days, unopened."}]},
    {"role": "user", "content": [{"text": "And opened items?"}]},
]

# The agent accepted both seams.
assert agent.system_prompt == system_prompt
assert len(agent.messages) == 3
assert [m["role"] for m in agent.messages] == ["user", "assistant", "user"]
