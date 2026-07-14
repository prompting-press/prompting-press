// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Loader guide — FileSystemLoader: map a key to a file in a base directory.
//! Uses the `assistant.yaml` fixture that lives next to this program.
//! Standalone — `cargo run --example guides_loader_filesystem`.

use prompting_press::loader::{FileSystemLoader, PromptLoader};
use prompting_press::Prompt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");

    // Construct from an existing directory (canonicalized at construction time).
    let loader = FileSystemLoader::with_base(dir)?;

    // "assistant" maps to {dir}/assistant.yaml (default suffix ".yaml").
    let raw = loader.load("assistant")?;
    let prompt = Prompt::from_yaml(&raw)?;
    assert_eq!(prompt.name(), "assistant");
    Ok(())
}
