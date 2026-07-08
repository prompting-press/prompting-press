/**
 * Integration gate — provenanceAttributes().
 *
 * Covers:
 * - result.provenanceAttributes() returns a Record<string,string> with exactly 4 entries
 * - All four prompting_press.prompt.* keys are present
 * - Values match result.name, result.variant, result.templateHash, result.renderHash
 * - variant is "default" when no variant selected
 * - Excluded fields: text, guard, output_model, metadata
 * - Deterministic: two identical renders produce identical maps
 * - Pure: calling twice does not mutate
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { Prompt } from "prompting-press";

const HEX64 = /^[0-9a-f]{64}$/;

const KEY_NAME = "prompting_press.prompt.name";
const KEY_VARIANT = "prompting_press.prompt.variant";
const KEY_TEMPLATE_HASH = "prompting_press.prompt.template_hash";
const KEY_RENDER_HASH = "prompting_press.prompt.render_hash";
const EXPECTED_KEYS = [KEY_NAME, KEY_VARIANT, KEY_TEMPLATE_HASH, KEY_RENDER_HASH];

const GREET_YAML = `
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name:  { type: string,  trusted: true }
  count: { type: integer, trusted: true }
`;

test("provenanceAttributes() returns a plain object with exactly 4 entries", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	assert.equal(typeof attrs, "object");
	assert.ok(attrs !== null);
	assert.equal(Object.keys(attrs).length, 4);
});

test("provenanceAttributes() contains all four library-owned prompting_press.prompt.* keys", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	for (const key of EXPECTED_KEYS) {
		assert.ok(Object.hasOwn(attrs, key), `missing key: ${key}`);
	}
});

test("provenanceAttributes() values equal the result fields", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });
	const attrs = result.provenanceAttributes();
	assert.equal(attrs[KEY_NAME], result.name);
	assert.equal(attrs[KEY_VARIANT], result.variant);
	assert.equal(attrs[KEY_TEMPLATE_HASH], result.templateHash);
	assert.equal(attrs[KEY_RENDER_HASH], result.renderHash);
});

test("provenanceAttributes() all values are strings", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	for (const [k, v] of Object.entries(attrs)) {
		assert.equal(typeof v, "string", `value for ${k} must be a string`);
	}
});

test("provenanceAttributes() KEY_VARIANT is 'default' when no variant selected", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Grace", count: 0 }).provenanceAttributes();
	assert.equal(attrs[KEY_VARIANT], "default");
});

test("provenanceAttributes() KEY_TEMPLATE_HASH and KEY_RENDER_HASH are 64-char lowercase hex", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	assert.match(attrs[KEY_TEMPLATE_HASH], HEX64);
	assert.match(attrs[KEY_RENDER_HASH], HEX64);
});

test("provenanceAttributes() excludes text, guard, output_model, metadata", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	assert.ok(!("text" in attrs));
	assert.ok(!("guard" in attrs));
	assert.ok(!("output_model" in attrs));
	assert.ok(!("metadata" in attrs));
	assert.equal(Object.keys(attrs).length, 4);
});

test("provenanceAttributes() is deterministic across identical renders", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const a = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	const b = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	assert.deepEqual(a, b);
});

test("provenanceAttributes() is pure — calling twice does not mutate the result", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });
	const a = result.provenanceAttributes();
	const b = result.provenanceAttributes();
	assert.deepEqual(a, b);
	assert.equal(result.name, "greet");
	assert.equal(result.variant, "default");
});
