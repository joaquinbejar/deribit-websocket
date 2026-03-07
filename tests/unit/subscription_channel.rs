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
    match channel {
        SubscriptionChannel::Perpetual {
            instrument,
            interval,
        } => {
            assert_eq!(instrument, "BTC-PERPETUAL");
            assert_eq!(interval, "raw");
        }
        _ => panic!("Expected Perpetual channel"),
    }
}

#[test]
fn test_parse_channel_perpetual_100ms() {
    let channel = SubscriptionChannel::from_string("perpetual.ETH-PERPETUAL.100ms");
    match channel {
        SubscriptionChannel::Perpetual {
            instrument,
            interval,
        } => {
            assert_eq!(instrument, "ETH-PERPETUAL");
            assert_eq!(interval, "100ms");
        }
        _ => panic!("Expected Perpetual channel"),
    }
}

// Test platform state channel parsing
#[test]
fn test_parse_channel_platform_state() {
    let channel = SubscriptionChannel::from_string("platform_state");
    assert!(matches!(channel, SubscriptionChannel::PlatformState));
}

#[test]
fn test_parse_channel_platform_state_public_methods() {
    let channel = SubscriptionChannel::from_string("platform_state.public_methods_state");
    assert!(matches!(
        channel,
        SubscriptionChannel::PlatformStatePublicMethods
    ));
}

// Test instrument state channel parsing
#[test]
fn test_parse_channel_instrument_state() {
    let channel = SubscriptionChannel::from_string("instrument.state.future.BTC");
    match channel {
        SubscriptionChannel::InstrumentState { kind, currency } => {
            assert_eq!(kind, "future");
            assert_eq!(currency, "BTC");
        }
        _ => panic!("Expected InstrumentState channel"),
    }
}

#[test]
fn test_parse_channel_instrument_state_option() {
    let channel = SubscriptionChannel::from_string("instrument.state.option.ETH");
    match channel {
        SubscriptionChannel::InstrumentState { kind, currency } => {
            assert_eq!(kind, "option");
            assert_eq!(currency, "ETH");
        }
        _ => panic!("Expected InstrumentState channel"),
    }
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
    let channel = SubscriptionChannel::Perpetual {
        instrument: "BTC-PERPETUAL".to_string(),
        interval: "raw".to_string(),
    };
    assert_eq!(channel.channel_name(), "perpetual.BTC-PERPETUAL.raw");
}

#[test]
fn test_channel_name_perpetual_100ms() {
    let channel = SubscriptionChannel::Perpetual {
        instrument: "ETH-PERPETUAL".to_string(),
        interval: "100ms".to_string(),
    };
    assert_eq!(channel.channel_name(), "perpetual.ETH-PERPETUAL.100ms");
}

#[test]
fn test_channel_name_platform_state() {
    let channel = SubscriptionChannel::PlatformState;
    assert_eq!(channel.channel_name(), "platform_state");
}

#[test]
fn test_channel_name_platform_state_public_methods() {
    let channel = SubscriptionChannel::PlatformStatePublicMethods;
    assert_eq!(
        channel.channel_name(),
        "platform_state.public_methods_state"
    );
}

#[test]
fn test_channel_name_instrument_state() {
    let channel = SubscriptionChannel::InstrumentState {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
    };
    assert_eq!(channel.channel_name(), "instrument.state.future.BTC");
}

