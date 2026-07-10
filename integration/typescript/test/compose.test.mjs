/**
 * Integration gate — Composition.
 *
 * Covers:
 * - Composition.fromMessages and append path preserve order and roles
 * - append returns void (non-fluent)
 * - Empty composition resolves to []
 * - Invalid vars at append → PromptValidationError, nothing stored
 * - Invalid entry in fromMessages → throws, no Composition returned
 * - Unknown variant at resolve → PromptRenderError, no partial result
 * - Variant entry field selects named arm
 * - No .chain() on class or instance (FR-013)
 * - Static (no-schema) entries accepted
 * - resolve() takes no arguments (post-008, no Registry)
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import {
	Composition,
	Prompt,
	PromptingPressError,
	PromptRenderError,
	PromptValidationError,
} from "prompting-press";
import { z } from "zod";

const Named = z.object({ name: z.string().min(1, "name must be non-empty") });
const EmptyVars = z.object({});

const SYS_PROMPT = new Prompt({
	name: "sys",
	role: "system",
	body: "You are a helpful assistant.",
	variables: {},
});

const GREET_PROMPT = new Prompt({
	name: "greet",
	role: "user",
	body: "Hi {{ name }}",
	variables: { name: { type: "string", trusted: true } },
});

const FAREWELL_PROMPT = new Prompt({
	name: "farewell",
	role: "user",
	body: "Bye {{ name }}",
	variables: { name: { type: "string", trusted: true } },
});

const VARIANT_PROMPT = new Prompt({
	name: "salute",
	role: "user",
	body: "Hi {{ name }}",
	variables: { name: { type: "string", trusted: true } },
	variants: { formal: { body: "Good day, {{ name }}" } },
});

test("Composition.fromMessages preserves order and roles", () => {
	const comp = Composition.fromMessages([
		{ prompt: SYS_PROMPT, schema: EmptyVars, data: {} },
		{ prompt: GREET_PROMPT, schema: Named, data: { name: "Ada" } },
	]);
	assert.equal(comp.length, 2);
	const msgs = comp.resolve();
	assert.equal(msgs.length, 2);
	assert.equal(msgs[0].role, "system");
	assert.equal(msgs[0].text, "You are a helpful assistant.");
	assert.equal(msgs[1].role, "user");
	assert.equal(msgs[1].text, "Hi Ada");
});

test("append preserves order, roles, text; returns void", () => {
	const comp = new Composition();
	const ret = comp.append({ prompt: SYS_PROMPT, schema: EmptyVars, data: {} });
	assert.equal(ret, undefined, "append must be non-fluent (void)");
	comp.append({ prompt: GREET_PROMPT, schema: Named, data: { name: "Bo" } });
	assert.equal(comp.length, 2);
	const msgs = comp.resolve();
	assert.deepEqual(
		msgs.map((m) => m.text),
		["You are a helpful assistant.", "Hi Bo"],
	);
});

test("empty Composition resolves to []", () => {
	const comp = new Composition();
	assert.equal(comp.length, 0);
	assert.deepEqual(comp.resolve(), []);
});

test("invalid vars at append throw PromptValidationError; nothing stored", () => {
	const comp = new Composition();
	comp.append({ prompt: GREET_PROMPT, schema: Named, data: { name: "ok" } });
	assert.equal(comp.length, 1);
	assert.throws(
		() => comp.append({ prompt: GREET_PROMPT, schema: Named, data: { name: "" } }),
		(err) => {
			assert.ok(err instanceof PromptValidationError);
			return true;
		},
	);
	assert.equal(comp.length, 1, "rejected append must store nothing");
});

test("fromMessages with invalid entry throws; no Composition returned", () => {
	assert.throws(
		() =>
			Composition.fromMessages([
				{ prompt: GREET_PROMPT, schema: Named, data: { name: "ok" } },
				{ prompt: GREET_PROMPT, schema: Named, data: { name: "" } },
			]),
		PromptValidationError,
	);
});

test("unknown variant at resolve throws PromptRenderError; no partial result returned", () => {
	const comp = Composition.fromMessages([
		{ prompt: GREET_PROMPT, data: { name: "X" }, variant: "nonexistent" },
	]);
	const sentinel = Symbol("not-set");
	let result = sentinel;
	assert.throws(
		() => {
			result = comp.resolve();
		},
		(err) => {
			assert.ok(err instanceof PromptRenderError);
			return true;
		},
	);
	assert.equal(result, sentinel, "resolve must throw, not return partial");
});

test("variant entry field selects named arm", () => {
	const comp = Composition.fromMessages([
		{ prompt: VARIANT_PROMPT, schema: Named, data: { name: "Di" }, variant: "formal" },
	]);
	assert.equal(comp.resolve()[0].text, "Good day, Di");
});

test("no variant field defaults to the reserved default arm", () => {
	const comp = Composition.fromMessages([
		{ prompt: VARIANT_PROMPT, schema: Named, data: { name: "Eli" } },
	]);
	assert.equal(comp.resolve()[0].text, "Hi Eli");
});

test("no .chain() on class or instance (FR-013)", () => {
	assert.equal(Composition.prototype.chain, undefined);
	const comp = new Composition();
	assert.equal(comp.chain, undefined);
});

test("static (no-schema) entries are accepted and marshaled directly", () => {
	const comp = Composition.fromMessages([
		{ prompt: SYS_PROMPT, data: {} },
		{ prompt: GREET_PROMPT, data: { name: "Zed" } },
	]);
	assert.deepEqual(
		comp.resolve().map((m) => m.text),
		["You are a helpful assistant.", "Hi Zed"],
	);
});

test("resolve() takes no argument (no Registry in post-008 surface)", () => {
	const comp = Composition.fromMessages([{ prompt: SYS_PROMPT, data: {} }]);
	const msgs = comp.resolve();
	assert.equal(msgs.length, 1);
});

test("mixed system + two user composition resolves to 3 ordered messages", () => {
	const comp = Composition.fromMessages([
		{ prompt: SYS_PROMPT, schema: EmptyVars, data: {} },
		{ prompt: GREET_PROMPT, schema: Named, data: { name: "Ada" } },
		{ prompt: FAREWELL_PROMPT, schema: Named, data: { name: "Bo" } },
	]);
	const msgs = comp.resolve();
	assert.equal(msgs.length, 3);
	assert.deepEqual(
		msgs.map((m) => [m.role, m.text]),
		[
			["system", "You are a helpful assistant."],
			["user", "Hi Ada"],
			["user", "Bye Bo"],
		],
	);
});

test("PromptRenderError and PromptValidationError are PromptingPressError instances", () => {
	assert.ok(PromptRenderError.prototype instanceof PromptingPressError);
	assert.ok(PromptValidationError.prototype instanceof PromptingPressError);
});
