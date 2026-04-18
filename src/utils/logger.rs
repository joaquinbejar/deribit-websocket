//! Logger setup utility for the deribit-websocket crate
//!
//! Provides a simple logger configuration based on environment variables.

use std::env;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static INIT: Once = Once::new();

/// Sets up the logger for the application.
///
/// The logger level is determined by the `DERIBIT_LOG_LEVEL` environment variable.
/// If the variable is not set, it defaults to `INFO`.
///
/// This function is safe to call multiple times - it will only initialize
/// the logger once.
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
