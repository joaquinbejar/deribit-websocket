//! WebSocket-specific types and models for Deribit API
//!
//! This module contains data structures specific to WebSocket communication,
//! including JSON-RPC message types, connection states, and WebSocket-specific
//! request/response structures.

use pretty_simple_display::{DebugPretty, DisplaySimple};
use serde::{Deserialize, Serialize};

/// WebSocket message types for JSON-RPC communication
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub enum WebSocketMessage {
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC response message
    Response(JsonRpcResponse),
    /// JSON-RPC notification message (no response expected)
    Notification(JsonRpcNotification),
}

/// JSON-RPC 2.0 request structure
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier for correlation with response
    pub id: serde_json::Value,
    /// Method name to call
    pub method: String,
    /// Optional parameters for the method
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier for correlation
    pub id: serde_json::Value,
    /// Result or error information
    #[serde(flatten)]
    pub result: JsonRpcResult,
}

/// JSON-RPC 2.0 result or error union
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
#[serde(untagged)]
pub enum JsonRpcResult {
    /// Successful result
    Success {
        /// Result data
        result: serde_json::Value,
    },
    /// Error result
    Error {
        /// Error information
        error: JsonRpcError,
    },
}

/// JSON-RPC 2.0 error structure
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional error data
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 notification structure (no response expected)
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Optional parameters
    pub params: Option<serde_json::Value>,
}

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected but not authenticated
    Connected,
    /// Connected and authenticated
    Authenticated,
    /// Attempting to reconnect
    Reconnecting,
    /// Connection failed
    Failed,
}

/// Heartbeat monitoring status
#[derive(Debug, Clone)]
pub struct HeartbeatStatus {
    /// Last ping sent timestamp
    pub last_ping: Option<std::time::Instant>,
    /// Last pong received timestamp
    pub last_pong: Option<std::time::Instant>,
    /// Number of consecutive missed pongs
    pub missed_pongs: u32,
}

/// WebSocket subscription channel types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubscriptionChannel {
    /// Ticker data for a specific instrument
    Ticker(String),
    /// Order book data for a specific instrument
    OrderBook(String),
    /// Trade data for a specific instrument
    Trades(String),
    /// Chart trade data for a specific instrument with resolution
    ChartTrades {
        /// The trading instrument (e.g., "BTC-PERPETUAL")
        instrument: String,
        /// Chart resolution (e.g., "1", "5", "15", "60" for minutes)
        resolution: String,
    },
    /// User's order updates
    UserOrders,
    /// User's trade updates
    UserTrades,
    /// User's portfolio updates
    UserPortfolio,
    /// User's position changes for a specific instrument with interval
    UserChanges {
        /// The trading instrument (e.g., "BTC-PERPETUAL")
        instrument: String,
        /// Update interval (e.g., "raw", "100ms")
        interval: String,
    },
    /// Price index updates
    PriceIndex(String),
    /// Estimated delivery price
    EstimatedExpirationPrice(String),
    /// Mark price updates
    MarkPrice(String),
    /// Funding rate updates
    Funding(String),
    /// Perpetual updates with configurable interval
    Perpetual {
        /// The trading instrument (e.g., "BTC-PERPETUAL")
        instrument: String,
        /// Update interval (e.g., "raw", "100ms")
        interval: String,
    },
    /// Quote updates
    Quote(String),
    /// Platform state updates
    PlatformState,
    /// Platform state public methods state updates
    PlatformStatePublicMethods,
    /// Instrument state changes for a specific kind and currency
    InstrumentState {
        /// Instrument kind (e.g., "future", "option", "spot")
        kind: String,
        /// Currency (e.g., "BTC", "ETH")
        currency: String,
    },
    /// Grouped order book with configurable depth and interval
    GroupedOrderBook {
        /// The trading instrument (e.g., "BTC-PERPETUAL")
        instrument: String,
        /// Grouping level for aggregation
        group: String,
        /// Order book depth (e.g., "1", "10", "20")
        depth: String,
        /// Update interval (e.g., "100ms", "agg2")
        interval: String,
    },
    /// Incremental ticker updates for a specific instrument
    IncrementalTicker(String),
    /// Trades by instrument kind (e.g., future, option) and currency
    TradesByKind {
        /// Instrument kind (e.g., "future", "option", "spot", "any")
        kind: String,
        /// Currency (e.g., "BTC", "ETH", "any")
        currency: String,
        /// Update interval (e.g., "raw", "100ms")
        interval: String,
    },
    /// Price ranking data for an index
    PriceRanking(String),
    /// Price statistics for an index
    PriceStatistics(String),
    /// Volatility index data
    VolatilityIndex(String),
    /// Block RFQ trades for a specific currency
    BlockRfqTrades(String),
    /// Block trade confirmations (all currencies)
    BlockTradeConfirmations,
    /// Block trade confirmations for a specific currency
    BlockTradeConfirmationsByCurrency(String),
    /// Unknown or unrecognized channel
    Unknown(String),
}

