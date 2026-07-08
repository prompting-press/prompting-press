/**
 * Loader guide — custom loaders: implement the `PromptLoader` interface.
 *
 * A class with an async `load(key): Promise<string>` satisfies the contract.
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import type { PromptLoader } from "prompting-press";
import { LOAD_NOT_FOUND, Prompt, PromptLoadError } from "prompting-press";

const GREET_YAML = `\
name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
`;

class InlineLoader implements PromptLoader {
	readonly #map: Record<string, string>;

	constructor(map: Record<string, string>) {
		this.#map = map;
	}

	async load(key: string): Promise<string> {
		const text = this.#map[key];
		if (text === undefined) {
			throw new PromptLoadError(`key not found: \`${key}\``, [
				{
					field: "",
					code: LOAD_NOT_FOUND,
					message: `key not found: \`${key}\``,
				},
			]);
		}
		return text;
	}
}

test("custom loader: class implementing PromptLoader", async () => {
	const loader = new InlineLoader({ greet: GREET_YAML });

	const raw = await loader.load("greet");
	const prompt = Prompt.fromYaml(raw);
	assert.equal(prompt.name, "greet");

	// A missing key rejects with PromptLoadError.
	await assert.rejects(() => loader.load("missing"), PromptLoadError);
});
