//! Unit tests for client module

use deribit_websocket::client::DeribitWebSocketClient;
use deribit_websocket::config::WebSocketConfig;

#[test]
fn test_client_creation() {
    let config = WebSocketConfig::default();
    let result = DeribitWebSocketClient::new(&config);

    assert!(result.is_ok());
}

#[test]
fn test_client_creation_with_custom_config() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(std::time::Duration::from_secs(60))
        .with_max_reconnect_attempts(10);

    let result = DeribitWebSocketClient::new(&config);
    assert!(result.is_ok());
}

#[test]
fn test_client_new_default() {
    let config = WebSocketConfig::default();
    let result = DeribitWebSocketClient::new(&config);
    assert!(result.is_ok());
}

#[test]
fn test_client_new_production() {
    let result = DeribitWebSocketClient::new_production();
    assert!(result.is_ok());
}

#[test]
fn test_client_new_with_url() {
    let result =
        DeribitWebSocketClient::new_with_url("wss://test.deribit.com/ws/api/v2".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_client_new_with_invalid_url() {
    let result = DeribitWebSocketClient::new_with_url("invalid-url".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_initial_connection_state() {
    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).unwrap();

    // Initially should not be connected
    assert!(!client.is_connected().await);
}

#[tokio::test]
async fn test_client_subscription_management() {
    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).unwrap();

    // Initially should have no subscriptions
    let subscriptions = client.get_subscriptions().await;
    assert!(subscriptions.is_empty());
}

#[test]
fn test_client_message_handler_management() {
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config).unwrap();

    // Initially should not have a message handler
    assert!(!client.has_message_handler());

    // Set a message handler
    client.set_message_handler(|_message| Ok(()), |_message, _error| {});

    assert!(client.has_message_handler());

    // Clear the message handler
    client.clear_message_handler();
    assert!(!client.has_message_handler());
}

#[test]
fn test_client_debug() {
    let config = WebSocketConfig::default();
    let client = DeribitWebSocketClient::new(&config).unwrap();

    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("DeribitWebSocketClient"));
}

#[test]
fn test_client_parse_channel_type() {
    // `SubscriptionChannel::from_string` is the sole parser used by the
    // client's subscribe/unsubscribe reconciliation, so pin its behaviour
    // for the channel formats the client ingests most frequently.
    use deribit_websocket::model::SubscriptionChannel;

    assert_eq!(
        SubscriptionChannel::from_string("ticker.BTC-PERPETUAL"),
        SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string())
    );
    assert_eq!(
        SubscriptionChannel::from_string("book.ETH-PERPETUAL.raw"),
        SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string())
    );
    assert_eq!(
        SubscriptionChannel::from_string("trades.BTC-PERPETUAL.raw"),
        SubscriptionChannel::Trades("BTC-PERPETUAL".to_string())
    );
    // Unknown channels round-trip through the catch-all variant.
    assert!(SubscriptionChannel::from_string("totally.unknown.channel").is_unknown());
}

#[test]
fn test_client_extract_instrument() {
    // The canonical round-trip: build a channel from its typed variant and
    // confirm the same string re-parses to the same variant. Keeps the
    // instrument extraction honest against drift in either direction.
    use deribit_websocket::model::SubscriptionChannel;

    for original in [
        SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
        SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string()),
        SubscriptionChannel::Trades("BTC-PERPETUAL".to_string()),
        SubscriptionChannel::Quote("SOL-PERPETUAL".to_string()),
    ] {
        let channel_name = original.channel_name();
        let reparsed = SubscriptionChannel::from_string(&channel_name);
        assert_eq!(
            original, reparsed,
            "channel {} must survive a round-trip through from_string",
            channel_name
        );
    }
}
