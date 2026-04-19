//! Build a `DeribitWebSocketClient` wired to a [`MockServer`].
//!
//! The configuration uses tight timeouts so tests fail fast when
//! something hangs, and disables logging so test output stays clean.

use std::time::Duration;

use deribit_websocket::prelude::*;

use super::mock_server::MockServer;

/// Build a [`WebSocketConfig`] pointing at `server` with deterministic,
/// tight timeouts and logging disabled. Callers that need different
/// timeouts (for example the request-timeout test) should chain
/// additional `with_*` calls on the returned value.
pub(crate) fn mock_config(server: &MockServer) -> WebSocketConfig {
    WebSocketConfig::with_url(&server.ws_url())
        .expect("mock server URL parses")
        .with_connection_timeout(Duration::from_millis(500))
        .with_request_timeout(Duration::from_secs(2))
        .with_logging(false)
}

/// Build a [`DeribitWebSocketClient`] using [`mock_config`].
pub(crate) fn mock_client(server: &MockServer) -> DeribitWebSocketClient {
    DeribitWebSocketClient::new(&mock_config(server)).expect("client construction succeeds")
}
