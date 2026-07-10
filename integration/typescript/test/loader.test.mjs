/**
 * Integration gate — loaders (MemoryLoader, FileSystemLoader, custom PromptLoader).
 *
 * Covers:
 * - MemoryLoader: hit returns raw text; miss → PromptLoadError (load_not_found)
 * - MemoryLoader accepts Record and Map
 * - FileSystemLoader: hit returns raw text; miss → PromptLoadError (load_not_found)
 * - FileSystemLoader: traversal rejection (../, absolute, NUL, backslash, empty, ".", foo/./bar)
 * - FileSystemLoader: read cap exceeded → PromptLoadError (load_io)
 * - Custom async loader implementing PromptLoader interface
 * - Composition: loader.load(k) + Prompt.fromYaml(raw)
 * - PromptLoadError is a PromptingPressError; LoadError is NOT a PromptLoadError (SC-010)
 * - LOAD_NOT_FOUND and LOAD_IO code constants
 * - load() returns Promise<string> (async interface)
 */

import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { test } from "node:test";
import {
	FileSystemLoader,
	LOAD_IO,
	LOAD_NOT_FOUND,
	LoadError,
	MemoryLoader,
	Prompt,
	PromptingPressError,
	PromptLoadError,
} from "prompting-press";

const VALID_YAML = `name: test\nrole: user\nbody: "Hello {{ name }}"\nvariables:\n  name: { type: string, trusted: true }\n`;

async function withPromptDir(key, content = VALID_YAML) {
	const dir = await mkdtemp(path.join(tmpdir(), "pp-int-loader-"));
	await writeFile(path.join(dir, `${key}.yaml`), content, "utf-8");
	return dir;
}

// ─── MemoryLoader ─────────────────────────────────────────────────────────────

test("MemoryLoader hit returns mapped raw text", async () => {
	const loader = new MemoryLoader({ greet: VALID_YAML });
	const raw = await loader.load("greet");
	assert.equal(raw, VALID_YAML);
});

test("MemoryLoader miss rejects with PromptLoadError (load_not_found)", async () => {
	const loader = new MemoryLoader();
	await assert.rejects(
		() => loader.load("missing"),
		(err) => {
			assert.ok(err instanceof PromptLoadError);
			assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
			return true;
		},
	);
});

test("MemoryLoader accepts a Map", async () => {
	const loader = new MemoryLoader(new Map([["greet", VALID_YAML]]));
	const raw = await loader.load("greet");
	assert.equal(raw, VALID_YAML);
});

// ─── FileSystemLoader ─────────────────────────────────────────────────────────

test("FileSystemLoader hit returns raw text", async () => {
	const dir = await withPromptDir("greet");
	try {
		const loader = new FileSystemLoader(dir);
		const raw = await loader.load("greet");
		assert.ok(raw.includes("name: test"));
	} finally {
		await rm(dir, { recursive: true, force: true });
	}
});

test("FileSystemLoader miss rejects with PromptLoadError (load_not_found)", async () => {
	const dir = await withPromptDir("greet");
	try {
		const loader = new FileSystemLoader(dir);
		await assert.rejects(
			() => loader.load("missing_key"),
			(err) => {
				assert.ok(err instanceof PromptLoadError);
				assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
				return true;
			},
		);
	} finally {
		await rm(dir, { recursive: true, force: true });
	}
});

const TRAVERSAL_KEYS = [
	"../secret",
	"../../etc/passwd",
	"/etc/passwd",
	"/absolute",
	"foo\0bar",
	"foo\\bar",
	"",
	".",
	"foo/./bar",
	"../",
];

for (const badKey of TRAVERSAL_KEYS) {
	test(`FileSystemLoader rejects traversal key: ${JSON.stringify(badKey)}`, async () => {
		const dir = await withPromptDir("greet");
		try {
			const loader = new FileSystemLoader(dir);
			await assert.rejects(
				() => loader.load(badKey),
				(err) => {
					assert.ok(err instanceof PromptLoadError);
					assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
					return true;
				},
			);
		} finally {
			await rm(dir, { recursive: true, force: true });
		}
	});
}

