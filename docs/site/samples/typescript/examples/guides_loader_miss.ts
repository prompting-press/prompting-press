/**
 * Loader guide — a missing key rejects with PromptLoadError (load_not_found),
 * distinct from LoadError (the parse/shape error thrown on malformed YAML).
 *
 * `catch (e) { if (e instanceof PromptLoadError) }` does NOT catch a malformed-YAML `LoadError`.
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import {
	LOAD_NOT_FOUND,
	LoadError,
	MemoryLoader,
	Prompt,
	PromptLoadError,
} from "prompting-press";

test("missing key rejects with PromptLoadError (load_not_found)", async () => {
	const loader = new MemoryLoader({});

	// A missing key rejects with PromptLoadError — not a parse error.
	await assert.rejects(
		() => loader.load("missing"),
		(err: unknown) => {
			assert.ok(err instanceof PromptLoadError);
			assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
			return true;
		},
	);

	// PromptLoadError is distinct from LoadError.
	// Parsing bad YAML throws LoadError — a different type on a different path.
	assert.throws(
		() => Prompt.fromYaml("not: valid: yaml: ["),
		(err: unknown) => {
			assert.ok(err instanceof LoadError);
			return true;
		},
	);
});