/// WebSocket request structure for Deribit API
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct WsRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID for correlation with responses
    pub id: serde_json::Value,
    /// API method name to call
    pub method: String,
    /// Parameters for the API method
    pub params: Option<serde_json::Value>,
}

/// WebSocket response structure for Deribit API
#[derive(Clone, Serialize, Deserialize, PartialEq, DebugPretty, DisplaySimple)]
pub struct WsResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID for correlation (None for notifications)
    pub id: Option<serde_json::Value>,
    /// Result data if the request was successful
    pub result: Option<serde_json::Value>,
    /// Error information if the request failed
    pub error: Option<JsonRpcError>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new<T: Serialize>(id: serde_json::Value, method: &str, params: Option<T>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: params.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
        }
    }
}

impl JsonRpcResponse {
    /// Create a new successful JSON-RPC response
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: JsonRpcResult::Success { result },
        }
    }

    /// Create a new error JSON-RPC response
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: JsonRpcResult::Error { error },
        }
    }
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new<T: Serialize>(method: &str, params: Option<T>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: params.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
        }
    }
}

impl SubscriptionChannel {
    /// Convert subscription channel to channel name
    pub fn channel_name(&self) -> String {
        match self {
            SubscriptionChannel::Ticker(instrument) => format!("ticker.{}", instrument),
            SubscriptionChannel::OrderBook(instrument) => format!("book.{}.raw", instrument),
            SubscriptionChannel::Trades(instrument) => format!("trades.{}.raw", instrument),
            SubscriptionChannel::ChartTrades {
                instrument,
                resolution,
            } => {
                format!("chart.trades.{}.{}", instrument, resolution)
            }
            SubscriptionChannel::UserOrders => "user.orders.any.any.raw".to_string(),
            SubscriptionChannel::UserTrades => "user.trades.any.any.raw".to_string(),
            SubscriptionChannel::UserPortfolio => "user.portfolio.any".to_string(),
            SubscriptionChannel::UserChanges {
                instrument,
                interval,
            } => {
                format!("user.changes.{}.{}", instrument, interval)
            }
            SubscriptionChannel::PriceIndex(currency) => {
                format!("deribit_price_index.{}_usd", currency.to_lowercase())
            }
            SubscriptionChannel::EstimatedExpirationPrice(instrument) => {
                format!("estimated_expiration_price.{}", instrument)
            }
            SubscriptionChannel::MarkPrice(instrument) => {
                format!("markprice.options.{}", instrument)
            }
            SubscriptionChannel::Funding(instrument) => format!("perpetual.{}.raw", instrument),
            SubscriptionChannel::Perpetual {
                instrument,
                interval,
            } => {
                format!("perpetual.{}.{}", instrument, interval)
            }
            SubscriptionChannel::Quote(instrument) => format!("quote.{}", instrument),
            SubscriptionChannel::PlatformState => "platform_state".to_string(),
            SubscriptionChannel::PlatformStatePublicMethods => {
                "platform_state.public_methods_state".to_string()
            }
            SubscriptionChannel::InstrumentState { kind, currency } => {
                format!("instrument.state.{}.{}", kind, currency)
            }
            SubscriptionChannel::GroupedOrderBook {
                instrument,
                group,
                depth,
                interval,
            } => {
                format!("book.{}.{}.{}.{}", instrument, group, depth, interval)
            }
            SubscriptionChannel::IncrementalTicker(instrument) => {
                format!("incremental_ticker.{}", instrument)
            }
            SubscriptionChannel::TradesByKind {
                kind,
                currency,
                interval,
            } => {
                format!("trades.{}.{}.{}", kind, currency, interval)
            }
            SubscriptionChannel::PriceRanking(index_name) => {
                format!("deribit_price_ranking.{}", index_name)
            }
            SubscriptionChannel::PriceStatistics(index_name) => {
                format!("deribit_price_statistics.{}", index_name)
            }
            SubscriptionChannel::VolatilityIndex(index_name) => {
                format!("deribit_volatility_index.{}", index_name)
            }
            SubscriptionChannel::BlockRfqTrades(currency) => {
                format!("block_rfq.trades.{}", currency)
            }
            SubscriptionChannel::BlockTradeConfirmations => "block_trade_confirmations".to_string(),
            SubscriptionChannel::BlockTradeConfirmationsByCurrency(currency) => {
                format!("block_trade_confirmations.{}", currency)
            }
            SubscriptionChannel::Unknown(channel) => channel.clone(),
        }
    }

