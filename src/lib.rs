//! # Deribit WebSocket Client
//!
//! This crate provides a comprehensive WebSocket client for the Deribit trading platform.
//! It implements JSON-RPC over WebSocket with full support for:
//!
//! ## Features
//!
//! - 🔌 **WebSocket Connection Management** - Robust connection handling with automatic reconnection
//! - 📡 **JSON-RPC Protocol** - Complete JSON-RPC 2.0 implementation for Deribit API
//! - 📊 **Real-time Subscriptions** - Market data, order updates, and trade notifications
//! - 🔐 **Authentication** - Support for API key and signature-based authentication
//! - 🛡️ **Error Handling** - Comprehensive error types and recovery mechanisms
//! - ⚡ **Async/Await** - Full async support with tokio runtime
//! - 📈 **Rate Limiting** - Built-in rate limiting to comply with Deribit API limits
//! - 🧪 **Testing Support** - Complete test coverage and examples
//!
//! ## Protocol Support
//!
//! | Feature | Status |
//! |---------|--------|
//! | JSON-RPC over WebSocket | ✅ Full Support |
//! | Real-time Subscriptions | ✅ Full Support |
//! | Authentication | ✅ API Key + Signature |
//! | Market Data | ✅ All Channels |
//! | Trading Operations | ✅ Orders, Positions |
//! | Account Management | ✅ Portfolio, Balances |
//!
//! ## Usage
//!
//! The WebSocket client provides callback-based message handling where incoming messages
//! are processed with a primary callback that returns a Result, and an error callback
//! that handles any errors from the primary callback.
//!
//! Basic usage involves:
//! 1. Creating a WebSocket configuration (testnet or production)
//! 2. Creating a client instance with the configuration
//! 3. Setting up message and error callbacks for processing
//! 4. Connecting to the Deribit WebSocket API
//! 5. Subscribing to desired channels (market data, user data)
//! 6. Starting the message processing loop
//!
//! See the examples directory for complete working examples including:
//! - Basic client usage with market data subscriptions
//! - Callback-based message handling with error recovery
//! - Authentication and user-specific data subscriptions

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod callback;
pub mod client;
pub mod config;
pub mod connection;
pub mod constants;
pub mod error;
pub mod message;
pub mod model;
/// Prelude module with commonly used types
pub mod prelude;
pub mod session;
pub mod subscriptions;

// Re-export common types from deribit-base
pub use deribit_base;
