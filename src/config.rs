//! Configuration for WebSocket client

use std::time::Duration;
use url::Url;

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
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            ws_url: Url::parse("wss://test.deribit.com/ws/api/v2").unwrap(),
            heartbeat_interval: Duration::from_secs(30),
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_millis(1000),
        }
    }
}

impl WebSocketConfig {
    /// Create a new configuration for testnet
    pub fn testnet() -> Self {
        Self {
            ws_url: Url::parse("wss://test.deribit.com/ws/api/v2").unwrap(),
            ..Default::default()
        }
    }

    /// Create a new configuration for production
    pub fn production() -> Self {
        Self {
            ws_url: Url::parse("wss://www.deribit.com/ws/api/v2").unwrap(),
            ..Default::default()
        }
    }

    /// Create a new configuration with custom URL
    pub fn with_url(url: &str) -> Result<Self, url::ParseError> {
        Ok(Self {
            ws_url: Url::parse(url)?,
            ..Default::default()
        })
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
}
