//! Prelude module for commonly used types and traits
//!
//! This module re-exports the most commonly used types from the deribit-websocket crate,
//! making it easy to import everything needed with a single `use` statement.

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
