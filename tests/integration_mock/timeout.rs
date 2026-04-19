//! Per-request timeout enforcement.
//!
//! When the server accepts a request but never replies, the client
//! must surface [`WebSocketError::Timeout`] within roughly the
//! configured `request_timeout`, not hang forever.

use std::time::Duration;

use deribit_websocket::prelude::*;
use futures_util::StreamExt;
use tokio::time::timeout;

use super::helpers::client::mock_config;
use super::helpers::mock_server::spawn_mock_server;

/// Configured `request_timeout` for this test. Short enough that CI
/// wall-clock noise is dwarfed by the deadline, long enough that
/// scheduling jitter on a slow runner still gets the timer fired
/// before the scenario's own sleep expires.
const REQUEST_TIMEOUT: Duration = Duration::from_millis(150);

/// Upper bound on how long the client may take to surface the timeout.
/// 5x headroom over the deadline, matching the convention used in the
/// dispatcher unit tests.
const TIMEOUT_OBSERVATION_BOUND: Duration = Duration::from_millis(750);

#[tokio::test]
async fn request_timeout_fires_on_silent_server() {
    let server = spawn_mock_server(|_sink, mut stream| async move {
        // Read the request and deliberately never reply.
        let _ = stream.next().await;
        tokio::time::sleep(Duration::from_secs(2)).await;
    })
    .await;

    let config = mock_config(&server).with_request_timeout(REQUEST_TIMEOUT);
    let client = DeribitWebSocketClient::new(&config).expect("client construction");

    timeout(Duration::from_secs(5), async {
        client.connect().await.expect("connect");

        let start = std::time::Instant::now();
        let result = client.get_time().await;
        let elapsed = start.elapsed();

        assert!(
            matches!(result, Err(WebSocketError::Timeout(_))),
            "expected Timeout, got {:?}",
            result
        );
        assert!(
            elapsed < TIMEOUT_OBSERVATION_BOUND,
            "timeout must fire within {:?} of the {:?} deadline, took {:?}",
            TIMEOUT_OBSERVATION_BOUND,
            REQUEST_TIMEOUT,
            elapsed
        );

        client.disconnect().await.expect("disconnect");
    })
    .await
    .expect("timeout flow finishes within 5s");
}
