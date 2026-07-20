# Changelog

## [0.5.0](https://github.com/prompting-press/prompting-press/compare/v0.4.0...v0.5.0) (2026-07-20)


### ⚠ BREAKING CHANGES

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317))
* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))


### Bug Fixes

* align remaining license metadata with MPL-2.0 relicense ([#319](https://github.com/prompting-press/prompting-press/issues/319)) ([0aea2b9](https://github.com/prompting-press/prompting-press/commit/0aea2b90ec5221061c70fc6967928c399d430c60))
* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))


### Miscellaneous Chores

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317)) ([f3c9d04](https://github.com/prompting-press/prompting-press/commit/f3c9d04540f9e3053921c12aa7d28f913acb143d))

## [0.4.0](https://github.com/prompting-press/prompting-press/compare/v0.3.2...v0.4.0) (2026-07-17)


### ⚠ BREAKING CHANGES

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317))

### Bug Fixes

* align remaining license metadata with MPL-2.0 relicense ([#319](https://github.com/prompting-press/prompting-press/issues/319)) ([0aea2b9](https://github.com/prompting-press/prompting-press/commit/0aea2b90ec5221061c70fc6967928c399d430c60))


### Miscellaneous Chores

* relicense from Apache-2.0 to MPL-2.0 ([#317](https://github.com/prompting-press/prompting-press/issues/317)) ([f3c9d04](https://github.com/prompting-press/prompting-press/commit/f3c9d04540f9e3053921c12aa7d28f913acb143d))

## [0.3.2](https://github.com/prompting-press/prompting-press/compare/v0.3.1...v0.3.2) (2026-07-10)


### ⚠ BREAKING CHANGES

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))


### Bug Fixes

* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))

## [0.3.1](https://github.com/prompting-press/prompting-press/compare/prompting-press-core-v0.3.0...prompting-press-core-v0.3.1) (2026-07-09)


### Bug Fixes

* restore working release config so 0.3.1 can publish ([#290](https://github.com/prompting-press/prompting-press/issues/290)) ([d9d2bb1](https://github.com/prompting-press/prompting-press/commit/d9d2bb13eeb09ab1853bf4b4a0e76d88b22e533d))

## [0.3.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-core-v0.2.0...prompting-press-core-v0.3.0) (2026-07-08)


### ⚠ BREAKING CHANGES

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))

## [0.2.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-core-v0.1.0...prompting-press-core-v0.2.0) (2026-07-01)


### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))


### Bug Fixes

* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))

## [0.1.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-core-v0.1.0...prompting-press-core-v0.1.0) (2026-07-01)


### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))

### Features

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))

## 0.1.0 (2026-06-30)


### ⚠ BREAKING CHANGES

* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* auto-generate the Rust/Python/TypeScript API reference pages from source doc comments ([#196](https://github.com/prompting-press/prompting-press/issues/196)) ([21d5a96](https://github.com/prompting-press/prompting-press/commit/21d5a9625c725c18ab50e7f0d4735b82a3ccde84))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* typed prompt-template engine with variable-agreement checking, variants, and provenance hashing ([#73](https://github.com/prompting-press/prompting-press/issues/73)) ([1b0e321](https://github.com/prompting-press/prompting-press/commit/1b0e3212f9f461ef3b96f90ad3d86f2f140c2210))


### Bug Fixes

* configure release-please per-crate (literal versions + linked-versions) ([f5f16ec](https://github.com/prompting-press/prompting-press/commit/f5f16ecf2c960b70214f456741f073054d2070a2))
