//! Integration tests for deribit-websocket crate

// Integration tests routinely use `.unwrap()` / `.expect()` for brevity and
// to surface failures with clear panic messages. Silence the strict lints
// that are enforced on the library crate here.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod integration;
mod unit;

// This file serves as the main entry point for all tests.
// Tests are organized by module in the unit/ and integration/ subdirectories.
