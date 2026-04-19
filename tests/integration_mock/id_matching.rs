//! Concurrent request/response id matching against the mock server.
//!
//! Spawns three concurrent [`DeribitWebSocketClient::get_time`] calls
//! and has the mock reply in REVERSE order to prove that the
//! dispatcher's id-correlation does not depend on response ordering —
//! each caller's future must resolve to the value keyed by the id it
//! sent, never another caller's result.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::helpers::client::mock_client;
use super::helpers::frames::u64_result;
use super::helpers::mock_server::spawn_mock_server;

const CONCURRENT_REQUESTS: usize = 3;

/// Per-request marker the mock multiplies the request id by, producing
/// a distinct response value per id.
const RESULT_MULTIPLIER: u64 = 1_000;

#[tokio::test]
async fn concurrent_requests_match_responses_by_id() {
    let server = spawn_mock_server(|mut sink, mut stream| async move {
        // Read all N requests before sending any reply, then flush them
        // back in reverse order. The reverse flush is what proves id
        // matching is not order-dependent.
        let mut requests: Vec<Value> = Vec::with_capacity(CONCURRENT_REQUESTS);
        for _ in 0..CONCURRENT_REQUESTS {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let v: Value = serde_json::from_str(&text).expect("request parses as JSON");
                    requests.push(v);
                }
                _ => return,
            }
        }
        for request in requests.into_iter().rev() {
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            let id_num = id.as_u64().unwrap_or(0);
            let value = id_num.saturating_mul(RESULT_MULTIPLIER);
            let _ = sink
                .send(Message::Text(u64_result(&id, value).into()))
                .await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    })
    .await;

    let client = Arc::new(mock_client(&server));

    timeout(Duration::from_secs(5), async {
        client.connect().await.expect("connect");

        let mut handles = Vec::with_capacity(CONCURRENT_REQUESTS);
        for _ in 0..CONCURRENT_REQUESTS {
            let c = Arc::clone(&client);
            handles.push(tokio::spawn(async move { c.get_time().await }));
        }

        let mut results: Vec<u64> = Vec::with_capacity(CONCURRENT_REQUESTS);
        for handle in handles {
            let value = handle
                .await
                .expect("task did not panic")
                .expect("get_time succeeds");
            results.push(value);
        }

        // Each request got id N (N >= 1), and the mock replies with
        // N * RESULT_MULTIPLIER. After collection, the set of results
        // must be exactly {N * MULT for N in 1..=CONCURRENT_REQUESTS}
        // regardless of the order the tasks finished.
        results.sort_unstable();
        let expected: Vec<u64> = (1..=CONCURRENT_REQUESTS as u64)
            .map(|i| i.saturating_mul(RESULT_MULTIPLIER))
            .collect();
        assert_eq!(
            results, expected,
            "concurrent requests must resolve to their own ids: got {:?}, expected {:?}",
            results, expected
        );

        client.disconnect().await.expect("disconnect");
    })
    .await
    .expect("id-matching flow finishes within 5s");
}
