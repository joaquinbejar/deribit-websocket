//! Error handling module for WebSocket client

pub(crate) mod display;
pub(crate) mod redaction;

use serde_json::Value;

use crate::model::ws_types::{JsonRpcError, JsonRpcRequest};

/// WebSocket-specific errors
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection failed: {0}")]
    /// Connection failed with error message
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    /// Authentication failed with error message
    AuthenticationFailed(String),

    #[error("Subscription failed: {0}")]
    /// Subscription failed with error message
    SubscriptionFailed(String),

    #[error("Invalid message format: {0}")]
    /// Invalid message format
    InvalidMessage(String),

    #[error("Connection closed unexpectedly")]
    /// Connection was closed
    ConnectionClosed,

    #[error("Heartbeat timeout")]
    /// Heartbeat timeout occurred
    HeartbeatTimeout,

    /// API error returned by Deribit, enriched with request + response
    /// context for debugging.
    ///
    /// The base `Display` form is `"API error <code>: <message>"` so
    /// callers that only call `.to_string()` see the legacy shape. When
    /// `method` and/or `params` are present the suffix
    /// `" (method=..., params=<truncated JSON>)"` is appended; `params`
    /// is truncated to the first 512 characters.
    ///
    /// Sensitive keys (`access_token`, `refresh_token`, `client_secret`,
    /// `signature`, `password`) inside `params` and `raw_response` are
    /// recursively replaced with `"***"` at construction time, before
    /// the value is stored — `Debug` output is therefore also safe.
    #[error(
        "API error {code}: {message}{}",
        display::fmt_api_context(method, params)
    )]
    ApiError {
        /// Deribit API error code.
        code: i64,
        /// Human-readable error message from the server.
        message: String,
        /// JSON-RPC method of the originating request, when available.
        method: Option<String>,
        /// Request parameters after sensitive-key redaction, when
        /// available.
        params: Option<Value>,
        /// Server response JSON after sensitive-key redaction, when
        /// available.
        raw_response: Option<String>,
    },

    #[error("Operation timed out: {0}")]
    /// Operation timed out (e.g., `send_request` awaiting a matching response)
    Timeout(String),

    #[error("Dispatcher task is not running")]
    /// The background dispatcher task is not running (never started, shut
    /// down, or panicked). No further I/O can be performed through it.
    DispatcherDead,

    #[error("Serialization error: {0}")]
    /// JSON serialization or deserialization failed.
    ///
    /// Typically raised when a request contains a numeric field whose value
    /// cannot be represented in JSON (e.g. `NaN` or `Infinity` in an `f64`),
    /// or when parsing a malformed response payload.
    Serialization(#[from] serde_json::Error),
}

impl WebSocketError {
    /// Construct an enriched `ApiError` from the originating request and
    /// the server-side error payload.
    ///
    /// Applies recursive, case-insensitive redaction of the sensitive
    /// keys `access_token`, `refresh_token`, `client_secret`,
    /// `signature`, and `password` to both `params` (cloned from
    /// `request`) and the caller-supplied `raw_response` before storing
    /// them, so the returned value is safe to log or surface through
    /// `Display` / `Debug`.
    #[must_use]
    pub fn api_error_from_parts(
        request: &JsonRpcRequest,
        error: JsonRpcError,
        raw_response: Option<String>,
    ) -> Self {
        Self::ApiError {
            code: i64::from(error.code),
            message: error.message,
            method: Some(request.method.clone()),
            params: request.params.clone().map(redaction::redact_params),
            raw_response: raw_response.map(|r| redaction::redact_raw_response(&r)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_request(method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_owned(),
            id: json!(1),
            method: method.to_owned(),
            params,
        }
    }

    fn make_rpc_error(code: i32, message: &str) -> JsonRpcError {
        JsonRpcError {
            code,
            message: message.to_owned(),
            data: None,
        }
    }

    #[test]
    fn api_error_display_without_context_matches_legacy_prefix() {
        let err = WebSocketError::ApiError {
            code: 10_000,
            message: "not_allowed".to_owned(),
            method: None,
            params: None,
            raw_response: None,
        };
        assert_eq!(err.to_string(), "API error 10000: not_allowed");
    }

    #[test]
    fn api_error_display_includes_method_when_present() {
        let request = make_request("public/get_time", None);
        let rpc_err = make_rpc_error(11_050, "bad_arguments");
        let err = WebSocketError::api_error_from_parts(&request, rpc_err, None);
        let text = err.to_string();
        assert!(text.contains("API error 11050: bad_arguments"));
        assert!(text.contains("method=public/get_time"));
    }

    #[test]
    fn api_error_display_includes_truncated_params() {
        let big_string = "a".repeat(5_000);
        let params = json!({ "blob": big_string });
        let request = make_request("private/buy", Some(params));
        let rpc_err = make_rpc_error(10_001, "invalid_params");
        let err = WebSocketError::api_error_from_parts(&request, rpc_err, None);
        let text = err.to_string();
        assert!(text.contains("method=private/buy"));
        assert!(text.contains("params="));
        assert!(
            text.chars().count() < 1_024,
            "Display should be truncated, got {} chars",
            text.chars().count()
        );
    }

    #[test]
    fn api_error_from_parts_redacts_access_token_in_display() {
        let request = make_request("public/auth", Some(json!({ "access_token": "leaky" })));
        let rpc_err = make_rpc_error(13_004, "invalid_credentials");
        let err = WebSocketError::api_error_from_parts(&request, rpc_err, None);
        let text = err.to_string();
        assert!(!text.contains("leaky"), "access_token leaked: {text}");
        assert!(text.contains("***"));
    }

    #[test]
    fn api_error_from_parts_redacts_refresh_token_in_display() {
        let request = make_request(
            "public/auth",
            Some(json!({ "refresh_token": "refresh-leak" })),
        );
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(13_004, "err"), None);
        assert!(!err.to_string().contains("refresh-leak"));
    }

    #[test]
    fn api_error_from_parts_redacts_client_secret_in_display() {
        let request = make_request(
            "public/auth",
            Some(json!({ "client_secret": "client-secret-leak" })),
        );
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(13_004, "err"), None);
        assert!(!err.to_string().contains("client-secret-leak"));
    }

