/**
 * Loader guide — loading JSON and TOML.
 *
 * The loader is format-agnostic (returns raw text), so only the FileSystemLoader
 * suffix and the `from*` parser change. Uses the `assistant.json` / `assistant.toml`
 * fixtures next to this program.
 */

import assert from "node:assert/strict";
import nodepath from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { FileSystemLoader, Prompt } from "prompting-press";

const dir = nodepath.dirname(fileURLToPath(import.meta.url));

test("FileSystemLoader: JSON and TOML are format-agnostic", async () => {
	// JSON: suffix ".json" -> loads {dir}/assistant.json, parsed with fromJson.
	const jsonLoader = new FileSystemLoader(dir, ".json");
	const fromJson = Prompt.fromJson(await jsonLoader.load("assistant"));
	assert.equal(fromJson.name, "assistant");

	// TOML: suffix ".toml" -> loads {dir}/assistant.toml, parsed with fromToml.
	const tomlLoader = new FileSystemLoader(dir, ".toml");
	const fromToml = Prompt.fromToml(await tomlLoader.load("assistant"));
	assert.equal(fromToml.name, "assistant");

	// Empty suffix -> the extension lives in the key instead (same file either way).
	const bareLoader = new FileSystemLoader(dir, "");
	const bare = Prompt.fromJson(await bareLoader.load("assistant.json"));
	assert.equal(bare.name, "assistant");
});
