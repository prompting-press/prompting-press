//! The prompt [`Registry`] — a library-owned map of prompt name → loaded
//! [`PromptDefinition`] (FR-008a, clarify Q2).
//!
//! Backed by a [`BTreeMap`] so iteration order is **deterministic** — `check()` (a later
//! phase) walks the registry and must produce stable, reproducible findings ordering for a
//! CI gate.
//!
//! This phase ships only construction + in-memory population (`new`, `insert`, `get`). The
//! dual-input loaders (`load_yaml` / `load_json`) arrive in US2 (T013); the crate does no
//! I/O — the caller hands in already-read text or a constructed object (C-03).

use std::collections::BTreeMap;

use prompting_press_core::PromptDefinition;

/// A name → [`PromptDefinition`] map. The single in-memory home for loaded prompts;
/// `render` / `get_source` / `check` resolve a prompt by name against it (absent ⇒
/// [`crate::ConsumerError::UnknownPrompt`], wired in a later phase).
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// BTreeMap keyed by [`PromptDefinition::name`] → deterministic iteration for `check()`.
    prompts: BTreeMap<String, PromptDefinition>,
}

impl Registry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a constructed [`PromptDefinition`], keyed by its `name`.
    ///
    /// The key is the prompt's [`name`](PromptDefinition::name) (a `#[serde(transparent)]`
    /// newtype that derefs to `String`). An existing entry with the same name is replaced.
    pub fn insert(&mut self, def: PromptDefinition) {
        let key = def.name.to_string();
        self.prompts.insert(key, def);
    }

    /// Look up a prompt by name. Returns `None` when absent — callers that need a hard error
    /// map the absence to [`crate::ConsumerError::UnknownPrompt`] (FR-008a).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PromptDefinition> {
        self.prompts.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(name: &str) -> PromptDefinition {
        serde_json::from_str(&format!(
            r#"{{ "name": "{name}", "role": "user", "body": "Hello {{{{ x }}}}" }}"#
        ))
        .expect("valid prompt definition")
    }

    #[test]
    fn insert_then_get_round_trips_by_name() {
        let mut reg = Registry::new();
        reg.insert(def("greet"));

        let got = reg.get("greet").expect("present after insert");
        assert_eq!(got.name.to_string(), "greet");
        assert!(reg.get("absent").is_none());
    }

    #[test]
    fn insert_replaces_same_name() {
        let mut reg = Registry::new();
        reg.insert(def("greet"));
        reg.insert(def("greet"));
        // Still exactly one logical entry under that name.
        assert!(reg.get("greet").is_some());
    }

    #[test]
    fn empty_registry_resolves_nothing() {
        let reg = Registry::new();
        assert!(reg.get("anything").is_none());
    }
}
