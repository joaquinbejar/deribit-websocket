//! WebSocket subscription management

/// Subscription channels
#[derive(Debug, Clone)]
pub enum SubscriptionChannel {
    Ticker(String),
    OrderBook(String),
    Trades(String),
    UserOrders,
    UserTrades,
}

impl SubscriptionChannel {
    /// Convert to channel string
    pub fn to_string(&self) -> String {
        match self {
            Self::Ticker(instrument) => format!("ticker.{instrument}.raw"),
            Self::OrderBook(instrument) => format!("book.{instrument}.raw"),
            Self::Trades(instrument) => format!("trades.{instrument}.raw"),
            Self::UserOrders => "user.orders.any.any.raw".to_string(),
            Self::UserTrades => "user.trades.any.any.raw".to_string(),
        }
    }
}
