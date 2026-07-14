// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! The Node [`NapiPrompt`] `#[napi]` class — a thin wrapper over the Rust consumer's
//! [`prompting_press::Prompt`] (spec 008, T042–T046).
//!
//! ## Design: Rust consumer owns ALL validation (C-01 / Principle I)
//!
//! Every invariant — reserved-name check, template parse, agreement check — runs inside
//! [`prompting_press::Prompt::new`] / `from_yaml` / `from_json` / `from_toml`. This binding
//! adds **zero engine logic**: it calls the consumer, marshals errors through the existing
//! [`consumer_error_to_napi_err`] path (SEC-004 scrub preserved), and surfaces the result.
//!
//! ## Construction errors surface as structured napi errors
//!
//! A construction failure (agreement violation, parse error, reserved name, load error) becomes
//! a [`napi::Error`] whose `reason` is the JSON payload `{ code, errors: [{field,code,message}] }`
//! — the same contract as every other error path (research D4). The TS facade decodes it into
//! the matching `PromptingPressError` subclass via `decodeAddonError`, so callers get
//! `instanceof PromptValidationError` or `instanceof LoadError` etc., never a raw napi error.
//!
//! ## Render: validation is owned in TypeScript (Q1)
//!
//! As with the registry-keyed `render` path, Zod `safeParse` runs in the TS facade before the
//! addon's `render_prompt` is called. `render_prompt` receives an already-validated
//! `serde_json::Value`, marshals it via [`crate::marshal::to_kernel_value`], and calls
//! `prompting_press_core::render` directly (critique E1 / C-01). Zero validation in Rust here.
//!
//! ## TOML: no JS TOML parser needed (decision: route to Rust)
//!
//! `fromToml` routes the raw TOML text to [`prompting_press::Prompt::from_toml`] (research R3,
//! `toml@1.1.2`). This is consistent with `fromYaml`/`fromJson` — raw text in, the consumer
//! parses it (Q3 / Principle I). `smol-toml` is therefore NOT added to `packages/typescript/
//! package.json`.
//!
//! ## FFI isolation (Principle II)
//!
//! This module delegates to `prompting_press::Prompt` — the consumer crate — for ALL logic.
//! No `pyo3` / no `napi` in the consumer crate (ci:check-ffi stays green).

use napi_derive::napi;

use prompting_press::{merge_definitions, MergeStrategy};
use prompting_press_core::GuardConfig as KernelGuardConfig;

use crate::check::{CheckReport, Finding};
use crate::error::{consumer_error_to_napi_err, kernel_error_to_napi_err};
use crate::marshal::to_kernel_value;
use crate::render::{GuardConfig, RenderResult};

// ── MergeStrategy marshaling ─────────────────────────────────────────────────

/// Marshal a TS `MergeStrategy` string value (`"replace"` / `"merge"`) into the Rust
/// consumer's [`MergeStrategy`] enum.
///
/// # Errors
///
/// Returns a `load`-coded napi error for unrecognized values (FR-011).
fn marshal_strategy(s: Option<String>) -> napi::Result<MergeStrategy> {
    match s.as_deref() {
        None | Some("replace") => Ok(MergeStrategy::Replace),
        Some("merge") => Ok(MergeStrategy::Merge),
        Some(other) => Err(consumer_error_to_napi_err(
            prompting_press::ConsumerError::Load(format!(
                "unknown MergeStrategy value: {other:?}; expected \"replace\" or \"merge\""
            )),
        )),
    }
}

/// The Node `NapiPrompt` class — an immutable, fully-validated prompt handle.
///
/// Wraps [`prompting_press::Prompt`]; all construction invariants (shape-valid,
/// template-parseable, agreement-sound, reserved-name clean) are enforced at construction
/// time — through the Rust consumer, not re-implemented here (Principle I). The inner
/// `Prompt` is the single source of truth; this type adds only the napi marshaling (C-02).
///
/// Construction is via the napi free-functions [`prompt_new`], [`prompt_from_yaml`],
/// [`prompt_from_json`], [`prompt_from_toml`], each of which maps any consumer error through
/// [`consumer_error_to_napi_err`] so the TS facade always receives the structured JSON payload.
///
/// The `#[napi(constructor)]` is present so napi-rs can build the JS class, but the TS
/// `Prompt` facade calls the free-functions — not `new NapiPrompt()` directly — so the raw
/// constructor is not part of the public TS API.
#[derive(Debug)]
#[napi]
pub struct NapiPrompt {
    inner: prompting_press::Prompt,
}

