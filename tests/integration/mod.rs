//! Integration tests for deribit-websocket
//!
//! These tests verify the complete functionality of WebSocket connection,
//! authentication, subscription management, and message handling.
//!
//! Note: Integration tests require valid Deribit API credentials and are only
//! enabled when the `integration-tests` feature is active.

#[cfg(feature = "integration-tests")]
pub mod authentication;
#[cfg(feature = "integration-tests")]
pub mod connection;
#[cfg(feature = "integration-tests")]
pub mod error_handling;
#[cfg(feature = "integration-tests")]
pub mod market_data;
#[cfg(feature = "integration-tests")]
pub mod subscriptions;
