/**
 * Integration gate — coreVersion().
 *
 * Covers:
 * - coreVersion() returns a non-empty string
 * - The string is callable and stable across repeated calls
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { coreVersion } from "prompting-press";

test("coreVersion() returns a non-empty string", () => {
	const v = coreVersion();
	assert.equal(typeof v, "string");
	assert.ok(v.length > 0, `coreVersion() returned empty string: ${JSON.stringify(v)}`);
});

test("coreVersion() is stable across repeated calls", () => {
	assert.equal(coreVersion(), coreVersion());
});
