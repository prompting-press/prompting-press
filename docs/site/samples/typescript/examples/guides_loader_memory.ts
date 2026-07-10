/**
 * Loader guide — MemoryLoader: load raw text by key, then construct a Prompt.
 *
 * The kernel stays I/O-free; the loader is a separate, caller-invoked I/O leaf.
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { MemoryLoader, Prompt } from "prompting-press";

const GREET_YAML = `\
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
`;

test("MemoryLoader: load raw text then construct", async () => {
	const loader = new MemoryLoader({ greet: GREET_YAML });

	// load() returns raw text — parsing is a separate step.
	const raw = await loader.load("greet");
	const prompt = Prompt.fromYaml(raw);
	assert.equal(prompt.name, "greet");
});
