//! Trading model definitions for Deribit WebSocket API
//!
//! This module provides types for buy, sell, cancel, and edit order operations.

use pretty_simple_display::{DebugPretty, DisplaySimple};
use serde::{Deserialize, Serialize};

/// Order type enumeration
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DebugPretty, DisplaySimple)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Limit order - executes at specified price or better
    Limit,
    /// Market order - executes immediately at best available price
    Market,
    /// Stop limit order - becomes limit order when stop price is reached
    StopLimit,
    /// Stop market order - becomes market order when stop price is reached
    StopMarket,
    /// Take limit order - limit order to take profit
    TakeLimit,
    /// Take market order - market order to take profit
    TakeMarket,
    /// Market limit order - market order with limit price protection
    MarketLimit,
    /// Trailing stop order - stop order that trails the market price
    TrailingStop,
}

impl OrderType {
    /// Returns the string representation of the order type
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "limit",
            OrderType::Market => "market",
            OrderType::StopLimit => "stop_limit",
            OrderType::StopMarket => "stop_market",
            OrderType::TakeLimit => "take_limit",
            OrderType::TakeMarket => "take_market",
            OrderType::MarketLimit => "market_limit",
            OrderType::TrailingStop => "trailing_stop",
        }
    }
}

/// Time in force specification for orders
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DebugPretty, DisplaySimple)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    /// Good till cancelled - order remains active until filled or cancelled
    #[serde(rename = "good_til_cancelled")]
    GoodTilCancelled,
    /// Good till day - order expires at end of trading day
    #[serde(rename = "good_til_day")]
    GoodTilDay,
    /// Fill or kill - order must be filled immediately and completely or cancelled
    #[serde(rename = "fill_or_kill")]
    FillOrKill,
    /// Immediate or cancel - fill what can be filled immediately, cancel the rest
    #[serde(rename = "immediate_or_cancel")]
    ImmediateOrCancel,
}

impl TimeInForce {
    /// Returns the string representation of the time in force
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeInForce::GoodTilCancelled => "good_til_cancelled",
            TimeInForce::GoodTilDay => "good_til_day",
            TimeInForce::FillOrKill => "fill_or_kill",
            TimeInForce::ImmediateOrCancel => "immediate_or_cancel",
        }
    }
}

/// Trigger type for conditional orders
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DebugPretty, DisplaySimple)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    /// Trigger based on index price
    IndexPrice,
    /// Trigger based on mark price
    MarkPrice,
    /// Trigger based on last traded price
    LastPrice,
}

/// Order request parameters for buy/sell operations
#[derive(Clone, Serialize, Deserialize, DebugPretty, DisplaySimple)]
pub struct OrderRequest {
    /// Instrument name (e.g., "BTC-PERPETUAL")
    pub instrument_name: String,
    /// Order amount (positive number)
    pub amount: f64,
    /// Order type (limit, market, etc.)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    /// User-defined label for the order (max 64 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Limit price for the order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Time in force specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Maximum amount to show in order book (for iceberg orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_show: Option<f64>,
    /// Whether the order should only be posted (not taken)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Whether this order only reduces position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Trigger price for conditional orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    /// Trigger type for conditional orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<Trigger>,
    /// Advanced order type (usd or implv)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced: Option<String>,
    /// Market maker protection flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmp: Option<bool>,
    /// Order validity timestamp (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
}

impl OrderRequest {
    /// Create a new limit order request
    #[must_use]
    pub fn limit(instrument_name: String, amount: f64, price: f64) -> Self {
        Self {
            instrument_name,
            amount,
            order_type: Some(OrderType::Limit),
            label: None,
            price: Some(price),
            time_in_force: None,
            max_show: None,
            post_only: None,
            reduce_only: None,
            trigger_price: None,
            trigger: None,
            advanced: None,
            mmp: None,
            valid_until: None,
        }
    }

