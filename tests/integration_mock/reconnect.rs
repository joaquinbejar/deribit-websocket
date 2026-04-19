//! Manual reconnect flow.
//!
//! The library does not ship an automatic reconnect loop yet; reconnect
//! is caller-driven via `disconnect()` + `connect()` (or just a fresh
//! `connect()`, which tears down the old dispatcher). This test proves
//! the caller-driven path works end-to-end against the mock:
//!
//! 1. Connect, subscribe, receive ACK.
//! 2. Mock closes the socket.
//! 3. Client calls `connect()` again on the same URL; the mock listener
//!    accepts a second handshake.
//! 4. Client replays the subscription (using
//!    [`SubscriptionManager::get_all_channels`]) and the mock echoes
//!    the confirmed channel list.
//! 5. Assert that the mock observed two subscribe frames and the
//!    client's subscription manager still reports the channel active.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::helpers::client::mock_client;
use super::helpers::frames::subscribe_success;
use super::helpers::mock_server::spawn_mock_server_twice;

const CHANNEL: &str = "ticker.BTC-PERPETUAL.raw";

#[tokio::test]
async fn manual_reconnect_replays_subscription() {
    let observed_methods: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let observed_first = Arc::clone(&observed_methods);
    let observed_second = Arc::clone(&observed_methods);

    let server = spawn_mock_server_twice(
        |mut sink, mut stream| async move {
            // First connection: handle the subscribe, then close the
            // socket to simulate a server-initiated disconnect.
            if let Some(Ok(Message::Text(text))) = stream.next().await {
                let request: Value = serde_json::from_str(&text).expect("subscribe request parses");
                let id = request.get("id").cloned().unwrap_or(Value::Null);
                let method = request["method"].as_str().unwrap_or("").to_owned();
                observed_first.lock().await.push(method);
                let _ = sink
                    .send(Message::Text(subscribe_success(&id, &[CHANNEL]).into()))
                    .await;
            }
            let _ = sink.close().await;
        },
        |mut sink, mut stream| async move {
            // Second connection: the client replays the subscription.
            if let Some(Ok(Message::Text(text))) = stream.next().await {
                let request: Value = serde_json::from_str(&text).expect("subscribe replay parses");
                let id = request.get("id").cloned().unwrap_or(Value::Null);
                let method = request["method"].as_str().unwrap_or("").to_owned();
                observed_second.lock().await.push(method);
                let _ = sink
                    .send(Message::Text(subscribe_success(&id, &[CHANNEL]).into()))
                    .await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        },
    )
    .await;

    let client = mock_client(&server);

    timeout(Duration::from_secs(5), async {
        // Phase 1: initial connect + subscribe.
        client.connect().await.expect("initial connect");
        client
            .subscribe(vec![CHANNEL.to_string()])
            .await
            .expect("initial subscribe");

        // Phase 2: manual reconnect. `connect()` tears down the old
        // (EOF) dispatcher and installs a fresh one against the
        // listener's second accept.
        client.connect().await.expect("reconnect");

        // Phase 3: replay every known channel through the new
        // dispatcher. `get_all_channels` includes previously subscribed
        // channels whose entries are preserved across reconnects.
        let channels = client
            .subscription_manager()
            .lock()
            .await
            .get_all_channels();
        assert!(
            channels.iter().any(|c| c == CHANNEL),
            "manager must remember the channel across reconnect, got {:?}",
            channels
        );
        client.subscribe(channels).await.expect("replay subscribe");

        let active = client
            .subscription_manager()
            .lock()
            .await
            .get_active_channels();
        assert!(
            active.iter().any(|c| c == CHANNEL),
            "{} must be active after reconnect, got {:?}",
            CHANNEL,
            active
        );

        client.disconnect().await.expect("disconnect");
    })
    .await
    .expect("reconnect flow finishes within 5s");

    let methods = observed_methods.lock().await;
    assert_eq!(
        methods.len(),
        2,
        "mock must have observed two subscribe frames, got {:?}",
        methods
    );
    assert!(
        methods.iter().all(|m| m == "public/subscribe"),
        "both observed frames must be public/subscribe, got {:?}",
        methods
    );
}
