// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Advisory lint types and shared helpers for the agreement + trusted/guard check.
//!
//! The lint runs **per-`Prompt`** via [`Prompt::check`](crate::Prompt::check). Construction
//! enforces the **hard** invariants (template parseable, referenced roots ⊆ declared
//! variables, no reserved variant name). The only LIVE finding `Prompt::check()` can surface
//! is the advisory:
//!
//! 1. **Trust / guard advisory.** A `Prompt` that declares one or more `trusted: false`
//!    variables but carries no `"guard"` key in its `metadata` map →
//!    [`FindingKind::UntrustedWithoutGuard`] per uncovered field.
//!
//! `UntrustedWithoutGuard` is the only `FindingKind` variant; the hard invariants (undeclared
//! variables, analysis errors, reserved variant names) are enforced at construction and never
//! appear in a `CheckReport` from a live `Prompt`.
//!
//! ## The `metadata.guard` convention
//!
//! A prompt has a guard configured iff a `"guard"` key is present in its `metadata` map.
//! The map is a library-opaque `serde_json::Map`; this module reads it
//! **read-only** and checks only for the *presence* of the top-level key — not its shape.
//!
//! ## Purity
//!
//! [`Prompt::check`](crate::Prompt::check) takes `&self`, never renders, never mutates.
//! Its only output is a [`CheckReport`].

use prompting_press_core::PromptDefinition;

/// The opaque-metadata key whose *presence* (in `metadata`) marks a prompt as
/// having a guard configured (the `metadata.guard` convention — see module docs).
pub(crate) const GUARD_KEY: &str = "guard";

/// One actionable lint finding: it names the prompt, the variant where applicable,
/// the failure `kind`, and a human-readable `detail`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The prompt's name.
    pub prompt: String,
    /// The variant the finding pertains to (`Some("default")` / `Some("<name>")` for an
    /// agreement or analysis finding); `None` for a prompt-level trust/guard finding.
    pub variant: Option<String>,
    /// The kind of failure (the discriminant a consumer matches on).
    pub kind: FindingKind,
    /// A human-readable, actionable description. Carries no bound-value content.
    pub detail: String,
}

/// The closed set of lint-failure classes.
///
/// `UntrustedWithoutGuard` is the only advisory class that `Prompt::check` can surface.
/// All other hard invariants (undeclared variables, analysis errors, reserved variant names)
/// are enforced at construction and are structurally unreachable from a live `Prompt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    /// The prompt declares `field` as `trusted: false` but configures no guard for it
    /// (see the `metadata.guard` convention in module docs). The only advisory class
    /// surfaced by `Prompt::check`.
    UntrustedWithoutGuard {
        /// The uncovered `trusted: false` field name.
        field: String,
    },
}

/// The output of `Prompt::check`: an ordered list of [`Finding`]s. Empty ⇒ pass.
///
/// Carries **only** findings — no rendered text, no mutated state.
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

/// `true` iff a top-level `"guard"` key is present in the prompt's `metadata` map
/// (the `metadata.guard` convention — module docs / C-09). Read-only; presence only (contents
/// opaque). Used by both `Prompt::check` (via `prompt::check_origin_advisory`) and the
/// per-prompt advisory helper.
pub(crate) fn has_guard_configured(def: &PromptDefinition) -> bool {
    def.metadata.contains_key(GUARD_KEY)
}
