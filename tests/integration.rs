//! Integration tests against a local mock WebSocket server.
//!
//! These tests are self-contained: they spawn a `tokio-tungstenite`
//! server bound to `127.0.0.1:0`, wire a `DeribitWebSocketClient` to
//! that ephemeral port, and exercise the public client API without
//! touching the real Deribit service. No credentials or network access
//! required.
//!
//! Covers the flows listed in issue #53: auth, subscribe/unsubscribe,
//! notification delivery, reconnect, request/response id matching, and
//! timeout handling.
//!
//! Run with `cargo test --test integration`.

// Integration tests routinely use `.unwrap()` / `.expect()` for brevity
// and to surface failures with clear panic messages. Silence the strict
// lints that are enforced on the library crate here.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod integration_mock;