    /// Create a new market order request
    #[must_use]
    pub fn market(instrument_name: String, amount: f64) -> Self {
        Self {
            instrument_name,
            amount,
            order_type: Some(OrderType::Market),
            label: None,
            price: None,
            time_in_force: None,
            max_show: None,
            post_only: None,
            reduce_only: None,
            trigger_price: None,
            trigger: None,
            advanced: None,
            mmp: None,
            valid_until: None,
        }
    }

    /// Set the order label
    #[must_use]
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set time in force
    #[must_use]
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set post-only flag
    #[must_use]
    pub fn with_post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }

    /// Set reduce-only flag
    #[must_use]
    pub fn with_reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }

    /// Set max show amount for iceberg orders
    #[must_use]
    pub fn with_max_show(mut self, max_show: f64) -> Self {
        self.max_show = Some(max_show);
        self
    }

    /// Set trigger price for conditional orders
    #[must_use]
    pub fn with_trigger(mut self, trigger_price: f64, trigger: Trigger) -> Self {
        self.trigger_price = Some(trigger_price);
        self.trigger = Some(trigger);
        self
    }

    /// Set MMP flag
    #[must_use]
    pub fn with_mmp(mut self, mmp: bool) -> Self {
        self.mmp = Some(mmp);
        self
    }
}

/// Edit order request parameters
#[derive(Clone, Serialize, Deserialize, DebugPretty, DisplaySimple)]
pub struct EditOrderRequest {
    /// Order ID to edit
    pub order_id: String,
    /// New amount for the order
    pub amount: f64,
    /// New price for the order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Whether to only reduce the position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Whether this order only reduces position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Advanced order type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced: Option<String>,
    /// New trigger price for conditional orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    /// Market maker protection flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmp: Option<bool>,
    /// Order validity timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
}

impl EditOrderRequest {
    /// Create a new edit order request
    #[must_use]
    pub fn new(order_id: String, amount: f64) -> Self {
        Self {
            order_id,
            amount,
            price: None,
            post_only: None,
            reduce_only: None,
            advanced: None,
            trigger_price: None,
            mmp: None,
            valid_until: None,
        }
    }

    /// Set new price
    #[must_use]
    pub fn with_price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    /// Set post-only flag
    #[must_use]
    pub fn with_post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }

    /// Set reduce-only flag
    #[must_use]
    pub fn with_reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }
}

/// Trade execution information
#[derive(Clone, Serialize, Deserialize, DebugPretty, DisplaySimple)]
pub struct TradeExecution {
    /// Trade ID
    pub trade_id: String,
    /// Instrument name
    pub instrument_name: String,
    /// Trade direction (buy/sell)
    pub direction: String,
    /// Trade amount
    pub amount: f64,
    /// Trade price
    pub price: f64,
    /// Trade fee
    pub fee: f64,
    /// Fee currency
    pub fee_currency: String,
    /// Order ID associated with this trade
    pub order_id: String,
    /// Order type
    pub order_type: String,
    /// Trade timestamp in milliseconds
    pub timestamp: u64,
    /// Liquidity type (maker/taker)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity: Option<String>,
    /// Index price at time of trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_price: Option<f64>,
    /// Mark price at time of trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
    /// Profit/loss from the trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_loss: Option<f64>,
}

