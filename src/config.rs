//! Configuration for WebSocket client

use std::env;
use std::time::Duration;
use url::Url;

use crate::constants;

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL
    pub ws_url: Url,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Maximum reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection delay
    pub reconnect_delay: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Enable logging
    pub enable_logging: bool,
    /// Log level
    pub log_level: String,
    /// Test mode
    pub test_mode: bool,
    /// Client ID for authentication
    pub client_id: Option<String>,
    /// Client secret for authentication
    pub client_secret: Option<String>,
    /// Per-request timeout for [`DeribitWebSocketClient::send_request`].
    ///
    /// Every call that awaits a matching JSON-RPC response is bounded by
    /// this duration. If the response does not arrive in time, the call
    /// returns [`WebSocketError::Timeout`].
    pub request_timeout: Duration,
    /// Notification channel capacity (frames buffered for the consumer).
    ///
    /// Depth of the bounded `tokio::sync::mpsc` that carries server-pushed
    /// notifications (and any unmatched frames) from the dispatcher task
    /// to [`DeribitWebSocketClient::receive_message`] /
    /// `start_message_processing_loop`.
    ///
    /// # Backpressure — Strategy A (await-send)
    ///
    /// When the channel is full the dispatcher task blocks on
    /// `send().await`; it therefore stops polling the WebSocket stream,
    /// the TCP recv buffer fills, and the Deribit server applies flow
    /// control. **No frames are ever dropped.** Every full-channel event
    /// emits a `tracing::warn!` with the channel capacity so slow
    /// consumers are visible in logs.
    ///
    /// Sizing: the default of `1024` is sufficient for normal liquid
    /// instruments. Raise it when the consumer performs heavy synchronous
    /// work between `next_notification` calls; lower it to tighten
    /// end-to-end memory bounds at the cost of more frequent
    /// backpressure warnings.
    pub notification_channel_capacity: usize,
    /// Dispatcher command channel capacity (in-flight outbound commands).
    ///
    /// Depth of the bounded `tokio::sync::mpsc` that carries outbound
    /// commands (request sends, cancel-request on timeout, shutdown)
    /// from callers to the dispatcher task.
    ///
    /// # Backpressure — Strategy A (await-send)
    ///
    /// When the channel is full, callers of
    /// [`DeribitWebSocketClient::send_request`] /
    /// [`DeribitWebSocketClient::disconnect`] block on `send().await`
    /// until the dispatcher drains a slot. Blocking here means the
    /// application is issuing requests faster than the dispatcher can
    /// write them to the socket; the `request_timeout` bound on
    /// `send_request` still applies, so the caller sees a
    /// [`WebSocketError::Timeout`] if the deadline elapses while
    /// waiting on the command channel.
    pub dispatcher_command_capacity: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self::try_new().unwrap_or_else(|_| {
            // Reached only when a user-supplied `DERIBIT_WS_URL` is invalid.
            // Fall back to the compile-time constant
            // [`constants::PRODUCTION_WS_URL`], whose parse-ability is locked
            // by the `test_production_ws_url_parses` unit test — making this
            // branch unreachable in practice.
            #[allow(
                clippy::expect_used,
                reason = "PRODUCTION_WS_URL is a compile-time constant validated by test_production_ws_url_parses"
            )]
            let ws_url = Url::parse(constants::PRODUCTION_WS_URL)
                .expect("PRODUCTION_WS_URL is a compile-time constant validated by tests");
            Self::from_parts(ws_url)
        })
    }
}

impl WebSocketConfig {
    /// Construct a configuration from environment variables, propagating
    /// parse errors for any user-supplied URL.
    ///
    /// Loads `.env` once via [`Self::load_env`], reads `DERIBIT_WS_URL`
    /// (falling back to [`constants::PRODUCTION_WS_URL`] when unset), and
    /// parses it. All other fields follow the same env-or-default strategy as
    /// [`Default`] but never fail.
    ///
    /// Prefer this over [`Default::default`] when the caller needs to surface
    /// an invalid `DERIBIT_WS_URL` as an error instead of silently falling
    /// back to the production URL.
    ///
    /// # Errors
    ///
    /// Returns [`url::ParseError`] when `DERIBIT_WS_URL` is set to a value
    /// that cannot be parsed as a URL.
    pub fn try_new() -> Result<Self, url::ParseError> {
        Self::load_env();
        let ws_url_str =
            env::var("DERIBIT_WS_URL").unwrap_or_else(|_| constants::PRODUCTION_WS_URL.to_string());
        let ws_url = Url::parse(&ws_url_str)?;
        Ok(Self::from_parts(ws_url))
    }

