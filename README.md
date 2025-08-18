<div style="text-align: center;">
<img src="https://raw.githubusercontent.com/joaquinbejar/deribit-websocket/refs/heads/main/doc/images/logo.png" alt="deribit-websocket" style="width: 80%; height: 80%;">
</div>

[![Dual License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/deribit-websocket.svg)](https://crates.io/crates/deribit-websocket)
[![Downloads](https://img.shields.io/crates/d/deribit-websocket.svg)](https://crates.io/crates/deribit-websocket)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/deribit-websocket.svg)](https://github.com/joaquinbejar/deribit-websocket/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/deribit-websocket.svg)](https://github.com/joaquinbejar/deribit-websocket/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/deribit-websocket.svg)](https://github.com/joaquinbejar/deribit-websocket/pulls)
[![Build Status](https://img.shields.io/github/workflow/status/joaquinbejar/deribit-websocket/CI)](https://github.com/joaquinbejar/deribit-websocket/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/deribit-websocket)](https://codecov.io/gh/joaquinbejar/deribit-websocket)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/deribit-websocket)](https://libraries.io/github/joaquinbejar/deribit-websocket)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/deribit-websocket)
[![Wiki](https://img.shields.io/badge/wiki-latest-blue.svg)](https://deepwiki.com/joaquinbejar/deribit-websocket)

## Deribit WebSocket Client

A high-performance, production-ready WebSocket client for the Deribit cryptocurrency derivatives exchange.
This crate provides comprehensive real-time market data streaming, trading operations, and account management
through Deribit's WebSocket API v2.

### Features

- 🔌 **WebSocket Connection Management** - Robust connection handling with automatic reconnection and heartbeat
- 📡 **JSON-RPC Protocol** - Complete JSON-RPC 2.0 implementation for Deribit API
- 📊 **Real-time Market Data** - Live ticker, order book, trades, and chart data streaming
- 📈 **Advanced Subscriptions** - Chart data aggregation and user position change notifications
- 🔐 **Authentication** - Secure API key and signature-based authentication
- 🛡️ **Error Handling** - Comprehensive error types with detailed recovery mechanisms
- ⚡ **Async/Await** - Full async support with tokio runtime for high concurrency
- 🔄 **Callback System** - Flexible message processing with primary and error callbacks
- 📋 **Subscription Management** - Intelligent subscription tracking and channel management
- 🧪 **Testing Support** - Complete test coverage with working examples

### Supported Subscription Channels

#### Market Data Channels
- `ticker.{instrument}` - Real-time ticker updates
- `book.{instrument}.{group}` - Order book snapshots and updates
- `trades.{instrument}` - Live trade executions
- `chart.trades.{instrument}.{resolution}` - Aggregated chart data for technical analysis

#### User Data Channels (Requires Authentication)
- `user.orders` - Order status updates and fills
- `user.trades` - User trade executions
- `user.changes.{instrument}.{interval}` - Position and portfolio changes

### Protocol Support

| Feature | Status | Description |
|---------|--------|-------------|
| JSON-RPC over WebSocket | ✅ Full Support | Complete JSON-RPC 2.0 implementation |
| Market Data Subscriptions | ✅ Full Support | All public channels supported |
| User Data Subscriptions | ✅ Full Support | Private channels with authentication |
| Chart Data Streaming | ✅ Full Support | Real-time OHLCV data aggregation |
| Authentication | ✅ API Key + Signature | Secure credential-based auth |
| Connection Management | ✅ Auto-reconnect | Robust connection handling |
| Error Recovery | ✅ Comprehensive | Detailed error types and handling |

### Quick Start

```rust
use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for TLS connections
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Create client for testnet
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message processing
    client.set_message_handler(
        |message| {
            tracing::info!("Received: {}", message);
            Ok(())
        },
        |message, error| {
            tracing::error!("Error processing {}: {}", message, error);
        }
    );

    // Connect and subscribe
    client.connect().await?;
    client.subscribe(vec!["ticker.BTC-PERPETUAL".to_string()]).await?;

    // Start processing messages
    client.start_message_processing_loop().await?;
    Ok(())
}
```

### Advanced Usage

The client supports advanced subscription patterns for professional trading applications:

#### Chart Data Streaming
```rust
// Subscribe to 1-minute chart data for BTC perpetual
client.subscribe(vec!["chart.trades.BTC-PERPETUAL.1".to_string()]).await?;
```

#### Position Change Monitoring
```rust
// Monitor real-time position changes (requires authentication)
client.authenticate("client_id", "client_secret").await?;
client.subscribe(vec!["user.changes.BTC-PERPETUAL.raw".to_string()]).await?;
```

### Examples

The crate includes comprehensive examples demonstrating:
- **`basic_client.rs`** - Basic connection, subscription, and message handling
- **`callback_example.rs`** - Advanced callback system with error handling
- **`advanced_subscriptions.rs`** - Chart data and position change subscriptions

### Architecture

The client is built with a modular architecture:
- **Connection Layer** - Low-level WebSocket connection management
- **Session Layer** - Protocol-aware session handling with authentication
- **Message Layer** - JSON-RPC request/response and notification handling
- **Subscription Layer** - Channel management and subscription tracking
- **Callback Layer** - Flexible message processing with error recovery

## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:

**Joaquin Bejar Garcia**
- Email: jb@taunais.com
- GitHub: [joaquinbejar](https://github.com/joaquinbejar)

We appreciate your interest and look forward to your contributions!

## ✍️ License

Licensed under MIT license

## Disclaimer

This software is not officially associated with Deribit. Trading financial instruments carries risk, and this library is provided as-is without any guarantees. Always test thoroughly with a demo account before using in a live trading environment.
