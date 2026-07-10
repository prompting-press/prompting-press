# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""from_messages builds a Composition from a list of (prompt, vars) or
(prompt, vars, variant) tuples. All validation runs before any state is created.
Standalone."""

from prompting_press import Composition, Prompt
from pydantic import BaseModel


class SysVars(BaseModel):
    instructions: str


class UserVars(BaseModel):
    query: str


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

comp = Composition.from_messages(
    [
        (sys_prompt, SysVars(instructions="Be concise.")),
        (user_prompt, UserVars(query="What is Rust?")),
    ]
)
messages = comp.resolve()
for m in messages:
    print(m.role, m.text)
    # "system" "Be concise."
    # "user"   "What is Rust?"

assert [(m.role, m.text) for m in messages] == [
    ("system", "Be concise."),
    ("user", "What is Rust?"),
]
