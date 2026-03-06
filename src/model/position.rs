//! Position management model definitions for Deribit WebSocket API
//!
//! This module provides types for position management operations including
//! closing positions and moving positions between subaccounts.

use serde::{Deserialize, Serialize};

/// Trade information from a close_position response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseTrade {
    /// Trade sequence number
    #[serde(default)]
    pub trade_seq: Option<u64>,
    /// Unique trade identifier
    #[serde(default)]
    pub trade_id: Option<String>,
    /// Trade timestamp in milliseconds
    #[serde(default)]
    pub timestamp: Option<u64>,
    /// Tick direction (0, 1, 2, 3)
    #[serde(default)]
    pub tick_direction: Option<i32>,
    /// Trade state (filled, etc.)
    #[serde(default)]
    pub state: Option<String>,
    /// Whether this was a reduce-only trade
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Trade price
    #[serde(default)]
    pub price: Option<f64>,
    /// Whether this was a post-only order
    #[serde(default)]
    pub post_only: Option<bool>,
    /// Order type (limit, market)
    #[serde(default)]
    pub order_type: Option<String>,
    /// Order ID
    #[serde(default)]
    pub order_id: Option<String>,
    /// Matching ID
    #[serde(default)]
    pub matching_id: Option<String>,
    /// Mark price at time of trade
    #[serde(default)]
    pub mark_price: Option<f64>,
    /// Liquidity type (T = taker, M = maker)
    #[serde(default)]
    pub liquidity: Option<String>,
    /// Instrument name
    #[serde(default)]
    pub instrument_name: Option<String>,
    /// Index price at time of trade
    #[serde(default)]
    pub index_price: Option<f64>,
    /// Fee currency
    #[serde(default)]
    pub fee_currency: Option<String>,
    /// Fee amount
    #[serde(default)]
    pub fee: Option<f64>,
    /// Trade direction (buy/sell)
    #[serde(default)]
    pub direction: Option<String>,
    /// Trade amount
    #[serde(default)]
    pub amount: Option<f64>,
}

/// Order information from a close_position response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseOrder {
    /// Whether the order was created via web
    #[serde(default)]
    pub web: Option<bool>,
    /// Time in force
    #[serde(default)]
    pub time_in_force: Option<String>,
    /// Whether the order was replaced
    #[serde(default)]
    pub replaced: Option<bool>,
    /// Whether this is a reduce-only order
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Order price
    #[serde(default)]
    pub price: Option<f64>,
    /// Whether this is a post-only order
    #[serde(default)]
    pub post_only: Option<bool>,
    /// Order type (limit, market)
    #[serde(default)]
    pub order_type: Option<String>,
    /// Order state (open, filled, cancelled)
    #[serde(default)]
    pub order_state: Option<String>,
    /// Order ID
    #[serde(default)]
    pub order_id: Option<String>,
    /// Maximum display amount
    #[serde(default)]
    pub max_show: Option<f64>,
    /// Last update timestamp
    #[serde(default)]
    pub last_update_timestamp: Option<u64>,
    /// Order label
    #[serde(default)]
    pub label: Option<String>,
    /// Whether this is a rebalance order
    #[serde(default)]
    pub is_rebalance: Option<bool>,
    /// Whether this is a liquidation order
    #[serde(default)]
    pub is_liquidation: Option<bool>,
    /// Instrument name
    #[serde(default)]
    pub instrument_name: Option<String>,
    /// Filled amount
    #[serde(default)]
    pub filled_amount: Option<f64>,
    /// Order direction (buy/sell)
    #[serde(default)]
    pub direction: Option<String>,
    /// Creation timestamp
    #[serde(default)]
    pub creation_timestamp: Option<u64>,
    /// Average fill price
    #[serde(default)]
    pub average_price: Option<f64>,
    /// Whether created via API
    #[serde(default)]
    pub api: Option<bool>,
    /// Order amount
    #[serde(default)]
    pub amount: Option<f64>,
}

/// Response from close_position API call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosePositionResponse {
    /// List of trades executed to close the position
    #[serde(default)]
    pub trades: Vec<CloseTrade>,
    /// The order placed to close the position
    #[serde(default)]
    pub order: Option<CloseOrder>,
}

/// Trade specification for move_positions request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePositionTrade {
    /// Instrument name (e.g., "BTC-PERPETUAL")
    pub instrument_name: String,
    /// Amount to move
    pub amount: f64,
    /// Optional price at which to move the position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
}

impl MovePositionTrade {
    /// Create a new move position trade
    ///
    /// # Arguments
    ///
    /// * `instrument_name` - The instrument name
    /// * `amount` - The amount to move
    #[must_use]
    pub fn new(instrument_name: &str, amount: f64) -> Self {
        Self {
            instrument_name: instrument_name.to_string(),
            amount,
            price: None,
        }
    }

