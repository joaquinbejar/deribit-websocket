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
        // Accept the subscribe first, assert its method is literally
        // `public/subscribe`, then echo the channel list.
        let subscribe_text = match stream.next().await {
            Some(Ok(Message::Text(t))) => t,
            _ => return,
        };
        let subscribe_request: Value =
            serde_json::from_str(&subscribe_text).expect("subscribe request parses as JSON");
        let subscribe_id = subscribe_request.get("id").cloned().unwrap_or(Value::Null);
        assert_eq!(
            subscribe_request["method"].as_str().unwrap_or(""),
            "public/subscribe",
            "first request must be public/subscribe, got {}",
            subscribe_request["method"]
        );
        let _ = sink
            .send(Message::Text(
                subscribe_success(&subscribe_id, &[CHANNEL]).into(),
            ))
            .await;

        // Then the unsubscribe, same assertion pattern.
        let unsubscribe_text = match stream.next().await {
            Some(Ok(Message::Text(t))) => t,
            _ => return,
        };
        let unsubscribe_request: Value =
            serde_json::from_str(&unsubscribe_text).expect("unsubscribe request parses as JSON");
        let unsubscribe_id = unsubscribe_request
            .get("id")
            .cloned()
            .unwrap_or(Value::Null);
        assert_eq!(
            unsubscribe_request["method"].as_str().unwrap_or(""),
            "public/unsubscribe",
            "second request must be public/unsubscribe, got {}",
            unsubscribe_request["method"]
        );
        let _ = sink
            .send(Message::Text(
                unsubscribe_success(&unsubscribe_id, &[CHANNEL]).into(),
            ))
            .await;
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
