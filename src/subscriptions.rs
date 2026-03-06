//! WebSocket subscription management

use std::fmt;

/// Subscription channels (legacy enum - use `model::SubscriptionChannel` for full support)
#[derive(Debug, Clone)]
#[deprecated(
    since = "0.2.0",
    note = "Use model::SubscriptionChannel instead which supports all channel types"
)]
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
    /// Unknown or unrecognized channel
    Unknown(String),
}

#[allow(deprecated)]
impl fmt::Display for SubscriptionChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let channel_str = match self {
            Self::Ticker(instrument) => format!("ticker.{instrument}.raw"),
            Self::OrderBook(instrument) => format!("book.{instrument}.raw"),
            Self::Trades(instrument) => format!("trades.{instrument}.raw"),
            Self::UserOrders => "user.orders.any.any.raw".to_string(),
            Self::UserTrades => "user.trades.any.any.raw".to_string(),
            Self::Unknown(channel) => channel.clone(),
        };
        write!(f, "{channel_str}")
    }
}
