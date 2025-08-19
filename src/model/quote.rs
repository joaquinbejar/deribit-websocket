//! Quote and Mass Quote model definitions for Deribit WebSocket API

use deribit_base::{impl_json_debug_pretty, impl_json_display};
use serde::{Deserialize, Serialize};

/// Represents a single quote in a mass quote request
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    /// Instrument name (e.g., "BTC-PERPETUAL")
    pub instrument_name: String,
    /// Quote side: "buy" or "sell"
    pub side: String,
    /// Quote amount (positive number)
    pub amount: f64,
    /// Quote price
    pub price: f64,
    /// Optional quote set ID for grouping quotes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_set_id: Option<String>,
    /// Optional post-only flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Optional time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
}

impl_json_display!(Quote);
impl_json_debug_pretty!(Quote);

/// Mass quote request parameters
#[derive(Clone, Serialize, Deserialize)]
pub struct MassQuoteRequest {
    /// MMP group name for this mass quote
    pub mmp_group: String,
    /// List of quotes to place
    pub quotes: Vec<Quote>,
    /// User-defined quote ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    /// Whether to return detailed error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed: Option<bool>,
}

impl_json_display!(MassQuoteRequest);
impl_json_debug_pretty!(MassQuoteRequest);

/// Mass quote response
#[derive(Clone, Serialize, Deserialize)]
pub struct MassQuoteResult {
    /// Number of successful quotes placed
    pub success_count: u32,
    /// Number of failed quotes
    pub error_count: u32,
    /// Detailed error information (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<QuoteError>>,
}

impl_json_display!(MassQuoteResult);
impl_json_debug_pretty!(MassQuoteResult);

/// Quote error information
#[derive(Clone, Serialize, Deserialize)]
pub struct QuoteError {
    /// Instrument name that failed
    pub instrument_name: String,
    /// Side that failed
    pub side: String,
    /// Error code
    pub error_code: i32,
    /// Error message
    pub error_message: String,
}

impl_json_display!(QuoteError);
impl_json_debug_pretty!(QuoteError);

/// Quote cancellation request parameters
#[derive(Clone, Serialize, Deserialize)]
pub struct CancelQuotesRequest {
    /// Optional currency to filter cancellations (e.g., "BTC")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Optional instrument kind filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Optional specific instrument name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_name: Option<String>,
    /// Optional quote set ID to cancel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_set_id: Option<String>,
    /// Optional delta range for options (min, max)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_range: Option<(f64, f64)>,
}

impl_json_display!(CancelQuotesRequest);
impl_json_debug_pretty!(CancelQuotesRequest);

/// Quote cancellation response
#[derive(Clone, Serialize, Deserialize)]
pub struct CancelQuotesResponse {
    /// Number of quotes cancelled
    pub cancelled_count: u32,
}

impl_json_display!(CancelQuotesResponse);
impl_json_debug_pretty!(CancelQuotesResponse);

/// MMP (Market Maker Protection) group configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct MmpGroupConfig {
    /// MMP group name (unique across account)
    pub mmp_group: String,
    /// Quantity limit for this group (max amount per quote)
    pub quantity_limit: f64,
    /// Delta limit (must be < quantity_limit)
    pub delta_limit: f64,
    /// Interval in milliseconds for MMP triggers
    pub interval: u64,
    /// Frozen time in milliseconds after MMP trigger
    pub frozen_time: u64,
    /// Whether the group is enabled
    pub enabled: bool,
}

impl_json_display!(MmpGroupConfig);
impl_json_debug_pretty!(MmpGroupConfig);

/// MMP group status information
#[derive(Clone, Serialize, Deserialize)]
pub struct MmpGroupStatus {
    /// MMP group name
    pub mmp_group: String,
    /// Current configuration
    pub config: MmpGroupConfig,
    /// Reserved initial margin for this group
    pub reserved_margin: f64,
    /// Number of active quotes in this group
    pub active_quotes: u32,
    /// Whether the group is currently frozen
    pub is_frozen: bool,
    /// Timestamp when freeze will end (if frozen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeze_end_time: Option<u64>,
}

impl_json_display!(MmpGroupStatus);
impl_json_debug_pretty!(MmpGroupStatus);

/// Quote information from get_open_orders
#[derive(Clone, Serialize, Deserialize)]
pub struct QuoteInfo {
    /// Quote ID
    pub quote_id: String,
    /// Instrument name
    pub instrument_name: String,
    /// Quote side
    pub side: String,
    /// Quote amount
    pub amount: f64,
    /// Quote price
    pub price: f64,
    /// Quote set ID (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_set_id: Option<String>,
    /// MMP group name
    pub mmp_group: String,
    /// Quote creation timestamp
    pub creation_timestamp: u64,
    /// Quote state (e.g., "open", "filled", "cancelled")
    pub state: String,
    /// Filled amount
    pub filled_amount: f64,
    /// Average fill price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price: Option<f64>,
    /// Quote priority in order book
    pub priority: u64,
}

