// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

/**
 * Loader guide — FileSystemLoader: map a key to a file in a base directory.
 *
 * Uses the `assistant.yaml` fixture that lives next to this program.
 */

import assert from "node:assert/strict";
import nodepath from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { FileSystemLoader, Prompt } from "prompting-press";

const dir = nodepath.dirname(fileURLToPath(import.meta.url));

test("FileSystemLoader: map key to file and construct", async () => {
	// Construct from an existing directory.
	const loader = new FileSystemLoader(dir);

	// "assistant" maps to {dir}/assistant.yaml (default suffix ".yaml").
	const raw = await loader.load("assistant");
	const prompt = Prompt.fromYaml(raw);
	assert.equal(prompt.name, "assistant");
});
