//! Unit tests for config module

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::constants;
use std::time::Duration;

#[test]
fn test_default_config() {
    let config = unsafe {
        std::env::set_var("DERIBIT_WS_URL", "wss://www.deribit.com/ws/api/v2");
        std::env::remove_var("DERIBIT_TEST_MODE");
        std::env::remove_var("DERIBIT_RECONNECT_DELAY");
        WebSocketConfig::default()
    };

    assert_eq!(config.ws_url.as_str(), "wss://www.deribit.com/ws/api/v2");
    assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    assert_eq!(config.max_reconnect_attempts, 3);
    assert_eq!(config.reconnect_delay, Duration::from_millis(5000));
}

#[test]
fn test_custom_url_config() {
    let custom_url = "wss://custom.example.com/ws";
    let config = WebSocketConfig::with_url(custom_url).unwrap();

    assert_eq!(config.ws_url.as_str(), custom_url);
}

#[test]
fn test_invalid_url_config() {
    let invalid_url = "not-a-valid-url";
    let result = WebSocketConfig::with_url(invalid_url);

    assert!(result.is_err());
}

#[test]
fn test_config_builder_pattern() {
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(Duration::from_secs(60))
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(Duration::from_millis(2000));

    assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
    assert_eq!(config.max_reconnect_attempts, 10);
    assert_eq!(config.reconnect_delay, Duration::from_millis(2000));
}

#[test]
fn test_config_chaining() {
    let config = unsafe {
        std::env::set_var("DERIBIT_WS_URL", "wss://www.deribit.com/ws/api/v2");
        std::env::remove_var("DERIBIT_TEST_MODE");
        std::env::remove_var("DERIBIT_RECONNECT_DELAY");
        WebSocketConfig::default()
    }
    .with_heartbeat_interval(Duration::from_secs(45))
    .with_max_reconnect_attempts(3)
    .with_reconnect_delay(Duration::from_millis(500));

    assert_eq!(config.ws_url.as_str(), "wss://www.deribit.com/ws/api/v2");
    assert_eq!(config.heartbeat_interval, Duration::from_secs(45));
    assert_eq!(config.max_reconnect_attempts, 3);
    assert_eq!(config.reconnect_delay, Duration::from_millis(500));
}

#[test]
fn test_config_clone() {
    let original = WebSocketConfig::default().with_heartbeat_interval(Duration::from_secs(120));

    let cloned = original.clone();

    assert_eq!(original.ws_url, cloned.ws_url);
    assert_eq!(original.heartbeat_interval, cloned.heartbeat_interval);
    assert_eq!(
        original.max_reconnect_attempts,
        cloned.max_reconnect_attempts
    );
    assert_eq!(original.reconnect_delay, cloned.reconnect_delay);
}

#[test]
fn test_config_debug() {
    let config = WebSocketConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("WebSocketConfig"));
    assert!(debug_str.contains("ws_url"));
    assert!(debug_str.contains("heartbeat_interval"));
}

#[test]
fn test_config_with_connection_timeout() {
    let config = WebSocketConfig::default().with_connection_timeout(Duration::from_secs(120));

    assert_eq!(config.connection_timeout, Duration::from_secs(120));
}

#[test]
fn test_config_with_credentials() {
    let config = WebSocketConfig::default()
        .with_credentials("client_id".to_string(), "client_secret".to_string());

    assert_eq!(config.client_id, Some("client_id".to_string()));
    assert_eq!(config.client_secret, Some("client_secret".to_string()));
}

#[test]
fn test_config_with_client_id() {
    let config = WebSocketConfig::default().with_client_id("my_client_id".to_string());

    assert_eq!(config.client_id, Some("my_client_id".to_string()));
}

#[test]
fn test_config_with_client_secret() {
    let config = WebSocketConfig::default().with_client_secret("my_secret".to_string());

    assert_eq!(config.client_secret, Some("my_secret".to_string()));
}

#[test]
fn test_config_with_logging() {
    let config = WebSocketConfig::default().with_logging(true);

    assert!(config.enable_logging);
}

#[test]
fn test_config_with_logging_disabled() {
    let config = WebSocketConfig::default().with_logging(false);

    assert!(!config.enable_logging);
}