#[napi]
impl NapiPrompt {
    // ── constructor (napi machinery; not the public TS API entry point) ───────────────────

    /// Napi constructor — present to satisfy napi-rs class registration. The TS `Prompt`
    /// calls the free-functions (`promptNew`, `promptFromYaml`, etc.) rather than this
    /// directly. Delegates to `prompt_new` so the same validation path runs.
    #[napi(constructor)]
    pub fn new_napi(shape: serde_json::Value) -> napi::Result<Self> {
        prompt_new(shape)
    }

    // ── read-only accessors ───────────────────────────────────────────────────────────────

    /// `prompt.name` — the prompt's `name` field.
    #[napi(getter)]
    #[must_use]
    pub fn name(&self) -> String {
        self.inner.name().to_owned()
    }

    /// `prompt.role` — the conversational role (`"system"` / `"user"` / `"assistant"`).
    #[napi(getter)]
    #[must_use]
    pub fn role(&self) -> String {
        self.inner.role().to_string()
    }

    /// `prompt.body` — the root body template source (unrendered).
    #[napi(getter)]
    #[must_use]
    pub fn body(&self) -> String {
        self.inner.body().to_owned()
    }

    /// `prompt.variables` — the declared variables map as a plain JSON object.
    ///
    /// Returns an opaque `serde_json::Value` (an object); the TS facade types it as the
    /// `PromptDefinition["variables"]` shape.
    #[napi(getter)]
    pub fn variables(&self) -> napi::Result<serde_json::Value> {
        serde_json::to_value(self.inner.variables()).map_err(|e| {
            consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
        })
    }

    /// `prompt.variants` — the named variants map as a plain JSON object.
    #[napi(getter)]
    pub fn variants(&self) -> napi::Result<serde_json::Value> {
        serde_json::to_value(self.inner.variants()).map_err(|e| {
            consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
        })
    }

    /// `prompt.outputModel` — the output model reference, if declared; `null` if absent.
    #[napi(getter)]
    pub fn output_model(&self) -> Option<String> {
        self.inner.output_model().map(str::to_owned)
    }

    /// `prompt.metadata` — the `metadata` opaque map as a plain JSON object.
    #[napi(getter)]
    pub fn metadata(&self) -> napi::Result<serde_json::Value> {
        serde_json::to_value(self.inner.metadata()).map_err(|e| {
            consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
        })
    }

    // ── operations ────────────────────────────────────────────────────────────────────────

    /// `prompt.renderPrompt(value, variant?, guard?, unsafeRevealRenderDetail?)` — render the
    /// prompt with the already-validated `value`.
    ///
    /// Validation has **already happened** in the TS facade (`schema.safeParse(data)`)
    /// before this is called. This function:
    /// 1. Marshals the validated value through [`to_kernel_value`].
    /// 2. Calls [`prompting_press_core::render`] directly.
    /// 3. Normalizes any [`KernelError`] through the consumer seam with the per-call opt-in,
    ///    then maps the result to a napi error (error detail scrubbed when flag is false).
    ///
    /// ## `unsafe_reveal_render_detail` — off-by-default render-error detail opt-in
    ///
    /// Default `None` / `false`. When `true`, the full underlying render-error detail is
    /// surfaced in the returned napi error's `errors[0].message` instead of the fixed
    /// scrubbed string.
    ///
    /// **Risk:** enabling this may place **bound-value content** — untrusted input, PII,
    /// secrets — into the thrown error and into any log line or stack trace derived from it.
    /// Use only in a controlled debug context with a trusted log destination, and only after
    /// deliberately accepting that exposure. Never set `true` by default or via ambient
    /// configuration.
    ///
    /// # Errors
    ///
    /// A kernel code (`unknown_variant` / `undefined_variable` / `parse` / `render` /
    /// `excluded_feature`) — the kernel rejected the render. `render` detail scrubbed unless
    /// `unsafe_reveal_render_detail = true`. `parse` detail always preserved.
    #[napi]
    pub fn render_prompt(
        &self,
        value: serde_json::Value,
        variant: Option<String>,
        guard: Option<GuardConfig>,
        unsafe_reveal_render_detail: Option<bool>,
    ) -> napi::Result<RenderResult> {
        let values = to_kernel_value(value);
        let guard_cfg = guard.map_or_else(KernelGuardConfig::default, KernelGuardConfig::from);
        let reveal = unsafe_reveal_render_detail.unwrap_or(false);

        prompting_press_core::render(
            self.inner.definition(),
            variant.as_deref(),
            values,
            &guard_cfg,
        )
        .map(RenderResult::from)
        .map_err(|e| {
            let consumer = prompting_press::ConsumerError::from_kernel_revealing(e, reveal);
            consumer_error_to_napi_err(consumer)
        })
    }

