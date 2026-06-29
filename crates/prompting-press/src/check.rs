//! Advisory lint types and shared helpers for the agreement + origin check
//! (spec 008 reshape; FR-016..020).
//!
//! Post-reshape, the lint runs **per-`Prompt`** via [`Prompt::check`](crate::Prompt::check),
//! not over a registry. Construction enforces the **hard** invariants (template parseable,
//! referenced roots ⊆ declared variables, no reserved variant name) — those arms are
//! structurally unreachable for a constructed `Prompt`. The only LIVE finding
//! `Prompt::check()` can surface is the advisory:
//!
//! 1. **Origin / guard advisory (FR-018, reframed).** A `Prompt` that declares one or more
//!    `untrusted`/`external` variables but carries no `"guard"` key in its `meta` or
//!    `metadata` map → [`FindingKind::UntrustedWithoutGuard`] per uncovered field.
//!
//! The remaining `FindingKind` variants (`UndeclaredVariable`, `AnalysisError`,
//! `ReservedVariantName`) are still part of the public type because they appear in
//! `CheckReport` values returned by the method — a caller must be able to match on all
//! variants even though construction makes them unreachable from a live `Prompt`.
//!
//! ## The `meta.guard` convention (C-09)
//!
//! A prompt has a guard configured iff a `"guard"` key is present in its `meta` map OR its
//! `metadata` map. Both maps are library-opaque `serde_json::Map`s; this module reads them
//! **read-only** and checks only for the *presence* of the top-level key — not its shape.
//!
//! ## The reserved `default` name (CR-1 — enforced at construction)
//!
//! `"default"` is the kernel's reserved name for the root body. A prompt declaring a variant
//! with that name is rejected at **construction** (`Prompt::new` returns an error). The
//! `ReservedVariantName` variant stays in `FindingKind` for completeness, but a constructed
//! `Prompt` can never carry it from `check()`.
//!
//! ## Purity (FR-019)
//!
//! [`Prompt::check`](crate::Prompt::check) takes `&self`, never renders, never mutates.
//! Its only output is a [`CheckReport`].

use prompting_press_core::PromptDefinition;

/// The opaque-metadata key whose *presence* (in `meta` or `metadata`) marks a prompt as
/// having a guard configured (the `meta.guard` convention — module docs / C-09).
pub(crate) const GUARD_KEY: &str = "guard";

/// One actionable lint finding (FR-020): it names the prompt, the variant where applicable,
/// the failure `kind`, and a human-readable `detail`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The prompt's name.
    pub prompt: String,
    /// The variant the finding pertains to (`Some("default")` / `Some("<name>")` for an
    /// agreement or analysis finding); `None` for a prompt-level origin finding.
    pub variant: Option<String>,
    /// The kind of failure (the discriminant a consumer matches on).
    pub kind: FindingKind,
    /// A human-readable, actionable description (FR-020). Carries no bound-value content.
    pub detail: String,
}

/// The closed set of lint-failure classes.
///
/// `UndeclaredVariable`, `AnalysisError`, and `ReservedVariantName` are enforced at
/// construction time post-reshape — a constructed `Prompt` never emits them from
/// `check()`. They remain public so a caller can match exhaustively on values returned
/// from any code path that constructs a `CheckReport`.
///
/// `UntrustedWithoutGuard` is the one LIVE advisory class that `Prompt::check()` can
/// surface (C-09 / FR-018).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    /// A template references `name`, but `name` is not in the prompt's declared `variables`
    /// (the agreement half — FR-016/017). Enforced at construction; unreachable from a live
    /// `Prompt::check()`.
    UndeclaredVariable {
        /// The undeclared root variable name the template referenced.
        name: String,
    },
    /// The prompt declares `field` as `untrusted`/`external` but configures no guard for it
    /// (the origin advisory — FR-018; the `meta.guard` convention, module docs). The LIVE
    /// advisory class surfaced by `Prompt::check()`.
    UntrustedWithoutGuard {
        /// The uncovered untrusted/external field name.
        field: String,
    },
    /// The kernel could not analyze a variant's template (a parse failure or an excluded
    /// feature). Enforced at construction; unreachable from a live `Prompt::check()`.
    AnalysisError {
        /// A stable, scrubbed reason code (e.g. `"parse"`, `"excluded_feature"`).
        reason: String,
    },
    /// The prompt declares a variant literally named `"default"` — the kernel's reserved
    /// name for the root body. Enforced at construction; unreachable from a live
    /// `Prompt::check()`.
    ReservedVariantName {
        /// The reserved variant name (always `"default"`).
        name: String,
    },
}

/// The output of `Prompt::check()`: an ordered list of [`Finding`]s. Empty ⇒ pass.
///
/// Carries **only** findings — no rendered text, no mutated state (FR-019).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckReport {
    /// Every advisory finding, in deterministic order. Empty ⇒ pass.
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

/// `true` iff a top-level `"guard"` key is present in the prompt's `meta` OR `metadata` map
/// (the `meta.guard` convention — module docs / C-09). Read-only; presence only (contents
/// opaque). Used by both `Prompt::check` (via `prompt::check_origin_advisory`) and the
/// per-prompt advisory helper.
pub(crate) fn has_guard_configured(def: &PromptDefinition) -> bool {
    def.meta.contains_key(GUARD_KEY) || def.metadata.contains_key(GUARD_KEY)
}
