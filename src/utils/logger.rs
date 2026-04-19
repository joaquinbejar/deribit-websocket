//! `tracing` subscriber setup, driven by `DERIBIT_LOG_LEVEL`.
//!
//! Installs a [`FmtSubscriber`] as the process-global default. The
//! level comes from the `DERIBIT_LOG_LEVEL` environment variable read
//! on the first call; subsequent calls are no-ops.

use std::env;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static INIT: Once = Once::new();

/// Install the `tracing` subscriber used by every example and binary
/// in the crate.
///
/// The level is read from the `DERIBIT_LOG_LEVEL` environment variable
/// once, on the first call. Recognised values: `TRACE`, `DEBUG`,
/// `INFO`, `WARN`, `ERROR`. Anything else (or an unset variable)
/// defaults to `INFO`.
///
/// The subscriber is registered as the *process-global* default via
/// [`tracing::subscriber::set_global_default`], which `tracing` allows
/// only once per process. A [`Once`] guard makes subsequent calls to
/// this function safe: they return immediately without touching the
/// already-installed subscriber, so applications that wire logging
/// from multiple entry points (tests, libs, `main`) do not race.
///
/// Changing `DERIBIT_LOG_LEVEL` after the first call has no effect —
/// tracing does not support replacing the global default at runtime.
///
/// # Example
///
/// ```rust,no_run
/// use deribit_websocket::utils::setup_logger;
///
/// // Set log level via environment variable (unsafe in Rust 2024 edition)
/// unsafe {
///     std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
/// }
///
/// // Initialize the logger
/// setup_logger();
///
/// // Now you can use tracing macros
/// tracing::info!("Logger initialized");
/// ```
pub fn setup_logger() {
    INIT.call_once(|| {
        let log_level = env::var("DERIBIT_LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".to_string())
            .to_uppercase();

        let level = match log_level.as_str() {
            "DEBUG" => Level::DEBUG,
            "ERROR" => Level::ERROR,
            "WARN" => Level::WARN,
            "TRACE" => Level::TRACE,
            _ => Level::INFO,
        };

        let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

        if tracing::subscriber::set_global_default(subscriber).is_ok() {
            tracing::debug!("Log level set to: {}", level);
        }
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_logger_can_be_called_multiple_times() {
        // First call
        setup_logger();
        // Second call should not panic
        setup_logger();
    }
}
