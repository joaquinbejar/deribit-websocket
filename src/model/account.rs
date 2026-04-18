//! Account model definitions for Deribit WebSocket API
//!
//! This module provides types for account queries including positions,
//! account summaries, and order state.

use serde::{Deserialize, Serialize};

/// Direction of a position (buy/sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Buy/long position
    Buy,
    /// Sell/short position
    Sell,
    /// Zero position (no direction)
    Zero,
}

impl Direction {
    /// Returns the direction as a string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Buy => "buy",
            Direction::Sell => "sell",
            Direction::Zero => "zero",
        }
    }
}

/// Position structure representing a user's position in an instrument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Average price of the position
    pub average_price: f64,
    /// Average price in USD
    #[serde(default)]
    pub average_price_usd: Option<f64>,
    /// Delta (price sensitivity) of the position
    #[serde(default)]
    pub delta: Option<f64>,
    /// Direction of the position (buy/sell)
    pub direction: Direction,
    /// Estimated liquidation price
    #[serde(default)]
    pub estimated_liquidation_price: Option<f64>,
    /// Floating profit/loss
    #[serde(default)]
    pub floating_profit_loss: Option<f64>,
    /// Floating profit/loss in USD
    #[serde(default)]
    pub floating_profit_loss_usd: Option<f64>,
    /// Gamma (delta sensitivity) of the position
    #[serde(default)]
    pub gamma: Option<f64>,
    /// Current index price
    #[serde(default)]
    pub index_price: Option<f64>,
    /// Initial margin requirement
    #[serde(default)]
    pub initial_margin: Option<f64>,
    /// Name of the instrument
    pub instrument_name: String,
    /// Interest value
    #[serde(default)]
    pub interest_value: Option<f64>,
    /// Instrument kind (future, option, etc.)
    #[serde(default)]
    pub kind: Option<String>,
    /// Leverage used for the position
    #[serde(default)]
    pub leverage: Option<i32>,
    /// Maintenance margin requirement
    #[serde(default)]
    pub maintenance_margin: Option<f64>,
    /// Current mark price
    #[serde(default)]
    pub mark_price: Option<f64>,
    /// Margin used by open orders
    #[serde(default)]
    pub open_orders_margin: Option<f64>,
    /// Realized funding payments
    #[serde(default)]
    pub realized_funding: Option<f64>,
    /// Realized profit/loss
    #[serde(default)]
    pub realized_profit_loss: Option<f64>,
    /// Settlement price
    #[serde(default)]
    pub settlement_price: Option<f64>,
    /// Position size
    pub size: f64,
    /// Position size in currency units
    #[serde(default)]
    pub size_currency: Option<f64>,
    /// Theta (time decay) of the position
    #[serde(default)]
    pub theta: Option<f64>,
    /// Total profit/loss
    #[serde(default)]
    pub total_profit_loss: Option<f64>,
    /// Vega (volatility sensitivity) of the position
    #[serde(default)]
    pub vega: Option<f64>,
}

/// Per-currency account summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySummary {
    /// Currency of the summary
    pub currency: String,
    /// The account's balance
    pub balance: f64,
    /// The account's current equity
    pub equity: f64,
    /// The account's available funds
    pub available_funds: f64,
    /// The account's margin balance
    pub margin_balance: f64,
    /// Total profit and loss
    #[serde(default)]
    pub total_pl: Option<f64>,
    /// Session realized profit and loss
    #[serde(default)]
    pub session_rpl: Option<f64>,
    /// Session unrealized profit and loss
    #[serde(default)]
    pub session_upl: Option<f64>,
    /// The maintenance margin
    pub maintenance_margin: f64,
    /// The account's initial margin
    pub initial_margin: f64,
    /// The account's available to withdrawal funds
    #[serde(default)]
    pub available_withdrawal_funds: Option<f64>,
    /// When true cross collateral is enabled for user
    #[serde(default)]
    pub cross_collateral_enabled: Option<bool>,
    /// The sum of position deltas
    #[serde(default)]
    pub delta_total: Option<f64>,
    /// Futures profit and Loss
    #[serde(default)]
    pub futures_pl: Option<f64>,
    /// Futures session realized profit and Loss
    #[serde(default)]
    pub futures_session_rpl: Option<f64>,
    /// Futures session unrealized profit and Loss
    #[serde(default)]
    pub futures_session_upl: Option<f64>,
    /// Options summary delta
    #[serde(default)]
    pub options_delta: Option<f64>,
    /// Options summary gamma
    #[serde(default)]
    pub options_gamma: Option<f64>,
    /// Options profit and Loss
    #[serde(default)]
    pub options_pl: Option<f64>,
    /// Options session realized profit and Loss
    #[serde(default)]
    pub options_session_rpl: Option<f64>,
    /// Options session unrealized profit and Loss
    #[serde(default)]
    pub options_session_upl: Option<f64>,
    /// Options summary theta
    #[serde(default)]
    pub options_theta: Option<f64>,
    /// Options summary vega
    #[serde(default)]
    pub options_vega: Option<f64>,
    /// true when portfolio margining is enabled for user
    #[serde(default)]
    pub portfolio_margining_enabled: Option<bool>,
}

