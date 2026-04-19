//! Subscribe / unsubscribe round-trip against the mock server.
//!
//! Verifies that [`DeribitWebSocketClient::subscribe`] reconciles the
//! local [`SubscriptionManager`] from the server-confirmed channel list
//! and that [`DeribitWebSocketClient::unsubscribe`] removes them again.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::helpers::client::mock_client;
use super::helpers::frames::{subscribe_success, unsubscribe_success};
use super::helpers::mock_server::spawn_mock_server;

const CHANNEL: &str = "ticker.BTC-PERPETUAL.raw";

#[tokio::test]
async fn subscribe_and_unsubscribe_update_manager() {
    let server = spawn_mock_server(|mut sink, mut stream| async move {
        // Accept two requests: subscribe first, then unsubscribe. Both
        // echo the single channel back as a success result.
        for _ in 0..2 {
            let text = match stream.next().await {
                Some(Ok(Message::Text(t))) => t,
                _ => return,
            };
            let request: Value = serde_json::from_str(&text).expect("request parses as JSON");
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            let method = request["method"].as_str().unwrap_or("");
            let reply = if method.ends_with("/subscribe") {
                subscribe_success(&id, &[CHANNEL])
            } else {
                unsubscribe_success(&id, &[CHANNEL])
            };
            let _ = sink.send(Message::Text(reply.into())).await;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    })
    .await;

    let client = mock_client(&server);

    timeout(Duration::from_secs(5), async {
        client.connect().await.expect("connect");

        client
            .subscribe(vec![CHANNEL.to_string()])
            .await
            .expect("subscribe");

        let active_after_sub = client
            .subscription_manager()
            .lock()
            .await
            .get_active_channels();
        assert!(
            active_after_sub.iter().any(|c| c == CHANNEL),
            "{} must be active after subscribe, got {:?}",
            CHANNEL,
            active_after_sub
        );

        client
            .unsubscribe(vec![CHANNEL.to_string()])
            .await
            .expect("unsubscribe");

        let active_after_unsub = client
            .subscription_manager()
            .lock()
            .await
            .get_active_channels();
        assert!(
            !active_after_unsub.iter().any(|c| c == CHANNEL),
            "{} must be gone after unsubscribe, got {:?}",
            CHANNEL,
            active_after_unsub
        );

        client.disconnect().await.expect("disconnect");
    })
    .await
    .expect("subscribe flow finishes within 5s");

    server.finish().await;
}
