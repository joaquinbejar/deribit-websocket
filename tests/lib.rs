//! Integration tests for deribit-websocket crate

// Integration tests routinely use `.unwrap()` / `.expect()` for brevity and
// to surface failures with clear panic messages. Silence the strict lints
// that are enforced on the library crate here.
#![allow(clippy::unwrap_used, clippy::expect_used)]

// Disambiguate from the sibling `tests/integration.rs` test binary
// (issue #53 mock-server suite). Without this attribute, Rust cannot
// decide whether `mod integration;` refers to `tests/integration.rs`
// or `tests/integration/mod.rs` and rejects the build with E0761.
#[path = "integration/mod.rs"]
mod integration;
mod unit;

// This file serves as the main entry point for all tests.
// Tests are organized by module in the unit/ and integration/ subdirectories.
