//! # Deribit WebSocket Client
//!
//! A high-performance, production-ready WebSocket client for the Deribit cryptocurrency derivatives exchange.
//! This crate provides comprehensive real-time market data streaming, trading operations, and account management
//! through Deribit's WebSocket API v2.
//!
//! ## Features
//!
//! - 🔌 **WebSocket Connection Management** - Robust connection handling with automatic reconnection and heartbeat
//! - 📡 **JSON-RPC Protocol** - Complete JSON-RPC 2.0 implementation for Deribit API
//! - 📊 **Real-time Market Data** - Live ticker, order book, trades, and chart data streaming
//! - 📈 **Advanced Subscriptions** - Chart data aggregation and user position change notifications
//! - 💰 **Mass Quote System** - High-performance mass quoting with MMP (Market Maker Protection) groups
//! - 🔐 **Authentication** - Secure API key and signature-based authentication
//! - 🛡️ **Error Handling** - Comprehensive error types with detailed recovery mechanisms
//! - ⚡ **Async/Await** - Full async support with tokio runtime for high concurrency
//! - 🔄 **Callback System** - Flexible message processing with primary and error callbacks
//! - 📋 **Subscription Management** - Intelligent subscription tracking and channel management
//! - 🧪 **Testing Support** - Complete test coverage with working examples
//!
//! ## Supported Subscription Channels
//!
//! ### Market Data Channels
//! - `ticker.{instrument}` - Real-time ticker updates
//! - `book.{instrument}.{group}` - Order book snapshots and updates
//! - `trades.{instrument}` - Live trade executions
//! - `chart.trades.{instrument}.{resolution}` - Aggregated chart data for technical analysis
//!
//! ### User Data Channels (Requires Authentication)
//! - `user.orders` - Order status updates and fills
//! - `user.trades` - User trade executions
//! - `user.changes.{instrument}.{interval}` - Position and portfolio changes
//!
//! ## Protocol Support
//!
//! | Feature | Status | Description |
//! |---------|--------|-------------|
//! | JSON-RPC over WebSocket | ✅ Full Support | Complete JSON-RPC 2.0 implementation |
//! | Market Data Subscriptions | ✅ Full Support | All public channels supported |
//! | User Data Subscriptions | ✅ Full Support | Private channels with authentication |
//! | Chart Data Streaming | ✅ Full Support | Real-time OHLCV data aggregation |
//! | Authentication | ✅ API Key + Signature | Secure credential-based auth |
//! | Connection Management | ✅ Auto-reconnect | Robust connection handling |
//! | Error Recovery | ✅ Comprehensive | Detailed error types and handling |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use deribit_websocket::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize crypto provider for TLS connections
//!     rustls::crypto::aws_lc_rs::default_provider()
//!         .install_default()
//!         .map_err(|_| "Failed to install crypto provider")?;
//!
//!     // Create client for testnet
//!     let config = WebSocketConfig::default();
//!     let mut client = DeribitWebSocketClient::new(&config)?;
//!
//!     // Set up message processing
//!     client.set_message_handler(
//!         |message| {
//!             tracing::info!("Received: {}", message);
//!             Ok(())
//!         },
//!         |message, error| {
//!             tracing::error!("Error processing {}: {}", message, error);
//!         }
//!     );
//!
//!     // Connect and subscribe
//!     client.connect().await?;
//!     client.subscribe(vec!["ticker.BTC-PERPETUAL".to_string()]).await?;
//!
//!     // Start processing messages
//!     client.start_message_processing_loop().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! The client supports advanced subscription patterns for professional trading applications:
//!
//! ### Chart Data Streaming
//! ```rust,no_run
//! # use deribit_websocket::prelude::*;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let config = WebSocketConfig::default();
//! # let client = DeribitWebSocketClient::new(&config)?;
//! // Subscribe to 1-minute chart data for BTC perpetual
//! client.subscribe(vec!["chart.trades.BTC-PERPETUAL.1".to_string()]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Position Change Monitoring
//! ```rust,no_run
//! # use deribit_websocket::prelude::*;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let config = WebSocketConfig::default();
//! # let client = DeribitWebSocketClient::new(&config)?;
//! // Monitor real-time position changes (requires authentication)
//! client.authenticate("client_id", "client_secret").await?;
//! client.subscribe(vec!["user.changes.BTC-PERPETUAL.raw".to_string()]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Mass Quote System
//! ```rust,no_run
//! # use deribit_websocket::prelude::*;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let config = WebSocketConfig::default();
//! # let mut client = DeribitWebSocketClient::new(&config)?;
//! # client.connect().await?;
//! # client.authenticate("client_id", "client_secret").await?;
//! // Set up MMP group for mass quoting
//! let mmp_config = MmpGroupConfig::new(
//!     "btc_market_making".to_string(),
//!     10.0,  // quantity_limit
//!     5.0,   // delta_limit  
//!     1000,  // interval (ms)
//!     5000,  // frozen_time (ms)
//! )?;
//! client.set_mmp_config(mmp_config).await?;
//!
//! // Create and place mass quotes
//! let quotes = vec![
//!     Quote::buy("BTC-PERPETUAL".to_string(), 0.1, 45000.0),
//!     Quote::sell("BTC-PERPETUAL".to_string(), 0.1, 55000.0),
//! ];
//! let request = MassQuoteRequest::new("btc_market_making".to_string(), quotes);
//! let response = client.mass_quote(request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Examples
//!
//! The crate includes comprehensive examples demonstrating:
//! - **`basic_client.rs`** - Basic connection, subscription, and message handling
//! - **`callback_example.rs`** - Advanced callback system with error handling
//! - **`advanced_subscriptions.rs`** - Chart data and position change subscriptions
//! - **`mass_quote_basic.rs`** - Basic mass quoting with MMP group setup
//! - **`mass_quote_advanced.rs`** - Advanced mass quoting with multiple MMP groups and monitoring
//! - **`mass_quote_options.rs`** - Options-specific mass quoting with delta management
//!
//! ## Architecture
//!
//! The client is built with a modular architecture:
//! - **Connection Layer** - Low-level WebSocket connection management
//! - **Session Layer** - Protocol-aware session handling with authentication
//! - **Message Layer** - JSON-RPC request/response and notification handling
//! - **Subscription Layer** - Channel management and subscription tracking
//! - **Callback Layer** - Flexible message processing with error recovery

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
