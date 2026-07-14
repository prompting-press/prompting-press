// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Provenance attributes helper — spec 018.
//!
//! Provides the four library-owned key constants and `provenance_attributes_of`, a pure
//! function that builds a flat string-keyed map from the four provenance fields already
//! present on every [`RenderResult`]. Also exposes [`ProvenanceExt`], an optional extension
//! trait that lets Rust callers write `result.provenance_attributes()`.
//!
//! ## Key invariants
//!
//! - Exactly **four entries** are emitted — an explicit allowlist, never reflection.
//!   A future field added to `RenderResult` cannot leak into the map (FR-007, SC-004).
//! - Keys are **library-owned** (`prompting_press.prompt.*`). They are NOT `OTel` `gen_ai.*`
//!   keys. A consumer may remap them onto their tracer's convention (FR-003).
//! - Pure: no I/O, no mutation, no callbacks (FR-004, FR-005).
//! - No telemetry / observability dependency is introduced (FR-006, SC-002).
//! - The kernel is unchanged; this module is consumer-layer only (FR-010, SC-006).

use std::collections::BTreeMap;

use prompting_press_core::RenderResult;

// ── Library-owned key constants (FR-003) ─────────────────────────────────────────────────

/// Attribute key for the prompt name (`prompting_press.prompt.name`).
///
/// Value: the `name` field of the rendered prompt definition.
pub const KEY_NAME: &str = "prompting_press.prompt.name";

/// Attribute key for the resolved variant name (`prompting_press.prompt.variant`).
///
/// Value: `"default"` when no variant arm was selected; the named arm otherwise (INV-3).
pub const KEY_VARIANT: &str = "prompting_press.prompt.variant";

/// Attribute key for the template hash (`prompting_press.prompt.template_hash`).
///
/// Value: lowercase-hex `SHA256(resolved variant source)`. Content-addresses the template,
/// not the rendered output. Unchanged by guard mode (the template source is not modified).
pub const KEY_TEMPLATE_HASH: &str = "prompting_press.prompt.template_hash";

/// Attribute key for the render hash (`prompting_press.prompt.render_hash`).
///
/// Value: lowercase-hex `SHA256(rendered output)`. Content-addresses the rendered text.
/// When the guard is enabled the hash is over the delimited body; when disabled it is over
/// the plain body. Both are deterministic. The hash is an identifier — never the content
/// itself (SEC-002).
pub const KEY_RENDER_HASH: &str = "prompting_press.prompt.render_hash";

// ── Shared builder (FR-009 shared-key-assembly — one place, all bindings delegate) ──────

/// Build the provenance attribute map from the four field values.
///
/// Returns a [`BTreeMap<String, String>`] containing exactly four entries — one per
/// library-owned key constant. `BTreeMap` guarantees deterministic iteration order,
/// satisfying the D1 canonical serialized-form requirement for cross-binding parity (D1).
///
/// This is a **free function over the four field values** (not a method on `RenderResult`)
/// because the consumer crate re-exports the kernel's `RenderResult` and an inherent `impl`
/// there would violate the orphan rule (E0116). Python and TypeScript call the equivalent
/// logic on their own binding-local `RenderResult` type; Rust callers may use the
/// [`ProvenanceExt`] trait for the ergonomic `result.provenance_attributes()` form.
///
/// # Allowlist discipline
///
/// The four keys are an **explicit allowlist** — this function will never emit `text`,
/// `guard`, `output_model`, or any other field, even if `RenderResult` gains new fields
/// in the future (FR-007, SC-004, SEC-001).
pub fn provenance_attributes_of(
    name: &str,
    variant: &str,
    template_hash: &str,
    render_hash: &str,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert(KEY_NAME.to_string(), name.to_string());
    map.insert(KEY_VARIANT.to_string(), variant.to_string());
    map.insert(KEY_TEMPLATE_HASH.to_string(), template_hash.to_string());
    map.insert(KEY_RENDER_HASH.to_string(), render_hash.to_string());
    map
}

// ── Extension trait (optional ergonomic surface for Rust callers) ────────────────────────

