//! Unit tests for subscriptions.rs (legacy SubscriptionChannel)

#[allow(deprecated)]
use deribit_websocket::subscriptions::SubscriptionChannel;

#[test]
#[allow(deprecated)]
fn test_subscription_channel_ticker_display() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    assert_eq!(format!("{}", channel), "ticker.BTC-PERPETUAL.raw");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_orderbook_display() {
    let channel = SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string());
    assert_eq!(format!("{}", channel), "book.ETH-PERPETUAL.raw");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_trades_display() {
    let channel = SubscriptionChannel::Trades("BTC-PERPETUAL".to_string());
    assert_eq!(format!("{}", channel), "trades.BTC-PERPETUAL.raw");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_user_orders_display() {
    let channel = SubscriptionChannel::UserOrders;
    assert_eq!(format!("{}", channel), "user.orders.any.any.raw");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_user_trades_display() {
    let channel = SubscriptionChannel::UserTrades;
    assert_eq!(format!("{}", channel), "user.trades.any.any.raw");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_unknown_display() {
    let channel = SubscriptionChannel::Unknown("custom.channel".to_string());
    assert_eq!(format!("{}", channel), "custom.channel");
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_debug() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let debug = format!("{:?}", channel);
    assert!(debug.contains("Ticker"));
    assert!(debug.contains("BTC-PERPETUAL"));
}

#[test]
#[allow(deprecated)]
fn test_subscription_channel_clone() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let cloned = channel.clone();
    assert_eq!(format!("{}", cloned), "ticker.BTC-PERPETUAL.raw");
}
