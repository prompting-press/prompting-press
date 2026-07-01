/**
 * build-versions.mjs — multi-version Starlight build orchestrator (Phase 7, TC02/TC03).
 *
 * Reads src/data/versions.json and produces one complete native Starlight build
 * per version, assembled into dist/:
 *
 *   dist/next/          — `next` build (working-tree src/content/docs/)
 *   dist/v0.1/          — released-minor build (frozen src/versions/v0.1/)
 *   dist/index.html     — thin root redirect → /v{latest}/ (TC03)
 *
 * For each version the script:
 *   1. STAGES the correct content tree into src/content/docs/ (or uses it as-is
 *      for `next`, which is already there).
 *   2. Runs `astro build` with:
 *        PP_DOCS_BASE=/vX.Y/   (or /next/)
 *        PP_DOCS_VERSION=vX.Y  (or next)
 *        PP_DOCS_IS_LATEST=true|false
 *        PP_SKIP_PREBUILD=1   (frozen versions only — skips the expensive
 *                              gen-shape-table/gen-api-refs prebuild that needs
 *                              nightly Rust + griffe + typedoc; frozen trees
 *                              already carry their reference pages)
 *      targeting --outDir dist/<prefix>
 *   3. RESTORES src/content/docs/ from a backup taken before staging.
 *      Restore runs in a finally block — guaranteed even on build failure.
 *
 * After all version builds, emits dist/index.html (the root redirect to /v{latest}/).
 *
 * Requirements:
 *   - Stable Node APIs only (fs, path, child_process) — runs under node >=22.12.0.
 *   - No new dependencies beyond what the docs site already has.
 *   - Deterministic: two runs produce identical dist/ (modulo Pagefind hash churn,
 *     which is an Astro/Pagefind concern, not ours).
 *
 * Usage:
 *   node docs/site/scripts/build-versions.mjs
 *   (from repo root or docs/site/ — the script resolves paths from its own location)
 */

import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const SITE_ROOT   = resolve(__dirname, "..");
const SRC_DOCS    = resolve(SITE_ROOT, "src/content/docs");
const SRC_VERSIONS = resolve(SITE_ROOT, "src/versions");
const MANIFEST_PATH = resolve(SITE_ROOT, "src/data/versions.json");
const DIST_ROOT   = resolve(SITE_ROOT, "dist");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function log(msg) {
  console.log(`[build-versions] ${msg}`);
}

function err(msg) {
  console.error(`[build-versions] ERROR: ${msg}`);
}

/**
 * Run `astro build` for a given version.
 *
 * Prebuild (gen-shape-table / gen-api-refs) is NOT run here — the caller is
 * responsible for running those for the `next` build before calling this.
 * Frozen-version builds skip prebuild entirely (their trees already have
 * their reference pages).
 *
 * @param {object} opts
 * @param {string} opts.prefix     e.g. "/v0.1/" or "/next/"
 * @param {string} opts.version    e.g. "v0.1" or "next"
 * @param {boolean} opts.isLatest
 * @param {string}  opts.outDir    absolute path for --outDir
 * @returns {boolean} true on success
 */
function runAstroBuild({ prefix, version, isLatest, outDir }) {
  // Normalise prefix: ensure leading + trailing slash.
  const base = prefix.startsWith("/") ? prefix : `/${prefix}`;

  const env = {
    ...process.env,
    PP_DOCS_BASE:      base,
    PP_DOCS_VERSION:   version,
    PP_DOCS_IS_LATEST: isLatest ? "true" : "false",
  };

  // `pnpm exec astro build` invokes Astro directly, bypassing the npm `build`
  // script lifecycle (which would re-trigger the prebuild).
  const cmd  = "pnpm";
  const args = ["exec", "astro", "build", "--outDir", outDir];

  log(`  Running: ${cmd} ${args.join(" ")}`);
  log(`    PP_DOCS_BASE=${base} PP_DOCS_VERSION=${version} PP_DOCS_IS_LATEST=${isLatest}`);

  const result = spawnSync(cmd, args, {
    stdio: "inherit",
    cwd: SITE_ROOT,
    env,
  });

  if (result.error) {
    err(`spawn error: ${result.error.message}`);
    return false;
  }
  if (result.status !== 0) {
    err(`astro build exited with status ${result.status}`);
    return false;
  }
  return true;
}

/**
 * Backup src/content/docs/ to a temp path (backup persists across the entire loop).
 * Returns the backup path.  Never removes the backup — caller does that.
 */
function backupDocs() {
  const backup = `${SRC_DOCS}.build-versions-backup`;
  if (existsSync(backup)) {
    rmSync(backup, { recursive: true, force: true });
  }
  cpSync(SRC_DOCS, backup, { recursive: true });
  return backup;
}

/**
 * Restore src/content/docs/ from the backup (source = backup path).
 * Does NOT remove the backup — the backup stays valid for the full loop.
 * Pass `removeBackup: true` only at the very end.
 */
function restoreDocs(backup, { removeBackup = false } = {}) {
  if (!existsSync(backup)) {
    err(`Backup not found at ${backup} — cannot restore src/content/docs/`);
    return;
  }
  rmSync(SRC_DOCS, { recursive: true, force: true });
  cpSync(backup, SRC_DOCS, { recursive: true });
  if (removeBackup) {
    rmSync(backup, { recursive: true, force: true });
  }
  log(`Restored src/content/docs/${removeBackup ? " (backup removed)" : ""}`);
}

/**
 * Stage a frozen version tree into src/content/docs/.
 * The backup is already taken — this just swaps the active content.
 */