test("FileSystemLoader rejects symlink escape", async () => {
	const root = await mkdtemp(path.join(tmpdir(), "pp-int-sym-"));
	try {
		const secretDir = path.join(root, "secret");
		await mkdir(secretDir);
		await writeFile(path.join(secretDir, "secret.yaml"), "secret content", "utf-8");
		const baseDir = path.join(root, "base");
		await mkdir(baseDir);
		await symlink(secretDir, path.join(baseDir, "escape"));
		const loader = new FileSystemLoader(baseDir);
		await assert.rejects(
			() => loader.load("escape/secret"),
			(err) => {
				assert.ok(err instanceof PromptLoadError);
				assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
				return true;
			},
		);
	} finally {
		await rm(root, { recursive: true, force: true });
	}
});

test("FileSystemLoader read cap exceeded rejects with load_io", async () => {
	const dir = await mkdtemp(path.join(tmpdir(), "pp-int-cap-"));
	try {
		await writeFile(path.join(dir, "big.yaml"), "x".repeat(100), "utf-8");
		const loader = new FileSystemLoader(dir, ".yaml", 50);
		await assert.rejects(
			() => loader.load("big"),
			(err) => {
				assert.ok(err instanceof PromptLoadError);
				assert.equal(err.errors[0].code, LOAD_IO);
				return true;
			},
		);
	} finally {
		await rm(dir, { recursive: true, force: true });
	}
});

// ─── Custom async loader implementing PromptLoader ────────────────────────────

test("custom async loader implementing PromptLoader interface works with fromYaml", async () => {
	const store = { greet: VALID_YAML };
	const customLoader = {
		load: async (key) => {
			if (key in store) return store[key];
			const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
			throw new PromptLoadError(`key not found: \`${key}\``, errors);
		},
	};
	const raw = await customLoader.load("greet");
	const p = Prompt.fromYaml(raw);
	assert.equal(p.name, "test");
});

test("custom loader miss is a PromptLoadError distinct from LoadError", async () => {
	const failLoader = {
		load: async (key) => {
			const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
			throw new PromptLoadError(`key not found: \`${key}\``, errors);
		},
	};
	await assert.rejects(
		() => failLoader.load("any"),
		(err) => {
			assert.ok(err instanceof PromptLoadError);
			assert.ok(!(err instanceof LoadError));
			return true;
		},
	);
});

// ─── Composition: load then parse ─────────────────────────────────────────────

test("load returns raw text; compose with Prompt.fromYaml (two separate steps)", async () => {
	const loader = new MemoryLoader({ greet: VALID_YAML });
	const raw = await loader.load("greet");
	const p = Prompt.fromYaml(raw);
	assert.equal(p.name, "test");
	const result = p.render({ name: "Ada" });
	assert.equal(result.text, "Hello Ada");
});

// ─── Error hierarchy ──────────────────────────────────────────────────────────

test("PromptLoadError is a PromptingPressError", async () => {
	const loader = new MemoryLoader();
	await assert.rejects(
		() => loader.load("missing"),
		(err) => {
			assert.ok(err instanceof PromptingPressError);
			assert.ok(err instanceof PromptLoadError);
			return true;
		},
	);
});

test("SC-010: PromptLoadError does NOT catch malformed-YAML LoadError", () => {
	assert.throws(
		() => Prompt.fromYaml("not: valid: yaml: : :"),
		(err) => {
			assert.ok(err instanceof LoadError);
			assert.ok(!(err instanceof PromptLoadError));
			return true;
		},
	);
});

test("LoadError is not a PromptLoadError", () => {
	assert.ok(!(LoadError.prototype instanceof PromptLoadError));
});

// ─── Code constants ───────────────────────────────────────────────────────────

test("LOAD_NOT_FOUND constant equals 'load_not_found'", () => {
	assert.equal(LOAD_NOT_FOUND, "load_not_found");
});

test("LOAD_IO constant equals 'load_io'", () => {
	assert.equal(LOAD_IO, "load_io");
});
