//! Unit tests for SubscriptionChannel parsing

use deribit_websocket::prelude::*;

// Test ticker channel parsing
#[test]
fn test_parse_channel_ticker() {
    let channel = SubscriptionChannel::from_string("ticker.BTC-PERPETUAL");
    assert!(matches!(channel, SubscriptionChannel::Ticker(ref inst) if inst == "BTC-PERPETUAL"));
}

#[test]
fn test_parse_channel_ticker_with_interval() {
    let channel = SubscriptionChannel::from_string("ticker.BTC-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::Ticker(ref inst) if inst == "BTC-PERPETUAL"));
}

// Test order book channel parsing
#[test]
fn test_parse_channel_orderbook() {
    let channel = SubscriptionChannel::from_string("book.ETH-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::OrderBook(ref inst) if inst == "ETH-PERPETUAL"));
}

#[test]
fn test_parse_channel_orderbook_with_depth() {
    let channel = SubscriptionChannel::from_string("book.BTC-PERPETUAL.100.100ms");
    assert!(matches!(channel, SubscriptionChannel::OrderBook(ref inst) if inst == "BTC-PERPETUAL"));
}

// Test trades channel parsing
#[test]
fn test_parse_channel_trades() {
    let channel = SubscriptionChannel::from_string("trades.BTC-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::Trades(ref inst) if inst == "BTC-PERPETUAL"));
}

#[test]
fn test_parse_channel_trades_with_interval() {
    let channel = SubscriptionChannel::from_string("trades.ETH-PERPETUAL.100ms");
    assert!(matches!(channel, SubscriptionChannel::Trades(ref inst) if inst == "ETH-PERPETUAL"));
}

// Test chart trades channel parsing
#[test]
fn test_parse_channel_chart_trades() {
    let channel = SubscriptionChannel::from_string("chart.trades.BTC-PERPETUAL.1");
    match channel {
        SubscriptionChannel::ChartTrades {
            instrument,
            resolution,
        } => {
            assert_eq!(instrument, "BTC-PERPETUAL");
            assert_eq!(resolution, "1");
        }
        _ => panic!("Expected ChartTrades variant"),
    }
}

#[test]
fn test_parse_channel_chart_trades_60min() {
    let channel = SubscriptionChannel::from_string("chart.trades.ETH-PERPETUAL.60");
    match channel {
        SubscriptionChannel::ChartTrades {
            instrument,
            resolution,
        } => {
            assert_eq!(instrument, "ETH-PERPETUAL");
            assert_eq!(resolution, "60");
        }
        _ => panic!("Expected ChartTrades variant"),
    }
}

// Test user orders channel parsing
#[test]
fn test_parse_channel_user_orders() {
    let channel = SubscriptionChannel::from_string("user.orders.any.any.raw");
    assert!(matches!(channel, SubscriptionChannel::UserOrders));
}

#[test]
fn test_parse_channel_user_orders_specific() {
    let channel = SubscriptionChannel::from_string("user.orders.BTC-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::UserOrders));
}

// Test user trades channel parsing
#[test]
fn test_parse_channel_user_trades() {
    let channel = SubscriptionChannel::from_string("user.trades.any.any.raw");
    assert!(matches!(channel, SubscriptionChannel::UserTrades));
}

#[test]
fn test_parse_channel_user_trades_specific() {
    let channel = SubscriptionChannel::from_string("user.trades.BTC-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::UserTrades));
}

// Test user portfolio channel parsing
#[test]
fn test_parse_channel_user_portfolio() {
    let channel = SubscriptionChannel::from_string("user.portfolio.any");
    assert!(matches!(channel, SubscriptionChannel::UserPortfolio));
}

#[test]
fn test_parse_channel_user_portfolio_btc() {
    let channel = SubscriptionChannel::from_string("user.portfolio.BTC");
    assert!(matches!(channel, SubscriptionChannel::UserPortfolio));
}

// Test user changes channel parsing
#[test]
fn test_parse_channel_user_changes() {
    let channel = SubscriptionChannel::from_string("user.changes.BTC-PERPETUAL.raw");
    match channel {
        SubscriptionChannel::UserChanges {
            instrument,
            interval,
        } => {
            assert_eq!(instrument, "BTC-PERPETUAL");
            assert_eq!(interval, "raw");
        }
        _ => panic!("Expected UserChanges variant"),
    }
}

#[test]
fn test_parse_channel_user_changes_100ms() {
    let channel = SubscriptionChannel::from_string("user.changes.ETH-PERPETUAL.100ms");
    match channel {
        SubscriptionChannel::UserChanges {
            instrument,
            interval,
        } => {
            assert_eq!(instrument, "ETH-PERPETUAL");
            assert_eq!(interval, "100ms");
        }
        _ => panic!("Expected UserChanges variant"),
    }
}

// Test price index channel parsing
#[test]
fn test_parse_channel_price_index() {
    let channel = SubscriptionChannel::from_string("deribit_price_index.btc_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceIndex(ref curr) if curr == "BTC"));
}

#[test]
fn test_parse_channel_price_index_eth() {
    let channel = SubscriptionChannel::from_string("deribit_price_index.eth_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceIndex(ref curr) if curr == "ETH"));
}

// Test estimated expiration price channel parsing
#[test]
fn test_parse_channel_estimated_expiration_price() {
    let channel = SubscriptionChannel::from_string("estimated_expiration_price.BTC-PERPETUAL");
    assert!(matches!(
        channel,
        SubscriptionChannel::EstimatedExpirationPrice(ref inst) if inst == "BTC-PERPETUAL"
    ));
}

// Test mark price channel parsing
#[test]
fn test_parse_channel_mark_price() {
    let channel = SubscriptionChannel::from_string("markprice.options.BTC");
    assert!(matches!(channel, SubscriptionChannel::MarkPrice(ref inst) if inst == "BTC"));
}

// Test perpetual channel parsing
#[test]
fn test_parse_channel_perpetual() {
    let channel = SubscriptionChannel::from_string("perpetual.BTC-PERPETUAL.raw");
    assert!(matches!(channel, SubscriptionChannel::Perpetual(ref inst) if inst == "BTC-PERPETUAL"));
}

// Test quote channel parsing
#[test]
fn test_parse_channel_quote() {
    let channel = SubscriptionChannel::from_string("quote.BTC-PERPETUAL");
    assert!(matches!(channel, SubscriptionChannel::Quote(ref inst) if inst == "BTC-PERPETUAL"));
}

// Test unknown channel parsing
#[test]
fn test_parse_channel_unknown() {
    let channel = SubscriptionChannel::from_string("unknown.channel.type");
    assert!(
        matches!(channel, SubscriptionChannel::Unknown(ref ch) if ch == "unknown.channel.type")
    );
}

#[test]
fn test_parse_channel_unknown_empty() {
    let channel = SubscriptionChannel::from_string("");
    assert!(matches!(channel, SubscriptionChannel::Unknown(ref ch) if ch.is_empty()));
}

#[test]
fn test_parse_channel_unknown_single_word() {
    let channel = SubscriptionChannel::from_string("foobar");
    assert!(matches!(channel, SubscriptionChannel::Unknown(ref ch) if ch == "foobar"));
}

// Test is_unknown helper
#[test]
fn test_is_unknown_true() {
    let channel = SubscriptionChannel::from_string("not.a.valid.channel");
    assert!(channel.is_unknown());
}

#[test]
fn test_is_unknown_false() {
    let channel = SubscriptionChannel::from_string("ticker.BTC-PERPETUAL");
    assert!(!channel.is_unknown());
}

// Test channel_name roundtrip
#[test]
fn test_channel_name_ticker() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "ticker.BTC-PERPETUAL");
}

#[test]
fn test_channel_name_orderbook() {
    let channel = SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "book.ETH-PERPETUAL.raw");
}

#[test]
fn test_channel_name_trades() {
    let channel = SubscriptionChannel::Trades("BTC-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "trades.BTC-PERPETUAL.raw");
}

#[test]
fn test_channel_name_chart_trades() {
    let channel = SubscriptionChannel::ChartTrades {
        instrument: "BTC-PERPETUAL".to_string(),
        resolution: "15".to_string(),
    };
    assert_eq!(channel.channel_name(), "chart.trades.BTC-PERPETUAL.15");
}

#[test]
fn test_channel_name_user_orders() {
    let channel = SubscriptionChannel::UserOrders;
    assert_eq!(channel.channel_name(), "user.orders.any.any.raw");
}

#[test]
fn test_channel_name_user_trades() {
    let channel = SubscriptionChannel::UserTrades;
    assert_eq!(channel.channel_name(), "user.trades.any.any.raw");
}

#[test]
fn test_channel_name_user_portfolio() {
    let channel = SubscriptionChannel::UserPortfolio;
    assert_eq!(channel.channel_name(), "user.portfolio.any");
}

#[test]
fn test_channel_name_user_changes() {
    let channel = SubscriptionChannel::UserChanges {
        instrument: "BTC-PERPETUAL".to_string(),
        interval: "raw".to_string(),
    };
    assert_eq!(channel.channel_name(), "user.changes.BTC-PERPETUAL.raw");
}

#[test]
fn test_channel_name_price_index() {
    let channel = SubscriptionChannel::PriceIndex("BTC".to_string());
    assert_eq!(channel.channel_name(), "deribit_price_index.btc_usd");
}

#[test]
fn test_channel_name_estimated_expiration_price() {
    let channel = SubscriptionChannel::EstimatedExpirationPrice("BTC-PERPETUAL".to_string());
    assert_eq!(
        channel.channel_name(),
        "estimated_expiration_price.BTC-PERPETUAL"
    );
}

#[test]
fn test_channel_name_mark_price() {
    let channel = SubscriptionChannel::MarkPrice("BTC".to_string());
    assert_eq!(channel.channel_name(), "markprice.options.BTC");
}

#[test]
fn test_channel_name_perpetual() {
    let channel = SubscriptionChannel::Perpetual("BTC-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "perpetual.BTC-PERPETUAL.raw");
}

#[test]
fn test_channel_name_quote() {
    let channel = SubscriptionChannel::Quote("BTC-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "quote.BTC-PERPETUAL");
}

#[test]
fn test_channel_name_unknown() {
    let channel = SubscriptionChannel::Unknown("custom.channel".to_string());
    assert_eq!(channel.channel_name(), "custom.channel");
}

// Test Display trait
#[test]
fn test_display_ticker() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    assert_eq!(format!("{}", channel), "ticker.BTC-PERPETUAL");
}

#[test]
fn test_display_unknown() {
    let channel = SubscriptionChannel::Unknown("my.custom.channel".to_string());
    assert_eq!(format!("{}", channel), "my.custom.channel");
}

// Test equality
#[test]
fn test_subscription_channel_equality() {
    let channel1 = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let channel2 = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let channel3 = SubscriptionChannel::Ticker("ETH-PERPETUAL".to_string());

    assert_eq!(channel1, channel2);
    assert_ne!(channel1, channel3);
}

#[test]
fn test_subscription_channel_clone() {
    let channel = SubscriptionChannel::ChartTrades {
        instrument: "BTC-PERPETUAL".to_string(),
        resolution: "1".to_string(),
    };
    let cloned = channel.clone();
    assert_eq!(channel, cloned);
}
