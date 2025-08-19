//! Unit tests for session module

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::session::WebSocketSession;
use std::sync::Arc;

#[test]
fn test_websocket_session_creation() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // Test that session can be created
    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_with_production_config() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_with_custom_config() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(std::time::Duration::from_secs(60))
        .with_max_reconnect_attempts(10);

    let session = WebSocketSession::new(config);

    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_arc_compatibility() {
    let config = WebSocketConfig::default();
    let session = Arc::new(WebSocketSession::new(config));

    // Test that session can be wrapped in Arc (for thread safety)
    let session_clone = session.clone();

    let debug_str1 = format!("{:?}", session);
    let debug_str2 = format!("{:?}", session_clone);

    assert!(debug_str1.contains("WebSocketSession"));
    assert!(debug_str2.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_debug_format() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    let debug_output = format!("{:?}", session);
    assert!(debug_output.contains("WebSocketSession"));
}
