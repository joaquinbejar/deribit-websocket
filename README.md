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

This crate provides a comprehensive WebSocket client for the Deribit trading platform.
It implements JSON-RPC over WebSocket with full support for:

### Features

- 🔌 **WebSocket Connection Management** - Robust connection handling with automatic reconnection
- 📡 **JSON-RPC Protocol** - Complete JSON-RPC 2.0 implementation for Deribit API
- 📊 **Real-time Subscriptions** - Market data, order updates, and trade notifications
- 🔐 **Authentication** - Support for API key and signature-based authentication
- 🛡️ **Error Handling** - Comprehensive error types and recovery mechanisms
- ⚡ **Async/Await** - Full async support with tokio runtime
- 📈 **Rate Limiting** - Built-in rate limiting to comply with Deribit API limits
- 🧪 **Testing Support** - Complete test coverage and examples

### Protocol Support

| Feature | Status |
|---------|--------|
| JSON-RPC over WebSocket | ✅ Full Support |
| Real-time Subscriptions | ✅ Full Support |
| Authentication | ✅ API Key + Signature |
| Market Data | ✅ All Channels |
| Trading Operations | ✅ Orders, Positions |
| Account Management | ✅ Portfolio, Balances |

### Usage

The WebSocket client provides callback-based message handling where incoming messages
are processed with a primary callback that returns a Result, and an error callback
that handles any errors from the primary callback.

Basic usage involves:
1. Creating a WebSocket configuration (testnet or production)
2. Creating a client instance with the configuration
3. Setting up message and error callbacks for processing
4. Connecting to the Deribit WebSocket API
5. Subscribing to desired channels (market data, user data)
6. Starting the message processing loop

See the examples directory for complete working examples including:
- Basic client usage with market data subscriptions
- Callback-based message handling with error recovery
- Authentication and user-specific data subscriptions

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
