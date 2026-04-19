//! `Display` formatting helpers for enriched error contexts.

use serde_json::Value;

use super::redaction::truncate_for_display;

/// Format the optional `method` / `params` context for an `ApiError`.
///
/// Returns the empty string when both fields are `None`, so the base
/// `"API error <code>: <message>"` prefix stays byte-identical to the
/// legacy variant for callers that only inspect `to_string()`.
///
/// When `params` is present, its JSON serialisation is truncated at
/// [`MAX_PAYLOAD_DISPLAY_LEN`] characters via
/// [`truncate_for_display`] so long payloads do not flood logs.
///
/// [`MAX_PAYLOAD_DISPLAY_LEN`]: super::redaction::MAX_PAYLOAD_DISPLAY_LEN
#[must_use]
pub(crate) fn fmt_api_context(method: &Option<String>, params: &Option<Value>) -> String {
    match (method.as_deref(), params.as_ref()) {
        (None, None) => String::new(),
        (Some(m), None) => format!(" (method={m})"),
        (None, Some(p)) => {
            let rendered = serde_json::to_string(p).unwrap_or_else(|_| "<unserializable>".into());
            format!(" (params={})", truncate_for_display(&rendered))
        }
        (Some(m), Some(p)) => {
            let rendered = serde_json::to_string(p).unwrap_or_else(|_| "<unserializable>".into());
            format!(" (method={m}, params={})", truncate_for_display(&rendered))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn no_context_returns_empty_string() {
        assert_eq!(fmt_api_context(&None, &None), "");
    }

    #[test]
    fn method_only() {
        let method = Some("public/get_time".to_owned());
        assert_eq!(fmt_api_context(&method, &None), " (method=public/get_time)");
    }

    #[test]
    fn params_only() {
        let params = Some(json!({ "instrument_name": "BTC-PERPETUAL" }));
        let out = fmt_api_context(&None, &params);
        assert!(out.starts_with(" (params="));
        assert!(out.contains("BTC-PERPETUAL"));
    }

    #[test]
    fn method_and_params() {
        let method = Some("private/buy".to_owned());
        let params = Some(json!({ "amount": 10 }));
        let out = fmt_api_context(&method, &params);
        assert!(out.starts_with(" (method=private/buy, params="));
        assert!(out.contains("\"amount\":10"));
    }

    #[test]
    fn long_params_are_truncated() {
        let big = "x".repeat(10_000);
        let params = Some(json!({ "blob": big }));
        let out = fmt_api_context(&None, &params);
        // Length bounded by MAX_PAYLOAD_DISPLAY_LEN + small constant suffix.
        assert!(out.chars().count() < 1_000);
    }
}
