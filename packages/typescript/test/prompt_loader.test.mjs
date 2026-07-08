/**
 * Spec 019 — PromptLoader tests for the TypeScript facade (T011).
 *
 * Covers:
 * - FileSystemLoader: hit + miss (load_not_found)
 * - Traversal guard: ../, absolute, NUL, backslash, symlink escape,
 *   key="", key=".", intermediate "." (SC-008)
 * - Read cap: exceed maxBytes → load_io (SC-009)
 * - MemoryLoader: hit + miss (async)
 * - Function coercion (FR-001)
 * - Compose Prompt.fromYaml(await loader.load(k)) (US1)
 * - catch (PromptLoadError) does NOT catch malformed-YAML LoadError (SC-010)
 * - Dependency-injection: swap FileSystemLoader ↔ MemoryLoader (SC-002)
 */

import assert from "node:assert/strict";
import { mkdtemp, rm, symlink, writeFile, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { test, describe } from "node:test";
import { fileURLToPath } from "node:url";

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

// ─── fixtures ──────────────────────────────────────────────────────────────

const VALID_YAML = `name: test\nrole: user\nbody: "Hello {{ name }}"\nvariables:\n  name: { type: string, trusted: true }\n`;

/** Create a temp directory, write one YAML file for `key`, return the dir path. */
async function withPromptDir(key, content = VALID_YAML) {
  const dir = await mkdtemp(path.join(tmpdir(), "pp-loader-test-"));
  await writeFile(path.join(dir, `${key}.yaml`), content, "utf-8");
  return dir;
}

// ─── FileSystemLoader: happy path ──────────────────────────────────────────

test("FileSystemLoader hit returns raw text", async () => {
  const dir = await withPromptDir("greet");
  try {
    const loader = new FileSystemLoader(dir);
    const raw = await loader.load("greet");
    assert.ok(raw.includes("name: test"), "raw text contains YAML content");
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("FileSystemLoader miss rejects with load_not_found", async () => {
  const dir = await withPromptDir("greet");
  try {
    const loader = new FileSystemLoader(dir);
    await assert.rejects(() => loader.load("missing_key"), (err) => {
      assert.ok(err instanceof PromptLoadError, "is PromptLoadError");
      assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
      return true;
    });
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("FileSystemLoader custom suffix", async () => {
  const dir = await mkdtemp(path.join(tmpdir(), "pp-loader-test-"));
  try {
    await writeFile(path.join(dir, "greet.json"), VALID_YAML, "utf-8");
    const loader = new FileSystemLoader(dir, ".json");
    const raw = await loader.load("greet");
    assert.ok(raw.length > 0);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

// ─── FileSystemLoader: traversal guard (SC-008) ────────────────────────────

const BAD_KEYS = [
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

for (const badKey of BAD_KEYS) {
  test(`traversal guard rejects key: ${JSON.stringify(badKey)}`, async () => {
    const dir = await withPromptDir("greet");
    try {
      const loader = new FileSystemLoader(dir);
      await assert.rejects(() => loader.load(badKey), (err) => {
        assert.ok(err instanceof PromptLoadError, `is PromptLoadError for key ${JSON.stringify(badKey)}`);
        assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
        return true;
      });
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
}

test("traversal guard: escaping symlink rejected", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "pp-loader-sym-"));
  try {
    const secretDir = path.join(root, "secret");
    await mkdir(secretDir);
    await writeFile(path.join(secretDir, "secret.yaml"), "secret content", "utf-8");

    const baseDir = path.join(root, "base");
    await mkdir(baseDir);
    await symlink(secretDir, path.join(baseDir, "escape"));

    const loader = new FileSystemLoader(baseDir);
    await assert.rejects(() => loader.load("escape/secret"), (err) => {
      assert.ok(err instanceof PromptLoadError);
      assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
      return true;
    });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("traversal error message does not leak base path", async () => {
  const dir = await withPromptDir("greet");
  try {
    const loader = new FileSystemLoader(dir);
    await assert.rejects(() => loader.load("../../../etc/passwd"), (err) => {
      assert.ok(err instanceof PromptLoadError);
      const msg = err.errors[0].message;
      assert.ok(!msg.includes(dir), `base path leaked: ${msg}`);
      // The logical key IS in the message.
      assert.ok(msg.includes("../../../etc/passwd"), `key missing from message: ${msg}`);
      return true;
    });
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

// ─── FileSystemLoader: read cap (SC-009) ───────────────────────────────────

test("read cap exceeded rejects with load_io", async () => {
  const dir = await mkdtemp(path.join(tmpdir(), "pp-loader-test-"));
  try {
    await writeFile(path.join(dir, "big.yaml"), "x".repeat(100), "utf-8");
    const loader = new FileSystemLoader(dir, ".yaml", 50);
    await assert.rejects(() => loader.load("big"), (err) => {
      assert.ok(err instanceof PromptLoadError);
      assert.equal(err.errors[0].code, LOAD_IO);
      return true;
    });
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("read cap at exactly the limit succeeds", async () => {
  const dir = await mkdtemp(path.join(tmpdir(), "pp-loader-test-"));
  try {
    const content = "x".repeat(50);
    await writeFile(path.join(dir, "exact.yaml"), content, "utf-8");
    const loader = new FileSystemLoader(dir, ".yaml", 50);
    const result = await loader.load("exact");
    assert.equal(result, content);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

// ─── MemoryLoader ───────────────────────────────────────────────────────────

test("MemoryLoader hit returns mapped text", async () => {
  const loader = new MemoryLoader({ greet: VALID_YAML });
  const raw = await loader.load("greet");
  assert.equal(raw, VALID_YAML);
});

test("MemoryLoader miss rejects with load_not_found", async () => {
  const loader = new MemoryLoader();
  await assert.rejects(() => loader.load("missing"), (err) => {
    assert.ok(err instanceof PromptLoadError);
    assert.equal(err.errors[0].code, LOAD_NOT_FOUND);
    return true;
  });
});

test("MemoryLoader accepts a Map", async () => {
  const loader = new MemoryLoader(new Map([["greet", VALID_YAML]]));
  const raw = await loader.load("greet");
  assert.equal(raw, VALID_YAML);
});

// ─── Function coercion (FR-001) ────────────────────────────────────────────

test("plain async function satisfies loader contract", async () => {
  const myLoader = async (key) => {
    if (key === "greet") return VALID_YAML;
    const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
    throw new PromptLoadError(`key not found: \`${key}\``, errors);
  };
  const raw = await myLoader("greet");
  const prompt = Prompt.fromYaml(raw);
  assert.equal(prompt.name, "test");
});

test("function coercion: failure surfaces as PromptLoadError", async () => {
  const myLoader = async (key) => {
    const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
    throw new PromptLoadError(`key not found: \`${key}\``, errors);
  };
  await assert.rejects(() => myLoader("any"), (err) => {
    assert.ok(err instanceof PromptLoadError);
    return true;
  });
});

// ─── SC-010: PromptLoadError is distinct from LoadError ────────────────────

test("PromptLoadError does not catch malformed-YAML LoadError", () => {
  assert.throws(() => Prompt.fromYaml("not: valid: yaml: : :"), (err) => {
    assert.ok(err instanceof LoadError, `expected LoadError, got ${err?.constructor?.name}`);
    assert.ok(!(err instanceof PromptLoadError), "LoadError must not be PromptLoadError");
    return true;
  });
});

test("PromptLoadError is a PromptingPressError", async () => {
  const loader = new MemoryLoader();
  await assert.rejects(() => loader.load("missing"), (err) => {
    assert.ok(err instanceof PromptingPressError, "is PromptingPressError");
    assert.ok(err instanceof PromptLoadError, "is PromptLoadError");
    return true;
  });
});

test("LoadError is not a PromptLoadError", () => {
  assert.ok(!(LoadError.prototype instanceof PromptLoadError));
});

// ─── Compose: load + parse (US1) ───────────────────────────────────────────

test("compose: FileSystemLoader + Prompt.fromYaml", async () => {
  const dir = await withPromptDir("greet");
  try {
    const loader = new FileSystemLoader(dir);
    const raw = await loader.load("greet");
    const prompt = Prompt.fromYaml(raw);
    assert.equal(prompt.name, "test");
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("compose: MemoryLoader + Prompt.fromYaml", async () => {
  const loader = new MemoryLoader({ greet: VALID_YAML });
  const prompt = Prompt.fromYaml(await loader.load("greet"));
  assert.equal(prompt.name, "test");
});

// ─── SC-002: dependency injection ──────────────────────────────────────────

async function loadAndParse(loader, key) {
  return Prompt.fromYaml(await loader.load(key));
}

test("swap FileSystemLoader for MemoryLoader without changing call site", async () => {
  const dir = await withPromptDir("greet");
  try {
    const fsLoader = new FileSystemLoader(dir);
    const fromFs = await loadAndParse(fsLoader, "greet");

    const memLoader = new MemoryLoader({ greet: VALID_YAML });
    const fromMem = await loadAndParse(memLoader, "greet");

    assert.equal(fromFs.name, fromMem.name);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

// ─── US3: custom loader (T015) ──────────────────────────────────────────────

test("custom loader is interchangeable with built-ins", async () => {
  const store = { greet: VALID_YAML };
  const customLoader = {
    load: async (key) => {
      if (key in store) return store[key];
      const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
      throw new PromptLoadError(`key not found: \`${key}\``, errors);
    },
  };
  const fromCustom = await loadAndParse(customLoader, "greet");
  const fromBuiltin = await loadAndParse(new MemoryLoader(store), "greet");
  assert.equal(fromCustom.name, fromBuiltin.name);
});

test("custom loader failure is a PromptLoadError distinct from parse error", async () => {
  const failingLoader = {
    load: async (key) => {
      const errors = [{ field: "", code: LOAD_NOT_FOUND, message: `key not found: \`${key}\`` }];
      throw new PromptLoadError(`key not found: \`${key}\``, errors);
    },
  };
  await assert.rejects(() => failingLoader.load("greet"), (err) => {
    assert.ok(err instanceof PromptLoadError);
    assert.ok(!(err instanceof LoadError));
    return true;
  });
});

// ─── LOAD_NOT_FOUND / LOAD_IO code constants ────────────────────────────────

test("LOAD_NOT_FOUND constant is the stable string", () => {
  assert.equal(LOAD_NOT_FOUND, "load_not_found");
});

test("LOAD_IO constant is the stable string", () => {
  assert.equal(LOAD_IO, "load_io");
});
