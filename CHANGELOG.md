# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-04-19

### Migration note

The default TLS backend changed from `native-tls` to `rustls-aws-lc`. Downstream crates that relied on the previous default must now either accept the new default or opt back in explicitly:

```toml
deribit-websocket = { version = "0.3.0", default-features = false, features = ["native-tls"] }
```

Every binary that opens a WebSocket connection must also call `deribit_websocket::install_default_crypto_provider()` once at startup, before the first `connect()`. This is a no-op under `native-tls` and selects the right rustls provider under the two rustls backends. See the "TLS backend selection" section of the README.

### Added
- TLS backend feature flags `rustls-aws-lc` (new default), `rustls-ring`, and `native-tls`, mutually exclusive, enforced at compile time by a `compile_error!` mutex that rejects multi-backend builds (#55, PR #70).
- `install_default_crypto_provider()` helper that idempotently installs the correct rustls provider for the active feature and is a no-op under `native-tls` (#55).
- `WebSocketConfig::try_new()` fallible constructor that propagates `DERIBIT_WS_URL` parse errors instead of panicking (#48).
- Mock-server integration test suite covering connect, authenticate, subscribe/unsubscribe lifecycles and reconnect scenarios end-to-end (#53).
- `WebSocketError::ApiError` now carries the originating request method, params, and a size-bounded redacted raw response payload for post-mortem debugging (#52).
- Extended rustdoc on `message::builder`, `utils`, `utils::logger`, and the `prelude` module header: every public item has substantive docs with `# Errors` sections where applicable; prelude has a "what you get / what you do not get" checklist (#56).
- `#![warn(rustdoc::broken_intra_doc_links)]` and `#![warn(rustdoc::missing_crate_level_docs)]` in `lib.rs`, enforced in CI via `RUSTDOCFLAGS="-D warnings" cargo doc` (#56).
- CI matrix builds every example and every test target under each of the three TLS backends via `cargo build --all-targets --no-default-features --features <backend>` (#54).
- CI `mutex-check` negative job that confirms the `compile_error!` mutex still rejects builds activating two TLS backends simultaneously (#55).
- `#[must_use]` annotations on pure functions and builder returns across the message-builder surface per project guidelines (#56).
- README sections documenting the TLS backend selection, the `install_default_crypto_provider()` helper, connection/request timeouts, and backpressure strategy.

### Changed
- **Default TLS backend is now `rustls-aws-lc`** — was `native-tls`. See the migration note above (#55, PR #70).
- `WebSocketSession::new` now accepts `impl Into<Arc<WebSocketConfig>>` so both owned `WebSocketConfig` (backward compatible) and `Arc<WebSocketConfig>` (zero-copy) callers compile. `DeribitWebSocketClient::new` uses the zero-copy path internally, eliminating the duplicate struct clone that used to happen on every client construction (#57).
- `WebSocketConfig::default()` no longer panics if `DERIBIT_WS_URL` is malformed — it falls back to a hard-coded production-URL safe default, keeping the fallible-but-panicking behaviour inside `try_new` only (#48).
- `DeribitWebSocketClient::connect()` now bounds the WebSocket handshake by the configured `connection_timeout`; a stalled listener during the TLS/HTTP upgrade surfaces `WebSocketError::Timeout` instead of hanging (#50).
- `subscribe()` / `unsubscribe()` now reconcile the active-subscription set against the channel list echoed by the server rather than the caller's input list; rejected channels no longer poison local state (#62, #65).
- Notification channel backpressure is explicitly documented as Strategy A (await-send) — the dispatcher blocks on full, stops polling the socket, and emits a `tracing::warn!` per full-channel event; no frames are dropped unless the notification receiver has been closed (#51).
- `send_request` borrows its request on the hot path instead of cloning it; one allocation removed per outbound request (#52).

### Fixed
- Serde errors thrown while constructing JSON-RPC payloads now propagate as `WebSocketError::Serialization` with context instead of being stringified via `.to_string()` (#46).
- MMP config builder rejects non-finite floats (`NaN`, `±∞`) up front with a clear validation error rather than sending them over the wire (#46).
- TLS provider installation is idempotent and returns a typed `CryptoProviderError::AlreadyInstalled` rather than panicking when called a second time, so libraries that wire logging and TLS from multiple entry points no longer race (#55).
- Three examples (`new_channels_subscription`, `mass_quote_advanced`, `mass_quote_options`) degraded gracefully on accounts without authenticated raw-stream access or without Market Maker Protection activated; previously they crashed on the first `11050` or `13778` error.

### Documentation
- Expanded crate-level `lib.rs` header with a backpressure architecture section.
- Audited public items in `message::builder`, `utils`, and `prelude` to remove signature-restating one-liners and add examples + `# Errors` sections where meaningful (#56).

### CI
- New `rustdoc (-D warnings)` job enforcing the two new rustdoc lints.
- Existing TLS backend matrix upgraded from `cargo build` (lib only) to `cargo build --all-targets` so every example and test target is compiled under every backend on every PR.
- Negative `mutex-check` job verifies the `compile_error!` barrier in `src/tls.rs`.

## [0.2.1] - 2026-03-07

### Added
- Extended prelude with `ws_types` exports: `AuthResponse`, `ConnectionState`, `HeartbeatStatus`, `HelloResponse`, `JsonRpcError`, `JsonRpcNotification`, `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcResult`, `TestResponse`, `WebSocketMessage`

### Changed
- Updated Rust edition to 2024

## [0.2.0] - 2026-03-07

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
- Account methods over WebSocket: `get_positions()`, `get_account_summary()`, `get_order_state()`, `get_order_history_by_currency()`
- Account types: `Position`, `AccountSummary`, `CurrencySummary`, `Direction`
- Request builders for all account operations
- Comprehensive unit tests for account module (25+ tests)
- Position management methods over WebSocket: `close_position()`, `move_positions()`
- Position types: `ClosePositionResponse`, `CloseTrade`, `CloseOrder`, `MovePositionTrade`, `MovePositionResult`
- Request builders for position management operations
- Comprehensive unit tests for position module (25+ tests)
- Subscription management methods: `public_unsubscribe_all()`, `private_unsubscribe_all()`
- Request builders for unsubscribe_all operations
- Unit tests for unsubscribe_all and SubscriptionManager.clear()

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

[0.3.0]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/joaquinbejar/deribit-websocket/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/joaquinbejar/deribit-websocket/releases/tag/v0.1.0
