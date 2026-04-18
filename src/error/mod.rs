//! Error handling module for WebSocket client

/// WebSocket-specific errors
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection failed: {0}")]
    /// Connection failed with error message
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    /// Authentication failed with error message
    AuthenticationFailed(String),

    #[error("Subscription failed: {0}")]
    /// Subscription failed with error message
    SubscriptionFailed(String),

    #[error("Invalid message format: {0}")]
    /// Invalid message format
    InvalidMessage(String),

    #[error("Connection closed unexpectedly")]
    /// Connection was closed
    ConnectionClosed,

    #[error("Heartbeat timeout")]
    /// Heartbeat timeout occurred
    HeartbeatTimeout,

    #[error("API error {0}: {1}")]
    /// API error with code and message
    ApiError(i32, String),

    #[error("Operation timed out: {0}")]
    /// Operation timed out (e.g., `send_request` awaiting a matching response)
    Timeout(String),

    #[error("Dispatcher task is not running")]
    /// The background dispatcher task is not running (never started, shut
    /// down, or panicked). No further I/O can be performed through it.
    DispatcherDead,

    #[error("Serialization error: {0}")]
    /// JSON serialization or deserialization failed.
    ///
    /// Typically raised when a request contains a numeric field whose value
    /// cannot be represented in JSON (e.g. `NaN` or `Infinity` in an `f64`),
    /// or when parsing a malformed response payload.
    Serialization(#[from] serde_json::Error),
}
