//! WebSocket request message handling

use crate::model::{quote::*, ws_types::JsonRpcRequest};

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

    /// Build mass quote request
    pub fn build_mass_quote_request(&mut self, request: MassQuoteRequest) -> JsonRpcRequest {
        let params = serde_json::to_value(request).expect("Failed to serialize mass quote request");

        self.build_request("private/mass_quote", Some(params))
    }

    /// Build cancel quotes request
    pub fn build_cancel_quotes_request(&mut self, request: CancelQuotesRequest) -> JsonRpcRequest {
        let params =
            serde_json::to_value(request).expect("Failed to serialize cancel quotes request");

        self.build_request("private/cancel_quotes", Some(params))
    }

    /// Build set MMP config request
    pub fn build_set_mmp_config_request(&mut self, config: MmpGroupConfig) -> JsonRpcRequest {
        let mut params = serde_json::json!({
            "mmp_group": config.mmp_group,
            "quantity_limit": config.quantity_limit,
            "delta_limit": config.delta_limit,
            "interval": config.interval,
            "frozen_time": config.frozen_time
        });

        // If interval is 0, this disables the group
        if config.interval == 0 {
            params["interval"] = serde_json::Value::Number(serde_json::Number::from(0));
        }

        self.build_request("private/set_mmp_config", Some(params))
    }

    /// Build get MMP config request
    pub fn build_get_mmp_config_request(&mut self, mmp_group: Option<String>) -> JsonRpcRequest {
        let params = if let Some(group) = mmp_group {
            serde_json::json!({
                "mmp_group": group
            })
        } else {
            serde_json::json!({})
        };

        self.build_request("private/get_mmp_config", Some(params))
    }

    /// Build reset MMP request
    pub fn build_reset_mmp_request(&mut self, mmp_group: Option<String>) -> JsonRpcRequest {
        let params = if let Some(group) = mmp_group {
            serde_json::json!({
                "mmp_group": group
            })
        } else {
            serde_json::json!({})
        };

        self.build_request("private/reset_mmp", Some(params))
    }

    /// Build get open orders request
    pub fn build_get_open_orders_request(
        &mut self,
        currency: Option<String>,
        kind: Option<String>,
        type_filter: Option<String>,
    ) -> JsonRpcRequest {
        let mut params = serde_json::json!({});

        if let Some(currency) = currency {
            params["currency"] = serde_json::Value::String(currency);
        }
        if let Some(kind) = kind {
            params["kind"] = serde_json::Value::String(kind);
        }
        if let Some(type_filter) = type_filter {
            params["type"] = serde_json::Value::String(type_filter);
        }

        self.build_request("private/get_open_orders", Some(params))
    }
}
