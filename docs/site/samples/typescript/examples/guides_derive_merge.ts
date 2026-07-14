// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

/**
 * Derive guide — MergeStrategy.Merge: union the base's declared variables with the overlay's,
 * so a child prompt inherits `company` + `max_words` and adds its own `tone` without
 * hand-spreading the base's variables. The base is untouched.
 *
 * Standalone — the docs page displays this file verbatim; run it directly to check.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { MergeStrategy, Prompt } from "prompting-press";

// The caller reads the definition; the library does no file I/O itself.
// Resolve the file next to this program (a real app uses its own path).
const defFile = (name: string) => fileURLToPath(new URL(name, import.meta.url));

test("Merge unions the base's variables into the child, leaving the base untouched", () => {
	const base = Prompt.fromYaml(readFileSync(defFile("assistant.yaml"), "utf8"));

	// Merge unions the map-typed fields (variables/variants/metadata) at their top-level
	// keys — child-wins on collision. The base's `company` + `max_words` survive; the
	// overlay only needs to declare what it adds. `strategy` rides in the options object.
	const child = base.derive(
		{
			body: "You are a {{ tone }} assistant for {{ company }}. Keep replies under {{ max_words }} words.",
			variables: { tone: { type: "string", trusted: true } },
		},
		{ strategy: MergeStrategy.Merge },
	);

	// child inherited the base's two variables and gained its own — three in total.
	assert.deepEqual(
		new Set(Object.keys(child.variables ?? {})),
		new Set(["company", "max_words", "tone"]),
	);
	// base is untouched: no `tone` leaked back onto it.
	assert.ok(!("tone" in (base.variables ?? {})));
});
