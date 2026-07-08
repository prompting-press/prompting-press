/**
 * Integration gate — derive() and MergeStrategy.
 *
 * Covers:
 * - derive() default (Replace) is non-breaking: original Prompt untouched
 * - derive() with MergeStrategy.Replace is byte-identical to no-strategy
 * - derive() with MergeStrategy.Merge unions variables, variants, metadata (child-wins on collision)
 * - derive() carries validators forward from source by default
 * - derive(overlay, { validators }) overrides the bound validator on derived prompt
 * - Base immutability after derive with Merge
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { MergeStrategy, Prompt, PromptValidationError } from "prompting-press";
import { z } from "zod";
import { PromptRenderError } from "prompting-press";

const BASE_SHAPE = {
	name: "base",
	role: "user",
	body: "{{ extraction }}",
	variables: { extraction: { type: "string", trusted: true } },
};

const GREET_SHAPE = {
	name: "greet",
	role: "user",
	body: "Hi {{ name }}, you have {{ count }} messages",
	variables: {
		name: { type: "string", trusted: true },
		count: { type: "integer", trusted: true },
	},
};

const GreetSchema = z.object({
	name: z.string().min(1),
	count: z
		.number()
		.int()
		.refine((n) => n >= 0, "must be non-negative"),
});

test("derive() Replace default: derived has overlay applied, original is untouched", () => {
	const original = new Prompt(GREET_SHAPE);
	const originalBody = original.body;

	const derived = original.derive({ body: "Hey {{ name }}, you have {{ count }} messages" });

	assert.equal(derived.body, "Hey {{ name }}, you have {{ count }} messages");
	assert.equal(original.body, originalBody, "original body unchanged");
});

test("derive() with MergeStrategy.Replace is byte-identical to no-strategy", () => {
	const base = new Prompt(BASE_SHAPE);
	const overlay = { body: "Processed: {{ extraction }}" };
	const viaNoStrategy = base.derive(overlay);
	const viaReplace = base.derive(overlay, { strategy: MergeStrategy.Replace });
	assert.equal(viaNoStrategy.body, viaReplace.body);
	assert.equal(viaNoStrategy.name, viaReplace.name);
});

test("derive() with MergeStrategy.Merge unions variables; base vars unchanged", () => {
	const base = new Prompt(BASE_SHAPE);
	const derived = base.derive(
		{
			body: "{{ extraction }} {{ sentiment }}",
			variables: { sentiment: { type: "string", trusted: true } },
		},
		{ strategy: MergeStrategy.Merge },
	);
	const vars = derived.variables ?? {};
	assert.ok("extraction" in vars, "base var retained");
	assert.ok("sentiment" in vars, "overlay var added");
	assert.equal(Object.keys(vars).length, 2);

	// Base is untouched.
	const baseVars = base.variables ?? {};
	assert.ok(!("sentiment" in baseVars), "base variables unchanged after Merge");
});

test("derive() with MergeStrategy.Merge unions variants", () => {
	const base = new Prompt({
		name: "base",
		role: "user",
		body: "{{ name }}",
		variables: { name: { type: "string", trusted: true } },
		variants: { v1: { body: "v1: {{ name }}" } },
	});
	const derived = base.derive(
		{ variants: { v2: { body: "v2: {{ name }}" } } },
		{ strategy: MergeStrategy.Merge },
	);
	const variants = derived.variants ?? {};
	assert.ok("v1" in variants, "base variant retained");
	assert.ok("v2" in variants, "overlay variant added");
});

test("derive() with MergeStrategy.Merge: child wins on metadata collision", () => {
	const base = new Prompt({
		name: "base",
		role: "user",
		body: "{{ name }}",
		variables: { name: { type: "string", trusted: true } },
		metadata: { base_key: "base_val", shared_key: "from_base" },
	});
	const derived = base.derive(
		{ metadata: { overlay_key: "overlay_val", shared_key: "from_overlay" } },
		{ strategy: MergeStrategy.Merge },
	);
	const meta = derived.metadata ?? {};
	assert.equal(meta.base_key, "base_val", "base key retained");
	assert.equal(meta.overlay_key, "overlay_val", "overlay key added");
	assert.equal(meta.shared_key, "from_overlay", "child wins on collision");
});

test("derive() carries validators forward from source by default", () => {
	const p = new Prompt(GREET_SHAPE, GreetSchema);
	const derived = p.derive({ body: "Greetings {{ name }}, you have {{ count }} messages" });
	// Inherited GreetSchema rejects count=-1.
	assert.throws(() => derived.render({ name: "Ada", count: -1 }), PromptValidationError);
});

test("derive(overlay, { validators }) overrides bound validator on derived prompt", () => {
	const p = new Prompt(GREET_SHAPE, GreetSchema);
	const NoCheck = z.object({ name: z.string(), count: z.number() });
	const derived = p.derive(
		{ body: "Hi {{ name }}, you have {{ count }} messages" },
		{ validators: NoCheck },
	);
	// NoCheck has no refine on count → -1 passes.
	const result = derived.render({ name: "Eli", count: -1 });
	assert.equal(result.text, "Hi Eli, you have -1 messages");
});

test("derive() with overlay introducing undeclared variable throws PromptRenderError", () => {
	const base = new Prompt(GREET_SHAPE);
	assert.throws(
		() => base.derive({ body: "{{ name }} {{ ghost }}" }),
		(err) => {
			assert.ok(err instanceof PromptRenderError);
			assert.ok(err.errors.some((r) => r.code === "undefined_variable"));
			return true;
		},
	);
});
