//! Unit tests for config module

use deribit_websocket::config::WebSocketConfig;
use std::time::Duration;

#[test]
fn test_default_config() {
    let config = WebSocketConfig::default();

    assert_eq!(config.ws_url.as_str(), "wss://test.deribit.com/ws/api/v2");
    assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    assert_eq!(config.max_reconnect_attempts, 5);
    assert_eq!(config.reconnect_delay, Duration::from_millis(1000));
}


#[test]
fn test_custom_url_config() {
    let custom_url = "wss://custom.example.com/ws";
    let config = WebSocketConfig::with_url(custom_url).unwrap();

    assert_eq!(config.ws_url.as_str(), custom_url);
}

#[test]
fn test_invalid_url_config() {
    let invalid_url = "not-a-valid-url";
    let result = WebSocketConfig::with_url(invalid_url);

    assert!(result.is_err());
}

#[test]
fn test_config_builder_pattern() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(Duration::from_secs(60))
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(Duration::from_millis(2000));

    assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
    assert_eq!(config.max_reconnect_attempts, 10);
    assert_eq!(config.reconnect_delay, Duration::from_millis(2000));
}

#[test]
fn test_config_chaining() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(Duration::from_secs(45))
        .with_max_reconnect_attempts(3)
        .with_reconnect_delay(Duration::from_millis(500));

    assert_eq!(config.ws_url.as_str(), "wss://www.deribit.com/ws/api/v2");
    assert_eq!(config.heartbeat_interval, Duration::from_secs(45));
    assert_eq!(config.max_reconnect_attempts, 3);
    assert_eq!(config.reconnect_delay, Duration::from_millis(500));
}

#[test]
fn test_config_clone() {
    let original = WebSocketConfig::default().with_heartbeat_interval(Duration::from_secs(120));

    let cloned = original.clone();

    assert_eq!(original.ws_url, cloned.ws_url);
    assert_eq!(original.heartbeat_interval, cloned.heartbeat_interval);
    assert_eq!(
        original.max_reconnect_attempts,
        cloned.max_reconnect_attempts
    );
    assert_eq!(original.reconnect_delay, cloned.reconnect_delay);
}

#[test]
fn test_config_debug() {
    let config = WebSocketConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("WebSocketConfig"));
    assert!(debug_str.contains("ws_url"));
    assert!(debug_str.contains("heartbeat_interval"));
}
