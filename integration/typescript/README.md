# TypeScript Integration Gate

Full-surface integration test harness for the `prompting-press` TypeScript package. It depends
on the package via a workspace path dep, builds the native napi addon and the TypeScript facade,
then exercises the entire public API: construction, rendering, guard config, derive/merge,
provenance attributes, loaders, composition, check, error hierarchy, and the version probe.

To add a feature, add `test/<feature>.test.mjs` and run `pnpm test`.

**Note:** `pnpm test` runs `build` (addon + facade) first, then `node --test`. On a fresh checkout
run `pnpm test` directly; do not `pnpm install` before building because pnpm snapshots the package
at install time and must see `packages/typescript/dist/` already built.
