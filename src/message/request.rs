//! WebSocket request message handling

use crate::model::ws_types::JsonRpcRequest;

/// Request builder for WebSocket messages
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    id_counter: u64,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestBuilder {
    /// Create a new request builder
    pub fn new() -> Self {
        Self { id_counter: 1 }
    }

    /// Build a JSON-RPC request
    pub fn build_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> JsonRpcRequest {
        let id = self.id_counter;
        self.id_counter += 1;

        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(id)),
            method: method.to_string(),
            params,
        }
    }

    /// Build authentication request
    pub fn build_auth_request(&mut self, client_id: &str, client_secret: &str) -> JsonRpcRequest {
        let params = serde_json::json!({
            "grant_type": "client_credentials",
            "client_id": client_id,
            "client_secret": client_secret
        });

        self.build_request("public/auth", Some(params))
    }

    /// Build subscription request
    pub fn build_subscribe_request(&mut self, channels: Vec<String>) -> JsonRpcRequest {
        let params = serde_json::json!({
            "channels": channels
        });

        self.build_request("public/subscribe", Some(params))
    }

    /// Build unsubscription request
    pub fn build_unsubscribe_request(&mut self, channels: Vec<String>) -> JsonRpcRequest {
        let params = serde_json::json!({
            "channels": channels
        });

        self.build_request("public/unsubscribe", Some(params))
    }

    /// Build test request
    pub fn build_test_request(&mut self) -> JsonRpcRequest {
        self.build_request("public/test", None)
    }

    /// Build get time request
    pub fn build_get_time_request(&mut self) -> JsonRpcRequest {
        self.build_request("public/get_time", None)
    }
}
