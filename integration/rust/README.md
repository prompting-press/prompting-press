# Rust Integration Gate

This crate is the pre-release integration gate for `prompting-press`. It is a real downstream consumer (path-dependent on `crates/prompting-press`) that exercises the full public API surface across modular test files — one file per feature — so the harness is trivially extensible. To add coverage for a new feature, add `tests/<feature>.rs` and declare it as a `[[test]]` target (or rely on Cargo's auto-discovery). Run with `cargo test` from this directory.