    /// Parse subscription channel from string
    ///
    /// Returns the appropriate `SubscriptionChannel` variant for recognized channel patterns,
    /// or `Unknown(String)` for unrecognized patterns.
    #[must_use]
    pub fn from_string(s: &str) -> Self {
        let parts: Vec<&str> = s.split('.').collect();
        match parts.as_slice() {
            ["ticker", instrument] => SubscriptionChannel::Ticker(instrument.to_string()),
            ["ticker", instrument, _interval] => {
                SubscriptionChannel::Ticker(instrument.to_string())
            }
            ["book", instrument, "raw"] => SubscriptionChannel::OrderBook(instrument.to_string()),
            ["book", instrument, group, depth, interval] => SubscriptionChannel::GroupedOrderBook {
                instrument: instrument.to_string(),
                group: group.to_string(),
                depth: depth.to_string(),
                interval: interval.to_string(),
            },
            ["book", instrument, _depth, _interval] => {
                SubscriptionChannel::OrderBook(instrument.to_string())
            }
            ["incremental_ticker", instrument] => {
                SubscriptionChannel::IncrementalTicker(instrument.to_string())
            }
            ["trades", instrument, "raw"] => SubscriptionChannel::Trades(instrument.to_string()),
            ["trades", kind, currency, interval] if !Self::looks_like_instrument(kind) => {
                SubscriptionChannel::TradesByKind {
                    kind: kind.to_string(),
                    currency: currency.to_string(),
                    interval: interval.to_string(),
                }
            }
            ["trades", instrument, _interval] => {
                SubscriptionChannel::Trades(instrument.to_string())
            }
            ["chart", "trades", instrument, resolution] => SubscriptionChannel::ChartTrades {
                instrument: instrument.to_string(),
                resolution: resolution.to_string(),
            },
            ["user", "orders", ..] => SubscriptionChannel::UserOrders,
            ["user", "trades", ..] => SubscriptionChannel::UserTrades,
            ["user", "portfolio", ..] => SubscriptionChannel::UserPortfolio,
            ["user", "changes", instrument, interval] => SubscriptionChannel::UserChanges {
                instrument: instrument.to_string(),
                interval: interval.to_string(),
            },
            ["deribit_price_index", currency_pair] => {
                let currency = currency_pair
                    .strip_suffix("_usd")
                    .map(|c| c.to_uppercase())
                    .unwrap_or_else(|| currency_pair.to_uppercase());
                SubscriptionChannel::PriceIndex(currency)
            }
            ["estimated_expiration_price", instrument] => {
                SubscriptionChannel::EstimatedExpirationPrice(instrument.to_string())
            }
            ["markprice", "options", instrument] => {
                SubscriptionChannel::MarkPrice(instrument.to_string())
            }
            ["perpetual", instrument, interval] => SubscriptionChannel::Perpetual {
                instrument: instrument.to_string(),
                interval: interval.to_string(),
            },
            ["quote", instrument] => SubscriptionChannel::Quote(instrument.to_string()),
            ["platform_state"] => SubscriptionChannel::PlatformState,
            ["platform_state", "public_methods_state"] => {
                SubscriptionChannel::PlatformStatePublicMethods
            }
            ["instrument", "state", kind, currency] => SubscriptionChannel::InstrumentState {
                kind: kind.to_string(),
                currency: currency.to_string(),
            },
            ["deribit_price_ranking", index_name] => {
                SubscriptionChannel::PriceRanking(index_name.to_string())
            }
            ["deribit_price_statistics", index_name] => {
                SubscriptionChannel::PriceStatistics(index_name.to_string())
            }
            ["deribit_volatility_index", index_name] => {
                SubscriptionChannel::VolatilityIndex(index_name.to_string())
            }
            ["block_rfq", "trades", currency] => {
                SubscriptionChannel::BlockRfqTrades(currency.to_string())
            }
            ["block_trade_confirmations"] => SubscriptionChannel::BlockTradeConfirmations,
            ["block_trade_confirmations", currency] => {
                SubscriptionChannel::BlockTradeConfirmationsByCurrency(currency.to_string())
            }
            _ => SubscriptionChannel::Unknown(s.to_string()),
        }
    }

