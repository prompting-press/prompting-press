//! The immutable [`Prompt`] value object and its [`PromptOverlay`].
//!
//! [`Prompt`] is the library's primary public type: a validated, immutable facade over a
//! [`PromptDefinition`]. Construction (`new`, `from_yaml`, `from_json`, `from_toml`) is
//! **validating** — a `Prompt` that constructs is guaranteed to be:
//!
//! 1. **Shape-valid** — the document parsed to a `PromptDefinition` (serde layer).
//! 2. **Template-parseable and analyzable** — every variant body (including the root body)
//!    is parseable by the kernel and free of excluded features; construction FAILS on an
//!    un-analyzable body.
//! 3. **Agreement-sound** — every variable a variant template references is declared in
//!    `variables`; a referenced-but-undeclared variable is a construction failure. The
//!    agreement check therefore moves ONTO construction; a constructed `Prompt` carries no
//!    undeclared-variable references.
//! 4. **Reserved-name clean** — no variant is literally named `"default"` (the kernel's
//!    reserved root-body alias); that is a construction failure.
//!
//! After construction every operation is infallible with respect to the above invariants;
//! `check()` is a pure advisory pass that can only surface the trust/guard finding (a prompt
//! with `trusted: false` vars and no guard configured).
//!
//! ## `derive` — the sole mutator
//!
//! [`Prompt::derive`] shallow-replaces top-level fields via a [`PromptOverlay`] and routes the
//! merged definition through `Prompt::new` (full re-validation). The original `Prompt` is
//! untouched. In Rust the validator is generic `V` named at the `render` call site
//! (compile-time coverage); `PromptOverlay` therefore carries only data fields — no runtime
//! validator object (the Rust asymmetry from the constitution's per-language-idiom principle).
//!
//! ## No I/O
//!
//! The text-factory methods accept already-read text — the caller hands it in. This crate
//! reads no files.

use std::collections::HashMap;

use garde::Validate;
use prompting_press_core::{
    required_roots, untrusted_fields, GuardConfig, KernelError, RenderResult,
};
use serde::Serialize;

use crate::check::{has_guard_configured, CheckReport, Finding, FindingKind};
use crate::error::code;
use crate::{ConsumerError, FieldError};
use prompting_press_core::generated::prompt_definition::{
    PromptDefinition, PromptDefinitionName, PromptDefinitionRole, PromptVariable, PromptVariant,
};

// ─── constants ───────────────────────────────────────────────────────────────

/// The kernel's reserved variant name for the root body (mirrors `check::DEFAULT_VARIANT`).
/// Re-declared here so `prompt.rs` has no public dep on `check`'s internal constants.
const DEFAULT: &str = "default";

// ─── Prompt ──────────────────────────────────────────────────────────────────

/// An immutable, fully-validated prompt. Wraps a [`PromptDefinition`]; all invariants
/// (shape-valid, template-parseable, agreement-sound, reserved-name clean) are enforced at
/// construction time. There are no setters; the sole mutator is [`Prompt::derive`].
#[derive(Debug, Clone)]
pub struct Prompt {
    /// The validated definition. Private; exposed only through read-only accessors.
    def: PromptDefinition,
}

impl Prompt {
    // ── constructors ────────────────────────────────────────────────────────

    /// The primary validating constructor.
    ///
    /// Runs the construction invariants on `def`:
    /// 1. For each variant arm (root body + every named variant), asks the kernel for the
    ///    arm's [`required_roots`]. A kernel `Err` (parse failure or excluded feature) is a
    ///    **construction failure** — the `Prompt` is not built.
    /// 2. Each analyzable arm's referenced roots must be a subset of the declared `variables`.
    ///    Any root not declared is an agreement violation.
    /// 3. A variant named literally `"default"` is rejected — the kernel reserves that
    ///    name for the root body; the declared arm would be unreachable.
    ///
    /// On success the `Prompt` is returned. On any violation a structured
    /// [`ConsumerError`] is returned — never a panic.
    ///
    /// # Errors
    ///
    /// - [`ConsumerError::Kernel`] — a variant template could not be parsed or uses an
    ///   excluded feature (`{% include %}` / macros / inheritance).
    /// - [`ConsumerError::Kernel`] — a variant template references a variable not declared
    ///   in `variables` (agreement failure; `code::UNDEFINED_VARIABLE`).
    /// - [`ConsumerError::Kernel`] — a variant is literally named `"default"` (reserved;
    ///   `code::UNDEFINED_VARIABLE` with field `"variant"`).
    pub fn new(def: PromptDefinition) -> Result<Self, ConsumerError> {
        validate_prompt_def(&def)?;
        Ok(Self { def })
    }

    /// Deserialize a `Prompt` from already-read **YAML** text, then validate.
    ///
    /// Equivalent to `serde_yaml_ng::from_str(..)` + [`Prompt::new`]. A parse/shape error
    /// returns [`ConsumerError::Load`]; a validation error returns the same errors as `new`.
    ///
    /// The crate reads no files — the caller supplies already-read text.
    ///
    /// # Errors
    ///
    /// [`ConsumerError::Load`] if `text` is not valid YAML or does not match the
    /// `PromptDefinition` shape, or any error from [`Prompt::new`].
    pub fn from_yaml(text: &str) -> Result<Self, ConsumerError> {
        let def: PromptDefinition =
            serde_yaml_ng::from_str(text).map_err(|e| ConsumerError::Load(e.to_string()))?;
        Self::new(def)
    }

    /// Deserialize a `Prompt` from already-read **JSON** text, then validate.
    ///
    /// Equivalent to `serde_json::from_str(..)` + [`Prompt::new`]. Error semantics mirror
    /// [`from_yaml`](Self::from_yaml).
    ///
    /// # Errors
    ///
    /// [`ConsumerError::Load`] if `text` is not valid JSON or does not match the
    /// `PromptDefinition` shape, or any error from [`Prompt::new`].
    pub fn from_json(text: &str) -> Result<Self, ConsumerError> {
        let def: PromptDefinition =
            serde_json::from_str(text).map_err(|e| ConsumerError::Load(e.to_string()))?;
        Self::new(def)
    }

    /// Deserialize a `Prompt` from already-read **TOML** text, then validate.
    ///
    /// Uses `toml::from_str` (the serde-native TOML crate — research R3 / `toml@1.1.2`).
    /// Error semantics mirror [`from_yaml`](Self::from_yaml).
    ///
    /// # Errors
    ///
    /// [`ConsumerError::Load`] if `text` is not valid TOML or does not match the
    /// `PromptDefinition` shape, or any error from [`Prompt::new`].
    pub fn from_toml(text: &str) -> Result<Self, ConsumerError> {
        let def: PromptDefinition =
            toml::from_str(text).map_err(|e| ConsumerError::Load(e.to_string()))?;
        Self::new(def)
    }

