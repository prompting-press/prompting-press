# Changelog

All notable changes to **Prompting Press**, aggregated across every published
package (the Rust crates, the Python distribution, and the npm package). Every
release is lockstepped — one version number moves across all packages at once.

This file is generated from the per-package changelogs by
[`scripts/gen-root-changelog.mjs`](scripts/gen-root-changelog.mjs); edit those,
not this file. Each entry links to its pull request and commit.


## 0.5.0 (2026-07-20)

### ⚠ BREAKING CHANGES

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317))
* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))
* add cross-language conformance suite enforcing identical marshaling and schema validation ([#185](https://github.com/prompting-press/prompting-press/issues/185)) ([b2092c0](https://github.com/prompting-press/prompting-press/commit/b2092c0ffcdbf47682489fd630c2ae6a0470951b))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208)) ([163559b](https://github.com/prompting-press/prompting-press/commit/163559bafdfbc7deee1e47bdba10128bf712e4e5))
* pluggable prompt loader with filesystem and in-memory backends ([#275](https://github.com/prompting-press/prompting-press/issues/275)) ([42515d2](https://github.com/prompting-press/prompting-press/commit/42515d26432d0dc06fcfc56f4ed5ffe14bdba401))
* provenance attributes helper for telemetry span attribution ([#274](https://github.com/prompting-press/prompting-press/issues/274)) ([af29000](https://github.com/prompting-press/prompting-press/commit/af290001a6336f77337a3ce788ed89aa61586a04))
* typed prompt-template library for Rust — input validation, CI agreement checks, and multi-message composition ([#100](https://github.com/prompting-press/prompting-press/issues/100)) ([674a31f](https://github.com/prompting-press/prompting-press/commit/674a31fd613f445ef8f04113cb99bced7bc9064a))
* surface the guard advisory override to Python and TypeScript callers ([#221](https://github.com/prompting-press/prompting-press/issues/221)) ([c3e9589](https://github.com/prompting-press/prompting-press/commit/c3e95896db9595946b061d3c0247ebe76ed6ce36))

### Bug Fixes

* align remaining license metadata with MPL-2.0 relicense ([#319](https://github.com/prompting-press/prompting-press/issues/319)) ([0aea2b9](https://github.com/prompting-press/prompting-press/commit/0aea2b90ec5221061c70fc6967928c399d430c60))
* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))
* **release:** use full 0.1.0 version literal for the core dep requirement ([#235](https://github.com/prompting-press/prompting-press/issues/235)) ([2d13bd2](https://github.com/prompting-press/prompting-press/commit/2d13bd201fc80e6319bb3543124b3c1de9213991))
* generate THIRD-PARTY-LICENSES in CI instead of committing them ([#286](https://github.com/prompting-press/prompting-press/issues/286)) ([bb91278](https://github.com/prompting-press/prompting-press/commit/bb912781220b923b0e7e12b6dd2a0d926859ad6f))

### Miscellaneous Chores

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317)) ([f3c9d04](https://github.com/prompting-press/prompting-press/commit/f3c9d04540f9e3053921c12aa7d28f913acb143d))

### Documentation

* aggregated changelog surfaced on every registry ([#303](https://github.com/prompting-press/prompting-press/issues/303)) ([4793238](https://github.com/prompting-press/prompting-press/commit/4793238f8a3a064a0097752154bf9038dcd41767))
* use LLM-appropriate prompt examples (system prompt with company var) ([#245](https://github.com/prompting-press/prompting-press/issues/245)) ([c1660b6](https://github.com/prompting-press/prompting-press/commit/c1660b6738847c76ae05f11a678838b313f7ce24))

## 0.4.0 (2026-07-17)

### ⚠ BREAKING CHANGES

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317))

### Bug Fixes

* align remaining license metadata with MPL-2.0 relicense ([#319](https://github.com/prompting-press/prompting-press/issues/319)) ([0aea2b9](https://github.com/prompting-press/prompting-press/commit/0aea2b90ec5221061c70fc6967928c399d430c60))

### Miscellaneous Chores

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317)) ([f3c9d04](https://github.com/prompting-press/prompting-press/commit/f3c9d04540f9e3053921c12aa7d28f913acb143d))

### Documentation

* aggregated changelog surfaced on every registry ([#303](https://github.com/prompting-press/prompting-press/issues/303)) ([4793238](https://github.com/prompting-press/prompting-press/commit/4793238f8a3a064a0097752154bf9038dcd41767))

## 0.3.2 (2026-07-10)

### ⚠ BREAKING CHANGES

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))
* add cross-language conformance suite enforcing identical marshaling and schema validation ([#185](https://github.com/prompting-press/prompting-press/issues/185)) ([b2092c0](https://github.com/prompting-press/prompting-press/commit/b2092c0ffcdbf47682489fd630c2ae6a0470951b))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208)) ([163559b](https://github.com/prompting-press/prompting-press/commit/163559bafdfbc7deee1e47bdba10128bf712e4e5))
* pluggable prompt loader with filesystem and in-memory backends ([#275](https://github.com/prompting-press/prompting-press/issues/275)) ([42515d2](https://github.com/prompting-press/prompting-press/commit/42515d26432d0dc06fcfc56f4ed5ffe14bdba401))
* provenance attributes helper for telemetry span attribution ([#274](https://github.com/prompting-press/prompting-press/issues/274)) ([af29000](https://github.com/prompting-press/prompting-press/commit/af290001a6336f77337a3ce788ed89aa61586a04))
* typed prompt-template library for Rust — input validation, CI agreement checks, and multi-message composition ([#100](https://github.com/prompting-press/prompting-press/issues/100)) ([674a31f](https://github.com/prompting-press/prompting-press/commit/674a31fd613f445ef8f04113cb99bced7bc9064a))
* surface the guard advisory override to Python and TypeScript callers ([#221](https://github.com/prompting-press/prompting-press/issues/221)) ([c3e9589](https://github.com/prompting-press/prompting-press/commit/c3e95896db9595946b061d3c0247ebe76ed6ce36))

### Bug Fixes

* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))
* **release:** use full 0.1.0 version literal for the core dep requirement ([#235](https://github.com/prompting-press/prompting-press/issues/235)) ([2d13bd2](https://github.com/prompting-press/prompting-press/commit/2d13bd201fc80e6319bb3543124b3c1de9213991))
* generate THIRD-PARTY-LICENSES in CI instead of committing them ([#286](https://github.com/prompting-press/prompting-press/issues/286)) ([bb91278](https://github.com/prompting-press/prompting-press/commit/bb912781220b923b0e7e12b6dd2a0d926859ad6f))

### Documentation

* use LLM-appropriate prompt examples (system prompt with company var) ([#245](https://github.com/prompting-press/prompting-press/issues/245)) ([c1660b6](https://github.com/prompting-press/prompting-press/commit/c1660b6738847c76ae05f11a678838b313f7ce24))

## 0.3.1 (2026-07-09)

### Bug Fixes

* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))
* generate THIRD-PARTY-LICENSES in CI instead of committing them ([#286](https://github.com/prompting-press/prompting-press/issues/286)) ([bb91278](https://github.com/prompting-press/prompting-press/commit/bb912781220b923b0e7e12b6dd2a0d926859ad6f))

## 0.3.0 (2026-07-08)

### ⚠ BREAKING CHANGES

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* pluggable prompt loader with filesystem and in-memory backends ([#275](https://github.com/prompting-press/prompting-press/issues/275)) ([42515d2](https://github.com/prompting-press/prompting-press/commit/42515d26432d0dc06fcfc56f4ed5ffe14bdba401))
* provenance attributes helper for telemetry span attribution ([#274](https://github.com/prompting-press/prompting-press/issues/274)) ([af29000](https://github.com/prompting-press/prompting-press/commit/af290001a6336f77337a3ce788ed89aa61586a04))

### Documentation

* use LLM-appropriate prompt examples (system prompt with company var) ([#245](https://github.com/prompting-press/prompting-press/issues/245)) ([c1660b6](https://github.com/prompting-press/prompting-press/commit/c1660b6738847c76ae05f11a678838b313f7ce24))

## 0.2.0 (2026-07-01)

### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208))

### Features

* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))
* add cross-language conformance suite enforcing identical marshaling and schema validation ([#185](https://github.com/prompting-press/prompting-press/issues/185)) ([b2092c0](https://github.com/prompting-press/prompting-press/commit/b2092c0ffcdbf47682489fd630c2ae6a0470951b))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208)) ([163559b](https://github.com/prompting-press/prompting-press/commit/163559bafdfbc7deee1e47bdba10128bf712e4e5))
* typed prompt-template library for Rust — input validation, CI agreement checks, and multi-message composition ([#100](https://github.com/prompting-press/prompting-press/issues/100)) ([674a31f](https://github.com/prompting-press/prompting-press/commit/674a31fd613f445ef8f04113cb99bced7bc9064a))
* surface the guard advisory override to Python and TypeScript callers ([#221](https://github.com/prompting-press/prompting-press/issues/221)) ([c3e9589](https://github.com/prompting-press/prompting-press/commit/c3e95896db9595946b061d3c0247ebe76ed6ce36))

### Bug Fixes

* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
* **release:** use full 0.1.0 version literal for the core dep requirement ([#235](https://github.com/prompting-press/prompting-press/issues/235)) ([2d13bd2](https://github.com/prompting-press/prompting-press/commit/2d13bd201fc80e6319bb3543124b3c1de9213991))

## 0.1.0 (2026-07-01)

### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))

### Features

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