/// Account summary response containing user account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    /// Account id
    #[serde(default)]
    pub id: Option<u64>,
    /// User email
    #[serde(default)]
    pub email: Option<String>,
    /// System generated user nickname
    #[serde(default)]
    pub system_name: Option<String>,
    /// Account name (given by user)
    #[serde(default)]
    pub username: Option<String>,
    /// Time at which the account was created (milliseconds since the Unix epoch)
    #[serde(default)]
    pub creation_timestamp: Option<u64>,
    /// Account type
    #[serde(rename = "type", default)]
    pub account_type: Option<String>,
    /// Whether MMP is enabled
    #[serde(default)]
    pub mmp_enabled: Option<bool>,
    /// Aggregated list of per-currency account summaries
    #[serde(default)]
    pub summaries: Option<Vec<CurrencySummary>>,
    /// Currency (for single-currency response)
    #[serde(default)]
    pub currency: Option<String>,
    /// Balance (for single-currency response)
    #[serde(default)]
    pub balance: Option<f64>,
    /// Equity (for single-currency response)
    #[serde(default)]
    pub equity: Option<f64>,
    /// Available funds (for single-currency response)
    #[serde(default)]
    pub available_funds: Option<f64>,
    /// Margin balance (for single-currency response)
    #[serde(default)]
    pub margin_balance: Option<f64>,
    /// Initial margin (for single-currency response)
    #[serde(default)]
    pub initial_margin: Option<f64>,
    /// Maintenance margin (for single-currency response)
    #[serde(default)]
    pub maintenance_margin: Option<f64>,
    /// Delta total (for single-currency response)
    #[serde(default)]
    pub delta_total: Option<f64>,
    /// Options value (for single-currency response)
    #[serde(default)]
    pub options_value: Option<f64>,
    /// Futures profit and loss (for single-currency response)
    #[serde(default)]
    pub futures_pl: Option<f64>,
    /// Options profit and loss (for single-currency response)
    #[serde(default)]
    pub options_pl: Option<f64>,
    /// Total profit and loss (for single-currency response)
    #[serde(default)]
    pub total_pl: Option<f64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_as_str() {
        assert_eq!(Direction::Buy.as_str(), "buy");
        assert_eq!(Direction::Sell.as_str(), "sell");
        assert_eq!(Direction::Zero.as_str(), "zero");
    }

    #[test]
    fn test_direction_serialization() {
        let dir = Direction::Buy;
        let json = serde_json::to_string(&dir).expect("serialize");
        assert_eq!(json, "\"buy\"");
    }

    #[test]
    fn test_direction_deserialization() {
        let dir: Direction = serde_json::from_str("\"sell\"").expect("deserialize");
        assert_eq!(dir, Direction::Sell);
    }

    #[test]
    fn test_position_deserialization() {
        let json = r#"{
            "average_price": 50000.0,
            "direction": "buy",
            "instrument_name": "BTC-PERPETUAL",
            "size": 100.0,
            "floating_profit_loss": 50.0,
            "mark_price": 50050.0
        }"#;

        let position: Position = serde_json::from_str(json).expect("deserialize");
        assert_eq!(position.instrument_name, "BTC-PERPETUAL");
        assert_eq!(position.size, 100.0);
        assert_eq!(position.direction, Direction::Buy);
        assert_eq!(position.average_price, 50000.0);
    }

    #[test]
    fn test_currency_summary_deserialization() {
        let json = r#"{
            "currency": "BTC",
            "balance": 1.5,
            "equity": 1.6,
            "available_funds": 1.0,
            "margin_balance": 1.5,
            "maintenance_margin": 0.1,
            "initial_margin": 0.2
        }"#;

        let summary: CurrencySummary = serde_json::from_str(json).expect("deserialize");
        assert_eq!(summary.currency, "BTC");
        assert_eq!(summary.balance, 1.5);
        assert_eq!(summary.equity, 1.6);
    }

    #[test]
    fn test_account_summary_deserialization() {
        let json = r#"{
            "currency": "BTC",
            "balance": 1.5,
            "equity": 1.6,
            "available_funds": 1.0,
            "margin_balance": 1.5,
            "initial_margin": 0.2,
            "maintenance_margin": 0.1,
            "delta_total": 0.5
        }"#;

        let summary: AccountSummary = serde_json::from_str(json).expect("deserialize");
        assert_eq!(summary.currency, Some("BTC".to_string()));
        assert_eq!(summary.balance, Some(1.5));
    }
}