/// Order information response
#[derive(Clone, Serialize, Deserialize, DebugPretty, DisplaySimple)]
pub struct OrderInfo {
    /// Order ID
    pub order_id: String,
    /// Instrument name
    pub instrument_name: String,
    /// Order direction (buy/sell)
    pub direction: String,
    /// Order amount
    pub amount: f64,
    /// Filled amount
    #[serde(default)]
    pub filled_amount: f64,
    /// Order price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// Average fill price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price: Option<f64>,
    /// Order type
    pub order_type: String,
    /// Order state (open, filled, cancelled, etc.)
    pub order_state: String,
    /// Time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// User label
    #[serde(default)]
    pub label: String,
    /// Creation timestamp in milliseconds
    pub creation_timestamp: u64,
    /// Last update timestamp in milliseconds
    pub last_update_timestamp: u64,
    /// Whether placed via API
    #[serde(default)]
    pub api: bool,
    /// Whether placed via web interface
    #[serde(default)]
    pub web: bool,
    /// Whether this is a post-only order
    #[serde(default)]
    pub post_only: bool,
    /// Whether this order only reduces position
    #[serde(default)]
    pub reduce_only: bool,
    /// Whether this is a liquidation order
    #[serde(default)]
    pub is_liquidation: bool,
    /// Maximum show amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_show: Option<f64>,
    /// Profit/loss on this order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_loss: Option<f64>,
    /// USD value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
    /// Implied volatility (for options)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implv: Option<f64>,
    /// Trigger price for conditional orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    /// Trigger type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// Whether triggered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered: Option<bool>,
    /// Whether replaced
    #[serde(default)]
    pub replaced: bool,
    /// MMP flag
    #[serde(default)]
    pub mmp: bool,
    /// MMP cancelled flag
    #[serde(default)]
    pub mmp_cancelled: bool,
}

/// Order response containing order info and trades
#[derive(Clone, Serialize, Deserialize, DebugPretty, DisplaySimple)]
pub struct OrderResponse {
    /// Order information
    pub order: OrderInfo,
    /// List of trade executions for the order
    #[serde(default)]
    pub trades: Vec<TradeExecution>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_order_type_serialization() {
        let order_type = OrderType::Limit;
        let json = serde_json::to_string(&order_type).expect("serialize");
        assert_eq!(json, "\"limit\"");

        let order_type = OrderType::StopLimit;
        let json = serde_json::to_string(&order_type).expect("serialize");
        assert_eq!(json, "\"stop_limit\"");
    }

    #[test]
    fn test_time_in_force_serialization() {
        let tif = TimeInForce::GoodTilCancelled;
        let json = serde_json::to_string(&tif).expect("serialize");
        assert_eq!(json, "\"good_til_cancelled\"");

        let tif = TimeInForce::ImmediateOrCancel;
        let json = serde_json::to_string(&tif).expect("serialize");
        assert_eq!(json, "\"immediate_or_cancel\"");
    }

    #[test]
    fn test_order_request_limit() {
        let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
            .with_label("test_order".to_string())
            .with_post_only(true);

        assert_eq!(request.instrument_name, "BTC-PERPETUAL");
        assert_eq!(request.amount, 100.0);
        assert_eq!(request.price, Some(50000.0));
        assert_eq!(request.order_type, Some(OrderType::Limit));
        assert_eq!(request.label, Some("test_order".to_string()));
        assert_eq!(request.post_only, Some(true));
    }

    #[test]
    fn test_order_request_market() {
        let request =
            OrderRequest::market("ETH-PERPETUAL".to_string(), 10.0).with_reduce_only(true);

        assert_eq!(request.instrument_name, "ETH-PERPETUAL");
        assert_eq!(request.amount, 10.0);
        assert_eq!(request.price, None);
        assert_eq!(request.order_type, Some(OrderType::Market));
        assert_eq!(request.reduce_only, Some(true));
    }

    #[test]
    fn test_edit_order_request() {
        let request = EditOrderRequest::new("order123".to_string(), 200.0).with_price(51000.0);

        assert_eq!(request.order_id, "order123");
        assert_eq!(request.amount, 200.0);
        assert_eq!(request.price, Some(51000.0));
    }

    #[test]
    fn test_order_type_as_str() {
        assert_eq!(OrderType::Limit.as_str(), "limit");
        assert_eq!(OrderType::Market.as_str(), "market");
        assert_eq!(OrderType::StopLimit.as_str(), "stop_limit");
        assert_eq!(OrderType::TrailingStop.as_str(), "trailing_stop");
    }

    #[test]
    fn test_time_in_force_as_str() {
        assert_eq!(TimeInForce::GoodTilCancelled.as_str(), "good_til_cancelled");
        assert_eq!(TimeInForce::FillOrKill.as_str(), "fill_or_kill");
        assert_eq!(
            TimeInForce::ImmediateOrCancel.as_str(),
            "immediate_or_cancel"
        );
    }
}
