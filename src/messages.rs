//! WebSocket message types

use serde::{Deserialize, Serialize};

/// WebSocket request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// WebSocket response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}
