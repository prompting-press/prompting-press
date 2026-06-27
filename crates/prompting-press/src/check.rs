//! The agreement + provenance lint (spec 003, US3; T017–T019; FR-016..020).
//!
//! [`check`](check()) is the library's **headline differentiator** (constitution Principle IV /
//! C-04/C-09): a single, pure pass over a [`Registry`] that catches — *before render, in CI*
//! — two classes of prompt bug:
//!
//! 1. **Agreement (FR-016/017).** A template that references a variable the prompt never
//!    declared. Under a lenient engine that renders as a silent empty string; here it is a
//!    reported [`FindingKind::UndeclaredVariable`].
//! 2. **Provenance (FR-018, reframed — see below).** A prompt that declares an
//!    `untrusted`/`external` input but configures **no guard** for it →
//!    [`FindingKind::UntrustedWithoutGuard`].
//!
//! The lint owns **only the comparison and the registry walk**. The hard parts — the sound
//! referenced-roots set and the provenance view — are computed *once* by the kernel
//! ([`prompting_press_core::required_roots`] / [`prompting_press_core::provenance_view`]); this
//! module never re-derives them (constitution Principle I / FR-017 / C-01).
//!
//! ## Purity (FR-019)
//!
//! [`check`](check()) takes `&Registry` (a shared borrow — mutation is impossible through the type
//! system), never renders, and has no side effects. Its only output is the [`CheckReport`].
//!
//! ## Determinism
//!
//! Findings are emitted in a stable order: the registry iterates by name
//! ([`BTreeMap`](std::collections::BTreeMap)); each prompt's variants are visited in sorted
//! order (default arm first, then named variants sorted via a
//! [`BTreeSet`](std::collections::BTreeSet)); within a variant, undeclared roots are already
//! sorted (the kernel returns a [`BTreeSet`](std::collections::BTreeSet)); provenance
//! findings follow, fields sorted (the kernel's
//! [`ProvenanceView`](prompting_press_core::ProvenanceView) sets are sorted). So the report is
//! reproducible for a CI gate.
//!
//! ## The `meta.guard` convention (FR-018, reframed — analyze F1)
//!
//! The spec-002 kernel has **no in-template "guard position"** concept, so the literal
//! "untrusted field used outside a guard position" lint is not implementable against the
//! kernel surface. This crate therefore adopts a concrete, implementable interpretation of
//! "a guard is configured for this prompt":
//!
//! > **A prompt has a guard configured iff a `"guard"` key is present in its `meta` map OR
//! > its `metadata` map.**
//!
//! Both `meta` and `metadata` are library-**opaque** `serde_json::Map`s on the prompt
//! definition (the library never interprets their *contents*); this lint reads them
//! **read-only** and only checks for the *presence* of a top-level `"guard"` key — it does not
//! validate the guard's shape (that is the caller's concern, and a render-time
//! [`GuardConfig`](prompting_press_core::GuardConfig) is what actually drives guard expansion).
//! The rule:
//!
//! - `declared_untrusted = provenance_view(def).untrusted ∪ .external`.
//! - If `declared_untrusted` is non-empty **and** neither `meta` nor `metadata` carries a
//!   `"guard"` key → emit one [`FindingKind::UntrustedWithoutGuard`] per uncovered field.
//! - If a `"guard"` key **is** present → the obligation is satisfied; no provenance finding.
//!
//! This is the consumer's chosen, documented interpretation of C-09's "you declared untrusted
//! inputs and set up no guard for them," given the kernel surface available in v1.
//!
//! ## The reserved `default` name (CR-1)
//!
//! `"default"` is the kernel's reserved name for the **root body**: variant resolution maps
//! both `None` and `Some("default")` to `def.body`. A prompt may therefore declare a
//! `variants` map that *also* contains a key `"default"` — but that declared arm is
//! **unreachable**: the kernel never resolves to it (it always lands on the root body), so it
//! is never rendered, hashed, or analyzed.
//!
//! [`check`](check()) handles this in two parts:
//!
//! 1. **Dedup (no double-analysis).** [`variants_to_check`] always analyzes the default arm
//!    exactly once, via the root body. It **excludes** a `"default"` key from the named-variant
//!    set so the root body is not analyzed twice (and so the unreachable declared arm is not
//!    mistaken for an analyzable variant).
//! 2. **Flag the dead arm.** A declared `variants["default"]` is reported as a
//!    [`FindingKind::ReservedVariantName`] finding naming the prompt + the reserved name +
//!    that its declared body is unreachable/shadowed by the root body. The arm's own body is
//!    not analyzed (it is dead), but the prompt does not pass silently.
//!
//! ## Handling a `required_roots` error (T018)
//!
//! [`prompting_press_core::required_roots`] can return `Err` — a malformed template, or one
//! using an excluded feature (`{% include %}` / macros / inheritance — these never parse under
//! the kernel's feature set). Rather than make `check()` fallible (it must stay a `-> CheckReport`
//! CI pass — F7), such an error is recorded as a finding with the distinct
//! [`FindingKind::AnalysisError`] kind, so a broken template surfaces loudly in the report
//! instead of being swallowed. This keeps `check()` infallible while still failing the gate on
//! an un-analyzable template.

