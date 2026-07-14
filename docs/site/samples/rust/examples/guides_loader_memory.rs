// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Loader guide — MemoryLoader: load raw text by key, then construct a Prompt.
//! The kernel stays I/O-free; the loader is a separate, caller-invoked I/O leaf.
//! Standalone — `cargo run --example guides_loader_memory`.

use std::collections::HashMap;

use prompting_press::loader::{MemoryLoader, PromptLoader};
use prompting_press::Prompt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert(
        "greet".to_string(),
        r#"name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
"#
        .to_string(),
    );
    let loader = MemoryLoader::new(map);

    // load() returns raw text — parsing is a separate step.
    let raw = loader.load("greet")?;
    let prompt = Prompt::from_yaml(&raw)?;
    assert_eq!(prompt.name(), "greet");
    Ok(())
}