    // ── read-only accessors ──────────────────────────────────────────────────

    /// The prompt's name (the `name` field of the underlying definition).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.def.name
    }

    /// The conversational role (`system` / `user` / `assistant`).
    #[must_use]
    pub fn role(&self) -> &PromptDefinitionRole {
        &self.def.role
    }

    /// The root body template source (the default arm's unrendered template).
    #[must_use]
    pub fn body(&self) -> &str {
        &self.def.body
    }

    /// The declared variables map (`name → PromptVariable`).
    #[must_use]
    pub fn variables(&self) -> &HashMap<String, PromptVariable> {
        &self.def.variables
    }

    /// The named variants map (`name → PromptVariant`). Empty when the prompt has no named
    /// variants (only the implicit default arm).
    #[must_use]
    pub fn variants(&self) -> &HashMap<String, PromptVariant> {
        &self.def.variants
    }

    /// The output model reference, if declared (`output_model` field). Carried as metadata
    /// only — never parsed or resolved by this library.
    #[must_use]
    pub fn output_model(&self) -> Option<&str> {
        self.def.output_model.as_deref()
    }

    /// The `metadata` opaque map (library-defined top-level annotations, if any).
    #[must_use]
    pub fn metadata(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.def.metadata
    }

    // ── operations ───────────────────────────────────────────────────────────

    /// Validate-then-render this prompt.
    ///
    /// 1. Validates `vars` once via garde, BEFORE any templating. On failure returns
    ///    [`ConsumerError::Validation`] — the kernel is never reached.
    /// 2. Bridges the validated struct to the kernel's value type via
    ///    [`minijinja::Value::from_serialize`].
    /// 3. Delegates to [`prompting_press_core::render`], normalizing any
    ///    [`KernelError`] to [`ConsumerError::Kernel`].
    ///
    /// `variant = None` selects the default (root body) arm. `guard` is plumbed straight
    /// through to the kernel; `RenderResult::guard` is surfaced unchanged (guard *expansion*
    /// is the kernel's contract).
    ///
    /// `V::Context: Default` so the whole-struct [`Validate::validate`] convenience applies
    /// (one validation pass over the entire input set). Context-carrying validation
    /// is intentionally out of v1 scope (one concrete path per concern).
    ///
    /// ## Byte-identical output
    ///
    /// The `RenderResult` hashes are byte-identical across Rust, Python, and TypeScript for
    /// the same template and inputs, because all three bindings share this kernel render path.
    ///
    /// ## `reveal_render_detail` — unsafe, off-by-default render-error detail opt-in
    ///
    /// Pass `false` in all production call sites (the default). When `true`, the full
    /// underlying render-error detail is surfaced in the returned
    /// [`ConsumerError::Kernel`] message instead of the fixed scrubbed string.
    ///
    /// **Risk:** enabling this may place **bound-value content** — untrusted input, PII,
    /// secrets — into the returned error message and into any log line or stack trace
    /// derived from it. Use only in a controlled debug context where you own the log
    /// destination and deliberately accept that exposure. Never set `true` by default or
    /// in ambient/global configuration.
    ///
    /// # Errors
    ///
    /// - [`ConsumerError::Validation`] — garde rejected `vars`.
    /// - [`ConsumerError::Kernel`] — the kernel rejected the render (unknown variant,
    ///   strict-undefined reference, parse/render failure). `Render` detail scrubbed
    ///   unless `reveal_render_detail = true`. `Parse` detail always preserved.
    pub fn render<V>(
        &self,
        vars: &V,
        variant: Option<&str>,
        guard: &GuardConfig,
        reveal_render_detail: bool,
    ) -> Result<RenderResult, ConsumerError>
    where
        V: Serialize + Validate,
        V::Context: Default,
    {
        // 1. Validate once, BEFORE any templating.
        //    Validation errors use the plain From scrubber — never from_kernel_revealing
        //    (validation is not a kernel render error; error scrubbing applies to Render detail only).
        vars.validate().map_err(ConsumerError::from)?;

        // 2. Bridge the validated struct to the kernel's value type.
        //    `from_serialize` is infallible: a custom-Serialize failure would surface
        //    downstream as a strict-undefined kernel error, never silently here.
        let values = minijinja::Value::from_serialize(vars);

        // 3. Delegate to the kernel; normalize KernelError via the per-call opt-in.
        //    When reveal_render_detail=false this is byte-for-byte the same as From.
        //    The kernel receives ONLY already-validated values; the consumer adds
        //    no render/agreement/variant/hash logic of its own.
        prompting_press_core::render(&self.def, variant, values, guard)
            .map_err(|e| ConsumerError::from_kernel_revealing(e, reveal_render_detail))
    }

    /// Return a variant's unrendered template source (the exact string the kernel hashes
    /// into `template_hash`). Delegates to the kernel; no vars, no validation.
    ///
    /// `variant = None` returns the root body source.
    ///
    /// # Errors
    ///
    /// [`ConsumerError::Kernel`] — the kernel rejected the lookup (unknown variant name).
    pub fn get_source(&self, variant: Option<&str>) -> Result<&str, ConsumerError> {
        prompting_press_core::get_source(&self.def, variant).map_err(ConsumerError::from)
    }

    /// Pure advisory lint: returns a [`CheckReport`] containing only the trust/guard
    /// finding class.
    ///
    /// Construction already enforces agreement, parse, and reserved-name invariants, so those
    /// arms are structurally unreachable for a constructed `Prompt`. The only LIVE finding
    /// `check()` can surface is [`FindingKind::UntrustedWithoutGuard`] — a prompt declaring
    /// `trusted: false` vars but carrying no `"guard"` key in `metadata`.
    ///
    /// Pure: takes `&self`, never renders, never mutates.
    #[must_use]
    pub fn check(&self) -> CheckReport {
        let mut findings = Vec::new();
        check_origin_advisory(self.name(), &self.def, &mut findings);
        CheckReport { findings }
    }

    /// The sole mutator: shallow-replace top-level fields from `overlay` onto a clone of
    /// this prompt's definition, then route the merged definition through [`Prompt::new`]
    /// (full re-validation). The original `Prompt` is untouched.
    ///
    /// Each `Some(field)` in `overlay` replaces the corresponding field; `None` fields are
    /// left as-is. `name` IS overlayable (the overlay can rename a prompt). After the
    /// merge, every construction invariant is re-checked over the whole merged definition —
    /// so an overlay that introduces an agreement violation or a reserved variant name is
    /// rejected.
    ///
    /// Uses [`MergeStrategy::Replace`] (the default): each overlay-present map field
    /// replaces the base's map field wholesale — byte-identical to the pre-017 behavior.
    ///
    /// In Rust the validator is the generic `V` named at the `render` call site (garde
    /// covers all fields at compile time); `derive` takes `&self` and carries no runtime
    /// validator. `PromptOverlay` therefore contains only data fields.
    ///
    /// # Errors
    ///
    /// Same error classes as [`Prompt::new`]: a merged definition that fails any construction
    /// invariant returns the structured error.
    pub fn derive(&self, overlay: PromptOverlay) -> Result<Self, ConsumerError> {
        self.derive_with(overlay, DeriveOptions::default())
    }

    /// Strategy-aware mutator: merge `overlay` onto this prompt's definition using the
    /// selected [`MergeStrategy`], then re-validate the whole merged definition.
    ///
    /// - [`MergeStrategy::Replace`] — byte-identical to [`derive`](Self::derive). Each
    ///   overlay-present field replaces the base's field wholesale (the default).
    /// - [`MergeStrategy::Merge`] — for the three map-typed fields (`variables`, `variants`,
    ///   `metadata`) performs a **top-level key union** with child-wins-whole-entry on
    ///   collision (no recursion). Scalar fields (`name`, `role`, `body`, `output_model`)
    ///   still replace when overlay-present. Reserved axes `deep`/`none` are excluded (C-08).
    ///
    /// The merged definition is always routed through [`Prompt::new`] (full re-validation:
    /// agreement check, template parse, reserved-name check) — no invariant is weakened by
    /// the strategy choice.
    ///
    /// The original `Prompt` is untouched (immutability).
    ///
    /// # Errors
    ///
    /// Same error classes as [`Prompt::new`]: a merged definition that fails any construction
    /// invariant returns the structured error.
    pub fn derive_with(
        &self,
        overlay: PromptOverlay,
        options: DeriveOptions,
    ) -> Result<Self, ConsumerError> {
        // Serialize the current definition and overlay to JSON values, run the shared
        // merge helper, then deserialize back through the validating constructor.
        //
        // This JSON-round-trip approach is the single-source path (FR-018 / R8): the same
        // `merge_definitions` helper is called by both this typed path AND the Node binding,
        // guaranteeing byte-identical union across bindings by construction (Principle I / D1).
        let base_json = serde_json::to_value(&self.def).map_err(|e| {
            ConsumerError::Load(format!(
                "internal: failed to serialize base definition: {e}"
            ))
        })?;
        let overlay_json = serde_json::to_value(&overlay).map_err(|e| {
            ConsumerError::Load(format!("internal: failed to serialize overlay: {e}"))
        })?;

        let merged_json = merge_definitions(base_json, overlay_json, options.strategy)?;

        let merged: prompting_press_core::generated::prompt_definition::PromptDefinition =
            serde_json::from_value(merged_json).map_err(|e| {
                ConsumerError::Load(format!(
                    "internal: failed to deserialize merged definition: {e}"
                ))
            })?;

        // Re-validate the merged whole through the same construction path.
        Self::new(merged)
    }

    /// Borrow the underlying [`PromptDefinition`] for use by binding crates
    /// (e.g. `prompting-press-node`, `prompting-press-py`) that need to call the kernel
    /// directly for `render/get_source` (their validation is owned in the binding layer, not
    /// in Rust garde, so the consumer's generic `render<V>` is not usable there). Bindings
    /// call `prompting_press_core::render(prompt.definition(), ...)` directly after doing
    /// their own validation — the same zero-engine-logic pattern as the `Prompt::render`
    /// path. Also used by `Composition::resolve` within this crate.
    #[must_use]
    pub fn definition(&self) -> &PromptDefinition {
        &self.def
    }
}