    /// Set the price for the position move
    #[must_use]
    pub fn with_price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }
}

/// Result of a single position move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePositionResult {
    /// Target subaccount ID
    pub target_uid: u64,
    /// Source subaccount ID
    pub source_uid: u64,
    /// Price at which the position was moved
    pub price: f64,
    /// Instrument name
    pub instrument_name: String,
    /// Direction of the position (buy/sell)
    pub direction: String,
    /// Amount that was moved
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_position_trade_new() {
        let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0);
        assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
        assert_eq!(trade.amount, 100.0);
        assert!(trade.price.is_none());
    }

    #[test]
    fn test_move_position_trade_with_price() {
        let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0).with_price(50000.0);
        assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
        assert_eq!(trade.amount, 100.0);
        assert_eq!(trade.price, Some(50000.0));
    }

    #[test]
    fn test_move_position_trade_serialization() {
        let trade = MovePositionTrade::new("ETH-PERPETUAL", 50.0).with_price(3000.0);
        let json = serde_json::to_string(&trade).expect("serialize");
        assert!(json.contains("ETH-PERPETUAL"));
        assert!(json.contains("50"));
        assert!(json.contains("3000"));
    }

    #[test]
    fn test_move_position_trade_without_price_serialization() {
        let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0);
        let json = serde_json::to_string(&trade).expect("serialize");
        assert!(!json.contains("price"));
    }

    #[test]
    fn test_move_position_result_deserialization() {
        let json = r#"{
            "target_uid": 23,
            "source_uid": 3,
            "price": 35800.0,
            "instrument_name": "BTC-PERPETUAL",
            "direction": "buy",
            "amount": 110.0
        }"#;

        let result: MovePositionResult = serde_json::from_str(json).expect("deserialize");
        assert_eq!(result.target_uid, 23);
        assert_eq!(result.source_uid, 3);
        assert_eq!(result.price, 35800.0);
        assert_eq!(result.instrument_name, "BTC-PERPETUAL");
        assert_eq!(result.direction, "buy");
        assert_eq!(result.amount, 110.0);
    }

    #[test]
    fn test_close_position_response_deserialization() {
        let json = r#"{
            "trades": [{
                "trade_seq": 1966068,
                "trade_id": "ETH-2696097",
                "timestamp": 1590486335742,
                "tick_direction": 0,
                "state": "filled",
                "reduce_only": true,
                "price": 202.8,
                "post_only": false,
                "order_type": "limit",
                "order_id": "ETH-584864807",
                "mark_price": 202.79,
                "liquidity": "T",
                "instrument_name": "ETH-PERPETUAL",
                "index_price": 202.86,
                "fee_currency": "ETH",
                "fee": 0.00007766,
                "direction": "sell",
                "amount": 21.0
            }],
            "order": {
                "time_in_force": "good_til_cancelled",
                "reduce_only": true,
                "price": 198.75,
                "post_only": false,
                "order_type": "limit",
                "order_state": "filled",
                "order_id": "ETH-584864807",
                "instrument_name": "ETH-PERPETUAL",
                "filled_amount": 21.0,
                "direction": "sell",
                "creation_timestamp": 1590486335742,
                "average_price": 202.8,
                "api": true,
                "amount": 21.0
            }
        }"#;

        let response: ClosePositionResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(response.trades.len(), 1);
        assert!(response.order.is_some());

        let trade = &response.trades[0];
        assert_eq!(trade.trade_id, Some("ETH-2696097".to_string()));
        assert_eq!(trade.price, Some(202.8));
        assert_eq!(trade.direction, Some("sell".to_string()));

        let order = response.order.as_ref().expect("order");
        assert_eq!(order.order_id, Some("ETH-584864807".to_string()));
        assert_eq!(order.order_state, Some("filled".to_string()));
    }

    #[test]
    fn test_close_trade_deserialization() {
        let json = r#"{
            "trade_seq": 12345,
            "trade_id": "BTC-123456",
            "price": 50000.0,
            "amount": 1.0,
            "direction": "buy"
        }"#;

        let trade: CloseTrade = serde_json::from_str(json).expect("deserialize");
        assert_eq!(trade.trade_seq, Some(12345));
        assert_eq!(trade.trade_id, Some("BTC-123456".to_string()));
        assert_eq!(trade.price, Some(50000.0));
    }

    #[test]
    fn test_close_order_deserialization() {
        let json = r#"{
            "order_id": "BTC-123",
            "order_state": "open",
            "order_type": "limit",
            "price": 45000.0,
            "amount": 100.0,
            "direction": "sell"
        }"#;

        let order: CloseOrder = serde_json::from_str(json).expect("deserialize");
        assert_eq!(order.order_id, Some("BTC-123".to_string()));
        assert_eq!(order.order_state, Some("open".to_string()));
        assert_eq!(order.price, Some(45000.0));
    }
}
