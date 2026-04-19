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
//! - 🔐 **Authentication** - Secure API key and signature-based authentication with typed responses
//! - 📝 **Trading Operations** - Full order lifecycle: buy, sell, cancel, edit orders
//! - 💼 **Account Management** - Position queries, account summaries, order history
//! - 🔄 **Session Management** - Heartbeat control, client identification, cancel-on-disconnect
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
//!     // Install the rustls crypto provider that matches the active TLS feature.
//!     // See the crate-level "TLS backends" section or `Cargo features` in the
//!     // README for the available backends.
//!     deribit_websocket::install_default_crypto_provider()?;
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
//!
//! ### Core Examples
//! - **`basic_client.rs`** - Basic connection, subscription, and message handling
//! - **`callback_example.rs`** - Advanced callback system with error handling
//! - **`advanced_subscriptions.rs`** - Chart data and position change subscriptions
//!
//! ### Trading & Account Management (v0.2.0)
//! - **`trading_operations.rs`** - Buy, sell, cancel, edit orders
//! - **`account_operations.rs`** - Get positions, account summary, order history
//! - **`position_management.rs`** - Close positions, move positions between subaccounts
//!
//! ### Session Management (v0.2.0)
//! - **`session_management.rs`** - Hello, heartbeat, typed responses (AuthResponse, HelloResponse, TestResponse)
//! - **`cancel_on_disconnect.rs`** - Enable/disable/get cancel-on-disconnect status
//! - **`unsubscribe_all.rs`** - Public and private unsubscribe_all operations
//!
//! ### Market Data Subscriptions
//! - **`new_channels_subscription.rs`** - Grouped order book, incremental ticker, trades by kind
//! - **`perpetual_subscription.rs`** - Perpetual funding rate subscriptions
//! - **`quote_subscription.rs`** - Quote data subscriptions
//! - **`price_index_subscription.rs`** - Price index subscriptions
//!
//! ### Mass Quoting
//! - **`mass_quote_basic.rs`** - Basic mass quoting with MMP group setup
//! - **`mass_quote_advanced.rs`** - Advanced mass quoting with multiple MMP groups
//! - **`mass_quote_options.rs`** - Options-specific mass quoting with delta management
//!
//! ## Timeouts
//!
//! Two deadlines bound the most common sources of indefinite hangs in a
//! network client. Both live on `WebSocketConfig` and can be set via
//! builder methods or the corresponding environment variables:
//!
//! - **`connection_timeout`** (default 10s, env `DERIBIT_CONNECTION_TIMEOUT`) —
//!   upper bound on the WebSocket handshake (TCP + TLS + HTTP upgrade).
//!   A peer that accepts the TCP connection but never completes the
//!   upgrade makes `DeribitWebSocketClient::connect` / `Dispatcher::connect`
//!   fail with `WebSocketError::Timeout` instead of hanging.
//! - **`request_timeout`** (default 30s, env `DERIBIT_REQUEST_TIMEOUT`) —
//!   upper bound on each `send_request` call, covering enqueue, write,
//!   and response wait. On the deadline the dispatcher evicts the
//!   now-orphaned waiter so the id-map stays small under repeated
//!   timeouts.
//!
//! Planned follow-ups: `read_idle_timeout` (maximum gap between frames)
//! and granular per-operation overrides.
//!
//! ## Backpressure
//!
//! The client and dispatcher communicate over two **bounded**
//! `tokio::sync::mpsc` channels, both using **Strategy A (await-send)** —
//! the producer blocks on a full channel, so frames are not dropped due
//! to backpressure. Frames can still be discarded if the notification
//! receiver has already been closed (for example during shutdown or
//! disconnect).
//!
//! - **`notification_channel_capacity`** (default 1024) — notifications
//!   from the dispatcher to the consumer. When full, the dispatcher
//!   stops polling the WebSocket stream and the TCP recv buffer fills,
//!   which makes the Deribit server apply flow control. Every
//!   full-channel event emits a `tracing::warn!` so slow consumers are
//!   visible in logs.
//! - **`dispatcher_command_capacity`** — outbound commands from the
//!   client to the dispatcher (request sends, cancel-request on timeout,
//!   shutdown). When full, the caller blocks until the dispatcher drains
//!   a slot; `request_timeout` on `send_request` still applies, so the
//!   caller surfaces `WebSocketError::Timeout` if the deadline elapses
//!   while waiting on the channel.
//!
//! Strategy A was chosen over drop-oldest / drop-newest variants because
//! the notification stream carries private trading events (order
//! updates, trade reports) where silent loss is unacceptable.
//!
//! ## Architecture
//!
//! The client is built with a modular architecture:
//! - **Connection Layer** - Low-level WebSocket connection management
//! - **Session Layer** - Protocol-aware session handling with authentication
//! - **Message Layer** - JSON-RPC request/response and notification handling
//! - **Subscription Layer** - Channel management and subscription tracking
//! - **Callback Layer** - Flexible message processing with error recovery
//!
//! ## TLS backends
//!
//! `deribit-websocket` exposes three mutually-exclusive TLS backends as
//! Cargo features, with a compile-time mutex (see the `tls` module)
//! that rejects any other combination:
//!
//! | Feature          | Default | Behaviour                                                          |
//! | ---------------- | :-----: | ------------------------------------------------------------------ |
//! | `rustls-aws-lc`  | ✅      | `rustls` with the `aws-lc-rs` crypto provider + OS root store      |
//! | `rustls-ring`    |         | `rustls` with the `ring` crypto provider + OS root store           |
//! | `native-tls`     |         | OS-native TLS stack (SChannel / SecureTransport / OpenSSL)         |
//!
//! Selecting a non-default backend:
//!
//! ```toml
//! # Cargo.toml
//! [dependencies]
//! deribit-websocket = { version = "0.2", default-features = false, features = ["rustls-ring"] }
//! ```
//!
//! or, from the command line:
//!
//! ```sh
//! cargo add deribit-websocket --no-default-features --features native-tls
//! ```
//!
//! Applications must call `install_default_crypto_provider` once at
//! startup — it picks the right provider for the active feature and is
//! a no-op under `native-tls`.
//!
//! Because both `rustls-*` backends use the OS-native root store via
//! `rustls-native-certs`, minimal container images (Alpine, distroless)
//! must have `ca-certificates` (or equivalent) installed so the trust
//! store is populated.

#![warn(missing_docs)]
#![deny(unsafe_code)]
// Regression guard against future `std::sync::Mutex` use across `.await`.
// Tokio's mutex (which this crate uses) is intentionally allowed.
#![warn(clippy::await_holding_lock)]
// Ban `.unwrap()`/`.expect()` in library code. `#[cfg(test)]` modules inside
// `src/` keep the default behaviour so existing unit tests continue to work.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

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
pub mod tls;
/// Utility functions and helpers
pub mod utils;

pub use tls::{CryptoProviderError, install_default_crypto_provider};
