//! Unit tests for error module

use deribit_websocket::error::WebSocketError;
use std::error::Error;

#[test]
fn test_websocket_error_display() {
    let error = WebSocketError::ConnectionFailed("Connection timeout".to_string());
    assert_eq!(error.to_string(), "Connection failed: Connection timeout");

    let error = WebSocketError::InvalidMessage("Bad JSON".to_string());
    assert_eq!(error.to_string(), "Invalid message format: Bad JSON");

    let error = WebSocketError::AuthenticationFailed("Invalid credentials".to_string());
    assert_eq!(
        error.to_string(),
        "Authentication failed: Invalid credentials"
    );

    let error = WebSocketError::SubscriptionFailed("Channel not found".to_string());
    assert_eq!(error.to_string(), "Subscription failed: Channel not found");

    let error = WebSocketError::ConnectionClosed;
    assert_eq!(error.to_string(), "Connection closed unexpectedly");

    let error = WebSocketError::HeartbeatTimeout;
    assert_eq!(error.to_string(), "Heartbeat timeout");
}

#[test]
fn test_websocket_error_debug() {
    let error = WebSocketError::ConnectionFailed("Test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ConnectionFailed"));
    assert!(debug_str.contains("Test"));
}

#[test]
fn test_websocket_error_source() {
    let error = WebSocketError::ConnectionFailed("Test".to_string());
    assert!(error.source().is_none());
}

#[test]
fn test_websocket_error_variants() {
    // Test all error variants exist and can be created
    let errors = [
        WebSocketError::ConnectionFailed("test".to_string()),
        WebSocketError::InvalidMessage("test".to_string()),
        WebSocketError::AuthenticationFailed("test".to_string()),
        WebSocketError::SubscriptionFailed("test".to_string()),
        WebSocketError::ConnectionClosed,
        WebSocketError::HeartbeatTimeout,
    ];

    assert_eq!(errors.len(), 6);
}

#[test]
fn test_websocket_error_display_variants() {
    // Test that all variants can be displayed
    let error1 = WebSocketError::ConnectionClosed;
    let error2 = WebSocketError::HeartbeatTimeout;

    assert!(!error1.to_string().is_empty());
    assert!(!error2.to_string().is_empty());

    let error3 = WebSocketError::ConnectionFailed("test".to_string());
    let error4 = WebSocketError::ConnectionFailed("different".to_string());

    assert_ne!(error3.to_string(), error4.to_string());
}

#[test]
fn test_websocket_error_debug_format() {
    let original = WebSocketError::InvalidMessage("test message".to_string());
    let debug_str = format!("{:?}", original);

    assert!(debug_str.contains("InvalidMessage"));
    assert!(debug_str.contains("test message"));
}

#[test]
fn test_websocket_error_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<WebSocketError>();
    assert_sync::<WebSocketError>();
}

#[test]
fn test_websocket_error_from_string() {
    // Test that we can create errors from string messages
    let message = "Connection failed due to network error";
    let error = WebSocketError::ConnectionFailed(message.to_string());

    assert!(error.to_string().contains(message));
}
