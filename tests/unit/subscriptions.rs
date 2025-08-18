//! Unit tests for subscriptions module

use deribit_websocket::model::SubscriptionChannel;
use deribit_websocket::model::subscription::SubscriptionManager;

#[test]
fn test_subscription_manager_creation() {
    let manager = SubscriptionManager::new();

    // Initially should have no subscriptions
    assert!(manager.get_all_channels().is_empty());
}

#[test]
fn test_subscription_manager_add_subscription() {
    let mut manager = SubscriptionManager::new();

    let channel = "ticker.BTC-PERPETUAL".to_string();
    let channel_type = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let instrument = Some("BTC-PERPETUAL".to_string());

    manager.add_subscription(channel.clone(), channel_type, instrument);

    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0], channel);
}

#[test]
fn test_subscription_manager_remove_subscription() {
    let mut manager = SubscriptionManager::new();

    let channel = "ticker.BTC-PERPETUAL".to_string();
    let channel_type = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let instrument = Some("BTC-PERPETUAL".to_string());

    manager.add_subscription(channel.clone(), channel_type, instrument);
    assert_eq!(manager.get_all_channels().len(), 1);

    manager.remove_subscription(&channel);
    assert!(manager.get_all_channels().is_empty());
}

#[test]
fn test_subscription_manager_multiple_subscriptions() {
    let mut manager = SubscriptionManager::new();

    let channels = vec![
        (
            "ticker.BTC-PERPETUAL".to_string(),
            SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
        ),
        (
            "book.ETH-PERPETUAL.100ms".to_string(),
            SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string()),
        ),
        (
            "trades.BTC-PERPETUAL".to_string(),
            SubscriptionChannel::Trades("BTC-PERPETUAL".to_string()),
        ),
    ];

    for (channel, channel_type) in channels {
        let instrument = Some("BTC-PERPETUAL".to_string());
        manager.add_subscription(channel, channel_type, instrument);
    }

    let all_channels = manager.get_all_channels();
    assert_eq!(all_channels.len(), 3);
}

#[test]
fn test_subscription_manager_duplicate_subscription() {
    let mut manager = SubscriptionManager::new();

    let channel = "ticker.BTC-PERPETUAL".to_string();
    let channel_type = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let instrument = Some("BTC-PERPETUAL".to_string());

    // Add the same subscription twice
    manager.add_subscription(channel.clone(), channel_type.clone(), instrument.clone());
    manager.add_subscription(channel.clone(), channel_type, instrument);

    // Should still only have one subscription
    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 1);
}

#[test]
fn test_subscription_manager_remove_nonexistent() {
    let mut manager = SubscriptionManager::new();

    // Try to remove a subscription that doesn't exist
    manager.remove_subscription("nonexistent.channel");

    // Should still be empty
    assert!(manager.get_all_channels().is_empty());
}

#[test]
fn test_subscription_manager_user_channels() {
    let mut manager = SubscriptionManager::new();

    let user_orders = SubscriptionChannel::UserOrders;
    let user_trades = SubscriptionChannel::UserTrades;

    manager.add_subscription("user.orders".to_string(), user_orders, None);
    manager.add_subscription("user.trades".to_string(), user_trades, None);

    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 2);
    assert!(channels.contains(&"user.orders".to_string()));
    assert!(channels.contains(&"user.trades".to_string()));
}

#[test]
fn test_subscription_manager_chart_trades() {
    let mut manager = SubscriptionManager::new();

    let chart_channel = SubscriptionChannel::ChartTrades {
        instrument: "BTC-PERPETUAL".to_string(),
        resolution: "1".to_string(),
    };

    manager.add_subscription(
        "chart.trades.BTC-PERPETUAL.1".to_string(),
        chart_channel,
        Some("BTC-PERPETUAL".to_string()),
    );

    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0], "chart.trades.BTC-PERPETUAL.1");
}

#[test]
fn test_subscription_manager_user_changes() {
    let mut manager = SubscriptionManager::new();

    let user_changes = SubscriptionChannel::UserChanges {
        instrument: "ETH-PERPETUAL".to_string(),
        interval: "raw".to_string(),
    };

    manager.add_subscription(
        "user.changes.ETH-PERPETUAL.raw".to_string(),
        user_changes,
        Some("ETH-PERPETUAL".to_string()),
    );

    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0], "user.changes.ETH-PERPETUAL.raw");
}

#[test]
fn test_subscription_manager_debug() {
    let manager = SubscriptionManager::new();
    let debug_str = format!("{:?}", manager);

    assert!(debug_str.contains("SubscriptionManager"));
}
