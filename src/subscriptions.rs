//! WebSocket subscription management

use std::fmt;

/// Subscription channels
#[derive(Debug, Clone)]
pub enum SubscriptionChannel {
    /// Ticker data for a specific instrument
    Ticker(String),
    /// Order book data for a specific instrument
    OrderBook(String),
    /// Trade data for a specific instrument
    Trades(String),
    /// User's order updates
    UserOrders,
    /// User's trade updates
    UserTrades,
}

impl fmt::Display for SubscriptionChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let channel_str = match self {
            Self::Ticker(instrument) => format!("ticker.{instrument}.raw"),
            Self::OrderBook(instrument) => format!("book.{instrument}.raw"),
            Self::Trades(instrument) => format!("trades.{instrument}.raw"),
            Self::UserOrders => "user.orders.any.any.raw".to_string(),
            Self::UserTrades => "user.trades.any.any.raw".to_string(),
        };
        write!(f, "{channel_str}")
    }
}
