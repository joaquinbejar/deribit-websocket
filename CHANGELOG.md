# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD pipeline (build, lint, test, coverage, semver)
- Dependabot configuration for automated dependency updates

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