    /// Check if this channel is unknown/unrecognized
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, SubscriptionChannel::Unknown(_))
    }

    /// Check if a string looks like an instrument name (contains hyphen).
    ///
    /// Used to distinguish between `trades.{instrument}.{interval}` and
    /// `trades.{kind}.{currency}.{interval}` patterns.
    #[must_use]
    fn looks_like_instrument(s: &str) -> bool {
        s.contains('-')
    }
}

impl std::fmt::Display for SubscriptionChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.channel_name())
    }
}

impl ConnectionState {
    /// Check if the connection is in a connected state
    pub fn is_connected(&self) -> bool {
        matches!(
            self,
            ConnectionState::Connected | ConnectionState::Authenticated
        )
    }

    /// Check if the connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, ConnectionState::Authenticated)
    }

    /// Check if the connection is in a transitional state
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            ConnectionState::Connecting | ConnectionState::Reconnecting
        )
    }
}

impl HeartbeatStatus {
    /// Create a new heartbeat status
    pub fn new() -> Self {
        Self {
            last_ping: None,
            last_pong: None,
            missed_pongs: 0,
        }
    }

    /// Record a ping sent
    pub fn ping_sent(&mut self) {
        self.last_ping = Some(std::time::Instant::now());
    }

    /// Record a pong received
    pub fn pong_received(&mut self) {
        self.last_pong = Some(std::time::Instant::now());
        self.missed_pongs = 0;
    }

    /// Record a missed pong
    pub fn missed_pong(&mut self) {
        self.missed_pongs += 1;
    }

    /// Check if connection is considered stale
    pub fn is_stale(&self, max_missed_pongs: u32) -> bool {
        self.missed_pongs >= max_missed_pongs
    }
}

impl Default for HeartbeatStatus {
    fn default() -> Self {
        Self::new()
    }
}
