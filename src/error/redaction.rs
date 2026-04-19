//! Sensitive-value redaction helpers for enriched error contexts.
//!
//! The `ApiError` variant of [`WebSocketError`] optionally carries the
//! originating request `params` and the full server response. Some
//! requests (authentication, signed private calls) contain secrets that
//! must never leak to logs, metrics, or error reporters. This module
//! provides the redaction primitives used at construction time.
//!
//! [`WebSocketError`]: crate::error::WebSocketError

use std::borrow::Cow;

use serde_json::Value;

/// Keys whose values must be redacted, regardless of casing, at any depth.
pub(crate) const SENSITIVE_KEYS: &[&str] = &[
    "access_token",
    "refresh_token",
    "client_secret",
    "signature",
    "password",
];

/// Replacement string written in place of any redacted value.
pub(crate) const REDACTION_MARKER: &str = "***";

/// Maximum length of any payload rendered inside a `Display` impl.
///
/// Long request bodies are truncated to this many Unicode scalar values
/// (characters, not bytes) with an ellipsis suffix.
pub(crate) const MAX_PAYLOAD_DISPLAY_LEN: usize = 512;

/// Recursively redact sensitive values inside a JSON payload.
///
/// The comparison is case-insensitive and performed on object keys only.
/// Arrays and nested objects are traversed in place; scalar values that
/// are not under a sensitive key are left untouched.
#[must_use]
pub(crate) fn redact_params(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let redacted = map
                .into_iter()
                .map(|(key, v)| {
                    if is_sensitive_key(&key) {
                        (key, Value::String(REDACTION_MARKER.to_owned()))
                    } else {
                        (key, redact_params(v))
                    }
                })
                .collect();
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(redact_params).collect()),
        other => other,
    }
}

/// Placeholder substituted when [`redact_raw_response`] is given input
/// that is not parseable as JSON.
///
/// Failing closed prevents leakage of secrets that might appear as bare
/// substrings in non-JSON payloads (HTML error pages, raw log lines,
/// truncated responses).
pub(crate) const NON_JSON_PLACEHOLDER: &str = "<non-json response redacted>";

/// Redact a raw response string by parsing it as JSON and recursively
/// stripping sensitive keys.
///
/// **Fails closed**: input that is not valid JSON is replaced with
/// [`NON_JSON_PLACEHOLDER`] rather than returned verbatim. The redactor
/// can only reason about JSON structure, so a non-JSON payload that
/// happens to contain `password=hunter2` would otherwise leak. By
/// construction every caller in this crate hands `redact_raw_response`
/// a freshly-serialised JSON envelope (see
/// [`super::envelope::build_raw_error_response`]), so the placeholder
/// path is unreachable on the happy path; this is defence in depth for
/// future callers.
#[must_use]
pub(crate) fn redact_raw_response(raw: &str) -> String {
    match serde_json::from_str::<Value>(raw) {
        Ok(value) => {
            let redacted = redact_params(value);
            // Re-serialisation of a valid `Value` cannot fail.
            serde_json::to_string(&redacted).unwrap_or_else(|_| NON_JSON_PLACEHOLDER.to_owned())
        }
        Err(_) => NON_JSON_PLACEHOLDER.to_owned(),
    }
}

/// Truncate a string to [`MAX_PAYLOAD_DISPLAY_LEN`] Unicode scalar values
/// for safe inclusion in `Display` output.
///
/// Splitting on char boundaries (rather than bytes) avoids panicking on
/// multibyte input. Shorter inputs are borrowed unchanged.
#[must_use]
pub(crate) fn truncate_for_display(s: &str) -> Cow<'_, str> {
    match s.char_indices().nth(MAX_PAYLOAD_DISPLAY_LEN) {
        Some((byte_idx, _)) => {
            let mut truncated =
                String::with_capacity(byte_idx.saturating_add('…'.len_utf8()));
            truncated.push_str(&s[..byte_idx]);
            truncated.push('…');
            Cow::Owned(truncated)
        }
        None => Cow::Borrowed(s),
    }
}