impl_json_display!(QuoteInfo);
impl_json_debug_pretty!(QuoteInfo);

/// MMP trigger notification
#[derive(Clone, Serialize, Deserialize)]
pub struct MmpTrigger {
    /// Currency that triggered MMP
    pub currency: String,
    /// MMP group that was triggered (if specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmp_group: Option<String>,
    /// Trigger timestamp
    pub timestamp: u64,
    /// Trigger reason
    pub reason: String,
    /// Duration of freeze in milliseconds
    pub frozen_time: u64,
}

impl_json_display!(MmpTrigger);
impl_json_debug_pretty!(MmpTrigger);

impl Quote {
    /// Create a new buy quote
    pub fn buy(instrument_name: String, amount: f64, price: f64) -> Self {
        Self {
            instrument_name,
            side: "buy".to_string(),
            amount,
            price,
            quote_set_id: None,
            post_only: None,
            time_in_force: None,
        }
    }

    /// Create a new sell quote
    pub fn sell(instrument_name: String, amount: f64, price: f64) -> Self {
        Self {
            instrument_name,
            side: "sell".to_string(),
            amount,
            price,
            quote_set_id: None,
            post_only: None,
            time_in_force: None,
        }
    }

    /// Set quote set ID for this quote
    pub fn with_quote_set_id(mut self, quote_set_id: String) -> Self {
        self.quote_set_id = Some(quote_set_id);
        self
    }

    /// Set post-only flag for this quote
    pub fn with_post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }

    /// Set time in force for this quote
    pub fn with_time_in_force(mut self, time_in_force: String) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }
}

impl MassQuoteRequest {
    /// Create a new mass quote request
    pub fn new(mmp_group: String, quotes: Vec<Quote>) -> Self {
        Self {
            mmp_group,
            quotes,
            quote_id: None,
            detailed: None,
        }
    }

    /// Set quote ID for tracking
    pub fn with_quote_id(mut self, quote_id: String) -> Self {
        self.quote_id = Some(quote_id);
        self
    }

    /// Request detailed error information
    pub fn with_detailed_errors(mut self) -> Self {
        self.detailed = Some(true);
        self
    }

    /// Validate the mass quote request
    pub fn validate(&self) -> Result<(), String> {
        if self.quotes.is_empty() {
            return Err("Mass quote request must contain at least one quote".to_string());
        }

        if self.quotes.len() > 100 {
            return Err("Mass quote request cannot contain more than 100 quotes".to_string());
        }

        // Check that all quotes are for the same index (currency pair)
        let mut currencies = std::collections::HashSet::new();
        for quote in &self.quotes {
            let currency = quote
                .instrument_name
                .split('-')
                .next()
                .ok_or("Invalid instrument name format")?;
            currencies.insert(currency);
        }

        if currencies.len() > 1 {
            return Err(
                "All quotes in a mass quote request must be for the same currency".to_string(),
            );
        }

        // Check for duplicate quotes (same instrument, side, and price)
        let mut seen = std::collections::HashSet::new();
        for quote in &self.quotes {
            let key = (&quote.instrument_name, &quote.side, quote.price as u64);
            if !seen.insert(key) {
                return Err(format!(
                    "Duplicate quote found for {} {} at price {}",
                    quote.instrument_name, quote.side, quote.price
                ));
            }
        }

        Ok(())
    }
}

impl CancelQuotesRequest {
    /// Create a request to cancel all quotes
    pub fn all() -> Self {
        Self {
            currency: None,
            kind: None,
            instrument_name: None,
            quote_set_id: None,
            delta_range: None,
        }
    }

    /// Create a request to cancel quotes by currency
    pub fn by_currency(currency: String) -> Self {
        Self {
            currency: Some(currency),
            kind: None,
            instrument_name: None,
            quote_set_id: None,
            delta_range: None,
        }
    }

    /// Create a request to cancel quotes by instrument
    pub fn by_instrument(instrument_name: String) -> Self {
        Self {
            currency: None,
            kind: None,
            instrument_name: Some(instrument_name),
            quote_set_id: None,
            delta_range: None,
        }
    }