use std::collections::BTreeSet;

use prompting_press_core::{provenance_view, required_roots, PromptDefinition};

use crate::Registry;

/// The reserved name of the default arm (the root `body`), mirroring the kernel's
/// variant-resolution convention (`None` ⇒ `"default"`).
const DEFAULT_VARIANT: &str = "default";

/// The opaque-metadata key whose *presence* (in `meta` or `metadata`) marks a prompt as
/// having a guard configured (the documented `meta.guard` convention — module docs).
const GUARD_KEY: &str = "guard";

/// One actionable lint finding (FR-020): it names the prompt, the variant where applicable,
/// the failure `kind`, and a human-readable `detail`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The prompt's registry name.
    pub prompt: String,
    /// The variant the finding pertains to (`Some("default")` / `Some("<name>")` for an
    /// agreement or analysis finding); `None` for a prompt-level provenance finding.
    pub variant: Option<String>,
    /// The kind of failure (the discriminant a consumer matches on).
    pub kind: FindingKind,
    /// A human-readable, actionable description (FR-020). Carries no bound-value content.
    pub detail: String,
}

/// The closed set of lint-failure classes.
///
/// `UndeclaredVariable` and `UntrustedWithoutGuard` are the two C-04/C-09 lint classes
/// (FR-016/018). `AnalysisError` is the third, distinct kind used when the kernel cannot
/// analyze a variant's template (see module docs, "Handling a `required_roots` error") — it
/// keeps [`check`](check()) infallible while still failing the gate on an un-analyzable template.
/// `ReservedVariantName` flags a prompt that declares a variant literally named `"default"`
/// (see module docs, "The reserved `default` name").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    /// A template references `name`, but `name` is not in the prompt's declared `variables`
    /// (the agreement half — FR-016/017).
    UndeclaredVariable {
        /// The undeclared root variable name the template referenced.
        name: String,
    },
    /// The prompt declares `field` as `untrusted`/`external` but configures no guard for it
    /// (the reframed provenance half — FR-018; the `meta.guard` convention, module docs).
    UntrustedWithoutGuard {
        /// The uncovered untrusted/external field name.
        field: String,
    },
    /// The kernel could not analyze a variant's template (a parse failure or an excluded
    /// feature). Recorded as a finding so [`check`](check()) stays infallible (F7) while still failing
    /// the gate. The `detail` carries a scrubbed description (no bound-value content).
    AnalysisError {
        /// A stable, scrubbed reason code (e.g. `"parse"`, `"excluded_feature"`,
        /// `"unknown_variant"`).
        reason: String,
    },
    /// The prompt declares a variant literally named `"default"` — a name the kernel reserves
    /// for the root body. Its declared body is therefore **unreachable**: the kernel's
    /// variant resolution maps both `None` and `Some("default")` to the root body, so the
    /// declared `variants["default"]` arm is shadowed and never rendered or analyzed. Flagged
    /// so the dead arm surfaces loudly rather than passing silently (see module docs, "The
    /// reserved `default` name").
    ReservedVariantName {
        /// The reserved variant name (always `"default"`).
        name: String,
    },
}

/// The output of [`check`](check()): an ordered list of [`Finding`]s. Empty ⇒ the lint passes.
///
/// Carries **only** findings — no rendered text, no mutated state (FR-019). The findings are
/// in a deterministic order (see module docs, "Determinism").
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckReport {
    /// Every lint failure found, in deterministic order. Empty ⇒ pass.
    pub findings: Vec<Finding>,
}