#[test]
fn test_config_with_log_level() {
    let config = WebSocketConfig::default().with_log_level("debug".to_string());

    assert_eq!(config.log_level, "debug");
}

#[test]
fn test_config_with_test_mode() {
    let config = WebSocketConfig::default().with_test_mode(true);

    assert!(config.test_mode);
}

#[test]
fn test_config_has_credentials_true() {
    let config = WebSocketConfig::default()
        .with_credentials("client_id".to_string(), "client_secret".to_string());

    assert!(config.has_credentials());
}

#[test]
fn test_config_get_credentials_some() {
    let config = WebSocketConfig::default()
        .with_credentials("client_id".to_string(), "client_secret".to_string());

    let creds = config.get_credentials();
    assert!(creds.is_some());

    let (id, secret) = creds.unwrap();
    assert_eq!(id, "client_id");
    assert_eq!(secret, "client_secret");
}

// =============================================================================
// Fallible constructor + Default fallback tests (Issue #48)
//
// These tests mutate the `DERIBIT_WS_URL` environment variable, so they are
// serialised to prevent cross-test races. The process-global env is restored
// to its original state at the end of each test via a guard.
// =============================================================================

/// RAII guard that restores the previous value of `DERIBIT_WS_URL` on drop
/// so one test's env mutation does not leak into the next one.
struct WsUrlEnvGuard {
    previous: Option<String>,
}

impl WsUrlEnvGuard {
    fn capture() -> Self {
        Self {
            previous: std::env::var("DERIBIT_WS_URL").ok(),
        }
    }

    fn set(value: &str) {
        // SAFETY: tests carrying this guard are `#[serial_test::serial]`, so no
        // other thread in this process mutates the env concurrently.
        unsafe {
            std::env::set_var("DERIBIT_WS_URL", value);
        }
    }
}

impl Drop for WsUrlEnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => {
                // SAFETY: see `WsUrlEnvGuard::set`.
                unsafe {
                    std::env::set_var("DERIBIT_WS_URL", value);
                }
            }
            None => {
                // SAFETY: see `WsUrlEnvGuard::set`.
                unsafe {
                    std::env::remove_var("DERIBIT_WS_URL");
                }
            }
        }
    }
}

#[test]
#[serial_test::serial]
fn test_try_new_returns_ok_with_valid_env_url() {
    let _guard = WsUrlEnvGuard::capture();
    WsUrlEnvGuard::set("wss://custom.example.com/ws");

    let result = WebSocketConfig::try_new();
    assert!(
        result.is_ok(),
        "try_new should succeed with a valid user URL"
    );
    assert_eq!(
        result.unwrap().ws_url.as_str(),
        "wss://custom.example.com/ws"
    );
}

#[test]
#[serial_test::serial]
fn test_try_new_returns_err_on_invalid_env_url() {
    let _guard = WsUrlEnvGuard::capture();
    WsUrlEnvGuard::set("not-a-valid-url");

    let result = WebSocketConfig::try_new();
    assert!(
        result.is_err(),
        "try_new must propagate url::ParseError for invalid env URL, got {result:?}"
    );
}

#[test]
#[serial_test::serial]
fn test_default_falls_back_on_invalid_env_url() {
    let _guard = WsUrlEnvGuard::capture();
    WsUrlEnvGuard::set("not-a-valid-url");

    // Default must NEVER panic; it silently falls back to PRODUCTION_WS_URL.
    let config = WebSocketConfig::default();
    assert_eq!(config.ws_url.as_str(), constants::PRODUCTION_WS_URL);
}

#[test]
#[serial_test::serial]
fn test_with_url_accepts_valid_url() {
    // Regression guard for the `with_url` refactor (issue #48).
    let custom = "wss://custom.example.com/ws";
    let config = WebSocketConfig::with_url(custom).expect("valid URL must parse in with_url");
    assert_eq!(config.ws_url.as_str(), custom);
}

#[test]
#[serial_test::serial]
fn test_with_url_returns_err_on_invalid_url() {
    // Regression guard for the `with_url` refactor (issue #48).
    let result = WebSocketConfig::with_url("not-a-valid-url");
    assert!(
        result.is_err(),
        "with_url must propagate url::ParseError for invalid URL, got {result:?}"
    );
}
