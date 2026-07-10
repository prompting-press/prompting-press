/**
 * Integration gate — prompt.check() and CheckReport.
 *
 * Covers:
 * - Trusted-only prompt → report.passed() true, findings []
 * - Untrusted variable without guard → untrusted_without_guard finding
 * - Finding fields: prompt, kind, detail (detail mentions the variable name)
 * - Guard configured in metadata → report.passed() true
 * - check() is pure: calling twice returns same result
 * - report.passed() and report.isEmpty() are consistent
 * - report.findings is an array of Finding objects with stable fields
 */

import assert from "node:assert/strict";
import { test } from "node:test";
import { Prompt } from "prompting-press";

const KIND_UNTRUSTED = "untrusted_without_guard";

test("trusted-only prompt passes with empty findings", () => {
	const p = new Prompt({
		name: "greet",
		role: "user",
		body: "Hi {{ name }}",
		variables: { name: { type: "string", trusted: true } },
	});
	const report = p.check();
	assert.ok(report.passed(), "trusted-only prompt must pass");
	assert.ok(report.isEmpty(), "no findings → isEmpty()");
	assert.deepEqual(report.findings, []);
});

test("untrusted variable without guard produces untrusted_without_guard finding", () => {
	const p = new Prompt({
		name: "ask",
		role: "user",
		body: "{{ topic }}",
		variables: { topic: { type: "string", trusted: false } },
	});
	const report = p.check();
	assert.ok(!report.passed(), "unguarded untrusted var must fail");
	const kinds = report.findings.map((f) => f.kind);
	assert.ok(kinds.includes(KIND_UNTRUSTED), `expected ${KIND_UNTRUSTED}, got ${kinds}`);
});

test("finding has prompt, kind, detail fields; detail mentions the variable", () => {
	const p = new Prompt({
		name: "ask",
		role: "user",
		body: "{{ topic }}",
		variables: { topic: { type: "string", trusted: false } },
	});
	const f = p.check().findings[0];
	assert.equal(typeof f.prompt, "string");
	assert.equal(typeof f.kind, "string");
	assert.equal(typeof f.detail, "string");
	assert.equal(f.prompt, "ask");
	assert.ok(f.detail.includes("topic"), `detail must mention 'topic', got: ${f.detail}`);
});

test("guard configured in metadata → report.passed() true", () => {
	const p = new Prompt({
		name: "guarded",
		role: "user",
		body: "{{ payload }}",
		variables: { payload: { type: "string", trusted: false } },
		metadata: { guard: { enabled: true } },
	});
	assert.ok(p.check().passed(), "guard configured → check must pass");
});

test("check() is pure: calling twice returns same result", () => {
	const p = new Prompt({
		name: "ask",
		role: "user",
		body: "{{ topic }}",
		variables: { topic: { type: "string", trusted: false } },
	});
	const r1 = p.check();
	const r2 = p.check();
	assert.equal(r1.passed(), r2.passed());
	assert.equal(r1.findings.length, r2.findings.length);
	assert.equal(r1.findings[0]?.kind, r2.findings[0]?.kind);
});

test("report.passed() and report.isEmpty() are consistent", () => {
	const p = new Prompt({ name: "greet", role: "user", body: "Hi", variables: {} });
	const report = p.check();
	assert.equal(report.passed(), report.isEmpty());
});

test("report.findings is an array", () => {
	const p = new Prompt({ name: "greet", role: "user", body: "Hi", variables: {} });
	assert.ok(Array.isArray(p.check().findings));
});

test("check() is a method on a Prompt instance", () => {
	const p = new Prompt({ name: "greet", role: "user", body: "hi", variables: {} });
	assert.equal(typeof p.check, "function");
});
