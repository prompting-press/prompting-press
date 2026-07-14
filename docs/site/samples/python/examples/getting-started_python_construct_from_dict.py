# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Construct a Prompt from a plain dict (the Rust loader validates the shape on the way in)."""

from prompting_press import Prompt

# A plain dict works too — convenient when the shape comes from already-parsed
# config. The Rust loader validates the shape on the way in, the same as the typed form.
assistant = Prompt(
    {
        "name": "assistant",
        "role": "system",
        "body": "You are a support assistant for {{ company }}. Keep your replies under {{ max_words }} words.",
        "variables": {
            "company": {"type": "string", "trusted": True},
            "max_words": {"type": "integer", "trusted": True},
        },
    }
)

assert assistant.name == "assistant"
