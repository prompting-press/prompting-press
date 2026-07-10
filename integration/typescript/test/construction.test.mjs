/**
 * Integration gate — construction surface.
 *
 * Covers:
 * - Prompt.fromYaml / fromJson / fromToml with valid input
 * - new Prompt(shape) object path
 * - invalid input → LoadError (malformed text, shape violation)
 * - agreement violation at construction → PromptRenderError
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { LoadError, Prompt, PromptingPressError, PromptRenderError } from "prompting-press";

const VALID_YAML = `
name: base
role: user
body: "Hi {{ name }}"
variables:
  name: { type: string, trusted: true }
`;

const VALID_JSON = JSON.stringify({
	name: "base",
	role: "user",
	body: "Hi {{ name }}",
	variables: { name: { type: "string", trusted: true } },
});

const VALID_TOML = `
name = "base"
role = "user"
body = "Hi {{ name }}"

[variables.name]
type = "string"
trusted = true
`;

const VALID_SHAPE = {
	name: "base",
	role: "user",
	body: "Hi {{ name }}",
	variables: { name: { type: "string", trusted: true } },
};

test("fromYaml constructs a Prompt with correct accessors", () => {
	const p = Prompt.fromYaml(VALID_YAML);
	assert.equal(p.name, "base");
	assert.equal(p.role, "user");
	assert.ok(p.body.includes("{{ name }}"));
	assert.ok(p.variables !== undefined);
	assert.ok(p.variants !== undefined);
	assert.equal(p.outputModel, undefined);
});

test("fromJson constructs a Prompt with correct name", () => {
	const p = Prompt.fromJson(VALID_JSON);
	assert.equal(p.name, "base");
});

test("fromToml constructs a Prompt with correct name and body", () => {
	const p = Prompt.fromToml(VALID_TOML);
	assert.equal(p.name, "base");
	assert.ok(p.body.includes("{{ name }}"));
});

test("new Prompt(shape) constructs from a PromptDefinition object", () => {
	const p = new Prompt(VALID_SHAPE);
	assert.equal(p.name, "base");
	assert.equal(p.role, "user");
});

test("fromYaml with malformed YAML throws LoadError", () => {
	assert.throws(
		() => Prompt.fromYaml("name: [unterminated"),
		(err) => {
			assert.ok(err instanceof LoadError);
			assert.ok(err instanceof PromptingPressError);
			return true;
		},
	);
});

test("fromJson with malformed JSON throws LoadError", () => {
	assert.throws(
		() => Prompt.fromJson("{ not valid json"),
		(err) => {
			assert.ok(err instanceof LoadError);
			return true;
		},
	);
});

test("fromToml with malformed TOML throws LoadError", () => {
	assert.throws(
		() => Prompt.fromToml("name = [unterminated"),
		(err) => {
			assert.ok(err instanceof LoadError);
			return true;
		},
	);
});

test("new Prompt with missing body throws LoadError", () => {
	assert.throws(
		() => new Prompt({ name: "bad", role: "user" }),
		(err) => {
			assert.ok(err instanceof LoadError);
			return true;
		},
	);
});

test("fromYaml with missing body throws LoadError", () => {
	assert.throws(
		() => Prompt.fromYaml("name: bad\nrole: user\n"),
		(err) => {
			assert.ok(err instanceof LoadError);
			return true;
		},
	);
});

test("undeclared template variable throws PromptRenderError at construction", () => {
	assert.throws(
		() => new Prompt({ name: "bad", role: "user", body: "{{ ghost }}", variables: {} }),
		(err) => {
			assert.ok(err instanceof PromptRenderError);
			assert.ok(err.errors.some((r) => r.code === "undefined_variable"));
			return true;
		},
	);
});

test("three construction paths produce a Prompt for the same definition", () => {
	const pYaml = Prompt.fromYaml(VALID_YAML);
	const pJson = Prompt.fromJson(VALID_JSON);
	const pObj = new Prompt(VALID_SHAPE);
	assert.equal(pYaml.name, pJson.name);
	assert.equal(pJson.name, pObj.name);
});