function stageVersion(minor) {
  const src = resolve(SRC_VERSIONS, `v${minor}`);
  if (!existsSync(src)) {
    throw new Error(`Frozen tree not found: ${src}`);
  }
  rmSync(SRC_DOCS, { recursive: true, force: true });
  cpSync(src, SRC_DOCS, { recursive: true });
  log(`Staged src/versions/v${minor}/ → src/content/docs/`);
}

/**
 * Emit dist/index.html — thin root redirect to /v{latest}/ (TC03 / FR-004 / R11).
 *
 * Uses meta-refresh + <link rel="canonical"> + <noscript> + location.replace()
 * for SEO-friendly static-host redirect (no real 301 on GitHub Pages).
 */
function emitRootRedirect(latestMinor) {
  const target = `/v${latestMinor}/`;
  const html = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="0; url=${target}" />
    <link rel="canonical" href="${target}" />
    <title>Redirecting to latest docs…</title>
    <meta name="robots" content="noindex" />
  </head>
  <body>
    <noscript>
      <p>
        Redirecting to the latest documentation&hellip;
        <a href="${target}">Click here if you are not redirected</a>.
      </p>
    </noscript>
    <script>location.replace("${target}");<\/script>
  </body>
</html>
`;
  mkdirSync(DIST_ROOT, { recursive: true });
  writeFileSync(resolve(DIST_ROOT, "index.html"), html, "utf-8");
  log(`Emitted dist/index.html → redirect to ${target}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main() {
  log("Starting multi-version build");
  log(`Site root: ${SITE_ROOT}`);

  // Read the manifest.
  if (!existsSync(MANIFEST_PATH)) {
    err(`versions.json not found at ${MANIFEST_PATH}`);
    process.exit(1);
  }
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, "utf-8"));

  const latestMinor = manifest.latest;
  log(`Latest released minor: ${latestMinor}`);

  // Collect build targets in order: released minors first (newest→oldest), then next.
  // We build them in this order so dist/ is populated newest-first; all succeed or we abort.
  const releasedVersions = manifest.versions.filter(
    (v) => v.minor !== "next",
  );
  const nextEntry = manifest.versions.find((v) => v.minor === "next");

  const buildTargets = [
    ...releasedVersions,
    ...(nextEntry ? [nextEntry] : []),
  ];

  if (buildTargets.length === 0) {
    err("No versions found in versions.json — nothing to build.");
    process.exit(1);
  }

  log(`Build targets: ${buildTargets.map((v) => v.minor).join(", ")}`);

  // Clean dist/ before assembling (ensures no stale artefacts from prior runs).
  if (existsSync(DIST_ROOT)) {
    rmSync(DIST_ROOT, { recursive: true, force: true });
    log("Cleaned dist/");
  }
  mkdirSync(DIST_ROOT, { recursive: true });

  // Back up the working-tree src/content/docs/ ONCE before any staging.
  log("Backing up src/content/docs/...");
  const backup = backupDocs();

  let allPassed = true;

  try {
    for (const entry of buildTargets) {
      const { minor, isLatest } = entry;
      const isNext = minor === "next";

      // Prefix: "/next/" for next, "/vX.Y/" for released minors.
      const prefix  = isNext ? "/next/" : `/v${minor}/`;
      const version = isNext ? "next"   : `v${minor}`;
      const outDir  = resolve(DIST_ROOT, isNext ? "next" : `v${minor}`);

      log(`\n--- Building ${version} (base=${prefix}, isLatest=${isLatest}) ---`);

      if (isNext) {
        // next: restore the working-tree content into src/content/docs/ from the
        // backup — this undoes any prior staging without removing the backup.
        restoreDocs(backup);
        log("Using working-tree src/content/docs/ for next build");

        // Run the prebuild scripts for the next build — these regenerate the
        // per-language API reference pages and the shape table from live source.
        // Frozen-version builds skip this (their trees already have reference pages).
        log("Running prebuild (gen-shape-table + gen-api-refs) for next...");
        const prebuildScripts = ["scripts/gen-shape-table.mjs", "scripts/gen-api-refs.mjs"];
        for (const script of prebuildScripts) {
          const r = spawnSync("node", [script], {
            stdio: "inherit",
            cwd: SITE_ROOT,
            env: process.env,
          });
          if (r.error || r.status !== 0) {
            err(`Prebuild script ${script} failed (status ${r.status})`);
            allPassed = false;
            // Skip astro build for next if prebuild failed.
            continue;
          }
        }
        if (!allPassed) {
          log("Skipping next astro build due to prebuild failure.");
          continue;
        }
      } else {
        // Released minor: stage the frozen tree into src/content/docs/.
        // The backup is still intact for the finally-restore.
        stageVersion(minor);
      }

      // Run astro build (prebuild already handled above for next, or skipped for frozen).
      const ok = runAstroBuild({ prefix, version, isLatest, outDir });

      if (!ok) {
        err(`Build FAILED for ${version}`);
        allPassed = false;
        // Continue trying other versions to surface all failures,
        // but mark overall as failed.
      } else {
        log(`Build OK for ${version} → ${outDir}`);
      }
    }
  } finally {
    // Always restore src/content/docs/ from the backup and remove the backup.
    log("\nRestoring src/content/docs/ from backup...");
    restoreDocs(backup, { removeBackup: true });
  }

  if (!allPassed) {
    err("One or more version builds failed. See above for details.");
    process.exit(1);
  }

  // Emit the root redirect (TC03).
  emitRootRedirect(latestMinor);

  // Summary.
  log("\n=== Build summary ===");
  log(`dist/ contents:`);
  for (const entry of readdirSync(DIST_ROOT).sort()) {
    log(`  dist/${entry}`);
  }
  log(`Root redirect: dist/index.html → /v${latestMinor}/`);
  log("Done.");
}

try {
  main();
} catch (e) {
  err(`Unhandled error: ${e.message}`);
  console.error(e);
  process.exit(1);
}