// ─── MergeStrategy + DeriveOptions ───────────────────────────────────────────

/// Selects how [`Prompt::derive_with`] combines map-typed overlay fields with the base.
///
/// Two strategies are supported in this release (the merge/replace industry-standard pair —
/// RFC 7386 JSON Merge Patch, Kubernetes, Terraform `merge()`). The enum is designed so a
/// future value (e.g. `Deep`) can be added without a new method or a breaking signature
/// change (C-08 — reserved axis, earned by a future second consumer).
///
/// ## Semantics
///
/// - [`Replace`](MergeStrategy::Replace) — each overlay-present top-level field replaces
///   the base's field wholesale. This is the default and reproduces the pre-017 behavior
///   exactly (SC-002).
/// - [`Merge`](MergeStrategy::Merge) — for the three map-typed fields (`variables`,
///   `variants`, `metadata`) performs a **top-level key union** with child-wins-whole-entry
///   on collision (no recursion into entry contents — that would be the excluded `deep`
///   strategy). Scalar fields (`name`, `role`, `body`, `output_model`) still replace when
///   overlay-present.
///
/// ## Soundness boundary (FR-019)
///
/// The agreement check is name-only (`required_roots ⊆ declared variable names`). Consequently:
/// - A `Merge` that **removes** a variable a base variant body still references fails
///   construction via the agreement check (name-removal caught across all variant arms).
/// - A `Merge` that **replaces** a variable's declaration (changing its `type` or `trusted`
///   flag) is **accepted** — type/trust correctness is the validator's responsibility.
///
/// The merged definition is always re-validated through [`Prompt::new`] regardless of
/// strategy, so no construction invariant is weakened by the strategy choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeStrategy {
    /// Wholesale field replacement (the default). Each overlay-present field replaces the
    /// base's field; absent overlay fields are left untouched. Byte-identical to pre-017
    /// behavior (SC-002).
    #[default]
    Replace,
    /// Top-level key union for map-typed fields (`variables`, `variants`, `metadata`).
    /// Child-wins-whole-entry on key collision; no recursion (no `deep`). Scalar fields
    /// replace when overlay-present.
    Merge,
}

/// Options for [`Prompt::derive_with`]. Implements [`Default`] so callers use
/// `DeriveOptions { strategy: MergeStrategy::Merge, ..Default::default() }` — forward-
/// extensible without a breaking signature change when future options are added.
#[derive(Debug, Clone, Default)]
pub struct DeriveOptions {
    /// The merge strategy to apply. Defaults to [`MergeStrategy::Replace`].
    pub strategy: MergeStrategy,
}

// ─── shared merge helper (FR-018 / R8) ───────────────────────────────────────

