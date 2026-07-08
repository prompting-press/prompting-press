/**
 * Integration gate — render surface.
 *
 * Covers:
 * - Happy-path render with schema form, static form, bound-validator form
 * - Variant selection (named arm vs default)
 * - Unknown variant → PromptRenderError (code unknown_variant)
 * - Undefined variable → PromptRenderError (code undefined_variable, never silent)
 * - Validation failure before templating → PromptValidationError
 * - Result fields: text, name, variant, templateHash, renderHash (64-hex), guard=null
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { Prompt, PromptRenderError, PromptValidationError } from "prompting-press";
import { z } from "zod";

const HEX64 = /^[0-9a-f]{64}$/;

const GREET_YAML = `
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name: { type: string, trusted: true }
  count: { type: integer, trusted: true }
`;

const VARIANT_YAML = `
name: greetv
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
variants:
  formal: { body: "Good day, {{ name }}" }
`;

const GreetSchema = z.object({
	name: z.string().min(1),
	count: z
		.number()
		.int()
		.refine((n) => n >= 0, "must be non-negative"),
});

const NameSchema = z.object({ name: z.string() });

test("render (schema form) produces correct text and 64-hex hashes", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render(GreetSchema, { name: "Ada", count: 3 });
	assert.equal(result.text, "Hi Ada, you have 3 messages");
	assert.equal(result.name, "greet");
	assert.equal(result.variant, "default");
	assert.match(result.templateHash, HEX64);
	assert.match(result.renderHash, HEX64);
	assert.equal(result.guard, null);
});

test("render (static form) marshals data directly without Zod check", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Bo", count: 1 });
	assert.equal(result.text, "Hi Bo, you have 1 messages");
	assert.equal(result.variant, "default");
});

test("render (bound-validator form) validates via the bound schema", () => {
	const p = Prompt.fromYaml(GREET_YAML, GreetSchema);
	const result = p.render({ name: "Cy", count: 2 });
	assert.equal(result.text, "Hi Cy, you have 2 messages");
});

test("render with bound validator throws PromptValidationError on invalid data", () => {
	const p = Prompt.fromYaml(GREET_YAML, GreetSchema);
	assert.throws(
		() => p.render({ name: "Ada", count: -1 }),
		(err) => {
			assert.ok(err instanceof PromptValidationError);
			assert.ok(err.errors.some((r) => r.field === "count"));
			return true;
		},
	);
});

test("render with named variant returns the variant arm text", () => {
	const p = Prompt.fromYaml(VARIANT_YAML);
	const def = p.render(NameSchema, { name: "Di" });
	const formal = p.render(NameSchema, { name: "Di" }, { variant: "formal" });
	assert.equal(def.text, "Hi Di");
	assert.equal(formal.text, "Good day, Di");
	assert.equal(def.variant, "default");
	assert.equal(formal.variant, "formal");
	// Different arms → different template hashes.
	assert.notEqual(def.templateHash, formal.templateHash);
});

test("render with unknown variant throws PromptRenderError (unknown_variant)", () => {
	const p = Prompt.fromYaml(VARIANT_YAML);
	assert.throws(
		() => p.render(NameSchema, { name: "Ada" }, { variant: "nope" }),
		(err) => {
			assert.ok(err instanceof PromptRenderError);
			assert.ok(err.errors.some((r) => r.code === "unknown_variant"));
			return true;
		},
	);
});

test("render with mismatched field name throws PromptRenderError (undefined_variable, not silent)", () => {
	// The schema has `nam` but the template expects `name` — a loud error, never empty render.
	const MisnameSchema = z.object({ nam: z.string() });
	const p = Prompt.fromYaml(`
name: greet
role: user
body: "Hi {{ name }}!"
variables:
  name: { type: string, trusted: true }
`);
	assert.throws(
		() => p.render(MisnameSchema, { nam: "Ada" }),
		(err) => {
			assert.ok(err instanceof PromptRenderError);
			assert.ok(err.errors.some((r) => r.code === "undefined_variable"));
			return true;
		},
	);
});

test("validation failure throws PromptValidationError before kernel is reached", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	assert.throws(
		() => p.render(GreetSchema, { name: "Ada", count: -1 }),
		(err) => {
			assert.ok(err instanceof PromptValidationError);
			const offending = err.errors.filter((r) => r.field === "count");
			assert.ok(offending.length > 0);
			assert.ok(offending.every((r) => r.code === "validation"));
			return true;
		},
	);
});
