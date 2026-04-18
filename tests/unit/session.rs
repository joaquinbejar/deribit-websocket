//! Unit tests for session module

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::model::subscription::SubscriptionManager;
use deribit_websocket::session::WebSocketSession;
use std::sync::Arc;
use tokio::sync::Mutex;

fn make_subscription_manager() -> Arc<Mutex<SubscriptionManager>> {
    Arc::new(Mutex::new(SubscriptionManager::new()))
}

#[test]
fn test_websocket_session_creation() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    // Test that session can be created
    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_with_production_config() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_with_custom_config() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(std::time::Duration::from_secs(60))
        .with_max_reconnect_attempts(10);

    let session = WebSocketSession::new(config, make_subscription_manager());

    let debug_str = format!("{:?}", session);
    assert!(debug_str.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_arc_compatibility() {
    let config = WebSocketConfig::default();
    let session = Arc::new(WebSocketSession::new(config, make_subscription_manager()));

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
    let session = WebSocketSession::new(config, make_subscription_manager());

    let debug_output = format!("{:?}", session);
    assert!(debug_output.contains("WebSocketSession"));
}

#[test]
fn test_websocket_session_config_access() {
    let config =
        WebSocketConfig::default().with_heartbeat_interval(std::time::Duration::from_secs(45));

    let session = WebSocketSession::new(config, make_subscription_manager());

    // Test that config can be accessed
    let config_ref = session.config();
    assert!(format!("{:?}", config_ref).contains("WebSocketConfig"));
}

#[test]
fn test_websocket_session_subscription_manager() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    // Test that subscription manager can be accessed
    let manager = session.subscription_manager();
    assert!(format!("{:?}", manager).contains("Mutex"));
}

#[tokio::test]
async fn test_websocket_session_state() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    // Initial state should be Disconnected
    let state = session.state().await;
    assert!(format!("{:?}", state).contains("Disconnected"));
}

#[tokio::test]
async fn test_websocket_session_set_state() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    // Set state to Connected
    session.set_state(ConnectionState::Connected).await;
    let state = session.state().await;
    assert!(matches!(state, ConnectionState::Connected));
}

#[tokio::test]
async fn test_websocket_session_is_connected_false() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    // Initial state is Disconnected, so is_connected should be false
    assert!(!session.is_connected().await);
}

#[tokio::test]
async fn test_websocket_session_is_connected_true() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    session.set_state(ConnectionState::Connected).await;
    assert!(session.is_connected().await);
}

#[tokio::test]
async fn test_websocket_session_is_authenticated_false() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    assert!(!session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_is_authenticated_true() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    session.set_state(ConnectionState::Authenticated).await;
    assert!(session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_mark_authenticated() {
    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

    session.mark_authenticated().await;
    assert!(session.is_authenticated().await);
}

#[tokio::test]
async fn test_websocket_session_mark_disconnected() {
    use deribit_websocket::model::ConnectionState;

    let config = WebSocketConfig::default();
    let session = WebSocketSession::new(config, make_subscription_manager());

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
    let session = WebSocketSession::new(config, make_subscription_manager());

    // This should not panic
    session.reactivate_subscriptions().await;
}

// Regression tests for #43: client and session must share the same
// SubscriptionManager so that subscription state stays consistent on
// reconnect.

#[test]
fn test_client_and_session_share_subscription_manager() {
    use deribit_websocket::client::DeribitWebSocketClient;

    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).expect("client construction must succeed");

    let client_handle = client.subscription_manager();
    let session_handle = client.session.subscription_manager();

    assert!(
        Arc::ptr_eq(&client_handle, &session_handle),
        "client and session must share the same SubscriptionManager allocation"
    );
}

#[tokio::test]
async fn test_client_mutation_visible_via_session() {
    use deribit_websocket::client::DeribitWebSocketClient;
    use deribit_websocket::model::SubscriptionChannel;

    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).expect("client construction must succeed");

    {
        let client_handle = client.subscription_manager();
        let mut guard = client_handle.lock().await;
        guard.add_subscription(
            "ticker.BTC-PERPETUAL.100ms".to_string(),
            SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
            Some("BTC-PERPETUAL".to_string()),
        );
    }

    let session_handle = client.session.subscription_manager();
    let session_guard = session_handle.lock().await;
    assert!(
        session_guard
            .get_subscription("ticker.BTC-PERPETUAL.100ms")
            .is_some(),
        "subscription added via client handle must be visible via session handle"
    );
}

#[tokio::test]
async fn test_session_mark_disconnected_clears_client_view() {
    use deribit_websocket::client::DeribitWebSocketClient;
    use deribit_websocket::model::SubscriptionChannel;

    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).expect("client construction must succeed");

    {
        let client_handle = client.subscription_manager();
        let mut guard = client_handle.lock().await;
        guard.add_subscription(
            "ticker.BTC-PERPETUAL.100ms".to_string(),
            SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
            Some("BTC-PERPETUAL".to_string()),
        );
    }

    client.session.mark_disconnected().await;

    let client_handle = client.subscription_manager();
    let guard = client_handle.lock().await;
    assert!(
        guard.get_all_channels().is_empty(),
        "session.mark_disconnected() must clear the client's view of subscriptions"
    );
}

#[tokio::test]
async fn test_session_reactivate_restores_client_subscriptions() {
    use deribit_websocket::client::DeribitWebSocketClient;
    use deribit_websocket::model::SubscriptionChannel;

    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).expect("client construction must succeed");

    let channel = "ticker.BTC-PERPETUAL.100ms";

    {
        let client_handle = client.subscription_manager();
        let mut guard = client_handle.lock().await;
        guard.add_subscription(
            channel.to_string(),
            SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
            Some("BTC-PERPETUAL".to_string()),
        );
        guard.deactivate_subscription(channel);
        let sub = guard
            .get_subscription(channel)
            .expect("subscription must exist after add_subscription");
        assert!(
            !sub.active,
            "subscription must be inactive after deactivate_subscription"
        );
    }

    client.session.reactivate_subscriptions().await;

    let client_handle = client.subscription_manager();
    let guard = client_handle.lock().await;
    let sub = guard
        .get_subscription(channel)
        .expect("subscription must still exist via client handle after reactivate");
    assert!(
        sub.active,
        "session.reactivate_subscriptions() must restore active=true in the client's view"
    );
}
