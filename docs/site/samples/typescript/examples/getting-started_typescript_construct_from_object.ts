// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import assert from "node:assert/strict";
import { test } from "node:test";
import { Prompt, type PromptDefinition } from "prompting-press";

test("construct from an object", () => {
  // The constructor takes a typed PromptDefinition — an editor type-checks the
  // shape (field names, the role enum, each variable's `trusted` flag) at author time.
  const definition: PromptDefinition = {
    name: "assistant",
    role: "system",
    body: "You are a support assistant for {{ company }}. Keep your replies under {{ max_words }} words.",
    variables: {
      company: { type: "string", trusted: true },
      max_words: { type: "integer", trusted: true },
    },
  };

  const assistant = new Prompt(definition); // same validation as the from* factories

  assert.equal(assistant.name, "assistant"); // => "assistant"
});