    /// `prompt.getSourcePrompt(variant?)` — return a variant's unrendered template source.
    ///
    /// Calls the kernel's `get_source` directly (the same path the registry-keyed
    /// `get_source` uses after resolving the name). No vars, no validation, no marshaling.
    ///
    /// # Errors
    ///
    /// A kernel code (e.g. `unknown_variant`) — the kernel rejected the lookup.
    #[napi]
    pub fn get_source_prompt(&self, variant: Option<String>) -> napi::Result<String> {
        prompting_press_core::get_source(self.inner.definition(), variant.as_deref())
            .map(str::to_owned)
            .map_err(kernel_error_to_napi_err)
    }

    /// `prompt.checkPrompt()` — pure advisory lint: trust/guard finding only.
    ///
    /// Delegates to [`prompting_press::Prompt::check`] and converts the consumer's
    /// [`prompting_press::CheckReport`] to the napi [`CheckReport`] via [`Finding::from`]
    /// (the same `From` impl the registry-level `check` uses — no duplication).
    ///
    /// The only LIVE finding for a constructed `Prompt` is `UntrustedWithoutGuard`; agreement,
    /// parse, and reserved-name invariants are enforced at construction.
    #[napi]
    pub fn check_prompt(&self) -> CheckReport {
        let consumer_report = self.inner.check();
        let findings: Vec<Finding> = consumer_report
            .findings
            .into_iter()
            .map(Finding::from)
            .collect();
        CheckReport::from_findings(findings)
    }

    /// `prompt.derivePrompt(overlay, strategy?)` — merge overlay onto base and re-validate.
    ///
    /// `overlay` is a `serde_json::Value` object whose keys are a subset of the
    /// `PromptDefinition` top-level fields. `strategy` is the optional marshaled
    /// [`MergeStrategy`] string (`"replace"` or `"merge"`; default `"replace"`).
    ///
    /// Implementation — calls the single shared [`merge_definitions`] helper (FR-018 / R8):
    /// 1. Serialize the current definition to a `serde_json::Value` (an object).
    /// 2. Call `merge_definitions(base, overlay, strategy)` — the same helper the typed
    ///    Rust `derive_with` path calls. This guarantees byte-identical union across bindings
    ///    (Principle I / D1) — no second, independent union algorithm.
    /// 3. Deserialize the merged object to a `PromptDefinition` and hand it to
    ///    `Prompt::from_json` (full re-validation through `Prompt::new`).
    ///
    /// The `PromptOverlay` type deliberately does not implement `serde::Deserialize`;
    /// this JSON-space path avoids that dep while calling the same shared helper.
    ///
    /// The original `NapiPrompt` is untouched.
    ///
    /// # Errors
    ///
    /// Same error classes as construction: a merged definition that fails any construction
    /// invariant returns the structured error. An unknown `strategy` value returns a
    /// `load`-coded error (FR-011).
    #[napi]
    pub fn derive_prompt(
        &self,
        overlay: serde_json::Value,
        strategy: Option<String>,
    ) -> napi::Result<NapiPrompt> {
        let strategy = marshal_strategy(strategy)?;

        // Serialize the current definition to a JSON object, apply the shared merge helper,
        // then validate through the same construction path.
        let base = serde_json::to_value(self.inner.definition()).map_err(|e| {
            consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
        })?;

        let merged =
            merge_definitions(base, overlay, strategy).map_err(consumer_error_to_napi_err)?;

        let merged_json = serde_json::to_string(&merged).map_err(|e| {
            consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
        })?;

        let derived =
            prompting_press::Prompt::from_json(&merged_json).map_err(consumer_error_to_napi_err)?;
        Ok(NapiPrompt { inner: derived })
    }
}