impl CheckReport {
    /// `true` iff there are no findings (the lint passed). Equivalent to
    /// `self.findings.is_empty()`; reads more clearly at a CI gate call site.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }

    /// Alias for [`passed`](Self::passed): `true` iff there are no findings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Run the agreement + provenance lint over `reg` (FR-016..020). **Pure**: never mutates,
/// never renders, no side effects (FR-019). Returns a [`CheckReport`]; empty ⇒ pass.
///
/// For each prompt (iterated by name — deterministic), in order:
///
/// 1. **Agreement** — for the default arm and each named variant (sorted), ask the kernel for
///    the variant's referenced roots ([`required_roots`]) and subtract the prompt's declared
///    `variables` keys; each leftover root is an [`FindingKind::UndeclaredVariable`]. A kernel
///    analysis `Err` becomes an [`FindingKind::AnalysisError`] finding (keeping `check`
///    infallible — F7). A declared variant literally named `"default"` is flagged as a
///    [`FindingKind::ReservedVariantName`] (its arm is unreachable — see module docs, "The
///    reserved `default` name").
/// 2. **Provenance** — ask the kernel for the prompt's untrusted/external fields
///    ([`provenance_view`]); if any are declared and no `"guard"` key is present in `meta` or
///    `metadata` (the `meta.guard` convention — module docs), each uncovered field is an
///    [`FindingKind::UntrustedWithoutGuard`].
///
/// An empty registry yields an empty report (pass — F7).
#[must_use]
pub fn check(reg: &Registry) -> CheckReport {
    let mut findings = Vec::new();

    for (name, def) in reg.iter() {
        check_agreement(name, def, &mut findings);
        check_provenance(name, def, &mut findings);
    }

    CheckReport { findings }
}

/// The agreement half for one prompt (FR-016/017): for each variant, subtract the declared
/// `variables` from the kernel's referenced roots and emit a finding per leftover.
///
/// The consumer owns ONLY the subtraction; the roots come from the kernel (FR-017 / C-01).
fn check_agreement(name: &str, def: &PromptDefinition, findings: &mut Vec<Finding>) {
    // The authoritative declared set (clarify Q1): the definition's own `variables` keys.
    let declared: BTreeSet<&str> = def.variables.keys().map(String::as_str).collect();

    // CR-1: a variant literally named `"default"` is schema-reserved — the kernel resolves
    // `Some("default")` to the root body, so the declared arm is unreachable/shadowed. Flag
    // it (the arm itself is NOT analyzed; `variants_to_check` excludes it, and the root body
    // is analyzed once below via the leading `DEFAULT_VARIANT`).
    if def.variants.contains_key(DEFAULT_VARIANT) {
        findings.push(Finding {
            prompt: name.to_string(),
            variant: Some(DEFAULT_VARIANT.to_string()),
            kind: FindingKind::ReservedVariantName {
                name: DEFAULT_VARIANT.to_string(),
            },
            detail: format!(
                "variant `{DEFAULT_VARIANT}` uses the reserved name for the root body; its \
                 declared body is unreachable (shadowed by the root body) and is never \
                 rendered — rename it or move its body to the root",
            ),
        });
    }

    for variant in variants_to_check(def) {
        // `None` ⇒ the default arm (root body), matching the kernel's resolution rule.
        let variant_arg = if variant == DEFAULT_VARIANT {
            None
        } else {
            Some(variant.as_str())
        };

        match required_roots(def, variant_arg) {
            Ok(agreement) => {
                // Subtract declared from referenced; each leftover root is undeclared.
                // `required_roots` is a sorted `BTreeSet`, so leftovers are emitted sorted.
                for root in &agreement.required_roots {
                    if !declared.contains(root.as_str()) {
                        findings.push(Finding {
                            prompt: name.to_string(),
                            variant: Some(variant.clone()),
                            kind: FindingKind::UndeclaredVariable { name: root.clone() },
                            detail: format!(
                                "template references undeclared variable `{root}` \
                                 (variant `{variant}`); add it to the prompt's `variables`",
                            ),
                        });
                    }
                }
            }
            // A malformed / excluded-feature template can't be analyzed. Record it (keeping
            // `check` infallible — F7) rather than silently passing it. The reason is a
            // scrubbed code, never the kernel's raw detail (which may carry bound-value text).
            Err(err) => {
                let reason = analysis_error_reason(&err);
                findings.push(Finding {
                    prompt: name.to_string(),
                    variant: Some(variant.clone()),
                    kind: FindingKind::AnalysisError {
                        reason: reason.to_string(),
                    },
                    detail: format!(
                        "template for variant `{variant}` could not be analyzed ({reason})",
                    ),
                });
            }
        }
    }
}

/// The provenance half for one prompt (FR-018, reframed): if it declares any untrusted /
/// external field and carries no `"guard"` key in `meta`/`metadata`, flag each uncovered
/// field. Prompt-level (no variant) — the obligation is on the prompt, not a single arm.
fn check_provenance(name: &str, def: &PromptDefinition, findings: &mut Vec<Finding>) {
    let view = provenance_view(def);

    // The full obligation set: untrusted ∪ external (both sorted `BTreeSet`s). Chaining into
    // a fresh sorted set keeps the emit order deterministic and de-duplicated.
    let declared_untrusted: BTreeSet<&str> = view
        .untrusted
        .iter()
        .chain(view.external.iter())
        .map(String::as_str)
        .collect();

    if declared_untrusted.is_empty() {
        return; // No untrusted/external inputs → no guard obligation.
    }

    // A guard is "configured" iff a top-level `"guard"` key is present in either opaque map
    // (the documented `meta.guard` convention). Presence only — the contents are opaque.
    if has_guard_configured(def) {
        return;
    }

    for field in declared_untrusted {
        findings.push(Finding {
            prompt: name.to_string(),
            variant: None,
            kind: FindingKind::UntrustedWithoutGuard {
                field: field.to_string(),
            },
            detail: format!(
                "field `{field}` is declared untrusted/external but the prompt configures \
                 no guard (add a `guard` key under the prompt's `meta` or `metadata`)",
            ),
        });
    }
}

/// The set of variant identifiers to analyze for a prompt, in deterministic order: the
/// reserved [`DEFAULT_VARIANT`] (root body) first, then each named variant **sorted**
/// (`def.variants` is a `HashMap`, whose key order is non-deterministic — sorting via a
/// `BTreeSet` makes the report reproducible).
///
/// The reserved name [`DEFAULT_VARIANT`] is **excluded** from the named set before extending
/// (CR-1): the kernel maps both `None` and `Some("default")` to the root body, so a declared
/// `variants["default"]` arm is unreachable. Including it here would analyze the root body
/// **twice** (once via the leading `DEFAULT_VARIANT` push, once via the duplicate key) while
/// the declared arm's body never gets analyzed at all. Removing it keeps the default arm
/// analyzed exactly once (via the root-body path). The shadowed declared arm is reported
/// separately by [`check_agreement`] as a [`FindingKind::ReservedVariantName`].
fn variants_to_check(def: &PromptDefinition) -> Vec<String> {
    let mut named: BTreeSet<&str> = def.variants.keys().map(String::as_str).collect();
    // Exclude the reserved name (CR-1): `Some("default")` resolves to the root body, so a
    // declared `variants["default"]` arm is unreachable; analyzing it would duplicate the
    // root-body analysis. The default arm is always checked via the leading push below.
    named.remove(DEFAULT_VARIANT);
    let mut out = Vec::with_capacity(named.len() + 1);
    out.push(DEFAULT_VARIANT.to_string());
    out.extend(named.into_iter().map(str::to_string));
    out
}

/// `true` iff a top-level `"guard"` key is present in the prompt's `meta` OR `metadata` map
/// (the `meta.guard` convention — module docs). Read-only; presence only (contents opaque).
fn has_guard_configured(def: &PromptDefinition) -> bool {
    def.meta.contains_key(GUARD_KEY) || def.metadata.contains_key(GUARD_KEY)
}

/// Map a kernel analysis error to a stable, **scrubbed** reason code for an
/// [`FindingKind::AnalysisError`]. Never copies the kernel's raw `detail` (which may carry
/// bound-value content — SEC-004 / FR-015); only the variant class is surfaced.
fn analysis_error_reason(err: &prompting_press_core::KernelError) -> &'static str {
    use prompting_press_core::KernelError;
    match err {
        KernelError::UnknownVariant { .. } => "unknown_variant",
        KernelError::UndefinedVariable { .. } => "undefined_variable",
        KernelError::Parse { .. } => "parse",
        KernelError::Render { .. } => "render",
        KernelError::ExcludedFeature { .. } => "excluded_feature",
    }
}