/// Extension trait providing `provenance_attributes()` on [`RenderResult`].
///
/// Import this trait (`use prompting_press::ProvenanceExt`) to call
/// `result.provenance_attributes()` instead of [`provenance_attributes_of`].
///
/// This trait exists solely because the consumer re-exports the kernel's `RenderResult`,
/// making an inherent `impl RenderResult` in this crate a compiler error (E0116 — orphan
/// rule). A trait defined HERE and implemented for the re-exported type is the idiomatic
/// workaround (RFC 445).
pub trait ProvenanceExt {
    /// Return the four provenance fields as a flat `prompting_press.prompt.*` attribute map.
    ///
    /// Identical to calling [`provenance_attributes_of`] with the result's four fields.
    /// See that function's documentation for the full key semantics and allowlist contract.
    fn provenance_attributes(&self) -> BTreeMap<String, String>;
}

impl ProvenanceExt for RenderResult {
    fn provenance_attributes(&self) -> BTreeMap<String, String> {
        provenance_attributes_of(
            &self.name,
            &self.variant,
            &self.template_hash,
            &self.render_hash,
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// T003 / T007 — free function produces exactly 4 entries with the correct keys and values.
    #[test]
    fn provenance_attributes_of_returns_exact_four_entries() {
        let map = provenance_attributes_of(
            "my-prompt",
            "formal",
            "aabbccdd".repeat(8).as_str(), // 64-char placeholder
            "11223344".repeat(8).as_str(),
        );

        assert_eq!(map.len(), 4, "must be exactly 4 entries");
        assert_eq!(map[KEY_NAME], "my-prompt");
        assert_eq!(map[KEY_VARIANT], "formal");
        assert_eq!(map[KEY_TEMPLATE_HASH], "aabbccdd".repeat(8));
        assert_eq!(map[KEY_RENDER_HASH], "11223344".repeat(8));
    }

    /// T003 — values match the inputs exactly (no transformation).
    #[test]
    fn provenance_attributes_of_values_match_inputs() {
        let map = provenance_attributes_of("greet", "default", "abc123", "def456");
        assert_eq!(map[KEY_NAME], "greet");
        assert_eq!(map[KEY_VARIANT], "default");
        assert_eq!(map[KEY_TEMPLATE_HASH], "abc123");
        assert_eq!(map[KEY_RENDER_HASH], "def456");
    }

    /// T003 — deterministic iteration order (`BTreeMap` ensures alphabetical key order).
    #[test]
    fn provenance_attributes_of_is_deterministic() {
        let map1 = provenance_attributes_of("p", "v", "th", "rh");
        let map2 = provenance_attributes_of("p", "v", "th", "rh");
        let keys1: Vec<&str> = map1.keys().map(String::as_str).collect();
        let keys2: Vec<&str> = map2.keys().map(String::as_str).collect();
        assert_eq!(keys1, keys2, "key order must be deterministic across calls");
    }

    /// T003 — map excludes text, guard, and any other field (allowlist, not reflection).
    #[test]
    fn provenance_attributes_of_excludes_non_provenance_fields() {
        let map = provenance_attributes_of("n", "v", "th", "rh");
        // Negative assertions: these must never appear (FR-007, SC-004, SEC-001).
        assert!(!map.contains_key("text"), "text must not be in the map");
        assert!(!map.contains_key("guard"), "guard must not be in the map");
        assert!(
            !map.contains_key("output_model"),
            "output_model must not be in the map"
        );
        assert!(
            !map.contains_key("metadata"),
            "metadata must not be in the map"
        );
    }

    /// T007 — extension trait gives `result.provenance_attributes()`; default variant → "default".
    #[test]
    fn extension_trait_provenance_attributes_matches_free_fn() {
        use garde::Validate;
        use prompting_press_core::GuardConfig;
        use serde::Serialize;

        #[derive(Serialize, Validate)]
        struct Vars {
            #[garde(skip)]
            name: String,
        }

        let prompt = crate::Prompt::from_json(
            r#"{"name":"greet","role":"user","body":"Hi {{ name }}","variables":{"name":{"type":"string","trusted":true}}}"#,
        )
        .unwrap();

        let vars = Vars {
            name: "Ada".to_string(),
        };
        let result = prompt
            .render(&vars, None, &GuardConfig::default(), false)
            .unwrap();

        let via_trait = result.provenance_attributes();
        let via_fn = provenance_attributes_of(
            &result.name,
            &result.variant,
            &result.template_hash,
            &result.render_hash,
        );

        assert_eq!(
            via_trait, via_fn,
            "trait and free-fn must produce identical maps"
        );
        assert_eq!(via_trait.len(), 4);
        // Default variant is always "default" (INV-3).
        assert_eq!(via_trait[KEY_VARIANT], "default");
    }
}