// ── napi free-functions: the four construction paths ─────────────────────────────────────

/// Construct a `NapiPrompt` from a decoded shape object (the `new Prompt(shape)` path, T042).
///
/// `shape` is the `serde_json::Value` napi decoded from the TS `PromptDefinition`-shaped
/// object; it is re-serialized to JSON and handed to [`prompting_press::Prompt::from_json`]
/// so the same validation path runs as for the text paths (Q3 — one loader, one path).
///
/// # Errors
///
/// - `load` — `shape` does not match the `PromptDefinition` schema.
/// - kernel codes — agreement / reserved-name / parse failure.
#[napi]
pub fn prompt_new(shape: serde_json::Value) -> napi::Result<NapiPrompt> {
    let json = serde_json::to_string(&shape).map_err(|e| {
        consumer_error_to_napi_err(prompting_press::ConsumerError::Load(e.to_string()))
    })?;
    let prompt = prompting_press::Prompt::from_json(&json).map_err(consumer_error_to_napi_err)?;
    Ok(NapiPrompt { inner: prompt })
}

/// Construct a `NapiPrompt` from already-read **YAML** text (`Prompt.fromYaml` path, T042).
///
/// # Errors
///
/// - `load` — `text` is not valid YAML or does not match the prompt-definition shape.
/// - kernel codes — agreement / reserved-name / parse failure.
#[napi]
pub fn prompt_from_yaml(text: String) -> napi::Result<NapiPrompt> {
    let prompt = prompting_press::Prompt::from_yaml(&text).map_err(consumer_error_to_napi_err)?;
    Ok(NapiPrompt { inner: prompt })
}

/// Construct a `NapiPrompt` from already-read **JSON** text (`Prompt.fromJson` path, T042).
///
/// # Errors
///
/// - `load` — `text` is not valid JSON or does not match the prompt-definition shape.
/// - kernel codes — agreement / reserved-name / parse failure.
#[napi]
pub fn prompt_from_json(text: String) -> napi::Result<NapiPrompt> {
    let prompt = prompting_press::Prompt::from_json(&text).map_err(consumer_error_to_napi_err)?;
    Ok(NapiPrompt { inner: prompt })
}

