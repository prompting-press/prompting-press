/**
 * strip-jargon.mjs
 *
 * Shared helpers for stripping internal governance jargon and escaping MDX
 * cell content. Extracted from gen-shape-table.mjs (the one allowed touch of
 * that file per spec 011 T005) so that the three API-ref extractors
 * (extract-rust-api.mjs, extract-ts-api.mjs, extract-python-api.py via the
 * renderer) and the shape-table generator all apply exactly the same
 * sanitisation rules.
 *
 * Pure functions — no I/O, no side-effects.
 */

/**
 * Strip internal-governance jargon from a description string before it is
 * shown to end users. Schema descriptions and source doc-comments are written
 * for library maintainers and may cite constitution principles, roadmap
 * decisions (C-NN), FR-/SC-/SEC- IDs, and spec numbers — none of which mean
 * anything to a docs reader. We sanitize at render time rather than editing
 * the source (the source is the published contract / single source of truth).
 *
 * @param {string | null | undefined} str
 * @returns {string}
 */
export function stripJargon(str) {
	let s = String(str ?? "");
	// The internal-ID vocabulary that has no meaning to a docs reader. Kept as one
	// alternation reused by the parenthetical, bracketed, and bare passes below.
	//   FR-/SC-/SEC-/C-NN/TY-/CR-  IDs · spec NNN · constitution Principle X ·
	//   roadmap decision · task IDs (T036) · decision/critique/finding IDs
	//   (D2, E1/critique E1) · clarification/requirement tags (Q1, R7, F5, US3) ·
	//   "data-model §Foo" citations.
	const ID =
		"(?:constitution\\s+|roadmap\\s+decision\\s+)?(?:Principle\\s+[IVXLC]+(?:\\s+v[0-9]+(?:\\.[0-9]+)*)?|FR-[0-9A-Za-z]+|SC-[0-9]+|SEC-[0-9]+|C-[0-9]+|CR-[0-9]+|TY-[0-9]+|spec\\s+[0-9]+|spec-[0-9]+|T[0-9]{2,}|critique\\s+E[0-9]+|decision\\s+D[0-9]+|D[0-9]+|US[0-9]+(?:\\s+scenario\\s+[0-9]+)?|[FQR][0-9]+|data-model\\s+§[A-Za-z]+)";
	// 1. Parenthetical citation clusters, possibly multiple slash/comma-separated IDs:
	//    "(FR-010a)", "(spec 008)", "(critique E1 / C-01)", "(US3 / F5; FR-022)", "(T045)".
	const paren = new RegExp(
		`\\s*\\((?:${ID})(?:\\s*[/,;&]\\s*(?:${ID}|[^)]*?))*\\)`,
		"gi",
	);
	s = s.replace(paren, "");
	// 2. Bracketed trailing tags: "[FR-015]", "[FR-022]", "[n]".
	s = s.replace(new RegExp(`\\s*\\[(?:${ID})\\]`, "gi"), "");
	// 3. Inline " (renamed from `provenance` in spec 008)" style notes that name specs.
	s = s.replace(/\s*\(renamed from `provenance` in spec [0-9]+\)/gi, "");
	// 4. Bare trailing/inline references: "per roadmap decision C-09", "constitution Principle IV".
	s = s.replace(
		/\s*(?:per\s+)?(?:roadmap decision\s+C-[0-9]+|constitution Principle\s+[IVXLC]+(?:\s+v[0-9]+(?:\.[0-9]+)*)?)\b[^.]*/gi,
		"",
	);
	// Clean up punctuation/space the removals leave behind: " ." → ".", " ;" → ";",
	// "( )" leftovers, and doubled spaces.
	s = s.replace(/\(\s*\)/g, "");
	s = s.replace(/\s+([.,;)])/g, "$1");
	return s.replace(/\s{2,}/g, " ").trim();
}

/**
 * Escape content for use inside a Markdown table cell:
 *   - pipes would break column boundaries
 *   - newlines would break the row
 *
 * @param {string | null | undefined} str
 * @returns {string}
 */
export function escapeCell(str) {
	return String(str ?? "")
		// Escape backslashes FIRST so the pipe-escaping below cannot be
		// defeated by a literal backslash already in the input
		// (js/incomplete-sanitization). Order matters: \ → \\ before | → \|.
		.replace(/\\/g, "\\\\")
		.replace(/\|/g, "\\|")
		.replace(/\n/g, " ");
}
