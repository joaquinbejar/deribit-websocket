//! WebSocket response message handling

use crate::model::ws_types::{JsonRpcError, JsonRpcResponse, JsonRpcResult};

/// Response handler for WebSocket messages
#[derive(Debug, Clone)]
pub struct ResponseHandler;

impl ResponseHandler {
    /// Create a new response handler
    pub fn new() -> Self {
        Self
    }

    /// Parse a JSON-RPC response
    pub fn parse_response(&self, data: &str) -> Result<JsonRpcResponse, serde_json::Error> {
        serde_json::from_str(data)
    }

    /// Check if response is successful
    pub fn is_success(&self, response: &JsonRpcResponse) -> bool {
        matches!(response.result, JsonRpcResult::Success { .. })
    }

    /// Extract result from successful response
    pub fn extract_result<'a>(
        &self,
        response: &'a JsonRpcResponse,
    ) -> Option<&'a serde_json::Value> {
        match &response.result {
            JsonRpcResult::Success { result } => Some(result),
            JsonRpcResult::Error { .. } => None,
        }
    }

    /// Extract error from response
    pub fn extract_error<'a>(&self, response: &'a JsonRpcResponse) -> Option<&'a JsonRpcError> {
        match &response.result {
            JsonRpcResult::Success { .. } => None,
            JsonRpcResult::Error { error } => Some(error),
        }
    }
}

impl Default for ResponseHandler {
    fn default() -> Self {
        Self::new()
    }
}