    /// Create a request to cancel quotes by quote set ID
    pub fn by_quote_set_id(quote_set_id: String) -> Self {
        Self {
            currency: None,
            kind: None,
            instrument_name: None,
            quote_set_id: Some(quote_set_id),
            delta_range: None,
        }
    }

    /// Create a request to cancel quotes by delta range (options only)
    pub fn by_delta_range(min_delta: f64, max_delta: f64) -> Self {
        Self {
            currency: None,
            kind: None,
            instrument_name: None,
            quote_set_id: None,
            delta_range: Some((min_delta, max_delta)),
        }
    }
}

impl MmpGroupConfig {
    /// Create a new MMP group configuration
    pub fn new(
        mmp_group: String,
        quantity_limit: f64,
        delta_limit: f64,
        interval: u64,
        frozen_time: u64,
    ) -> Result<Self, String> {
        if delta_limit >= quantity_limit {
            return Err("Delta limit must be less than quantity limit".to_string());
        }

        // Check quantity limits (500 BTC, 5000 ETH equivalent)
        let currency = mmp_group.split('_').next().unwrap_or("");
        let max_limit = match currency.to_uppercase().as_str() {
            "BTC" => 500.0,
            "ETH" => 5000.0,
            _ => 500.0, // Default to BTC limit
        };

        if quantity_limit > max_limit {
            return Err(format!(
                "Quantity limit {} exceeds maximum allowed {} for {}",
                quantity_limit, max_limit, currency
            ));
        }

        Ok(Self {
            mmp_group,
            quantity_limit,
            delta_limit,
            interval,
            frozen_time,
            enabled: true,
        })
    }

    /// Disable the MMP group (sets interval to 0)
    pub fn disable(mut self) -> Self {
        self.interval = 0;
        self.enabled = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_creation() {
        let quote = Quote::buy("BTC-PERPETUAL".to_string(), 1.0, 50000.0)
            .with_quote_set_id("set1".to_string())
            .with_post_only(true);

        assert_eq!(quote.instrument_name, "BTC-PERPETUAL");
        assert_eq!(quote.side, "buy");
        assert_eq!(quote.amount, 1.0);
        assert_eq!(quote.price, 50000.0);
        assert_eq!(quote.quote_set_id, Some("set1".to_string()));
        assert_eq!(quote.post_only, Some(true));
    }

    #[test]
    fn test_mass_quote_validation() {
        let quotes = vec![
            Quote::buy("BTC-PERPETUAL".to_string(), 1.0, 50000.0),
            Quote::sell("BTC-PERPETUAL".to_string(), 1.0, 51000.0),
        ];

        let request = MassQuoteRequest::new("btc_group".to_string(), quotes);
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_mass_quote_validation_different_currencies() {
        let quotes = vec![
            Quote::buy("BTC-PERPETUAL".to_string(), 1.0, 50000.0),
            Quote::sell("ETH-PERPETUAL".to_string(), 1.0, 3000.0),
        ];

        let request = MassQuoteRequest::new("mixed_group".to_string(), quotes);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_mass_quote_validation_duplicate_quotes() {
        let quotes = vec![
            Quote::buy("BTC-PERPETUAL".to_string(), 1.0, 50000.0),
            Quote::buy("BTC-PERPETUAL".to_string(), 2.0, 49000.0),
        ];

        let request = MassQuoteRequest::new("btc_group".to_string(), quotes);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_mmp_group_config_validation() {
        let config = MmpGroupConfig::new("btc_group".to_string(), 100.0, 50.0, 1000, 5000);
        assert!(config.is_ok());

        let invalid_config = MmpGroupConfig::new(
            "btc_group".to_string(),
            50.0,
            100.0, // Delta limit > quantity limit
            1000,
            5000,
        );
        assert!(invalid_config.is_err());
    }

    #[test]
    fn test_cancel_quotes_builders() {
        let cancel_all = CancelQuotesRequest::all();
        assert!(cancel_all.currency.is_none());

        let cancel_btc = CancelQuotesRequest::by_currency("BTC".to_string());
        assert_eq!(cancel_btc.currency, Some("BTC".to_string()));

        let cancel_instrument = CancelQuotesRequest::by_instrument("BTC-PERPETUAL".to_string());
        assert_eq!(
            cancel_instrument.instrument_name,
            Some("BTC-PERPETUAL".to_string())
        );

        let cancel_set = CancelQuotesRequest::by_quote_set_id("set1".to_string());
        assert_eq!(cancel_set.quote_set_id, Some("set1".to_string()));

        let cancel_delta = CancelQuotesRequest::by_delta_range(0.3, 0.7);
        assert_eq!(cancel_delta.delta_range, Some((0.3, 0.7)));
    }
}
