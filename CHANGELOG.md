# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD pipeline (build, lint, test, coverage, semver)
- Dependabot configuration for automated dependency updates
- Local `utils` module with `setup_logger()` function
- Trading methods over WebSocket: `buy()`, `sell()`, `cancel()`, `cancel_all()`, `cancel_all_by_currency()`, `cancel_all_by_instrument()`, `edit()`
- New trading types: `OrderRequest`, `EditOrderRequest`, `OrderResponse`, `OrderInfo`, `OrderType`, `TimeInForce`, `Trigger`, `TradeExecution`
- Request builders for all trading operations
- Comprehensive unit tests for trading module (36 tests)
- `Unknown` variant to `SubscriptionChannel` for unrecognized channel patterns
- `is_unknown()` helper method on `SubscriptionChannel`
- Comprehensive tests for all subscription channel parsing patterns (45 tests)

### Fixed
- `parse_channel_type()` now correctly handles all 14 `SubscriptionChannel` variants instead of only 5
- Unknown channels no longer incorrectly default to `Ticker(String::new())`

### Deprecated
- `subscriptions::SubscriptionChannel` - use `model::SubscriptionChannel` instead for full channel support

### Removed
- **BREAKING**: Removed `deribit-base` dependency - crate is now fully self-contained
- Removed `pub use deribit_base;` re-export from `lib.rs`
- Removed `pub use deribit_base::prelude::*;` from `prelude.rs`

### Changed
- Added `tracing-subscriber` dependency for logger functionality

## [0.1.2] - 2024-03-06

### Changed
- Replace `deribit_base` macros with `pretty-simple-display` derive macros
- Update dependency versions to latest stable releases
  - tokio: 1.0 → 1.50
  - tokio-tungstenite: 0.27 → 0.28
  - deribit-base: 0.2 → 0.2.6
  - rustls: 0.23 → 0.23.31
  - thiserror: 2.0 → 2.0.18

## [0.1.1] - 2024-03-05

### Changed
- Bump `deribit-base` dependency to 0.2
- Remove commented-out workspace dependency

### Fixed
- Mass quote validation logic improvements
- WebSocket client resilience enhancements

## [0.1.0] - 2024-03-04

### Added
- Initial release
- WebSocket client for Deribit API real-time data
- JSON-RPC 2.0 message handling
- Subscription management for market data channels
- Mass quote operations support
- MMP (Market Maker Protection) group management
- Callback-based notification handling
- TLS/SSL secure connections via rustls
- Async/await support with Tokio runtime
- Comprehensive error handling
- Example implementations for common use cases

[Unreleased]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/joaquinbejar/deribit-websocket/releases/tag/v0.1.0