/// The **single-source** map-union algorithm shared by both the typed [`Prompt::derive_with`]
/// path and the Node binding's `derive_prompt`.
///
/// `base` and `overlay` must be `serde_json::Value::Object`s. Each top-level key in
/// `overlay` is applied to `base` according to `strategy`:
///
/// - [`MergeStrategy::Replace`]: each overlay-present key replaces the base's key wholesale
///   (the pre-017 behavior, preserved exactly).
/// - [`MergeStrategy::Merge`]: for the three map-typed fields (`variables`, `variants`,
///   `metadata`) the function unions top-level sub-keys with child-wins-whole-entry
///   (no recursion). All other keys (scalar fields) replace wholesale, same as `Replace`.
///
/// By operating entirely in `serde_json::Value` space this helper is the common denominator
/// for both the typed Rust path (which serializes `PromptDefinition` + `PromptOverlay` to
/// `Value` before calling this) and the Node binding (which is already JSON-native). This
/// guarantees byte-identical results across bindings by construction (Principle I / D1) —
/// per-binding date/decimal serialization would otherwise let a typed-map union and a
/// JSON-space union diverge.
///
/// # Errors
///
/// [`ConsumerError::Load`] if `base` or `overlay` is not a JSON object.
pub fn merge_definitions(
    base: serde_json::Value,
    overlay: serde_json::Value,
    strategy: MergeStrategy,
) -> Result<serde_json::Value, ConsumerError> {
    // The three map-typed fields subject to key-union under Merge.
    const MAP_FIELDS: &[&str] = &["variables", "variants", "metadata"];

    let serde_json::Value::Object(mut base_obj) = base else {
        return Err(ConsumerError::Load(
            "internal: base is not a JSON object".to_string(),
        ));
    };
    let serde_json::Value::Object(overlay_obj) = overlay else {
        return Err(ConsumerError::Load(
            "internal: overlay is not a JSON object".to_string(),
        ));
    };

    for (key, overlay_val) in overlay_obj {
        // Under Merge, attempt a top-level key union for the three map-typed fields.
        // Both sides must be JSON objects for the union to apply; otherwise fall through
        // to wholesale replace (conservative — construction will validate the result).
        if strategy == MergeStrategy::Merge
            && MAP_FIELDS.contains(&key.as_str())
            && overlay_val.is_object()
        {
            let base_is_object = matches!(
                base_obj.get(&key),
                Some(serde_json::Value::Object(_)) | None
            );
            if base_is_object {
                let base_sub = base_obj
                    .entry(&key)
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let (
                    serde_json::Value::Object(base_sub_obj),
                    serde_json::Value::Object(overlay_sub),
                ) = (base_sub, overlay_val)
                {
                    // Child-wins: extend base sub-map with overlay sub-map entries.
                    base_sub_obj.extend(overlay_sub);
                    continue;
                }
                // Unreachable: both checked above — but if we somehow fall through, the
                // borrow checker prevents us from re-using overlay_val. Exit gracefully.
                continue;
            }
        }
        // Replace (both strategies for scalars; Replace for all fields under Replace;
        // also used when overlay map value is not an object under Merge).
        base_obj.insert(key, overlay_val);
    }

    Ok(serde_json::Value::Object(base_obj))
}

// ─── PromptOverlay ───────────────────────────────────────────────────────────

/// A shallow-replacement overlay for [`Prompt::derive`].
///
/// Each field is `Option<T>`. A `Some(value)` replaces the corresponding field on the
/// cloned definition; a `None` leaves it unchanged. All fields are optional — pass only
/// what should change.
///
/// `name` is overlayable: a prompt can be renamed (useful for template-derived variants).
/// After merging, the full construction invariants (agreement, parse, reserved name) are
/// re-checked over the merged whole.
///
/// In Rust the validator is the generic `V` named at the call site; `PromptOverlay` carries
/// **only data fields** — no runtime validator object (the Rust compile-time asymmetry
/// documented in R6).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PromptOverlay {
    /// Replace the prompt's `name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<PromptDefinitionName>,
    /// Replace the prompt's `role`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<PromptDefinitionRole>,
    /// Replace the root body template source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Replace the full `variables` map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, PromptVariable>>,
    /// Replace the full `variants` map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, PromptVariant>>,
    /// Replace (or clear) the `output_model` reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_model: Option<Option<String>>,
    /// Replace the `metadata` opaque map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

// ─── internal helpers ────────────────────────────────────────────────────────

/// Run all construction invariants over `def`. Returns `Ok(())` on success; the first
/// violated invariant returns the structured `ConsumerError`.
///
/// Invariants (in order):
/// 1. Reserved variant name (`"default"` in `variants`) → rejected.
/// 2. For each variant arm: kernel `required_roots` must not `Err` (parse / excluded
///    feature → construction failure).
/// 3. Referenced roots must be a subset of declared `variables` (agreement check).
fn validate_prompt_def(def: &PromptDefinition) -> Result<(), ConsumerError> {
    // 1. Reject a variant literally named "default" (reserved name for the root body).
    if def.variants.contains_key(DEFAULT) {
        return Err(ConsumerError::Kernel(vec![FieldError {
            field: "variant".to_string(),
            code: code::UNDEFINED_VARIABLE.to_string(),
            message: format!(
                "variant `{DEFAULT}` uses the reserved name for the root body; rename it \
                 or move its body to the root"
            ),
        }]));
    }

    let declared: std::collections::BTreeSet<&str> =
        def.variables.keys().map(String::as_str).collect();

    // Variants to analyze: default arm first (None), then every named arm.
    // The "default" key is already excluded above (construction-failed).
    let mut arms: Vec<Option<&str>> = vec![None];
    arms.extend(def.variants.keys().map(|k| Some(k.as_str())));

    for variant_opt in arms {
        let variant_label = variant_opt.unwrap_or(DEFAULT);

        // 2. Parse + required_roots (construction fails on Err).
        let agreement = required_roots(def, variant_opt).map_err(|e| {
            let (field, msg, c) = kernel_analysis_error_to_field(&e);
            ConsumerError::Kernel(vec![FieldError {
                field: field.to_string(),
                code: c.to_string(),
                message: msg,
            }])
        })?;

        // 3. Agreement check: referenced roots ⊆ declared variables.
        for root in &agreement.required_roots {
            if !declared.contains(root.as_str()) {
                return Err(ConsumerError::Kernel(vec![FieldError {
                    field: root.clone(),
                    code: code::UNDEFINED_VARIABLE.to_string(),
                    message: format!(
                        "template references undeclared variable `{root}` \
                         (variant `{variant_label}`); add it to the prompt's `variables`"
                    ),
                }]));
            }
        }
    }

    Ok(())
}

