// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Loader guide — loading JSON and TOML: the loader is format-agnostic (returns raw
//! text), so only the FileSystemLoader suffix and the `from_*` parser change.
//! Uses the `assistant.json` / `assistant.toml` fixtures next to this program.
//! Standalone — `cargo run --example guides_loader_formats`.

use prompting_press::loader::{FileSystemLoader, PromptLoader};
use prompting_press::Prompt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");

    // JSON: suffix ".json" → loads {dir}/assistant.json, parsed with from_json.
    let json_loader = FileSystemLoader::new(dir, ".json", FileSystemLoader::DEFAULT_MAX_BYTES)?;
    let json_raw = json_loader.load("assistant")?;
    let from_json = Prompt::from_json(&json_raw)?;
    assert_eq!(from_json.name(), "assistant");

    // TOML: suffix ".toml" → loads {dir}/assistant.toml, parsed with from_toml.
    let toml_loader = FileSystemLoader::new(dir, ".toml", FileSystemLoader::DEFAULT_MAX_BYTES)?;
    let toml_raw = toml_loader.load("assistant")?;
    let from_toml = Prompt::from_toml(&toml_raw)?;
    assert_eq!(from_toml.name(), "assistant");

    // Empty suffix → the extension lives in the key instead (same file either way).
    let bare_loader = FileSystemLoader::new(dir, "", FileSystemLoader::DEFAULT_MAX_BYTES)?;
    let bare_raw = bare_loader.load("assistant.json")?;
    assert_eq!(Prompt::from_json(&bare_raw)?.name(), "assistant");

    Ok(())
}