#[test]
fn test_channel_name_instrument_state_option() {
    let channel = SubscriptionChannel::InstrumentState {
        kind: "option".to_string(),
        currency: "ETH".to_string(),
    };
    assert_eq!(channel.channel_name(), "instrument.state.option.ETH");
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

// =============================================================================
// Tests for new subscription channels (Issue #10)
// =============================================================================

// Test grouped order book channel parsing
#[test]
fn test_parse_channel_grouped_orderbook() {
    let channel = SubscriptionChannel::from_string("book.BTC-PERPETUAL.100.10.100ms");
    match channel {
        SubscriptionChannel::GroupedOrderBook {
            instrument,
            group,
            depth,
            interval,
        } => {
            assert_eq!(instrument, "BTC-PERPETUAL");
            assert_eq!(group, "100");
            assert_eq!(depth, "10");
            assert_eq!(interval, "100ms");
        }
        _ => panic!("Expected GroupedOrderBook variant"),
    }
}

#[test]
fn test_parse_channel_grouped_orderbook_agg2() {
    let channel = SubscriptionChannel::from_string("book.ETH-PERPETUAL.10.20.agg2");
    match channel {
        SubscriptionChannel::GroupedOrderBook {
            instrument,
            group,
            depth,
            interval,
        } => {
            assert_eq!(instrument, "ETH-PERPETUAL");
            assert_eq!(group, "10");
            assert_eq!(depth, "20");
            assert_eq!(interval, "agg2");
        }
        _ => panic!("Expected GroupedOrderBook variant"),
    }
}

// Test incremental ticker channel parsing
#[test]
fn test_parse_channel_incremental_ticker() {
    let channel = SubscriptionChannel::from_string("incremental_ticker.BTC-PERPETUAL");
    assert!(
        matches!(channel, SubscriptionChannel::IncrementalTicker(ref inst) if inst == "BTC-PERPETUAL")
    );
}

#[test]
fn test_parse_channel_incremental_ticker_eth() {
    let channel = SubscriptionChannel::from_string("incremental_ticker.ETH-PERPETUAL");
    assert!(
        matches!(channel, SubscriptionChannel::IncrementalTicker(ref inst) if inst == "ETH-PERPETUAL")
    );
}

// Test trades by kind channel parsing
#[test]
fn test_parse_channel_trades_by_kind() {
    let channel = SubscriptionChannel::from_string("trades.future.BTC.raw");
    match channel {
        SubscriptionChannel::TradesByKind {
            kind,
            currency,
            interval,
        } => {
            assert_eq!(kind, "future");
            assert_eq!(currency, "BTC");
            assert_eq!(interval, "raw");
        }
        _ => panic!("Expected TradesByKind variant"),
    }
}

#[test]
fn test_parse_channel_trades_by_kind_option() {
    let channel = SubscriptionChannel::from_string("trades.option.ETH.100ms");
    match channel {
        SubscriptionChannel::TradesByKind {
            kind,
            currency,
            interval,
        } => {
            assert_eq!(kind, "option");
            assert_eq!(currency, "ETH");
            assert_eq!(interval, "100ms");
        }
        _ => panic!("Expected TradesByKind variant"),
    }
}

#[test]
fn test_parse_channel_trades_by_kind_any() {
    let channel = SubscriptionChannel::from_string("trades.any.any.raw");
    match channel {
        SubscriptionChannel::TradesByKind {
            kind,
            currency,
            interval,
        } => {
            assert_eq!(kind, "any");
            assert_eq!(currency, "any");
            assert_eq!(interval, "raw");
        }
        _ => panic!("Expected TradesByKind variant"),
    }
}

// Test price ranking channel parsing
#[test]
fn test_parse_channel_price_ranking() {
    let channel = SubscriptionChannel::from_string("deribit_price_ranking.btc_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceRanking(ref idx) if idx == "btc_usd"));
}

#[test]
fn test_parse_channel_price_ranking_eth() {
    let channel = SubscriptionChannel::from_string("deribit_price_ranking.eth_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceRanking(ref idx) if idx == "eth_usd"));
}

// Test price statistics channel parsing
#[test]
fn test_parse_channel_price_statistics() {
    let channel = SubscriptionChannel::from_string("deribit_price_statistics.btc_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceStatistics(ref idx) if idx == "btc_usd"));
}

#[test]
fn test_parse_channel_price_statistics_eth() {
    let channel = SubscriptionChannel::from_string("deribit_price_statistics.eth_usd");
    assert!(matches!(channel, SubscriptionChannel::PriceStatistics(ref idx) if idx == "eth_usd"));
}

// Test volatility index channel parsing
#[test]
fn test_parse_channel_volatility_index() {
    let channel = SubscriptionChannel::from_string("deribit_volatility_index.btc_usd");
    assert!(matches!(channel, SubscriptionChannel::VolatilityIndex(ref idx) if idx == "btc_usd"));
}

#[test]
fn test_parse_channel_volatility_index_eth() {
    let channel = SubscriptionChannel::from_string("deribit_volatility_index.eth_usd");
    assert!(matches!(channel, SubscriptionChannel::VolatilityIndex(ref idx) if idx == "eth_usd"));
}

// Test channel_name for new variants
#[test]
fn test_channel_name_grouped_orderbook() {
    let channel = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    assert_eq!(channel.channel_name(), "book.BTC-PERPETUAL.100.10.100ms");
}

#[test]
fn test_channel_name_incremental_ticker() {
    let channel = SubscriptionChannel::IncrementalTicker("BTC-PERPETUAL".to_string());
    assert_eq!(channel.channel_name(), "incremental_ticker.BTC-PERPETUAL");
}

#[test]
fn test_channel_name_trades_by_kind() {
    let channel = SubscriptionChannel::TradesByKind {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
        interval: "raw".to_string(),
    };
    assert_eq!(channel.channel_name(), "trades.future.BTC.raw");
}

#[test]
fn test_channel_name_price_ranking() {
    let channel = SubscriptionChannel::PriceRanking("btc_usd".to_string());
    assert_eq!(channel.channel_name(), "deribit_price_ranking.btc_usd");
}

#[test]
fn test_channel_name_price_statistics() {
    let channel = SubscriptionChannel::PriceStatistics("btc_usd".to_string());
    assert_eq!(channel.channel_name(), "deribit_price_statistics.btc_usd");
}

#[test]
fn test_channel_name_volatility_index() {
    let channel = SubscriptionChannel::VolatilityIndex("btc_usd".to_string());
    assert_eq!(channel.channel_name(), "deribit_volatility_index.btc_usd");
}

// Test Display trait for new variants
#[test]
fn test_display_grouped_orderbook() {
    let channel = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    assert_eq!(format!("{}", channel), "book.BTC-PERPETUAL.100.10.100ms");
}

#[test]
fn test_display_incremental_ticker() {
    let channel = SubscriptionChannel::IncrementalTicker("BTC-PERPETUAL".to_string());
    assert_eq!(format!("{}", channel), "incremental_ticker.BTC-PERPETUAL");
}

#[test]
fn test_display_trades_by_kind() {
    let channel = SubscriptionChannel::TradesByKind {
        kind: "option".to_string(),
        currency: "ETH".to_string(),
        interval: "100ms".to_string(),
    };
    assert_eq!(format!("{}", channel), "trades.option.ETH.100ms");
}

// Test is_unknown returns false for new variants
#[test]
fn test_is_unknown_grouped_orderbook() {
    let channel = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    assert!(!channel.is_unknown());
}

#[test]
fn test_is_unknown_incremental_ticker() {
    let channel = SubscriptionChannel::IncrementalTicker("BTC-PERPETUAL".to_string());
    assert!(!channel.is_unknown());
}

#[test]
fn test_is_unknown_trades_by_kind() {
    let channel = SubscriptionChannel::TradesByKind {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
        interval: "raw".to_string(),
    };
    assert!(!channel.is_unknown());
}

#[test]
fn test_is_unknown_price_ranking() {
    let channel = SubscriptionChannel::PriceRanking("btc_usd".to_string());
    assert!(!channel.is_unknown());
}

#[test]
fn test_is_unknown_price_statistics() {
    let channel = SubscriptionChannel::PriceStatistics("btc_usd".to_string());
    assert!(!channel.is_unknown());
}

#[test]
fn test_is_unknown_volatility_index() {
    let channel = SubscriptionChannel::VolatilityIndex("btc_usd".to_string());
    assert!(!channel.is_unknown());
}

// Test equality for new variants
#[test]
fn test_grouped_orderbook_equality() {
    let channel1 = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    let channel2 = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    let channel3 = SubscriptionChannel::GroupedOrderBook {
        instrument: "ETH-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    assert_eq!(channel1, channel2);
    assert_ne!(channel1, channel3);
}

#[test]
fn test_trades_by_kind_equality() {
    let channel1 = SubscriptionChannel::TradesByKind {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
        interval: "raw".to_string(),
    };
    let channel2 = SubscriptionChannel::TradesByKind {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
        interval: "raw".to_string(),
    };
    let channel3 = SubscriptionChannel::TradesByKind {
        kind: "option".to_string(),
        currency: "BTC".to_string(),
        interval: "raw".to_string(),
    };
    assert_eq!(channel1, channel2);
    assert_ne!(channel1, channel3);
}
