//! Curated glob-importable re-exports for common use cases.
//!
//! ```rust,no_run
//! use deribit_websocket::prelude::*;
//! ```
//!
//! Type names below are rendered as plain code rather than as rustdoc
//! intra-doc links because this module doc is emitted before the
//! re-exports it describes, so the linker would not resolve them.
//!
//! # What you get
//!
//! - **Client + config** — `DeribitWebSocketClient`, `WebSocketConfig`,
//!   and the message-handler types (`MessageHandler`,
//!   `MessageHandlerBuilder`, `MessageCallback`, `ErrorCallback`)
//!   needed to drive a live connection.
//! - **Session / connection** — `WebSocketSession` and
//!   `WebSocketConnection` for consumers that want to hold the
//!   underlying session directly.
//! - **Error type** — `WebSocketError`. All fallible operations in this
//!   crate return `Result<T, WebSocketError>`.
//! - **DTOs commonly seen on the wire** — JSON-RPC envelopes
//!   (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcNotification`,
//!   `JsonRpcError`, `JsonRpcResult`), trading structures
//!   (`OrderRequest`, `OrderResponse`, `OrderInfo`, `TradeExecution`,
//!   `Position`, etc.), mass-quote structures (`Quote`,
//!   `MassQuoteRequest`, `MassQuoteResult`, `MmpGroupConfig`, …), and
//!   account/subscription helpers.
//! - **Message helpers** — `MessageBuilder`, `RequestBuilder`,
//!   `ResponseHandler`, `NotificationHandler`.
//! - **Constants** — everything from [`crate::constants`], including
//!   default URLs and channel name prefixes.
//! - **`setup_logger`** — quick `tracing` initialisation driven by
//!   `DERIBIT_LOG_LEVEL`.
//! - **A small slice of external types** — [`serde_json::Value`],
//!   [`serde_json::json`], and
//!   [`tokio_tungstenite::tungstenite::Message`] (re-exported as
//!   `TungsteniteMessage`) because they appear in almost every
//!   callback signature.
//!
//! # What you do *not* get
//!
//! The prelude is intentionally narrow. The following are **not**
//! re-exported and should be imported by path when needed:
//!
//! - Less-common model variants — browse [`crate::model`] for the full
//!   list (instrument metadata, chart candles, etc.).
//! - Low-level dispatcher / connection internals — see
//!   [`crate::connection`].
//! - TLS backend helpers — see [`crate::tls`] and the crate-level
//!   [`crate::install_default_crypto_provider`].
//! - The `error` module sub-types beyond `WebSocketError` — see
//!   [`crate::error`] for envelope builders and helpers.

// Callback system
pub use crate::callback::{ErrorCallback, MessageCallback, MessageHandler, MessageHandlerBuilder};

// Client and configuration
pub use crate::client::DeribitWebSocketClient;
pub use crate::config::WebSocketConfig;

// Connection management
pub use crate::connection::WebSocketConnection;

// Error handling
pub use crate::error::WebSocketError;

// Message types
pub use crate::message::{
    MessageBuilder, notification::NotificationHandler, request::RequestBuilder,
    response::ResponseHandler,
};

// Model types
pub use crate::model::{
    account::{AccountSummary, CurrencySummary, Direction, Position},
    position::{
        CloseOrder, ClosePositionResponse, CloseTrade, MovePositionResult, MovePositionTrade,
    },
    quote::{
        CancelQuotesRequest, CancelQuotesResponse, MassQuoteRequest, MassQuoteResult,
        MmpGroupConfig, MmpGroupStatus, MmpTrigger, Quote, QuoteError, QuoteInfo,
    },
    subscription::{Subscription, SubscriptionManager},
    trading::{
        EditOrderRequest, OrderInfo, OrderRequest, OrderResponse, OrderType, TimeInForce,
        TradeExecution, Trigger,
    },
    ws_types::{
        AuthResponse, ConnectionState, HeartbeatStatus, HelloResponse, JsonRpcError,
        JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, JsonRpcResult, TestResponse,
        WebSocketMessage,
    },
};

// Session management
pub use crate::session::WebSocketSession;

// Subscription management (re-export from model for full channel support)
pub use crate::model::SubscriptionChannel;

// Constants
pub use crate::constants::*;

// Utility functions
pub use crate::utils::setup_logger;

// Re-export commonly used external types
pub use serde_json::{Value, json};
pub use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
