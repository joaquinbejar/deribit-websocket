//! Shared helpers for the mock-server integration tests.
//!
//! - [`mock_server`] spawns a local `tokio-tungstenite` server bound to
//!   an ephemeral port.
//! - [`client`] builds a `DeribitWebSocketClient` wired to that server
//!   with deterministic, tight timeouts suitable for tests.
//! - [`frames`] constructs the JSON-RPC payloads the mock scenarios
//!   reply with.

pub(crate) mod client;
pub(crate) mod frames;
pub(crate) mod mock_server;
