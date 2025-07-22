//! WebSocket message types

use serde::{Deserialize, Serialize};

/// WebSocket request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest {
    /// JSON-RPC version (typically "2.0")
    pub jsonrpc: String,
    /// Request ID for correlation with responses
    pub id: u64,
    /// API method name to call
    pub method: String,
    /// Parameters for the API method
    pub params: serde_json::Value,
}

/// WebSocket response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    /// JSON-RPC version (typically "2.0")
    pub jsonrpc: String,
    /// Request ID for correlation (None for notifications)
    pub id: Option<u64>,
    /// Result data if the request was successful
    pub result: Option<serde_json::Value>,
    /// Error information if the request failed
    pub error: Option<serde_json::Value>,
}
