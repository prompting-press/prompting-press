/**
 * merge-changelogs.mjs — aggregate the six per-package release-please
 * CHANGELOG.md files into one deduplicated section for a given version.
 *
 * release-please is configured with `linked-versions` across six components
 * (crates/prompting-press-core, crates/prompting-press, crates/prompting-press-py,
 * crates/prompting-press-node, packages/python, packages/typescript) — they
 * always share a version number, but `separate-pull-requests: false` still
 * writes SIX independent CHANGELOG.md files, one per package, each covering
 * only the commits release-please attributed to that package's paths. There
 * is no single root CHANGELOG.md (snapshot-docs.mjs previously assumed one
 * existed and always fell through to a "no changelog entry" stub as a
 * result). This module reads all six, extracts each one's section for the
 * requested version, and merges them into one deduplicated list.
 *
 * Every release-please "Features"/"Bug Fixes" bullet ends with a trailing
 * `([shorthash](commit-url))` — that hash is a stable, exact dedup key
 * (unlike PR number or bullet text, which can't disambiguate two different
 * commits landed under the same PR). "⚠ BREAKING CHANGES" bullets carry no
 * commit hash (release-please emits them from the commit footer, not a
 * per-commit link), so those are deduped by exact bullet text instead —
 * confirmed byte-identical across packages for the same PR.
 *
 * Stable Node.js APIs only (fs/path) — runs under node >=22.12.0.
 */

import { existsSync, readFileSync } from "node:fs";

/** release-please's fixed subsection order; anything else is passed through, kept last. */
const KNOWN_SECTIONS = ["⚠ BREAKING CHANGES", "Features", "Bug Fixes"];

/**
 * Extract the changelog section for a specific version from a release-please
 * CHANGELOG body. Matches a heading line containing the exact version
 * (`## [1.2.0]`, `## 1.2.0`, `## [1.2.0](url) (date)`) and returns everything
 * up to the next `## ` heading. Returns null if not found.
 *
 * (Mirrors snapshot-docs.mjs's extractChangelogSection — kept independent
 * rather than shared so each script's contract stays self-contained.)
 *
 * @param {string} changelogText
 * @param {string} ver
 * @returns {string | null}
 */
export function extractChangelogSection(changelogText, ver) {
	const lines = changelogText.split(/\r?\n/);
	const verEsc = ver.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
	const headingRe = new RegExp(`^##\\s+\\[?${verEsc}\\b`);
	const anyHeadingRe = /^##\s+/;
	let start = -1;
	for (let i = 0; i < lines.length; i++) {
		if (headingRe.test(lines[i])) {
			start = i;
			break;
		}
	}
	if (start === -1) return null;
	let end = lines.length;
	for (let i = start + 1; i < lines.length; i++) {
		if (anyHeadingRe.test(lines[i])) {
			end = i;
			break;
		}
	}
	return lines.slice(start, end).join("\n").trim();
}

/**
 * Split a version section's body into { sectionTitle -> bullet[] }, in the
 * order subsections first appear. Only "### " subsections and their "* "
 * bullets are recognised; anything else (blank lines, the version heading
 * itself) is discarded — a merged page re-renders its own heading.
 *
 * @param {string} sectionText
 * @returns {Map<string, string[]>}
 */
function parseSubsections(sectionText) {
	const subsections = new Map();
	let current = null;
	for (const line of sectionText.split(/\r?\n/)) {
		const heading = /^###\s+(.+)$/.exec(line);
		if (heading) {
			current = heading[1].trim();
			if (!subsections.has(current)) subsections.set(current, []);
			continue;
		}
		if (line.startsWith("## ")) {
			current = null; // defensive: shouldn't occur, extractChangelogSection strips it
			continue;
		}
		if (current && line.startsWith("* ")) {
			subsections.get(current).push(line);
		}
	}
	return subsections;
}

/** Trailing `([shorthash](.../commit/...))` — release-please's per-commit link. */
const COMMIT_LINK_RE = /\(\[([0-9a-f]{7,40})]\([^)]*\/commit\/[0-9a-f]+\)\)\s*$/;

/**
 * Dedup key for one bullet line. Features/Bug Fixes bullets end with a
 * commit-hash link → dedup by that hash (exact, commit-level). Bullets
 * without one (BREAKING CHANGES, or any future section release-please adds
 * without a commit link) fall back to the full bullet text.
 *
 * @param {string} bulletLine
 * @returns {string}
 */
function dedupKey(bulletLine) {
	const m = COMMIT_LINK_RE.exec(bulletLine);
	return m ? m[1] : bulletLine;
}

/**
 * Merge N per-package version sections into one deduplicated section body
 * (no leading version heading — callers own their own page heading/frontmatter).
 *
 * @param {string[]} sections - Section bodies as returned by extractChangelogSection.
 * @returns {string} Merged Markdown body, or "" if every input section was empty/null.
 */
export function mergeSections(sections) {
	/** @type {Map<string, string[]>} */
	const merged = new Map();
	/** @type {Map<string, Set<string>>} */
	const seen = new Map();

	for (const sectionText of sections) {
		if (!sectionText) continue;
		const subsections = parseSubsections(sectionText);
		for (const [title, bullets] of subsections) {
			if (!merged.has(title)) {
				merged.set(title, []);
				seen.set(title, new Set());
			}
			const mergedBullets = merged.get(title);
			const seenKeys = seen.get(title);
			for (const bullet of bullets) {
				const key = dedupKey(bullet);
				if (seenKeys.has(key)) continue;
				seenKeys.add(key);
				mergedBullets.push(bullet);
			}
		}
	}

	if (merged.size === 0) return "";

	// Emit in release-please's fixed order, then anything unrecognised (stable
	// insertion order via Map) — future-proofs against a subsection type this
	// module doesn't know about yet, rather than silently dropping it.
	const orderedTitles = [
		...KNOWN_SECTIONS.filter((t) => merged.has(t)),
		...[...merged.keys()].filter((t) => !KNOWN_SECTIONS.includes(t)),
	];

	const out = [];
	for (const title of orderedTitles) {
		out.push(`### ${title}`, "");
		out.push(...merged.get(title));
		out.push("");
	}
	return out.join("\n").trim();
}

/**
 * Read and merge the given version's section across all package CHANGELOG.md
 * paths (repo-root-relative), returning the merged Markdown body ("" if none
 * of the packages have that version yet — e.g. a package added after the
 * version was cut).
 *
 * @param {string} repoRoot - Absolute path to the repo root.
 * @param {string[]} packagePaths - Repo-root-relative package dirs (each holding a CHANGELOG.md).
 * @param {string} version - Full semver, e.g. "0.2.0".
 * @returns {string}
 */
export function mergeChangelogsForVersion(repoRoot, packagePaths, version) {
	const sections = packagePaths.map((pkgPath) => {
		const changelogPath = `${repoRoot}/${pkgPath}/CHANGELOG.md`;
		if (!existsSync(changelogPath)) return null;
		return extractChangelogSection(readFileSync(changelogPath, "utf-8"), version);
	});
	return mergeSections(sections);
}