/// Map a kernel analysis error to `(field, message, code)` for a construction-failure
/// `ConsumerError`. Scrubbed — no bound-value content.
fn kernel_analysis_error_to_field(err: &KernelError) -> (&'static str, String, &'static str) {
    match err {
        KernelError::UnknownVariant { requested } => (
            "variant",
            format!("unknown variant: `{requested}`"),
            code::UNKNOWN_VARIANT,
        ),
        KernelError::UndefinedVariable { name } => (
            "template",
            format!("undefined variable at render: `{name}`"),
            code::UNDEFINED_VARIABLE,
        ),
        // detail may embed bound-value content — DO NOT copy it.
        KernelError::Parse { detail: _ } => {
            ("template", "template parse error".to_string(), code::PARSE)
        }
        KernelError::Render { detail: _ } => ("template", "render error".to_string(), code::RENDER),
        KernelError::ExcludedFeature { detail: _ } => (
            "template",
            "template uses an excluded feature".to_string(),
            code::EXCLUDED_FEATURE,
        ),
        // Only raised when a caller supplies an invalid guard advisory override.
        // The detail names the missing element(s) — no bound value content.
        KernelError::GuardAdvisoryInvalid { detail } => (
            "guard",
            format!("guard advisory override is invalid: {detail}"),
            code::RENDER,
        ),
    }
}