/// Construct a `NapiPrompt` from already-read **TOML** text (`Prompt.fromToml` path, T042).
///
/// Routes the raw text to [`prompting_press::Prompt::from_toml`] (`toml@1.1.2`). No JS TOML
/// parser is needed — raw text is routed to Rust exactly as with the YAML/JSON paths (Q3 /
/// Principle I). **`smol-toml` is NOT added to `packages/typescript/package.json`.**
///
/// # Errors
///
/// - `load` — `text` is not valid TOML or does not match the prompt-definition shape.
/// - kernel codes — agreement / reserved-name / parse failure.
#[napi]
pub fn prompt_from_toml(text: String) -> napi::Result<NapiPrompt> {
    let prompt = prompting_press::Prompt::from_toml(&text).map_err(consumer_error_to_napi_err)?;
    Ok(NapiPrompt { inner: prompt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompting_press::error::code;

    fn payload_of(err: &napi::Error) -> serde_json::Value {
        serde_json::from_str(&err.reason).expect("napi error reason is the JSON payload")
    }

    // ── T042: construction paths ─────────────────────────────────────────────────────────

    #[test]
    fn prompt_new_valid_shape_succeeds() {
        let shape = serde_json::json!({
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let p = prompt_new(shape).expect("valid shape must construct");
        assert_eq!(p.name(), "greet");
        assert_eq!(p.role(), "user");
        assert_eq!(p.body(), "Hi {{ name }}");
    }

    #[test]
    fn prompt_from_json_valid_constructs() {
        let json = r#"{"name":"greet","role":"user","body":"Hi {{ name }}","variables":{"name":{"type":"string","trusted":true}}}"#;
        let p = prompt_from_json(json.to_string()).expect("valid JSON");
        assert_eq!(p.name(), "greet");
    }

    #[test]
    fn prompt_from_yaml_valid_constructs() {
        let yaml = "name: greet\nrole: user\nbody: \"Hi {{ name }}\"\nvariables:\n  name:\n    type: string\n    trusted: true\n";
        let p = prompt_from_yaml(yaml.to_string()).expect("valid YAML");
        assert_eq!(p.name(), "greet");
    }

    #[test]
    fn prompt_from_toml_valid_constructs() {
        let toml_text = "name = \"greet\"\nrole = \"user\"\nbody = \"Hi {{ name }}\"\n[variables.name]\ntype = \"string\"\ntrusted = true\n";
        let p = prompt_from_toml(toml_text.to_string()).expect("valid TOML");
        assert_eq!(p.name(), "greet");
    }

    // ── T042: construction failures ──────────────────────────────────────────────────────

    #[test]
    fn undeclared_variable_maps_to_undefined_variable_code() {
        let shape = serde_json::json!({
            "name": "bad",
            "role": "user",
            "body": "{{ ghost }}",
            "variables": {},
        });
        let err = prompt_new(shape).expect_err("undeclared var must fail");
        let payload = payload_of(&err);
        assert_eq!(payload["errors"][0]["code"], code::UNDEFINED_VARIABLE);
    }

    #[test]
    fn missing_body_maps_to_load_code() {
        let shape = serde_json::json!({ "name": "bad", "role": "user" });
        let err = prompt_new(shape).expect_err("missing body must fail");
        let payload = payload_of(&err);
        assert_eq!(payload["code"], "load");
    }

    #[test]
    fn reserved_variant_name_default_maps_to_error() {
        let shape = serde_json::json!({
            "name": "bad",
            "role": "user",
            "body": "hi",
            "variables": {},
            "variants": { "default": { "body": "shadowed" } },
        });
        let err = prompt_new(shape).expect_err("reserved variant name must fail");
        let payload = payload_of(&err);
        // Emitted as a kernel error with undefined_variable code (the consumer's behaviour).
        let errors = payload["errors"].as_array().expect("errors array");
        assert!(
            errors.iter().any(|r| r["field"] == "variant"),
            "field must be 'variant', got {errors:?}"
        );
    }

    // ── T044: render_prompt ──────────────────────────────────────────────────────────────

    #[test]
    fn render_prompt_produces_text_and_hashes() {
        let shape = serde_json::json!({
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}!",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let p = prompt_new(shape).expect("valid");
        let result = p
            .render_prompt(serde_json::json!({ "name": "Ada" }), None, None, None)
            .expect("render succeeds");
        assert_eq!(result.text(), "Hi Ada!");
        assert_eq!(result.template_hash().len(), 64);
        assert_eq!(result.render_hash().len(), 64);
    }

    // ── T044: get_source_prompt ──────────────────────────────────────────────────────────

    #[test]
    fn get_source_prompt_returns_unrendered_body() {
        let shape = serde_json::json!({
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let p = prompt_new(shape).expect("valid");
        let src = p.get_source_prompt(None).expect("source");
        assert_eq!(src, "Hi {{ name }}");
    }

    // ── T044: check_prompt ───────────────────────────────────────────────────────────────

    #[test]
    fn check_prompt_returns_advisory_for_unguarded_untrusted() {
        let shape = serde_json::json!({
            "name": "unguarded",
            "role": "user",
            "body": "{{ payload }}",
            "variables": { "payload": { "type": "string", "trusted": false } },
        });
        let p = prompt_new(shape).expect("valid shape");
        let report = p.check_prompt();
        assert!(
            !report.passed(),
            "unguarded untrusted field must produce a finding"
        );
    }

    #[test]
    fn check_prompt_passes_when_guarded() {
        let shape = serde_json::json!({
            "name": "guarded",
            "role": "user",
            "body": "{{ payload }}",
            "variables": { "payload": { "type": "string", "trusted": false } },
            "metadata": { "guard": { "enabled": true } },
        });
        let p = prompt_new(shape).expect("valid shape");
        assert!(
            p.check_prompt().passed(),
            "guard configured → check must pass"
        );
    }

    // ── T045: derive_prompt ──────────────────────────────────────────────────────────────

    #[test]
    fn derive_prompt_valid_overlay_original_unchanged() {
        let shape = serde_json::json!({
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let original = prompt_new(shape).expect("valid");
        let original_body = original.body();

        let overlay = serde_json::json!({ "body": "Hey {{ name }}" });
        let derived = original
            .derive_prompt(overlay, None)
            .expect("valid overlay");

        assert_eq!(derived.body(), "Hey {{ name }}");
        assert_eq!(
            original.body(),
            original_body,
            "original unchanged (SC-004)"
        );
    }

    #[test]
    fn derive_prompt_undeclared_var_returns_error() {
        let shape = serde_json::json!({
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let original = prompt_new(shape).expect("valid");
        // Overlay introduces an undeclared variable.
        let overlay = serde_json::json!({ "body": "{{ name }} {{ ghost }}" });
        let err = original
            .derive_prompt(overlay, None)
            .expect_err("undeclared var must fail");
        let payload = payload_of(&err);
        let errors = payload["errors"].as_array().expect("errors");
        assert!(errors.iter().any(|r| r["code"] == code::UNDEFINED_VARIABLE));
    }

    // ── T045 Merge (spec 017): derive_prompt with strategy="merge" ──────────────────────

    #[test]
    fn derive_prompt_merge_unions_variables() {
        let shape = serde_json::json!({
            "name": "base",
            "role": "user",
            "body": "{{ extraction }}",
            "variables": { "extraction": { "type": "string", "trusted": true } },
        });
        let original = prompt_new(shape).expect("valid");

        let overlay = serde_json::json!({
            "body": "{{ extraction }} {{ sentiment }}",
            "variables": { "sentiment": { "type": "string", "trusted": true } },
        });
        let derived = original
            .derive_prompt(overlay, Some("merge".to_string()))
            .expect("Merge must succeed");

        let vars = derived.variables().expect("vars");
        let obj = vars.as_object().expect("vars is object");
        assert!(obj.contains_key("extraction"), "base var retained");
        assert!(obj.contains_key("sentiment"), "overlay var added");
        assert_eq!(obj.len(), 2);
    }

    #[test]
    fn derive_prompt_merge_replace_default_parity() {
        // derive_prompt(overlay, None) == derive_prompt(overlay, Some("replace")) (SC-002).
        let shape = serde_json::json!({
            "name": "base",
            "role": "user",
            "body": "{{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let base = prompt_new(shape).expect("valid");

        let overlay = serde_json::json!({ "body": "Hello {{ name }}!" });
        let via_none = base
            .derive_prompt(overlay.clone(), None)
            .expect("derive with None");
        let via_replace = base
            .derive_prompt(overlay, Some("replace".to_string()))
            .expect("derive with replace");

        assert_eq!(via_none.body(), via_replace.body());
    }

    #[test]
    fn derive_prompt_unknown_strategy_is_error() {
        let shape = serde_json::json!({
            "name": "base", "role": "user", "body": "{{ name }}",
            "variables": { "name": { "type": "string", "trusted": true } },
        });
        let base = prompt_new(shape).expect("valid");
        let err = base
            .derive_prompt(serde_json::json!({}), Some("deep".to_string()))
            .expect_err("unknown strategy must fail");
        let payload = payload_of(&err);
        assert_eq!(
            payload["code"], "load",
            "unknown strategy maps to load error (FR-011)"
        );
    }
}
