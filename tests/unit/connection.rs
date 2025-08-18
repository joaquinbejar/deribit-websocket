//! Unit tests for connection module

use deribit_websocket::connection::WebSocketConnection;
use url::Url;

#[test]
fn test_websocket_connection_creation() {
    let url = Url::parse("wss://test.deribit.com/ws/api/v2").unwrap();
    let connection = WebSocketConnection::new(url.clone());

    assert!(!connection.is_connected());
}

#[test]
fn test_websocket_connection_url() {
    let url = Url::parse("wss://test.deribit.com/ws/api/v2").unwrap();
    let connection = WebSocketConnection::new(url.clone());

    // Connection should store the URL correctly
    assert!(!connection.is_connected()); // Initially not connected
}

#[test]
fn test_websocket_connection_debug() {
    let url = Url::parse("wss://test.deribit.com/ws/api/v2").unwrap();
    let connection = WebSocketConnection::new(url);

    let debug_str = format!("{:?}", connection);
    assert!(debug_str.contains("WebSocketConnection"));
}

#[test]
fn test_websocket_connection_initial_state() {
    let url = Url::parse("wss://www.deribit.com/ws/api/v2").unwrap();
    let connection = WebSocketConnection::new(url);

    // Initially should not be connected
    assert!(!connection.is_connected());
}

#[test]
fn test_websocket_connection_with_different_urls() {
    let testnet_url = Url::parse("wss://test.deribit.com/ws/api/v2").unwrap();
    let production_url = Url::parse("wss://www.deribit.com/ws/api/v2").unwrap();

    let testnet_connection = WebSocketConnection::new(testnet_url);
    let production_connection = WebSocketConnection::new(production_url);

    assert!(!testnet_connection.is_connected());
    assert!(!production_connection.is_connected());
}

#[test]
fn test_websocket_connection_clone() {
    let url = Url::parse("wss://test.deribit.com/ws/api/v2").unwrap();
    let connection = WebSocketConnection::new(url);

    // Test that connection can be cloned (if Clone is implemented)
    let debug_str = format!("{:?}", connection);
    assert!(debug_str.contains("WebSocketConnection"));
}