#[inline]
fn is_sensitive_key(key: &str) -> bool {
    SENSITIVE_KEYS
        .iter()
        .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redact_params_scalar_passthrough() {
        assert_eq!(redact_params(Value::Null), Value::Null);
        assert_eq!(redact_params(json!(42)), json!(42));
        assert_eq!(redact_params(json!(true)), json!(true));
        assert_eq!(redact_params(json!("plain")), json!("plain"));
    }

    #[test]
    fn redact_params_redacts_top_level_access_token() {
        let input = json!({ "access_token": "secret-token", "other": 1 });
        let out = redact_params(input);
        assert_eq!(out, json!({ "access_token": "***", "other": 1 }));
    }

    #[test]
    fn redact_params_redacts_refresh_token() {
        let input = json!({ "refresh_token": "abc" });
        assert_eq!(redact_params(input), json!({ "refresh_token": "***" }));
    }

    #[test]
    fn redact_params_redacts_client_secret() {
        let input = json!({ "client_secret": "s3cret" });
        assert_eq!(redact_params(input), json!({ "client_secret": "***" }));
    }

    #[test]
    fn redact_params_redacts_signature() {
        let input = json!({ "signature": "deadbeef" });
        assert_eq!(redact_params(input), json!({ "signature": "***" }));
    }

    #[test]
    fn redact_params_redacts_password() {
        let input = json!({ "password": "hunter2" });
        assert_eq!(redact_params(input), json!({ "password": "***" }));
    }

    #[test]
    fn redact_params_redacts_case_insensitive_variants() {
        // `eq_ignore_ascii_case` varies letter case only, not structure:
        // `Access_Token` matches `access_token`, but `AccessToken`
        // (no underscore) does not. The issue #52 redaction list
        // uses snake_case names exactly, so only case variants are
        // expected to match.
        let input = json!({
            "Password": "a",
            "PASSWORD": "b",
            "Access_Token": "c",
            "REFRESH_TOKEN": "d",
            "Client_Secret": "e",
            "Signature": "f",
        });
        let out = redact_params(input);
        assert_eq!(
            out,
            json!({
                "Password": "***",
                "PASSWORD": "***",
                "Access_Token": "***",
                "REFRESH_TOKEN": "***",
                "Client_Secret": "***",
                "Signature": "***",
            })
        );
    }

    #[test]
    fn redact_params_does_not_redact_structurally_different_keys() {
        // Keys that differ from the snake_case spec by more than letter
        // case (e.g. CamelCase without underscore) are NOT redacted.
        let input = json!({
            "AccessToken": "keep-me",
            "clientsecret": "keep-me-too",
        });
        let out = redact_params(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn redact_params_redacts_nested_object() {
        let input = json!({
            "auth": {
                "client_id": "public-id",
                "client_secret": "shh",
                "nested": { "password": "deeper" }
            },
            "data": 1
        });
        let out = redact_params(input);
        assert_eq!(
            out,
            json!({
                "auth": {
                    "client_id": "public-id",
                    "client_secret": "***",
                    "nested": { "password": "***" }
                },
                "data": 1
            })
        );
    }

    #[test]
    fn redact_params_redacts_array_of_objects() {
        let input = json!([
            { "access_token": "t1", "keep": 1 },
            { "access_token": "t2", "keep": 2 },
        ]);
        let out = redact_params(input);
        assert_eq!(
            out,
            json!([
                { "access_token": "***", "keep": 1 },
                { "access_token": "***", "keep": 2 },
            ])
        );
    }

    #[test]
    fn redact_params_leaves_non_sensitive_keys_alone() {
        let input = json!({
            "instrument_name": "BTC-PERPETUAL",
            "amount": 10,
            "type": "limit",
        });
        let out = redact_params(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn redact_raw_response_valid_json_redacts() {
        let raw = r#"{"access_token":"leak","id":1}"#;
        let redacted = redact_raw_response(raw);
        assert!(!redacted.contains("leak"));
        assert!(redacted.contains("***"));
        assert!(redacted.contains("\"id\":1"));
    }

    #[test]
    fn redact_raw_response_invalid_json_replaced_with_placeholder() {
        let raw = "not json at all";
        assert_eq!(redact_raw_response(raw), NON_JSON_PLACEHOLDER);
    }

    #[test]
    fn redact_raw_response_invalid_json_does_not_leak_sensitive_like_substrings() {
        // Non-JSON text that happens to mention "password" must NOT be
        // returned verbatim — the redactor cannot reason about key/value
        // structure in arbitrary text, so it fails closed.
        let raw = "password=hunter2 (raw log line)";
        let out = redact_raw_response(raw);
        assert_eq!(out, NON_JSON_PLACEHOLDER);
        assert!(!out.contains("hunter2"));
    }

    #[test]
    fn redact_raw_response_truncated_json_fails_closed() {
        // A response that was cut off mid-JSON would parse-fail; ensure
        // we still don't leak any sensitive substrings it carried.
        let raw = r#"{"access_token":"leak-me","id":1"#;
        let out = redact_raw_response(raw);
        assert_eq!(out, NON_JSON_PLACEHOLDER);
        assert!(!out.contains("leak-me"));
    }

    #[test]
    fn truncate_for_display_short_borrows() {
        let s = "abc";
        let out = truncate_for_display(s);
        assert!(matches!(out, Cow::Borrowed(_)));
        assert_eq!(out, "abc");
    }

    #[test]
    fn truncate_for_display_long_truncates_at_char_boundary() {
        let s: String = "x".repeat(MAX_PAYLOAD_DISPLAY_LEN + 100);
        let out = truncate_for_display(&s);
        assert!(matches!(out, Cow::Owned(_)));
        // MAX_PAYLOAD_DISPLAY_LEN chars + one ellipsis char.
        assert_eq!(out.chars().count(), MAX_PAYLOAD_DISPLAY_LEN + 1);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn truncate_for_display_multibyte_does_not_panic() {
        // "€" is 3 bytes in UTF-8; repeating it well past the char cap
        // would panic if we sliced by bytes instead of chars.
        let s: String = "€".repeat(MAX_PAYLOAD_DISPLAY_LEN + 10);
        let out = truncate_for_display(&s);
        assert_eq!(out.chars().count(), MAX_PAYLOAD_DISPLAY_LEN + 1);
    }

    #[test]
    fn truncate_for_display_exact_cap_is_borrowed() {
        let s: String = "x".repeat(MAX_PAYLOAD_DISPLAY_LEN);
        let out = truncate_for_display(&s);
        assert!(matches!(out, Cow::Borrowed(_)));
        assert_eq!(out.chars().count(), MAX_PAYLOAD_DISPLAY_LEN);
    }
}
