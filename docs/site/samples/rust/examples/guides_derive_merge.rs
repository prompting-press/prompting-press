// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Derive guide — `MergeStrategy::Merge` via `derive_with`: union the base's declared
//! variables with the overlay's, so a child prompt inherits `company` + `max_words` and adds
//! its own `tone` without hand-spreading the base's variables. The base is untouched.
//! Standalone — `cargo run --example guides_derive_merge`.

use prompting_press::{DeriveOptions, MergeStrategy, Prompt, PromptOverlay};
use serde_json::json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");
    let base = Prompt::from_yaml(&fs::read_to_string(format!("{dir}/assistant.yaml"))?)?;

    // Merge unions the map-typed fields (variables/variants/metadata) at their top-level
    // keys — child-wins on collision. The base's `company` + `max_words` survive; the
    // overlay only declares what it adds. `derive_with` takes the strategy in DeriveOptions.
    let child = base.derive_with(
        PromptOverlay {
            body: Some(
                "You are a {{ tone }} assistant for {{ company }}. \
                 Keep replies under {{ max_words }} words."
                    .to_string(),
            ),
            variables: Some(serde_json::from_value(json!({
                "tone": { "type": "string", "trusted": true }
            }))?),
            ..Default::default()
        },
        DeriveOptions {
            strategy: MergeStrategy::Merge,
        },
    )?;

    // child inherited the base's two variables and gained its own — three in total.
    assert!(child.variables().contains_key("company"));
    assert!(child.variables().contains_key("max_words"));
    assert!(child.variables().contains_key("tone"));
    assert_eq!(child.variables().len(), 3);
    // base is untouched: no `tone` leaked back onto it.
    assert!(!base.variables().contains_key("tone"));
    Ok(())
}
