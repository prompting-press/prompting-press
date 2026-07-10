//! Loader guide — custom loaders: a closure or a struct implementing PromptLoader.
//! No struct is required — any `Fn(&str) -> Result<String, PromptLoadError>` closure works.
//! Standalone — `cargo run --example guides_loader_custom`.

use prompting_press::loader::PromptLoader;
use prompting_press::{Prompt, PromptLoadError};

const GREET_YAML: &str = r#"name: greet
role: user
body: "Hello {{ name }}"
variables:
  name: { type: string, trusted: true }
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A closure is a loader — no struct needed.
    let loader = |key: &str| -> Result<String, PromptLoadError> {
        match key {
            "greet" => Ok(GREET_YAML.to_string()),
            _ => Err(PromptLoadError::NotFound {
                key: key.to_string(),
            }),
        }
    };

    let raw = loader.load("greet")?;
    let prompt = Prompt::from_yaml(&raw)?;
    assert_eq!(prompt.name(), "greet");

    // A missing key returns NotFound.
    assert!(loader.load("missing").is_err());
    Ok(())
}
