//! Shared helpers for crate consumers.
//!
//! Currently a single entry point: [`crate::utils::setup_logger`],
//! which installs a `tracing` subscriber driven by the
//! `DERIBIT_LOG_LEVEL` environment variable. The helper is re-exported
//! from [`crate::prelude`] so `use deribit_websocket::prelude::*;`
//! makes it available without an explicit path.
//!
//! # Example
//!
//! ```rust,no_run
//! use deribit_websocket::utils::setup_logger;
//!
//! // Default level is INFO; override via env var before the first call.
//! unsafe {
//!     std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
//! }
//! setup_logger();
//! tracing::debug!("logger online");
//! ```

mod logger;

pub use logger::setup_logger;