    #[test]
    fn api_error_from_parts_redacts_signature_in_display() {
        let request = make_request("public/auth", Some(json!({ "signature": "sig-leak" })));
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(13_004, "err"), None);
        assert!(!err.to_string().contains("sig-leak"));
    }

    #[test]
    fn api_error_from_parts_redacts_password_in_display() {
        let request = make_request("public/auth", Some(json!({ "password": "pw-leak" })));
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(13_004, "err"), None);
        assert!(!err.to_string().contains("pw-leak"));
    }

    #[test]
    fn api_error_from_parts_redacts_all_keys_in_debug() {
        let request = make_request(
            "public/auth",
            Some(json!({
                "access_token": "a-leak",
                "refresh_token": "r-leak",
                "client_secret": "c-leak",
                "signature": "s-leak",
                "password": "p-leak",
            })),
        );
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(13_004, "err"), None);
        let debug = format!("{err:?}");
        for leak in ["a-leak", "r-leak", "c-leak", "s-leak", "p-leak"] {
            assert!(
                !debug.contains(leak),
                "{leak} leaked in Debug output: {debug}"
            );
        }
    }

    #[test]
    fn api_error_from_parts_redacts_nested_sensitive_keys() {
        let request = make_request(
            "private/xyz",
            Some(json!({
                "outer": {
                    "inner": {
                        "password": "deep-leak"
                    }
                }
            })),
        );
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(10_001, "err"), None);
        let debug = format!("{err:?}");
        assert!(!debug.contains("deep-leak"));
    }

    #[test]
    fn api_error_from_parts_redacts_case_insensitive_in_debug() {
        // Only letter-case varies here; the spec keys (snake_case) must
        // still be recognisable by `eq_ignore_ascii_case`.
        let request = make_request(
            "private/xyz",
            Some(json!({
                "Password": "UPPER-leak",
                "Access_Token": "upper-snake-leak",
                "REFRESH_TOKEN": "shouty-leak",
            })),
        );
        let err =
            WebSocketError::api_error_from_parts(&request, make_rpc_error(10_001, "err"), None);
        let debug = format!("{err:?}");
        assert!(!debug.contains("UPPER-leak"));
        assert!(!debug.contains("upper-snake-leak"));
        assert!(!debug.contains("shouty-leak"));
    }

    #[test]
    fn api_error_from_parts_sets_method_from_request() {
        let request = make_request("public/test", None);
        let err = WebSocketError::api_error_from_parts(&request, make_rpc_error(1, "x"), None);
        match err {
            WebSocketError::ApiError { method, .. } => {
                assert_eq!(method.as_deref(), Some("public/test"));
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[test]
    fn api_error_from_parts_handles_none_params() {
        let request = make_request("public/test", None);
        let err = WebSocketError::api_error_from_parts(&request, make_rpc_error(1, "x"), None);
        match err {
            WebSocketError::ApiError { params, .. } => assert!(params.is_none()),
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[test]
    fn api_error_from_parts_preserves_error_code_and_message() {
        let request = make_request("public/test", None);
        let err = WebSocketError::api_error_from_parts(
            &request,
            make_rpc_error(13_004, "invalid_credentials"),
            None,
        );
        match err {
            WebSocketError::ApiError { code, message, .. } => {
                assert_eq!(code, 13_004);
                assert_eq!(message, "invalid_credentials");
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[test]
    fn api_error_from_parts_redacts_raw_response() {
        let request = make_request("public/auth", None);
        let raw =
            r#"{"id":1,"error":{"code":13004,"message":"x","data":{"access_token":"raw-leak"}}}"#;
        let err = WebSocketError::api_error_from_parts(
            &request,
            make_rpc_error(13_004, "x"),
            Some(raw.to_owned()),
        );
        match err {
            WebSocketError::ApiError {
                raw_response: Some(stored),
                ..
            } => {
                assert!(!stored.contains("raw-leak"));
                assert!(stored.contains("***"));
            }
            other => panic!("expected ApiError with raw_response, got {other:?}"),
        }
    }

    #[test]
    fn api_error_matches_on_code_still_works() {
        let request = make_request("public/test", None);
        let err = WebSocketError::api_error_from_parts(&request, make_rpc_error(42, "oops"), None);
        match err {
            WebSocketError::ApiError { code, message, .. } => {
                assert_eq!(code, 42);
                assert_eq!(message, "oops");
            }
            _ => panic!("expected ApiError"),
        }
    }
}
