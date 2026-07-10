#!/usr/bin/env node
/**
 * gen-root-changelog.mjs — generate the repo-root CHANGELOG.md by aggregating
 * the six per-package release-please CHANGELOG.md files into one file covering
 * every released version.
 *
 * WHY THIS EXISTS
 * ---------------
 * release-please is configured `linked-versions` across six components, but
 * `separate-pull-requests: false` still writes SIX independent CHANGELOG.md
 * files — one per package, each scoped to the commits release-please attributed
 * to that package's paths. There is no single root CHANGELOG.md.
 *
 * Worse for discoverability: with `include-component-in-tag: false` the ONE
 * aggregated GitHub Release at `vX.Y.Z` carries only the FIRST-listed package's
 * notes (crates/prompting-press-core) — the other five collide on the shared tag
 * and are skipped as harmless duplicates (verified against release-please source).
 * So a change that touches only, say, the TypeScript package is absent from both
 * "the" release page and any single package file.
 *
 * This script produces the missing canonical artifact: a root CHANGELOG.md that
 * is the deduplicated UNION of all six packages, per version. It reuses the
 * already-tested merge logic the docs site uses for its per-version changelog
 * page (docs/site/scripts/lib/merge-changelogs.mjs) — one source of truth for
 * "what shipped in version X", rather than a second, drifting merge.
 *
 * IDEMPOTENT + SELF-HEALING: it regenerates the WHOLE file from the per-package
 * changelogs every run, so re-running after a new release simply prepends the new
 * version. No incremental state; the per-package files remain the source of truth.
 *
 * Usage:  node scripts/gen-root-changelog.mjs [--check]
 *   (no args)  write CHANGELOG.md at the repo root
 *   --check    exit 1 if the on-disk CHANGELOG.md differs from freshly generated
 *              (for a CI drift gate), printing nothing to the file
 *
 * Stable Node.js APIs only (fs/path/url) — runs under node >=22.12.0.
 */

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { mergeChangelogsForVersion } from "../docs/site/scripts/lib/merge-changelogs.mjs";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const OUT_PATH = resolve(REPO_ROOT, "CHANGELOG.md");

// The six linked packages, in the same order release-please lists them. Keep in
// sync with release-please-config.json `packages` and docs/site's snapshot list.
const PACKAGE_PATHS = [
	"crates/prompting-press-core",
	"crates/prompting-press",
	"crates/prompting-press-py",
	"crates/prompting-press-node",
	"packages/python",
	"packages/typescript",
];

/** A version heading in a release-please changelog: `## [1.2.0]...` or `## 1.2.0 (date)`. */
const VERSION_HEADING_RE = /^##\s+\[?(\d+\.\d+\.\d+)\]?[^\n]*$/gm;
/** Capture the release date, if present, from the same heading line. */
const DATE_RE = /\((\d{4}-\d{2}-\d{2})\)/;

/**
 * Scan every package changelog and collect the set of released versions with the
 * first release date we see for each (release-please stamps the same date across
 * all six for a lockstep release). Returned newest-first (semver-descending).
 *
 * @returns {Array<{ version: string, date: string | null }>}
 */
function collectVersions() {
	/** @type {Map<string, string | null>} */
	const versionToDate = new Map();
	for (const pkgPath of PACKAGE_PATHS) {
		const changelogPath = resolve(REPO_ROOT, pkgPath, "CHANGELOG.md");
		if (!existsSync(changelogPath)) continue;
		const text = readFileSync(changelogPath, "utf-8");
		for (const line of text.split(/\r?\n/)) {
			VERSION_HEADING_RE.lastIndex = 0;
			const m = VERSION_HEADING_RE.exec(line);
			if (!m) continue;
			const version = m[1];
			const date = DATE_RE.exec(line)?.[1] ?? null;
			// Keep the first non-null date we encounter for a version.
			if (!versionToDate.has(version) || (!versionToDate.get(version) && date)) {
				versionToDate.set(version, date);
			}
		}
	}
	return [...versionToDate.entries()]
		.map(([version, date]) => ({ version, date }))
		.sort((a, b) => b.version.localeCompare(a.version, undefined, { numeric: true }));
}

/** Assemble the full root CHANGELOG.md body. */
function generate() {
	const versions = collectVersions();
	const header =
		"# Changelog\n\n" +
		"All notable changes to **Prompting Press**, aggregated across every published\n" +
		"package (the Rust crates, the Python distribution, and the npm package). Every\n" +
		"release is lockstepped — one version number moves across all packages at once.\n\n" +
		"This file is generated from the per-package changelogs by\n" +
		"[`scripts/gen-root-changelog.mjs`](scripts/gen-root-changelog.mjs); edit those,\n" +
		"not this file. Each entry links to its pull request and commit.\n";

	const blocks = [header];
	for (const { version, date } of versions) {
		const section = mergeChangelogsForVersion(REPO_ROOT, PACKAGE_PATHS, version);
		if (!section) continue; // a version no package has an entry for (shouldn't happen)
		const heading = date ? `## ${version} (${date})` : `## ${version}`;
		blocks.push(`${heading}\n\n${section}`);
	}
	// One trailing newline; blocks joined by a blank line.
	return `${blocks.join("\n\n")}\n`;
}

// `--section <version>` prints ONLY that version's merged union body to stdout
// (no file write, no root header). The release workflow pipes this into
// `gh release edit --notes` so the ONE aggregated GitHub Release at vX.Y.Z shows
// the union of every package's notes, not just the first package's (release-please
// gives the single component-less release only the first-listed package's notes;
// a root '.' package yields the COMPLEMENT of package paths, not the union —
// verified via dry-run — so overwriting the body here is the robust fix).
const sectionIdx = process.argv.indexOf("--section");
if (sectionIdx !== -1) {
	const version = process.argv[sectionIdx + 1];
	if (!version) {
		console.error("[gen-root-changelog] --section requires a version, e.g. --section 0.3.3");
		process.exit(2);
	}
	const section = mergeChangelogsForVersion(REPO_ROOT, PACKAGE_PATHS, version);
	if (!section) {
		console.error(`[gen-root-changelog] no changelog entry found for ${version}`);
		process.exit(1);
	}
	// Bare stdout write (no trailing decoration) so the caller can redirect it.
	process.stdout.write(`${section}\n`);
	process.exit(0);
}

const content = generate();

if (process.argv.includes("--check")) {
	const current = existsSync(OUT_PATH) ? readFileSync(OUT_PATH, "utf-8") : "";
	if (current !== content) {
		console.error(
			"[gen-root-changelog] CHANGELOG.md is stale — run `node scripts/gen-root-changelog.mjs` and commit.",
		);
		process.exit(1);
	}
	console.log("[gen-root-changelog] CHANGELOG.md is up to date.");
} else {
	writeFileSync(OUT_PATH, content, "utf-8");
	console.log(`[gen-root-changelog] Wrote ${OUT_PATH}`);
}
