# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Build a Composition by appending (Prompt, vars) entries, then resolve it to
an ordered list of role-tagged messages. Standalone."""

from prompting_press import Composition, Prompt
from pydantic import BaseModel


class SysVars(BaseModel):
    instructions: str


class UserVars(BaseModel):
    query: str


# Build the two prompts inline from their shape, so the content is explicit.
sys_prompt = Prompt(
    {
        "name": "system-preamble",
        "role": "system",
        "body": "{{ instructions }}",
        "variables": {"instructions": {"type": "string", "trusted": True}},
    }
)
user_prompt = Prompt(
    {
        "name": "user-turn",
        "role": "user",
        "body": "{{ query }}",
        "variables": {"query": {"type": "string", "trusted": False}},
    }
)

comp = Composition()
comp.append(sys_prompt, SysVars(instructions="Be concise."))
comp.append(user_prompt, UserVars(query="What is Rust?"))

messages = comp.resolve()
for m in messages:
    print(m.role, m.text)
    # "system" "Be concise."
    # "user"   "What is Rust?"

assert [(m.role, m.text) for m in messages] == [
    ("system", "Be concise."),
    ("user", "What is Rust?"),
]
