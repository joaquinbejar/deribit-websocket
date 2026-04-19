//! JSON-RPC frame constructors for mock scenarios.
//!
//! Each helper returns the serialised JSON payload ready to wrap in a
//! `tokio_tungstenite::tungstenite::Message::Text`. The `id` is passed
//! in as a borrowed [`serde_json::Value`] so the scenario can echo back
//! whatever the client sent, preserving the JSON-RPC id-correlation
//! semantics the tests exercise.

use serde_json::{Value, json};

/// Build a `{jsonrpc, id, result}` success response.
pub(crate) fn response(id: &Value, result: Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
    .to_string()
}

/// Build a canned `public/auth` success response matching
/// [`deribit_websocket::model::AuthResponse`].
pub(crate) fn auth_success(id: &Value) -> String {
    response(
        id,
        json!({
            "access_token": "mock-access-token",
            "token_type": "bearer",
            "expires_in": 900,
            "refresh_token": "mock-refresh-token",
            "scope": "session:mock",
        }),
    )
}

/// Build a `public/subscribe` success response: JSON array of the
/// channels the server confirmed.
pub(crate) fn subscribe_success(id: &Value, channels: &[&str]) -> String {
    let arr: Vec<Value> = channels
        .iter()
        .map(|c| Value::String((*c).to_owned()))
        .collect();
    response(id, Value::Array(arr))
}

/// Build a `public/unsubscribe` success response: identical shape to
/// [`subscribe_success`].
pub(crate) fn unsubscribe_success(id: &Value, channels: &[&str]) -> String {
    subscribe_success(id, channels)
}

/// Build a server-pushed `subscription` notification for `channel`
/// with a small, deterministic ticker-like payload.
pub(crate) fn ticker_notification(channel: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "subscription",
        "params": {
            "channel": channel,
            "data": {
                "instrument_name": "BTC-PERPETUAL",
                "best_bid_price": 50000.0,
                "best_ask_price": 50001.0,
                "timestamp": 1_700_000_000_000_u64,
            },
        },
    })
    .to_string()
}

/// Build a `{jsonrpc, id, result}` frame carrying a plain integer
/// result, used by the id-matching test where the mock echoes the
/// request id scaled by 1000 so each response is distinguishable.
pub(crate) fn u64_result(id: &Value, value: u64) -> String {
    response(id, json!(value))
}
