//! Reconstruction of the raw JSON-RPC error envelope.
//!
//! When the server returns a JSON-RPC error, the client only retains
//! the parsed `JsonRpcResponse` (with `result` already destructured).
//! This module reassembles a wire-equivalent `{jsonrpc, id, error}`
//! payload from the still-accessible response fields so the enriched
//! [`super::WebSocketError::ApiError`] can carry the full context for
//! debugging.

use serde_json::Value;

use crate::model::ws_types::JsonRpcError;

/// Build the JSON string `{"jsonrpc": ..., "id": ..., "error": ...}`
/// for an error response.
///
/// This is the canonical helper used by every `ApiError` construction
/// site in the client. Keeping it in one place avoids copy/paste drift
/// of the envelope shape across the ~30 call sites.
///
/// `serde_json::to_string` on a `Value` is infallible (any well-formed
/// `Value` serialises), so this function returns `String` directly.
#[must_use]
#[inline]
pub(crate) fn build_raw_error_response(jsonrpc: &str, id: &Value, error: &JsonRpcError) -> String {
    serde_json::json!({
        "jsonrpc": jsonrpc,
        "id": id,
        "error": error,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_error(code: i32, message: &str, data: Option<Value>) -> JsonRpcError {
        JsonRpcError {
            code,
            message: message.to_owned(),
            data,
        }
    }

    fn parse(raw: &str) -> Value {
        match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("envelope must be valid JSON: {e}; raw={raw}"),
        }
    }

    #[test]
    fn envelope_round_trips_through_serde() {
        let raw = build_raw_error_response(
            "2.0",
            &json!(42),
            &make_error(13_004, "invalid_credentials", None),
        );
        let parsed = parse(&raw);
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 42);
        assert_eq!(parsed["error"]["code"], 13_004);
        assert_eq!(parsed["error"]["message"], "invalid_credentials");
        assert!(parsed["error"]["data"].is_null());
    }

    #[test]
    fn envelope_preserves_string_id() {
        let raw = build_raw_error_response(
            "2.0",
            &json!("client-correlation-id"),
            &make_error(1, "x", None),
        );
        let parsed = parse(&raw);
        assert_eq!(parsed["id"], "client-correlation-id");
    }

    #[test]
    fn envelope_preserves_error_data_payload() {
        let data = Some(json!({ "reason": "rate_limited", "retry_after_ms": 1500 }));
        let raw = build_raw_error_response("2.0", &json!(7), &make_error(10_028, "too_many", data));
        let parsed = parse(&raw);
        assert_eq!(parsed["error"]["data"]["reason"], "rate_limited");
        assert_eq!(parsed["error"]["data"]["retry_after_ms"], 1_500);
    }
}
