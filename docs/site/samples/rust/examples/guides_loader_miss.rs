//! Loader guide — a missing key raises PromptLoadError (load_not_found),
//! distinct from ConsumerError (the parse/shape error raised on malformed YAML).
//! Standalone — `cargo run --example guides_loader_miss`.

use std::collections::HashMap;

use prompting_press::error::code;
use prompting_press::loader::{MemoryLoader, PromptLoader};
use prompting_press::{Prompt, PromptLoadError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = MemoryLoader::new(HashMap::new());

    // A missing key returns PromptLoadError::NotFound — not a parse error.
    let err = loader.load("missing").unwrap_err();
    match &err {
        PromptLoadError::NotFound { key } => assert_eq!(key, "missing"),
        other => panic!("unexpected variant: {other:?}"),
    }
    // The normalized error row carries code "load_not_found".
    assert_eq!(err.to_field_error().code, code::LOAD_NOT_FOUND);

    // PromptLoadError is distinct from ConsumerError.
    // Parsing bad YAML raises ConsumerError::Load — a different type on a different path.
    let parse_result = Prompt::from_yaml("not: valid: yaml: [");
    assert!(parse_result.is_err());

    Ok(())
}
