# Changelog

## [0.3.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-py-v0.2.0...prompting-press-py-v0.3.0) (2026-07-08)


### ⚠ BREAKING CHANGES

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272))

### Features

* add merge strategy to derive() for composing prompt variables ([#272](https://github.com/prompting-press/prompting-press/issues/272)) ([3f1e27f](https://github.com/prompting-press/prompting-press/commit/3f1e27f6ad98e2cb60de1db14f293aab21214c98))
* pluggable prompt loader with filesystem and in-memory backends ([#275](https://github.com/prompting-press/prompting-press/issues/275)) ([42515d2](https://github.com/prompting-press/prompting-press/commit/42515d26432d0dc06fcfc56f4ed5ffe14bdba401))
* provenance attributes helper for telemetry span attribution ([#274](https://github.com/prompting-press/prompting-press/issues/274)) ([af29000](https://github.com/prompting-press/prompting-press/commit/af290001a6336f77337a3ce788ed89aa61586a04))


### Documentation

* use LLM-appropriate prompt examples (system prompt with company var) ([#245](https://github.com/prompting-press/prompting-press/issues/245)) ([c1660b6](https://github.com/prompting-press/prompting-press/commit/c1660b6738847c76ae05f11a678838b313f7ce24))

## [0.2.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-py-v0.1.0...prompting-press-py-v0.2.0) (2026-07-01)


### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* add cross-language conformance suite enforcing identical marshaling and schema validation ([#185](https://github.com/prompting-press/prompting-press/issues/185)) ([b2092c0](https://github.com/prompting-press/prompting-press/commit/b2092c0ffcdbf47682489fd630c2ae6a0470951b))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208)) ([163559b](https://github.com/prompting-press/prompting-press/commit/163559bafdfbc7deee1e47bdba10128bf712e4e5))
* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* surface the guard advisory override to Python and TypeScript callers ([#221](https://github.com/prompting-press/prompting-press/issues/221)) ([c3e9589](https://github.com/prompting-press/prompting-press/commit/c3e95896db9595946b061d3c0247ebe76ed6ce36))

## [0.1.0](https://github.com/prompting-press/prompting-press/compare/prompting-press-py-v0.1.0...prompting-press-py-v0.1.0) (2026-07-01)


### ⚠ BREAKING CHANGES

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227))

### Features

* per-version documentation sites + corrected, tested examples ([#227](https://github.com/prompting-press/prompting-press/issues/227)) ([688ec29](https://github.com/prompting-press/prompting-press/commit/688ec297b876f1e1fda6ac7bb8190d3acba754da))

## 0.1.0 (2026-06-30)


### ⚠ BREAKING CHANGES

* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193))

### Features

* add cross-language conformance suite enforcing identical marshaling and schema validation ([#185](https://github.com/prompting-press/prompting-press/issues/185)) ([b2092c0](https://github.com/prompting-press/prompting-press/commit/b2092c0ffcdbf47682489fd630c2ae6a0470951b))
* guard delimits untrusted values in the rendered prompt ([#214](https://github.com/prompting-press/prompting-press/issues/214)) ([1ac0250](https://github.com/prompting-press/prompting-press/commit/1ac02504af0bdccd4a84e91a3a482dccd6205a63))
* opt-in to full render-error detail for debugging ([#208](https://github.com/prompting-press/prompting-press/issues/208)) ([163559b](https://github.com/prompting-press/prompting-press/commit/163559bafdfbc7deee1e47bdba10128bf712e4e5))
* polyglot prompt-template foundations (shared core, schema, codegen, CI guardrails) ([#36](https://github.com/prompting-press/prompting-press/issues/36)) ([fef33c8](https://github.com/prompting-press/prompting-press/commit/fef33c8bac92d7f438536d8d76d93425db884c2b))
* rename to PromptVariable/PromptVariant, one metadata map per container, preserve parse-error detail ([#193](https://github.com/prompting-press/prompting-press/issues/193)) ([99d11cf](https://github.com/prompting-press/prompting-press/commit/99d11cfe63c0a006c67feee21931c8f8752d04a0))
* surface the guard advisory override to Python and TypeScript callers ([#221](https://github.com/prompting-press/prompting-press/issues/221)) ([c3e9589](https://github.com/prompting-press/prompting-press/commit/c3e95896db9595946b061d3c0247ebe76ed6ce36))