    /// Create a new configuration with a custom URL.
    ///
    /// Non-URL fields are populated from environment variables using the
    /// same rules as [`Default`]; only the URL is overridden. `.env` is
    /// loaded once via [`Self::load_env`] before any env var is read.
    ///
    /// # Errors
    ///
    /// Returns [`url::ParseError`] when `url` cannot be parsed.
    pub fn with_url(url: &str) -> Result<Self, url::ParseError> {
        Self::load_env();
        let ws_url = Url::parse(url)?;
        Ok(Self::from_parts(ws_url))
    }

    /// Centralised `.env` loader for every public constructor.
    ///
    /// Idempotent and harmless when called multiple times per process. Every
    /// public entry point ([`Self::try_new`], [`Self::with_url`], and
    /// [`Default::default`] via `try_new`) calls this exactly once before
    /// reading any env var, so [`Self::from_parts`] can assume the environment
    /// is already loaded.
    fn load_env() {
        let _ = dotenv::dotenv();
    }

    /// Private helper: populate every field except `ws_url` from environment
    /// variables (with sensible defaults) and combine them with the given URL.
    ///
    /// The caller is responsible for calling [`Self::load_env`] beforehand so
    /// that `.env` overrides are visible to `std::env::var`. Every public
    /// constructor satisfies this invariant.
    fn from_parts(ws_url: Url) -> Self {
        let heartbeat_interval = env::var("DERIBIT_HEARTBEAT_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(30));

        let max_reconnect_attempts = env::var("DERIBIT_RECONNECT_ATTEMPTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let reconnect_delay = env::var("DERIBIT_RECONNECT_DELAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(5));

        let connection_timeout = env::var("DERIBIT_CONNECTION_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(10));

        let enable_logging = env::var("DERIBIT_ENABLE_LOGGING")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let log_level = env::var("DERIBIT_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let test_mode = env::var("DERIBIT_TEST_MODE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let client_id = env::var("DERIBIT_CLIENT_ID").ok();
        let client_secret = env::var("DERIBIT_CLIENT_SECRET").ok();

        let request_timeout = env::var("DERIBIT_REQUEST_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(30));

        let notification_channel_capacity = env::var("DERIBIT_NOTIFICATION_CAPACITY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024);

        let dispatcher_command_capacity = env::var("DERIBIT_DISPATCHER_CAPACITY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(64);

        Self {
            ws_url,
            heartbeat_interval,
            max_reconnect_attempts,
            reconnect_delay,
            connection_timeout,
            enable_logging,
            log_level,
            test_mode,
            client_id,
            client_secret,
            request_timeout,
            notification_channel_capacity,
            dispatcher_command_capacity,
        }
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set maximum reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Set reconnection delay
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set client credentials
    pub fn with_credentials(mut self, client_id: String, client_secret: String) -> Self {
        self.client_id = Some(client_id);
        self.client_secret = Some(client_secret);
        self
    }

    /// Set client ID
    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Set client secret
    pub fn with_client_secret(mut self, client_secret: String) -> Self {
        self.client_secret = Some(client_secret);
        self
    }

    /// Enable or disable logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Set log level
    pub fn with_log_level(mut self, level: String) -> Self {
        self.log_level = level;
        self
    }

    /// Set test mode
    pub fn with_test_mode(mut self, test_mode: bool) -> Self {
        self.test_mode = test_mode;
        self
    }

    /// Check if credentials are available
    pub fn has_credentials(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }

    /// Get client credentials as tuple
    pub fn get_credentials(&self) -> Option<(&str, &str)> {
        match (&self.client_id, &self.client_secret) {
            (Some(id), Some(secret)) => Some((id, secret)),
            _ => None,
        }
    }

    /// Set the per-request timeout awaiting a matching JSON-RPC response.
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the notification channel capacity.
    ///
    /// This bounds the number of server-pushed frames buffered between the
    /// dispatcher task and the consumer.
    #[must_use]
    pub fn with_notification_channel_capacity(mut self, capacity: usize) -> Self {
        self.notification_channel_capacity = capacity;
        self
    }

    /// Set the dispatcher command channel capacity.
    ///
    /// Caps the number of outbound commands queued waiting for the
    /// dispatcher task to process them.
    #[must_use]
    pub fn with_dispatcher_command_capacity(mut self, capacity: usize) -> Self {
        self.dispatcher_command_capacity = capacity;
        self
    }
}
