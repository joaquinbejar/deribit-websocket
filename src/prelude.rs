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
pub use crate::model::subscription::{Subscription, SubscriptionManager};

// Session management
pub use crate::session::WebSocketSession;

// Subscription management
pub use crate::subscriptions::SubscriptionChannel;

// Constants
pub use crate::constants::*;

// Re-export commonly used types from deribit-base
pub use deribit_base::prelude::*;

// Re-export commonly used external types
pub use serde_json::{Value, json};
pub use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
