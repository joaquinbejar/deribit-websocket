//! Server-pushed notification delivery through
//! [`DeribitWebSocketClient::receive_message`].
//!
//! The mock accepts a subscribe, replies success, then pushes a
//! `subscription`-method notification. The client must surface that
//! raw frame text on the next `receive_message` call.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::helpers::client::mock_client;
use super::helpers::frames::{subscribe_success, ticker_notification};
use super::helpers::mock_server::spawn_mock_server;

const CHANNEL: &str = "ticker.BTC-PERPETUAL.raw";

#[tokio::test]
async fn server_push_reaches_receive_message() {
    let server = spawn_mock_server(|mut sink, mut stream| async move {
        // Handle the subscribe then push one notification.
        if let Some(Ok(Message::Text(text))) = stream.next().await {
            let request: Value = serde_json::from_str(&text).expect("subscribe request parses");
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            let _ = sink
                .send(Message::Text(subscribe_success(&id, &[CHANNEL]).into()))
                .await;
        }
        let _ = sink
            .send(Message::Text(ticker_notification(CHANNEL).into()))
            .await;
        // Hold the socket open long enough for the client to pull the
        // notification out of the mpsc channel.
        tokio::time::sleep(Duration::from_millis(200)).await;
    })
    .await;

    let client = mock_client(&server);

    timeout(Duration::from_secs(5), async {
        client.connect().await.expect("connect");
        client
            .subscribe(vec![CHANNEL.to_string()])
            .await
            .expect("subscribe");

        let raw = client
            .receive_message()
            .await
            .expect("notification arrives");
        let parsed: Value = serde_json::from_str(&raw).expect("notification parses as JSON");

        assert_eq!(parsed["method"], "subscription");
        assert_eq!(parsed["params"]["channel"], CHANNEL);
        assert!(
            parsed["params"]["data"].is_object(),
            "data must be present, got {}",
            parsed["params"]["data"]
        );

        client.disconnect().await.expect("disconnect");
    })
    .await
    .expect("notification flow finishes within 5s");

    server.finish().await;
}
