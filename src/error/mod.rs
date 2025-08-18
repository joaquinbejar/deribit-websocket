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
}
