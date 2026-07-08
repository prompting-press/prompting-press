/**
 * Provenance attributes tests for the TypeScript facade — spec 018 (T009, T012, T013).
 *
 * T009 (US1): `result.provenanceAttributes()` is a 4-entry Record<string,string> with the
 *             fixed library-owned `prompting_press.prompt.*` keys; values equal the result
 *             fields.
 * T012 (US2): exactly 4 keys; map excludes text/guard/metadata/output_model (per binding).
 * T013 (US3): the four provenance fields remain publicly readable for custom key maps.
 *
 * Note: these tests require the napi addon to be built (`pnpm run build`). They run as
 * part of `pnpm test`.
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { Prompt } from "prompting-press";

// ── Key constants (spec 018 FR-003, library-owned — NOT OTel gen_ai.* keys) ─────────────

const KEY_NAME = "prompting_press.prompt.name";
const KEY_VARIANT = "prompting_press.prompt.variant";
const KEY_TEMPLATE_HASH = "prompting_press.prompt.template_hash";
const KEY_RENDER_HASH = "prompting_press.prompt.render_hash";

const EXPECTED_KEYS = [KEY_NAME, KEY_VARIANT, KEY_TEMPLATE_HASH, KEY_RENDER_HASH];

// A lowercase 64-char hex string — the SHA-256 provenance hash shape.
const HEX64 = /^[0-9a-f]{64}$/;

// ── Prompt fixture ────────────────────────────────────────────────────────────────────────

const GREET_YAML = `
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name:  { type: string,  trusted: true }
  count: { type: integer, trusted: true }
`;

// ── T009 (US1) ───────────────────────────────────────────────────────────────────────────

test("provenanceAttributes() returns a Record with exactly 4 entries", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });

	const attrs = result.provenanceAttributes();

	// Must be a plain object (Record<string,string>).
	assert.equal(typeof attrs, "object");
	assert.ok(attrs !== null);

	// Exactly 4 keys (FR-002, SC-004).
	const keys = Object.keys(attrs);
	assert.equal(keys.length, 4, `expected 4 keys, got ${keys.length}: ${keys.join(", ")}`);
});

test("provenanceAttributes() contains all four library-owned keys", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });
	const attrs = result.provenanceAttributes();

	for (const key of EXPECTED_KEYS) {
		assert.ok(Object.prototype.hasOwnProperty.call(attrs, key), `missing key: ${key}`);
	}
});

test("provenanceAttributes() values equal the result field values", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });
	const attrs = result.provenanceAttributes();

	// All values are strings (FR-001).
	for (const [k, v] of Object.entries(attrs)) {
		assert.equal(typeof v, "string", `value for ${k} must be a string, got ${typeof v}`);
	}

	// Values equal result fields (note: napi exposes camelCase getters on RenderResult).
	assert.equal(attrs[KEY_NAME], result.name, "KEY_NAME value must equal result.name");
	assert.equal(attrs[KEY_VARIANT], result.variant, "KEY_VARIANT value must equal result.variant");
	assert.equal(
		attrs[KEY_TEMPLATE_HASH],
		result.templateHash,
		"KEY_TEMPLATE_HASH value must equal result.templateHash",
	);
	assert.equal(
		attrs[KEY_RENDER_HASH],
		result.renderHash,
		"KEY_RENDER_HASH value must equal result.renderHash",
	);
});

test("provenanceAttributes() KEY_VARIANT is 'default' when no variant selected (INV-3)", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Grace", count: 0 });
	const attrs = result.provenanceAttributes();

	assert.equal(attrs[KEY_VARIANT], "default");
});

test("provenanceAttributes() KEY_TEMPLATE_HASH and KEY_RENDER_HASH are 64-char lowercase hex", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });
	const attrs = result.provenanceAttributes();

	assert.match(attrs[KEY_TEMPLATE_HASH], HEX64, `KEY_TEMPLATE_HASH: ${attrs[KEY_TEMPLATE_HASH]}`);
	assert.match(attrs[KEY_RENDER_HASH], HEX64, `KEY_RENDER_HASH: ${attrs[KEY_RENDER_HASH]}`);
});

test("provenanceAttributes() is deterministic — two identical renders produce equal maps", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs1 = p.render({ name: "Ada", count: 3 }).provenanceAttributes();
	const attrs2 = p.render({ name: "Ada", count: 3 }).provenanceAttributes();

	assert.deepEqual(attrs1, attrs2, "identical renders must produce identical attribute maps (SC-003)");
});

// ── T012 (US2) — exactly 4 keys; named exclusions ────────────────────────────────────────

test("provenanceAttributes() does NOT include text, guard, output_model, or metadata (FR-007)", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const attrs = p.render({ name: "Ada", count: 3 }).provenanceAttributes();

	// Explicit exclusion assertions (FR-007, SC-004, SEC-001).
	assert.ok(!("text" in attrs), "text must NOT be in provenanceAttributes()");
	assert.ok(!("guard" in attrs), "guard must NOT be in provenanceAttributes()");
	assert.ok(!("output_model" in attrs), "output_model must NOT be in provenanceAttributes()");
	assert.ok(!("metadata" in attrs), "metadata must NOT be in provenanceAttributes()");

	// Still exactly 4.
	assert.equal(Object.keys(attrs).length, 4);
});

test("provenanceAttributes() is a pure projection — calling it twice does not mutate the result", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });

	// Call twice — result fields must remain consistent.
	const a = result.provenanceAttributes();
	const b = result.provenanceAttributes();

	assert.deepEqual(a, b);
	// Result getters are unchanged.
	assert.equal(result.name, "greet");
	assert.equal(result.variant, "default");
});

// ── T013 (US3) — custom key map is buildable from public fields ───────────────────────────

test("result fields are publicly readable and allow a custom-keyed map (FR-008)", () => {
	const p = Prompt.fromYaml(GREET_YAML);
	const result = p.render({ name: "Ada", count: 3 });

	// Consumer builds their own map with custom keys — no dependency on the helper.
	const custom = {
		"my.prompt.name": result.name,
		"my.prompt.variant": result.variant,
		"my.prompt.template_hash": result.templateHash,
		"my.prompt.render_hash": result.renderHash,
	};

	assert.equal(custom["my.prompt.name"], "greet");
	assert.equal(custom["my.prompt.variant"], "default");
	assert.match(custom["my.prompt.template_hash"], HEX64);
	assert.match(custom["my.prompt.render_hash"], HEX64);
});
