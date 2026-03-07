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

#[test]
fn test_subscription_manager_clear() {
    let mut manager = SubscriptionManager::new();

    // Add multiple subscriptions
    manager.add_subscription(
        "ticker.BTC-PERPETUAL".to_string(),
        SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
        Some("BTC-PERPETUAL".to_string()),
    );
    manager.add_subscription(
        "book.ETH-PERPETUAL.100ms".to_string(),
        SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string()),
        Some("ETH-PERPETUAL".to_string()),
    );
    manager.add_subscription(
        "user.orders".to_string(),
        SubscriptionChannel::UserOrders,
        None,
    );

    assert_eq!(manager.get_all_channels().len(), 3);

    // Clear all subscriptions
    manager.clear();

    assert!(manager.get_all_channels().is_empty());
    assert!(manager.active_subscriptions().is_empty());
}

#[test]
fn test_subscription_manager_clear_empty() {
    let mut manager = SubscriptionManager::new();

    // Clear an already empty manager should not panic
    manager.clear();

    assert!(manager.get_all_channels().is_empty());
}

// Request builder tests for unsubscribe_all

use deribit_websocket::prelude::RequestBuilder;

#[test]
fn test_request_builder_public_unsubscribe_all() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_public_unsubscribe_all_request();

    assert_eq!(request.method, "public/unsubscribe_all");
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_private_unsubscribe_all() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_private_unsubscribe_all_request();

    assert_eq!(request.method, "private/unsubscribe_all");
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_unsubscribe_all_incremental_ids() {
    let mut builder = RequestBuilder::new();

    let r1 = builder.build_public_unsubscribe_all_request();
    let r2 = builder.build_private_unsubscribe_all_request();

    assert_eq!(r1.id, serde_json::json!(1));
    assert_eq!(r2.id, serde_json::json!(2));
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_ticker() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    let channels = manager.get_all_channels();
    assert_eq!(channels.len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_orderbook() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_trades() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Trades("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_incremental_ticker() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::IncrementalTicker("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_chart_trades() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::ChartTrades {
        instrument: "BTC-PERPETUAL".to_string(),
        resolution: "1".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_grouped_orderbook() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "5".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_changes() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserChanges {
        instrument: "BTC-PERPETUAL".to_string(),
        interval: "raw".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_trades_by_kind() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::TradesByKind {
        currency: "BTC".to_string(),
        kind: "future".to_string(),
        interval: "raw".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_price_index() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::PriceIndex("btc_usd".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_price_ranking() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::PriceRanking("btc_usd".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_price_statistics() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::PriceStatistics("btc_usd".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_volatility_index() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::VolatilityIndex("btc_usd".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_estimated_expiration() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::EstimatedExpirationPrice("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_mark_price() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::MarkPrice("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_funding() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Funding("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_quote() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Quote("BTC-PERPETUAL".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_perpetual() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Perpetual {
        instrument: "BTC-PERPETUAL".to_string(),
        interval: "100ms".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_instrument_state() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::InstrumentState {
        currency: "BTC".to_string(),
        kind: "future".to_string(),
    };
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_block_rfq() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::BlockRfqTrades("BTC".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_block_trade_confirmations_currency() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::BlockTradeConfirmationsByCurrency("BTC".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_mmp_trigger() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserMmpTrigger("btc_usd".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_orders() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserOrders;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_trades() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserTrades;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_portfolio() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserPortfolio;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_platform_state() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::PlatformState;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_platform_state_public() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::PlatformStatePublicMethods;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_block_trade_confirmations() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::BlockTradeConfirmations;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_access_log() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserAccessLog;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_user_lock() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::UserLock;
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_add_subscription_from_channel_unknown() {
    let mut manager = SubscriptionManager::new();
    
    let channel = SubscriptionChannel::Unknown("custom.channel".to_string());
    manager.add_subscription_from_channel(channel);
    
    assert_eq!(manager.get_all_channels().len(), 1);
}

#[test]
fn test_subscription_manager_get_subscription() {
    let mut manager = SubscriptionManager::new();
    
    let channel = "ticker.BTC-PERPETUAL".to_string();
    let channel_type = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    manager.add_subscription(channel.clone(), channel_type, Some("BTC-PERPETUAL".to_string()));
    
    let subscription = manager.get_subscription(&channel);
    assert!(subscription.is_some());
    assert_eq!(subscription.unwrap().channel, channel);
}

#[test]
fn test_subscription_manager_get_subscription_not_found() {
    let manager = SubscriptionManager::new();
    
    let subscription = manager.get_subscription("nonexistent");
    assert!(subscription.is_none());
}

#[test]
fn test_subscription_manager_deactivate_subscription() {
    let mut manager = SubscriptionManager::new();
    
    let channel = "ticker.BTC-PERPETUAL".to_string();
    let channel_type = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    manager.add_subscription(channel.clone(), channel_type, Some("BTC-PERPETUAL".to_string()));
    
    // Should be active by default
    assert_eq!(manager.active_subscriptions().len(), 1);
    
    manager.deactivate_subscription(&channel);
    
    // Should now be inactive
    assert_eq!(manager.active_subscriptions().len(), 0);
}

#[test]
fn test_subscription_manager_deactivate_nonexistent() {
    let mut manager = SubscriptionManager::new();
    
    // Should not panic
    manager.deactivate_subscription("nonexistent");
}

#[test]
fn test_subscription_manager_reactivate_all() {
    let mut manager = SubscriptionManager::new();
    
    // Add and deactivate subscriptions
    let channel1 = "ticker.BTC-PERPETUAL".to_string();
    let channel2 = "book.ETH-PERPETUAL.100ms".to_string();
    
    manager.add_subscription(channel1.clone(), SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()), None);
    manager.add_subscription(channel2.clone(), SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string()), None);
    
    manager.deactivate_subscription(&channel1);
    manager.deactivate_subscription(&channel2);
    
    assert_eq!(manager.active_subscriptions().len(), 0);
    
    manager.reactivate_all();
    
    assert_eq!(manager.active_subscriptions().len(), 2);
}

#[test]
fn test_subscription_manager_get_active_channels() {
    let mut manager = SubscriptionManager::new();
    
    let channel1 = "ticker.BTC-PERPETUAL".to_string();
    let channel2 = "book.ETH-PERPETUAL.100ms".to_string();
    
    manager.add_subscription(channel1.clone(), SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()), None);
    manager.add_subscription(channel2.clone(), SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string()), None);
    
    manager.deactivate_subscription(&channel1);
    
    let active_channels = manager.get_active_channels();
    assert_eq!(active_channels.len(), 1);
    assert!(active_channels.contains(&channel2));
}

#[test]
fn test_subscription_manager_active_subscriptions() {
    let mut manager = SubscriptionManager::new();
    
    manager.add_subscription(
        "ticker.BTC-PERPETUAL".to_string(),
        SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string()),
        Some("BTC-PERPETUAL".to_string()),
    );
    
    let active = manager.active_subscriptions();
    assert_eq!(active.len(), 1);
    assert!(active[0].active);
}
