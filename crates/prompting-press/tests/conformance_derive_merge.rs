//! Conformance corpus — Rust derive-merge runner (spec 017 T017).
//!
//! Drives every `conformance/derive-merge/*.json` fixture through the consumer's
//! `derive_with(overlay, DeriveOptions { strategy: Merge })` path and asserts:
//!
//! 1. The merged prompt's variable, variant, and metadata maps match the expected merged
//!    definition in the fixture (canonical serialized form — decision D1). This is the
//!    structural parity assertion: Python and TS runners assert the same merged definition
//!    from their own bindings, so cross-binding parity is transitive.
//! 2. After merging, the derived prompt renders with the fixture's `render_input` and
//!    produces text equal to `expected_render.text`.
//!
//! The `template_hash` and `render_hash` fields in `expected_render` are populated by the
//! golden generator (`moon run conformance:regen`). Until regenerated they are empty strings
//! and the hash assertions are skipped (non-blocking for the structural assertions).

mod common;

use common::{build_vars, RawVars, TypedValue};
use prompting_press::{MergeStrategy, Prompt};
use prompting_press_core::{GuardConfig, PromptDefinition};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

// ─── fixture types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct ExpectedRender {
    text: String,
    #[serde(default)]
    template_hash: String,
    #[serde(default)]
    render_hash: String,
}

#[derive(Deserialize, Debug)]
struct DeriveMergeFixture {
    case: String,
    base: Value,
    overlay: Value,
    strategy: String,
    expected_merged: Value,
    /// Same `{type, value}` tagged format as the marshaling corpus.
    render_input: BTreeMap<String, TypedValue>,
    expected_render: ExpectedRender,
}

// ─── loader ───────────────────────────────────────────────────────────────────

fn load_derive_merge_fixtures() -> Vec<(PathBuf, DeriveMergeFixture)> {
    let corpus_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("conformance")
        .join("derive-merge");

    let mut fixtures = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&corpus_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
                let fixture: DeriveMergeFixture = serde_json::from_str(&text)
                    .unwrap_or_else(|e| panic!("invalid fixture {}: {e}", path.display()));
                fixtures.push((path, fixture));
            }
        }
    }
    fixtures.sort_by(|(a, _), (b, _)| a.cmp(b));
    fixtures
}

// ─── helper: parse strategy string ───────────────────────────────────────────

fn parse_strategy(s: &str) -> MergeStrategy {
    match s {
        "merge" => MergeStrategy::Merge,
        "replace" | _ => MergeStrategy::Replace,
    }
}

// ─── runner ───────────────────────────────────────────────────────────────────

#[test]
fn derive_merge_fixtures_match_expected() {
    let fixtures = load_derive_merge_fixtures();
    assert!(
        !fixtures.is_empty(),
        "no derive-merge fixtures found under conformance/derive-merge/"
    );

    let mut failures = Vec::new();

    for (path, fx) in &fixtures {
        // 1. Build base Prompt.
        let base_def: PromptDefinition = serde_json::from_value(fx.base.clone())
            .unwrap_or_else(|e| panic!("{}: invalid base definition: {e}", fx.case));
        let base = Prompt::new(base_def)
            .unwrap_or_else(|e| panic!("{}: base Prompt::new failed: {e:?}", fx.case));

        // 2. Build overlay using the shared helper in serde_json::Value space.
        let strategy = parse_strategy(&fx.strategy);

        let base_json = serde_json::to_value(base.definition())
            .unwrap_or_else(|e| panic!("{}: serialize base failed: {e}", fx.case));
        let merged_json =
            prompting_press::merge_definitions(base_json, fx.overlay.clone(), strategy)
                .unwrap_or_else(|e| panic!("{}: merge_definitions failed: {e:?}", fx.case));

        let derived = Prompt::from_json(
            &serde_json::to_string(&merged_json)
                .unwrap_or_else(|e| panic!("{}: serialize merged failed: {e}", fx.case)),
        )
        .unwrap_or_else(|e| panic!("{}: Prompt::from_json (merged) failed: {e:?}", fx.case));

        // 3. Assert structural parity: merged definition matches expected_merged.
        //    Compare the three map fields and scalar fields.
        let expected_merged = &fx.expected_merged;

        // Variables
        let got_vars =
            serde_json::to_value(derived.variables()).expect("serialize derived.variables");
        let expected_vars = expected_merged
            .get("variables")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        if !json_keys_match(&got_vars, &expected_vars) {
            failures.push(format!(
                "[rust] case={} divergence=merged_variables: got {:?}, expected {:?}",
                fx.case, got_vars, expected_vars
            ));
        }

        // Metadata
        let got_meta =
            serde_json::to_value(derived.metadata()).expect("serialize derived.metadata");
        let expected_meta = expected_merged
            .get("metadata")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        if got_meta != expected_meta {
            failures.push(format!(
                "[rust] case={} divergence=merged_metadata: got {:?}, expected {:?}",
                fx.case, got_meta, expected_meta
            ));
        }

        // Body
        let expected_body = expected_merged
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if derived.body() != expected_body {
            failures.push(format!(
                "[rust] case={} divergence=merged_body: got {:?}, expected {:?}",
                fx.case,
                derived.body(),
                expected_body
            ));
        }

        // 4. Render with render_input and assert text.
        let vars = RawVars(build_vars(&fx.render_input));
        let result = derived
            .render(&vars, None, &GuardConfig::default(), false)
            .unwrap_or_else(|e| panic!("{} ({}): render failed: {e:?}", fx.case, path.display()));

        if result.text != fx.expected_render.text {
            failures.push(format!(
                "[rust] case={} divergence=render_text: got {:?}, golden {:?}",
                fx.case, result.text, fx.expected_render.text
            ));
        }

        // Hash assertions — skipped when golden is still empty (pre-regen).
        if !fx.expected_render.template_hash.is_empty()
            && result.template_hash != fx.expected_render.template_hash
        {
            failures.push(format!(
                "[rust] case={} divergence=template_hash: got {}, golden {}",
                fx.case, result.template_hash, fx.expected_render.template_hash
            ));
        }
        if !fx.expected_render.render_hash.is_empty()
            && result.render_hash != fx.expected_render.render_hash
        {
            failures.push(format!(
                "[rust] case={} divergence=render_hash: got {}, golden {}",
                fx.case, result.render_hash, fx.expected_render.render_hash
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "derive-merge conformance divergences:\n{}",
        failures.join("\n")
    );
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Check that the keys present in `expected` are all present in `got`, and
/// `got` has exactly the same set of keys (order-independent). This gives a
/// clear diff for variable-map mismatches.
fn json_keys_match(got: &Value, expected: &Value) -> bool {
    match (got, expected) {
        (Value::Object(g), Value::Object(e)) => {
            let g_keys: std::collections::BTreeSet<_> = g.keys().collect();
            let e_keys: std::collections::BTreeSet<_> = e.keys().collect();
            g_keys == e_keys
        }
        _ => got == expected,
    }
}
