//! Authentication flow against the mock server.
//!
//! Verifies that `DeribitWebSocketClient::authenticate` sends a
//! `public/auth` JSON-RPC request, that the returned `AuthResponse` is
//! parsed from the server reply, and that the whole round-trip
//! completes without panics.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::helpers::client::mock_client;
use super::helpers::frames::auth_success;
use super::helpers::mock_server::spawn_mock_server;

#[tokio::test]
async fn authenticate_round_trip_against_mock() {
    let server = spawn_mock_server(|mut sink, mut stream| async move {
        // Read the single `public/auth` request and reply with a canned
        // AuthResponse carrying the id the client assigned.
        if let Some(Ok(Message::Text(text))) = stream.next().await {
            let request: Value = serde_json::from_str(&text).expect("auth request parses as JSON");
            assert_eq!(
                request["method"], "public/auth",
                "first frame must be public/auth, got {}",
                request["method"]
            );
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            let _ = sink.send(Message::Text(auth_success(&id).into())).await;
        }
        // Keep the sink alive briefly so the client can read the reply.
        tokio::time::sleep(Duration::from_millis(50)).await;
    })
    .await;

    let client = mock_client(&server);

    timeout(Duration::from_secs(5), async {
        client.connect().await.expect("connect succeeds");

        let auth = client
            .authenticate("mock-id", "mock-secret")
            .await
            .expect("authenticate succeeds");

        assert_eq!(auth.access_token, "mock-access-token");
        assert_eq!(auth.token_type, "bearer");
        assert_eq!(auth.expires_in, 900);
        assert_eq!(auth.refresh_token, "mock-refresh-token");
        assert_eq!(auth.scope, "session:mock");

        client.disconnect().await.expect("disconnect succeeds");
    })
    .await
    .expect("auth flow finishes within 5s");
}
