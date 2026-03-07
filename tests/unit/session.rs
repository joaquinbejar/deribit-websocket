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

#[test]
fn test_websocket_session_config_access() {
    let config =
        WebSocketConfig::default().with_heartbeat_interval(std::time::Duration::from_secs(45));

    let session = WebSocketSession::new(config);

    // Test that config can be accessed
    let config_ref = session.config();
    assert!(format!("{:?}", config_ref).contains("WebSocketConfig"));
}

#[test]
fn test_websocket_session_subscription_manager() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // Test that subscription manager can be accessed
    let manager = session.subscription_manager();
    assert!(format!("{:?}", manager).contains("Mutex"));
}

#[tokio::test]
async fn test_websocket_session_state() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // Initial state should be Disconnected
    let state = session.state().await;
    assert!(format!("{:?}", state).contains("Disconnected"));
}

#[tokio::test]
async fn test_websocket_session_set_state() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // Set state to Connected
    session.set_state(ConnectionState::Connected).await;
    let state = session.state().await;
    assert!(matches!(state, ConnectionState::Connected));
}

#[tokio::test]
async fn test_websocket_session_is_connected_false() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // Initial state is Disconnected, so is_connected should be false
    assert!(!session.is_connected().await);
}

#[tokio::test]
async fn test_websocket_session_is_connected_true() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    session.set_state(ConnectionState::Connected).await;
    assert!(session.is_connected().await);
}

#[tokio::test]
async fn test_websocket_session_is_authenticated_false() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    assert!(!session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_is_authenticated_true() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    session.set_state(ConnectionState::Authenticated).await;
    assert!(session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_mark_authenticated() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    session.mark_authenticated().await;
    assert!(session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_mark_disconnected() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // First connect and authenticate
    session.set_state(ConnectionState::Authenticated).await;
    assert!(session.is_authenticated().await);

    // Then disconnect
    session.mark_disconnected().await;
    assert!(!session.is_connected().await);
    assert!(!session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_reactivate_subscriptions() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config);

    // This should not panic
    session.reactivate_subscriptions().await;
}
