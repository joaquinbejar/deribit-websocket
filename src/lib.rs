//! # Deribit WebSocket Client
//!
//! This crate provides a WebSocket client for the Deribit trading platform.
//! It implements the common traits from `deribit-base` and provides real-time
//! market data and trading functionality through WebSocket connections.

pub mod client;
pub mod messages;
pub mod subscriptions;

pub use client::*;
pub use messages::*;
pub use subscriptions::*;

// Re-export common types from deribit-base
pub use deribit_base::*;
