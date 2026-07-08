/**
 * Integration gate — PromptingPressError hierarchy.
 *
 * Covers:
 * - PromptingPressError base class
 * - PromptValidationError / PromptRenderError / LoadError / PromptLoadError extend base
 * - .errors is [{field, code, message}] with correct types
 * - No native ZodError leaks: a validation failure is PromptValidationError, not ZodError
 * - LoadError and PromptLoadError are distinct (not instanceof each other)
 * - Registry is NOT exported (post-spec-008 removal)
 * - render free function is NOT exported (post-spec-008 removal)
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import {
	LoadError,
	MemoryLoader,
	Prompt,
	PromptingPressError,
	PromptLoadError,
	PromptRenderError,
	PromptValidationError,
} from "prompting-press";
import { z } from "zod";

const GREET_YAML = `
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name: { type: string, trusted: true }
  count: { type: integer, trusted: true }
`;

const GreetSchema = z.object({
	name: z.string(),
	count: z
		.number()
		.int()
		.refine((n) => n >= 0, "must be non-negative"),
});

test("PromptValidationError is a PromptingPressError", () => {
	assert.ok(PromptValidationError.prototype instanceof PromptingPressError);
});

test("PromptRenderError is a PromptingPressError", () => {
	assert.ok(PromptRenderError.prototype instanceof PromptingPressError);
});

test("LoadError is a PromptingPressError", () => {
	assert.ok(LoadError.prototype instanceof PromptingPressError);
});

test("PromptLoadError is a PromptingPressError", () => {
	assert.ok(PromptLoadError.prototype instanceof PromptingPressError);
});

test("PromptingPressError has .errors array of {field, code, message}", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	try {
		p.render(GreetSchema, { name: "Ada", count: -1 });
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof PromptingPressError);
		assert.ok(Array.isArray(err.errors));
		assert.ok(err.errors.length > 0);
		for (const row of err.errors) {
			assert.equal(typeof row.field, "string");
			assert.equal(typeof row.code, "string");
			assert.equal(typeof row.message, "string");
		}
	}
});

test("PromptValidationError .errors has code 'validation' on every row", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	try {
		p.render(GreetSchema, { name: "Ada", count: -1 });
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof PromptValidationError);
		assert.ok(err.errors.every((r) => r.code === "validation"));
	}
});

test("validation error is NOT a ZodError (SC-006: no native type leaks)", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	try {
		p.render(GreetSchema, { name: "Ada", count: -1 });
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof PromptValidationError);
		assert.notEqual(err.constructor.name, "ZodError");
		assert.ok(!(err instanceof z.ZodError));
	}
});

test("LoadError .errors have code 'load' on every row", () => {
	try {
		Prompt.fromYaml("name: [unterminated");
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof LoadError);
		assert.ok(err.errors.every((r) => r.code === "load"));
	}
});

test("LoadError is NOT a PromptLoadError", () => {
	assert.ok(!(LoadError.prototype instanceof PromptLoadError));
});

test("PromptLoadError is NOT a LoadError", () => {
	assert.ok(!(PromptLoadError.prototype instanceof LoadError));
});

test("PromptLoadError from MemoryLoader miss has correct code in errors[0]", async () => {
	const loader = new MemoryLoader();
	try {
		await loader.load("missing");
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof PromptLoadError);
		assert.ok(err instanceof PromptingPressError);
		assert.equal(err.errors[0].code, "load_not_found");
	}
});

test("PromptRenderError .errors have code undefined_variable for agreement gap", () => {
	try {
		new Prompt({ name: "bad", role: "user", body: "{{ ghost }}", variables: {} });
		assert.fail("should have thrown");
	} catch (err) {
		assert.ok(err instanceof PromptRenderError);
		assert.ok(err.errors.some((r) => r.code === "undefined_variable"));
	}
});

test("Registry is NOT exported (spec-008 removal)", async () => {
	const mod = await import("prompting-press");
	assert.equal(mod.Registry, undefined);
});

test("render free function is NOT exported (spec-008 removal)", async () => {
	const mod = await import("prompting-press");
	assert.equal(mod.render, undefined);
});