/// The trust/guard advisory check for a single prompt (the only LIVE finding class for a
/// constructed `Prompt`).
///
/// A prompt declaring variables with `trusted: false` that carry no `"guard"` key in
/// `metadata` gets one [`FindingKind::UntrustedWithoutGuard`] per uncovered field.
/// This mirrors `check::check_provenance` but operates on a single `Prompt`, not a registry.
pub(crate) fn check_origin_advisory(
    name: &str,
    def: &PromptDefinition,
    findings: &mut Vec<Finding>,
) {
    // Fields where trusted == false (sorted BTreeSet → deterministic order).
    let declared_untrusted = untrusted_fields(def);

    if declared_untrusted.is_empty() {
        return;
    }

    if has_guard_configured(def) {
        return;
    }

    for field in &declared_untrusted {
        findings.push(Finding {
            prompt: name.to_string(),
            variant: None,
            kind: FindingKind::UntrustedWithoutGuard {
                field: field.clone(),
            },
            detail: format!(
                "field `{field}` is declared untrusted (`trusted: false`) but the prompt \
                 configures no guard (add a `guard` key under the prompt's `metadata`)"
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn valid_json() -> &'static str {
        r#"{"name":"greet","role":"user","body":"Hi {{ name }}","variables":{"name":{"type":"string","trusted":true}}}"#
    }

    fn make_prompt() -> Prompt {
        Prompt::from_json(valid_json()).expect("valid prompt must construct")
    }

    // ── T033: construction valid ──────────────────────────────────────────────

    #[test]
    fn construct_valid_prompt_succeeds() {
        let p = make_prompt();
        assert_eq!(p.name(), "greet");
        assert_eq!(p.body(), "Hi {{ name }}");
        assert!(p.variables().contains_key("name"));
        assert!(p.variants().is_empty());
    }

    // ── T033: construction invalid — undeclared variable ─────────────────────

    #[test]
    fn construct_rejects_undeclared_variable() {
        // `body` references `ghost` which is not in `variables`.
        let json = r#"{"name":"bad","role":"user","body":"{{ ghost }}","variables":{"name":{"type":"string","trusted":true}}}"#;
        let err = Prompt::from_json(json).expect_err("undeclared var must fail construction");
        match &err {
            ConsumerError::Kernel(rows) => {
                assert!(
                    rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                    "expected undefined_variable, got {rows:?}"
                );
                assert!(
                    rows.iter().any(|r| r.message.contains("ghost")),
                    "error must name the offending variable, got {rows:?}"
                );
            }
            other => panic!("expected ConsumerError::Kernel, got {other:?}"),
        }
    }

    // ── T033: construction invalid — excluded feature in body ─────────────────

    #[test]
    fn construct_rejects_excluded_feature_body() {
        let json = r#"{"name":"bad","role":"user","body":"{% include \"x\" %}","variables":{}}"#;
        let err = Prompt::from_json(json).expect_err("excluded feature must fail construction");
        match &err {
            ConsumerError::Kernel(rows) => {
                let codes: Vec<&str> = rows.iter().map(|r| r.code.as_str()).collect();
                assert!(
                    codes.contains(&code::EXCLUDED_FEATURE) || codes.contains(&code::PARSE),
                    "expected excluded_feature or parse code, got {codes:?}"
                );
            }
            other => panic!("expected ConsumerError::Kernel, got {other:?}"),
        }
    }

    // ── T033: construction invalid — syntax error ─────────────────────────────

    #[test]
    fn construct_rejects_syntax_error() {
        let json = r#"{"name":"bad","role":"user","body":"{{ unclosed","variables":{}}"#;
        let err = Prompt::from_json(json).expect_err("syntax error must fail construction");
        match &err {
            ConsumerError::Kernel(rows) => {
                let codes: Vec<&str> = rows.iter().map(|r| r.code.as_str()).collect();
                assert!(
                    codes.contains(&code::PARSE) || codes.contains(&code::EXCLUDED_FEATURE),
                    "expected parse code, got {codes:?}"
                );
            }
            other => panic!("expected ConsumerError::Kernel, got {other:?}"),
        }
    }

    // ── T033: construction invalid — variant named "default" ─────────────────

    #[test]
    fn construct_rejects_reserved_variant_name() {
        let json = r#"{"name":"bad","role":"user","body":"Hi","variables":{},"variants":{"default":{"body":"shadowed"}}}"#;
        let err =
            Prompt::from_json(json).expect_err("reserved variant name must fail construction");
        match &err {
            ConsumerError::Kernel(rows) => {
                assert_eq!(rows[0].field, "variant", "field must be 'variant'");
                assert!(
                    rows[0].message.contains("reserved") || rows[0].message.contains("default"),
                    "message must mention the reserved name, got {:?}",
                    rows[0].message
                );
            }
            other => panic!("expected ConsumerError::Kernel, got {other:?}"),
        }
    }

    // ── T033: `with` adds a variant; original unchanged ───────────────────────

    #[test]
    fn derive_adds_variant_original_unchanged() {
        let original = make_prompt();
        let original_body = original.body().to_string();
        let original_variants_count = original.variants().len();

        // Overlay: add a named variant that references the same declared variable.
        let mut new_variants = original.variants().clone();
        new_variants.insert(
            "brief".to_string(),
            serde_json::from_value(serde_json::json!({"body": "Hey {{ name }}"}))
                .expect("valid variant"),
        );

        let derived = original
            .derive(PromptOverlay {
                variants: Some(new_variants),
                ..Default::default()
            })
            .expect("derive must succeed for a valid overlay");

        // Derived has the new variant.
        assert!(derived.variants().contains_key("brief"));

        // Original is untouched (immutability — SC-004).
        assert_eq!(original.body(), original_body);
        assert_eq!(original.variants().len(), original_variants_count);
    }

    // ── T033: `derive` producing undeclared var → Err ────────────────────────

    #[test]
    fn derive_undeclared_var_body_returns_err() {
        let original = make_prompt();

        // Overlay replaces body with one that references an undeclared variable.
        let err = original
            .derive(PromptOverlay {
                body: Some("{{ name }} {{ ghost }}".to_string()),
                ..Default::default()
            })
            .expect_err("overlay with undeclared var must fail");
        match &err {
            ConsumerError::Kernel(rows) => {
                assert!(
                    rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                    "expected undefined_variable, got {rows:?}"
                );
            }
            other => panic!("expected ConsumerError::Kernel, got {other:?}"),
        }
    }

    // ── T033: from_toml round-trips ────────────────────────────────────────────

    #[test]
    fn from_toml_round_trips() {
        let toml_text = r#"
name = "greeting"
role = "user"
body = "Hi {{ name }}"

[variables.name]
type = "string"
trusted = true
"#;
        let p = Prompt::from_toml(toml_text).expect("TOML must construct");
        assert_eq!(p.name(), "greeting");
        assert_eq!(p.body(), "Hi {{ name }}");
        assert!(p.variables().contains_key("name"));
    }

    // ── T033: render byte-identical hashes ────────────────────────────────────

    #[test]
    fn render_byte_identical_hashes_across_two_renders() {
        use garde::Validate;
        use serde::Serialize;

        #[derive(Serialize, Validate)]
        struct V {
            #[garde(length(min = 1))]
            name: String,
        }

        let p = make_prompt();
        let vars = V {
            name: "Ada".to_string(),
        };

        let r1 = p
            .render(&vars, None, &GuardConfig::default(), false)
            .expect("render 1");
        let r2 = p
            .render(&vars, None, &GuardConfig::default(), false)
            .expect("render 2");

        assert_eq!(r1.text, r2.text, "text must be byte-identical");
        assert_eq!(
            r1.template_hash, r2.template_hash,
            "template_hash must be byte-identical"
        );
        assert_eq!(
            r1.render_hash, r2.render_hash,
            "render_hash must be byte-identical"
        );
    }

    // ── T033: check() returns only origin/guard advisory ─────────────────────

    #[test]
    fn check_returns_origin_advisory_only() {
        // A prompt with an untrusted variable and no guard → should find UntrustedWithoutGuard.
        let json = r#"{"name":"unguarded","role":"user","body":"{{ payload }}","variables":{"payload":{"type":"string","trusted":false}}}"#;
        let p = Prompt::from_json(json).expect("valid shape, should construct");
        let report = p.check();
        assert!(
            !report.passed(),
            "unguarded untrusted field should produce a finding"
        );
        assert!(report
            .findings
            .iter()
            .all(|f| matches!(&f.kind, FindingKind::UntrustedWithoutGuard { .. })));
    }

    #[test]
    fn check_passes_for_guarded_untrusted_field() {
        let json = r#"{"name":"guarded","role":"user","body":"{{ payload }}","variables":{"payload":{"type":"string","trusted":false}},"metadata":{"guard":{"enabled":true}}}"#;
        let p = Prompt::from_json(json).expect("valid shape");
        assert!(p.check().passed(), "guard configured → check must pass");
    }

    // ── T033: get_source delegates to kernel ──────────────────────────────────

    #[test]
    fn get_source_returns_root_body() {
        let p = make_prompt();
        let src = p.get_source(None).expect("root source must resolve");
        assert_eq!(src, "Hi {{ name }}");
    }

    // ── spec 013 T002/T003: reveal_render_detail flag ─────────────────────────

    /// T002 (a): flag on vs off produces byte-identical output on the SUCCESS path (SC-005).
    /// Agreement is enforced at construction so post-construction render errors are rare;
    /// the reveal seam is unit-tested in error.rs (T001). Here we confirm the flag has no
    /// effect on the success path.
    #[test]
    fn reveal_flag_does_not_change_success_path() {
        #[derive(serde::Serialize, garde::Validate)]
        struct V {
            #[garde(length(min = 1))]
            name: String,
        }
        let vars = V {
            name: "Ada".to_string(),
        };
        let p = make_prompt();

        let r_false = p
            .render(&vars, None, &GuardConfig::default(), false)
            .expect("render with false must succeed");
        let r_true = p
            .render(&vars, None, &GuardConfig::default(), true)
            .expect("render with true must succeed");

        // SC-005: text and both hashes are byte-identical regardless of the flag.
        assert_eq!(r_false.text, r_true.text, "text must be byte-identical");
        assert_eq!(
            r_false.template_hash, r_true.template_hash,
            "template_hash must be byte-identical"
        );
        assert_eq!(
            r_false.render_hash, r_true.render_hash,
            "render_hash must be byte-identical"
        );
    }

    /// T003: validation errors are unchanged by the reveal flag (validation uses
    /// `ConsumerError::from`, not `from_kernel_revealing` — the kernel is never reached).
    #[test]
    fn reveal_flag_does_not_affect_validation_errors() {
        #[derive(serde::Serialize, garde::Validate)]
        struct V {
            #[garde(length(min = 1))]
            name: String,
        }
        let invalid_vars = V {
            name: String::new(), // always fails garde length(min=1)
        };
        let p = make_prompt();

        let err_false = p
            .render(&invalid_vars, None, &GuardConfig::default(), false)
            .expect_err("invalid vars must fail");
        let err_true = p
            .render(&invalid_vars, None, &GuardConfig::default(), true)
            .expect_err("invalid vars must fail");

        // Both must be Validation errors (the flag is irrelevant — kernel never reached).
        assert!(
            matches!(&err_false, ConsumerError::Validation(_)),
            "flag=false must produce Validation, got {err_false:?}"
        );
        assert!(
            matches!(&err_true, ConsumerError::Validation(_)),
            "flag=true must produce Validation, got {err_true:?}"
        );
        assert_eq!(
            err_false, err_true,
            "validation errors must be identical regardless of the reveal flag"
        );
    }

    // ── T008 [US2]: Replace parity — derive(overlay) == derive_with(overlay, Default) ──

    /// `derive(overlay)` and `derive_with(overlay, DeriveOptions::default())` produce identical
    /// output for a scalar-replace and a map-replace case (SC-002 / INV-2).
    #[test]
    fn derive_default_and_derive_with_default_are_identical() {
        let base = make_prompt();

        let overlay = PromptOverlay {
            body: Some("Hello {{ name }}!".to_string()),
            ..Default::default()
        };

        let via_derive = base.derive(overlay.clone()).expect("derive must succeed");
        let via_derive_with = base
            .derive_with(overlay, DeriveOptions::default())
            .expect("derive_with default must succeed");

        // Both produce the same body (SC-002).
        assert_eq!(via_derive.body(), via_derive_with.body());
        assert_eq!(via_derive.body(), "Hello {{ name }}!");
    }

    /// Replacing the `variables` map via derive (no strategy) drops the base's other variables
    /// — exactly the pre-017 wholesale-replace behavior.
    #[test]
    fn derive_replace_drops_other_variables() {
        // Base has `name`, overlay supplies only `color` — Replace drops `name`.
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ color }}","variables":{"name":{"type":"string","trusted":true},"color":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        let new_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"color":{"type":"string","trusted":true}}"#).expect("valid");
        let derived = base
            .derive(PromptOverlay {
                variables: Some(new_vars),
                ..Default::default()
            })
            .expect("derive must succeed");

        assert!(
            !derived.variables().contains_key("name"),
            "Replace drops base-only keys"
        );
        assert!(
            derived.variables().contains_key("color"),
            "overlay key present"
        );
    }

    // ── T010 [US1]: Merge unions variables ────────────────────────────────────

    /// Merge unions variables: base {extraction} + overlay {sentiment} → {extraction, sentiment}.
    #[test]
    fn merge_strategy_unions_variables() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid base");

        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"sentiment":{"type":"string","trusted":true}}"#)
            .expect("valid");
        let derived = base
            .derive_with(
                PromptOverlay {
                    variables: Some(overlay_vars),
                    body: Some("{{ extraction }} {{ sentiment }}".to_string()),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("Merge must succeed");

        assert!(
            derived.variables().contains_key("extraction"),
            "base key retained"
        );
        assert!(
            derived.variables().contains_key("sentiment"),
            "overlay key added"
        );
        assert_eq!(derived.variables().len(), 2);
        // Base is untouched (INV-1 / SC-005).
        assert!(!base.variables().contains_key("sentiment"));
    }

    /// Child-wins on key collision under Merge (whole-entry replace, INV-4).
    #[test]
    fn merge_child_wins_on_key_collision() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ field }}","variables":{"field":{"type":"string","trusted":true}}}"#,
        ).expect("valid base");

        // Overlay replaces `field` with a different declaration (trusted: false).
        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"field":{"type":"string","trusted":false}}"#).expect("valid");
        let derived = base
            .derive_with(
                PromptOverlay {
                    variables: Some(overlay_vars),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("Merge child-wins must succeed");

        assert_eq!(derived.variables().len(), 1, "still one variable");
        let field = derived.variables().get("field").expect("field present");
        assert!(!field.trusted, "overlay's trusted=false wins");
    }

    /// Body referencing merged vars constructs (agreement check runs over merged set).
    #[test]
    fn merge_agreement_check_over_merged_vars() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"sentiment":{"type":"string","trusted":true}}"#)
            .expect("valid");
        // New body references both variables — should pass the agreement check over merged set.
        let derived = base.derive_with(
            PromptOverlay {
                variables: Some(overlay_vars),
                body: Some("{{ extraction }} and {{ sentiment }}".to_string()),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        );
        assert!(
            derived.is_ok(),
            "agreement check passes with merged var set, got: {derived:?}"
        );
    }

    /// Empty overlay map under Merge leaves base variables unchanged (INV-5).
    #[test]
    fn merge_empty_overlay_map_leaves_base_unchanged() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        let derived = base
            .derive_with(
                PromptOverlay {
                    variables: Some(HashMap::new()),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("empty overlay under Merge must succeed");

        assert_eq!(derived.variables().len(), 1);
        assert!(derived.variables().contains_key("extraction"));
    }

    /// A Merge that breaks agreement fails at construction (SC-004 / INV-3).
    #[test]
    fn merge_agreement_violation_fails_construction() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        // New body references `ghost` which is in neither base nor overlay.
        let err = base
            .derive_with(
                PromptOverlay {
                    body: Some("{{ extraction }} {{ ghost }}".to_string()),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect_err("undeclared var must fail");

        assert!(
            matches!(&err, ConsumerError::Kernel(_)),
            "expected Kernel error, got {err:?}"
        );
    }

    // ── T010a [US1]: soundness tests (FR-019) ────────────────────────────────

    /// (a) A Merge whose union removes a variable a base variant body references → construction
    /// FAILS via the agreement check (name-removal caught across all variant arms).
    #[test]
    fn merge_removing_var_referenced_by_variant_fails() {
        // Base: body references `extraction`; variant `v1` also references `extraction`.
        let base = Prompt::from_json(r#"{
            "name":"base","role":"user","body":"{{ extraction }}",
            "variables":{"extraction":{"type":"string","trusted":true},"extra":{"type":"string","trusted":true}},
            "variants":{"v1":{"body":"variant: {{ extraction }}"}}
        }"#).expect("valid base");

        // Overlay under Merge replaces `variables` with only `extra` (dropping `extraction`).
        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"extra":{"type":"string","trusted":true}}"#).expect("valid");
        // Under Replace: replaces the whole map → extraction removed from declared vars but
        // body still references it → agreement check catches it. Under Merge with the new map
        // being {extra: ...}, the union is {extraction: ..., extra: ...} — so extraction is
        // NOT removed. To actually remove extraction under Merge we need to supply an overlay
        // that results in extraction being absent. Since Merge only adds/replaces keys, we
        // cannot remove a key — removal only happens under Replace. This is the
        // INV-5/FR-006 invariant: absence never removes.
        //
        // The test we want: base body references extraction + variant references extraction,
        // overlay body changes to not reference extraction, overlay variables removes extraction
        // (Replace strategy, since Merge can't remove keys). So this test uses Replace to
        // verify the agreement check across variant arms.
        let err = base
            .derive(PromptOverlay {
                variables: Some(overlay_vars),
                body: Some("{{ extra }}".to_string()),
                // Leave variants untouched — variant v1 still references extraction.
                ..Default::default()
            })
            .expect_err("agreement violation across variant arm must fail");

        match &err {
            ConsumerError::Kernel(rows) => {
                assert!(
                    rows.iter().any(|r| r.code == code::UNDEFINED_VARIABLE),
                    "expected undefined_variable, got {rows:?}"
                );
            }
            other => panic!("expected Kernel error, got {other:?}"),
        }
    }

    /// (b) A Merge that replaces a variable's declaration (type/trust swap) referenced by a
    /// base arm → construction SUCCEEDS (name-only boundary, FR-019).
    #[test]
    fn merge_type_swap_accepted_name_only_boundary() {
        let base = Prompt::from_json(
            r#"{
            "name":"base","role":"user","body":"{{ field }}",
            "variables":{"field":{"type":"string","trusted":true}},
            "variants":{"v1":{"body":"variant: {{ field }}"}}
        }"#,
        )
        .expect("valid base");

        // Overlay replaces `field` with a different type/trust under Merge.
        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"field":{"type":"integer","trusted":false}}"#).expect("valid");
        let result = base.derive_with(
            PromptOverlay {
                variables: Some(overlay_vars),
                ..Default::default()
            },
            DeriveOptions {
                strategy: MergeStrategy::Merge,
            },
        );
        assert!(
            result.is_ok(),
            "type/trust swap accepted (name-only boundary), got: {result:?}"
        );
        let derived = result.unwrap();
        let field = derived.variables().get("field").expect("field present");
        assert!(!field.trusted, "overlay trusted=false wins");
    }

    // ── T010b [US1]: variants + metadata union under Merge ────────────────────

    /// Merge unions variants: base variant + overlay variant → both present.
    #[test]
    fn merge_unions_variants() {
        let base = Prompt::from_json(
            r#"{
            "name":"base","role":"user","body":"{{ name }}",
            "variables":{"name":{"type":"string","trusted":true}},
            "variants":{"v1":{"body":"v1: {{ name }}"}}
        }"#,
        )
        .expect("valid");

        let overlay_variants: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariant,
        > = serde_json::from_str(r#"{"v2":{"body":"v2: {{ name }}"}}"#).expect("valid");
        let derived = base
            .derive_with(
                PromptOverlay {
                    variants: Some(overlay_variants),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("Merge unions variants");

        assert!(
            derived.variants().contains_key("v1"),
            "base variant retained"
        );
        assert!(
            derived.variants().contains_key("v2"),
            "overlay variant added"
        );
    }

    /// Merge unions metadata keys; a guard-key collision replaces the base's whole guard entry.
    #[test]
    fn merge_unions_metadata_guard_key_child_wins() {
        let base = Prompt::from_json(
            r#"{
            "name":"base","role":"user","body":"{{ name }}",
            "variables":{"name":{"type":"string","trusted":true}},
            "metadata":{"base_key":"base_val","guard":{"enabled":false}}
        }"#,
        )
        .expect("valid");

        let overlay_metadata: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(r#"{"overlay_key":"overlay_val","guard":{"enabled":true}}"#)
                .expect("valid");
        let derived = base
            .derive_with(
                PromptOverlay {
                    metadata: Some(overlay_metadata),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("Merge metadata union");

        let meta = derived.metadata();
        assert_eq!(
            meta.get("base_key").and_then(|v| v.as_str()),
            Some("base_val"),
            "base key retained"
        );
        assert_eq!(
            meta.get("overlay_key").and_then(|v| v.as_str()),
            Some("overlay_val"),
            "overlay key added"
        );
        // guard key: child wins whole entry (INV-4).
        let guard = meta.get("guard").expect("guard key present");
        assert_eq!(
            guard.get("enabled").and_then(serde_json::Value::as_bool),
            Some(true),
            "overlay guard wins"
        );
    }

    // ── T010c [US1]: error scrubbing preserved (SEC-001) ─────────────────────

    /// A failed Merge construction yields the SAME scrubbed error class as a failed plain
    /// derive — no overlay value content leaks into the default error message.
    #[test]
    fn merge_failed_construction_error_scrubbed() {
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        // Merge that adds `sentiment` but new body references undeclared `ghost`.
        let overlay_vars: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"sentiment":{"type":"string","trusted":true}}"#)
            .expect("valid");
        let err = base
            .derive_with(
                PromptOverlay {
                    variables: Some(overlay_vars),
                    body: Some("{{ extraction }} {{ sentiment }} {{ ghost }}".to_string()),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect_err("undeclared var must fail");

        match &err {
            ConsumerError::Kernel(rows) => {
                // The error names the undeclared variable — that is the agreed non-secret
                // structural information (the variable name, not a bound value).
                // It must NOT contain any bound value from the overlay body text.
                let msg = &rows[0].message;
                assert!(msg.contains("ghost"), "error names offending variable");
                assert!(
                    !msg.contains("extraction {{ sentiment }}"),
                    "bound value not leaked"
                );
            }
            other => panic!("expected Kernel error, got {other:?}"),
        }
    }

    // ── T018 [US3]: Merge add-a-key == manual-spread Replace ─────────────────

    /// derive({variables:{sentiment}, body:"..."}, Merge) == derive({variables:{extraction, sentiment}, body:"..."}, Replace)
    #[test]
    fn merge_add_key_equals_manual_spread_replace() {
        // Base declares only `extraction` and references it in the body.
        let base = Prompt::from_json(
            r#"{"name":"base","role":"user","body":"{{ extraction }}","variables":{"extraction":{"type":"string","trusted":true}}}"#,
        ).expect("valid");

        let sentiment_var: HashMap<
            String,
            prompting_press_core::generated::prompt_definition::PromptVariable,
        > = serde_json::from_str(r#"{"sentiment":{"type":"string","trusted":true}}"#)
            .expect("valid");
        let new_body = "{{ extraction }} {{ sentiment }}".to_string();

        // Merge strategy: add sentiment to the existing extraction; also update body.
        let via_merge = base
            .derive_with(
                PromptOverlay {
                    variables: Some(sentiment_var.clone()),
                    body: Some(new_body.clone()),
                    ..Default::default()
                },
                DeriveOptions {
                    strategy: MergeStrategy::Merge,
                },
            )
            .expect("Merge add-a-key");

        // Manual spread Replace: include both extraction and sentiment explicitly; same body.
        let mut both_vars = base.variables().clone();
        both_vars.extend(sentiment_var);
        let via_replace = base
            .derive(PromptOverlay {
                variables: Some(both_vars),
                body: Some(new_body),
                ..Default::default()
            })
            .expect("Replace with both vars");

        // The resulting variable sets should be equal.
        assert_eq!(
            via_merge
                .variables()
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            via_replace
                .variables()
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            "Merge add-a-key equals manual-spread Replace (US3)"
        );
    }
}
