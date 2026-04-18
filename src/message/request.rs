//! WebSocket request message handling

use serde::ser::Error as _;

use crate::error::WebSocketError;
use crate::model::{quote::*, trading::*, ws_types::JsonRpcRequest};

/// Build a [`WebSocketError::Serialization`] carrying `msg` as the underlying
/// `serde_json::Error`. Used to surface non-finite-float rejections the same
/// way a real JSON serialization failure would surface.
#[cold]
#[inline(never)]
fn serialization_error(msg: impl std::fmt::Display) -> WebSocketError {
    WebSocketError::Serialization(serde_json::Error::custom(msg))
}

/// Reject `NaN` and `+/- Infinity`. `serde_json` silently maps these to
/// `null` when serializing — which would otherwise corrupt outgoing requests.
#[inline]
fn check_finite(field: &'static str, value: f64) -> Result<(), WebSocketError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(serialization_error(format_args!(
            "field `{field}` must be finite, got {value}"
        )))
    }
}

/// Same as [`check_finite`] for optional values. `None` is always accepted.
#[inline]
fn check_finite_opt(field: &'static str, value: Option<f64>) -> Result<(), WebSocketError> {
    match value {
        Some(v) => check_finite(field, v),
        None => Ok(()),
    }
}

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

    /// Build public unsubscribe_all request
    ///
    /// Unsubscribes from all public channels. Takes no parameters.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for unsubscribing from all public channels
    pub fn build_public_unsubscribe_all_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PUBLIC_UNSUBSCRIBE_ALL,
            Some(serde_json::json!({})),
        )
    }

    /// Build private unsubscribe_all request
    ///
    /// Unsubscribes from all private channels. Takes no parameters.
    /// Requires authentication.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for unsubscribing from all private channels
    pub fn build_private_unsubscribe_all_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PRIVATE_UNSUBSCRIBE_ALL,
            Some(serde_json::json!({})),
        )
    }

    /// Build test request
    pub fn build_test_request(&mut self) -> JsonRpcRequest {
        self.build_request(crate::constants::methods::PUBLIC_TEST, None)
    }

    /// Build set_heartbeat request
    ///
    /// Enables heartbeat with specified interval. The server will send a heartbeat
    /// message every `interval` seconds, and expects a response within the same interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - Heartbeat interval in seconds (10-3600)
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for setting the heartbeat interval
    pub fn build_set_heartbeat_request(&mut self, interval: u64) -> JsonRpcRequest {
        let params = serde_json::json!({
            "interval": interval
        });
        self.build_request(
            crate::constants::methods::PUBLIC_SET_HEARTBEAT,
            Some(params),
        )
    }

    /// Build disable_heartbeat request
    ///
    /// Disables heartbeat messages. The server will stop sending heartbeat messages
    /// and test_request notifications.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for disabling heartbeats
    pub fn build_disable_heartbeat_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PUBLIC_DISABLE_HEARTBEAT,
            Some(serde_json::json!({})),
        )
    }

    /// Build hello request
    ///
    /// Sends client identification to the server. This is used for client tracking
    /// and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `client_name` - Name of the client application
    /// * `client_version` - Version of the client application
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for client identification
    pub fn build_hello_request(
        &mut self,
        client_name: &str,
        client_version: &str,
    ) -> JsonRpcRequest {
        let params = serde_json::json!({
            "client_name": client_name,
            "client_version": client_version
        });
        self.build_request(crate::constants::methods::PUBLIC_HELLO, Some(params))
    }

    /// Build get time request
    pub fn build_get_time_request(&mut self) -> JsonRpcRequest {
        self.build_request("public/get_time", None)
    }

    /// Build enable_cancel_on_disconnect request
    ///
    /// Enables automatic cancellation of all open orders when the WebSocket connection
    /// is lost. This is a safety feature to prevent unintended order execution when
    /// the client loses connectivity.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for enabling cancel-on-disconnect
    pub fn build_enable_cancel_on_disconnect_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PRIVATE_ENABLE_CANCEL_ON_DISCONNECT,
            Some(serde_json::json!({})),
        )
    }

    /// Build disable_cancel_on_disconnect request
    ///
    /// Disables automatic cancellation of orders on disconnect. Orders will remain
    /// active even if the WebSocket connection is lost.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for disabling cancel-on-disconnect
    pub fn build_disable_cancel_on_disconnect_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PRIVATE_DISABLE_CANCEL_ON_DISCONNECT,
            Some(serde_json::json!({})),
        )
    }

    /// Build get_cancel_on_disconnect request
    ///
    /// Retrieves the current cancel-on-disconnect status for the session.
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for getting the cancel-on-disconnect status
    pub fn build_get_cancel_on_disconnect_request(&mut self) -> JsonRpcRequest {
        self.build_request(
            crate::constants::methods::PRIVATE_GET_CANCEL_ON_DISCONNECT,
            Some(serde_json::json!({})),
        )
    }

    /// Build mass quote request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if the request contains values
    /// that cannot be represented in JSON (for example `NaN` or `Infinity` in
    /// any `f64` field such as `price` or `amount`).
    pub fn build_mass_quote_request(
        &mut self,
        request: MassQuoteRequest,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        for quote in &request.quotes {
            check_finite("quotes[].amount", quote.amount)?;
            check_finite("quotes[].price", quote.price)?;
        }
        let params = serde_json::to_value(request)?;
        Ok(self.build_request("private/mass_quote", Some(params)))
    }

    /// Build cancel quotes request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if the request contains values
    /// that cannot be represented in JSON (for example `NaN` or `Infinity` in
    /// the `delta_range` tuple).
    pub fn build_cancel_quotes_request(
        &mut self,
        request: CancelQuotesRequest,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        if let Some((min, max)) = request.delta_range {
            check_finite("delta_range.min", min)?;
            check_finite("delta_range.max", max)?;
        }
        let params = serde_json::to_value(request)?;
        Ok(self.build_request("private/cancel_quotes", Some(params)))
    }

    /// Build set MMP config request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if `config.quantity_limit` or
    /// `config.delta_limit` is `NaN` or `Infinity`. `MmpGroupConfig::new`
    /// enforces magnitude invariants but NaN comparisons always return false
    /// and silently bypass them, which is why the finite check is repeated
    /// here.
    pub fn build_set_mmp_config_request(
        &mut self,
        config: MmpGroupConfig,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        check_finite("quantity_limit", config.quantity_limit)?;
        check_finite("delta_limit", config.delta_limit)?;

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

        Ok(self.build_request("private/set_mmp_config", Some(params)))
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

    /// Build buy order request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if `request` contains values
    /// that cannot be represented in JSON (for example `NaN` or `Infinity` in
    /// `price`, `amount`, `max_show` or `trigger_price`).
    pub fn build_buy_request(
        &mut self,
        request: &OrderRequest,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        check_finite("amount", request.amount)?;
        check_finite_opt("price", request.price)?;
        check_finite_opt("max_show", request.max_show)?;
        check_finite_opt("trigger_price", request.trigger_price)?;
        let params = serde_json::to_value(request)?;
        Ok(self.build_request("private/buy", Some(params)))
    }

    /// Build sell order request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if `request` contains values
    /// that cannot be represented in JSON (for example `NaN` or `Infinity` in
    /// `price`, `amount`, `max_show` or `trigger_price`).
    pub fn build_sell_request(
        &mut self,
        request: &OrderRequest,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        check_finite("amount", request.amount)?;
        check_finite_opt("price", request.price)?;
        check_finite_opt("max_show", request.max_show)?;
        check_finite_opt("trigger_price", request.trigger_price)?;
        let params = serde_json::to_value(request)?;
        Ok(self.build_request("private/sell", Some(params)))
    }

    /// Build cancel order request
    pub fn build_cancel_request(&mut self, order_id: &str) -> JsonRpcRequest {
        let params = serde_json::json!({
            "order_id": order_id
        });

        self.build_request("private/cancel", Some(params))
    }

    /// Build cancel all orders request
    pub fn build_cancel_all_request(&mut self) -> JsonRpcRequest {
        self.build_request("private/cancel_all", Some(serde_json::json!({})))
    }

    /// Build cancel all orders by currency request
    pub fn build_cancel_all_by_currency_request(&mut self, currency: &str) -> JsonRpcRequest {
        let params = serde_json::json!({
            "currency": currency
        });

        self.build_request("private/cancel_all_by_currency", Some(params))
    }

    /// Build cancel all orders by instrument request
    pub fn build_cancel_all_by_instrument_request(
        &mut self,
        instrument_name: &str,
    ) -> JsonRpcRequest {
        let params = serde_json::json!({
            "instrument_name": instrument_name
        });

        self.build_request("private/cancel_all_by_instrument", Some(params))
    }

    /// Build edit order request
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if `request` contains values
    /// that cannot be represented in JSON (for example `NaN` or `Infinity` in
    /// `price`, `amount` or `trigger_price`).
    pub fn build_edit_request(
        &mut self,
        request: &EditOrderRequest,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        check_finite("amount", request.amount)?;
        check_finite_opt("price", request.price)?;
        check_finite_opt("trigger_price", request.trigger_price)?;
        let params = serde_json::to_value(request)?;
        Ok(self.build_request("private/edit", Some(params)))
    }

    // Account methods

    /// Build a get_positions request
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency filter (BTC, ETH, USDC, etc.) - optional
    /// * `kind` - Kind filter (future, option, spot, etc.) - optional
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for getting positions
    pub fn build_get_positions_request(
        &mut self,
        currency: Option<&str>,
        kind: Option<&str>,
    ) -> JsonRpcRequest {
        let mut params = serde_json::Map::new();

        if let Some(currency) = currency {
            params.insert(
                "currency".to_string(),
                serde_json::Value::String(currency.to_string()),
            );
        }

        if let Some(kind) = kind {
            params.insert(
                "kind".to_string(),
                serde_json::Value::String(kind.to_string()),
            );
        }

        if params.is_empty() {
            self.build_request(crate::constants::methods::PRIVATE_GET_POSITIONS, None)
        } else {
            self.build_request(
                crate::constants::methods::PRIVATE_GET_POSITIONS,
                Some(serde_json::Value::Object(params)),
            )
        }
    }

    /// Build a get_account_summary request
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency to get summary for (BTC, ETH, USDC, etc.)
    /// * `extended` - Whether to include extended information
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for getting account summary
    pub fn build_get_account_summary_request(
        &mut self,
        currency: &str,
        extended: Option<bool>,
    ) -> JsonRpcRequest {
        let mut params = serde_json::Map::new();
        params.insert(
            "currency".to_string(),
            serde_json::Value::String(currency.to_string()),
        );

        if let Some(extended) = extended {
            params.insert("extended".to_string(), serde_json::Value::Bool(extended));
        }

        self.build_request(
            crate::constants::methods::PRIVATE_GET_ACCOUNT_SUMMARY,
            Some(serde_json::Value::Object(params)),
        )
    }

    /// Build a get_order_state request
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to get state for
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for getting order state
    pub fn build_get_order_state_request(&mut self, order_id: &str) -> JsonRpcRequest {
        let params = serde_json::json!({
            "order_id": order_id
        });

        self.build_request(
            crate::constants::methods::PRIVATE_GET_ORDER_STATE,
            Some(params),
        )
    }

    /// Build a get_order_history_by_currency request
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency to get order history for
    /// * `kind` - Kind filter (future, option, spot, etc.) - optional
    /// * `count` - Number of items to return - optional
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for getting order history
    pub fn build_get_order_history_by_currency_request(
        &mut self,
        currency: &str,
        kind: Option<&str>,
        count: Option<u32>,
    ) -> JsonRpcRequest {
        let mut params = serde_json::Map::new();
        params.insert(
            "currency".to_string(),
            serde_json::Value::String(currency.to_string()),
        );

        if let Some(kind) = kind {
            params.insert(
                "kind".to_string(),
                serde_json::Value::String(kind.to_string()),
            );
        }

        if let Some(count) = count {
            params.insert(
                "count".to_string(),
                serde_json::Value::Number(serde_json::Number::from(count)),
            );
        }

        self.build_request(
            crate::constants::methods::PRIVATE_GET_ORDER_HISTORY_BY_CURRENCY,
            Some(serde_json::Value::Object(params)),
        )
    }

    // Position management methods

    /// Build a close_position request
    ///
    /// # Arguments
    ///
    /// * `instrument_name` - The instrument to close position for
    /// * `order_type` - Order type: "limit" or "market"
    /// * `price` - Price for limit orders (required if order_type is "limit")
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for closing a position
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if `price` is `NaN` or
    /// `Infinity`, which cannot be represented in JSON.
    pub fn build_close_position_request(
        &mut self,
        instrument_name: &str,
        order_type: &str,
        price: Option<f64>,
    ) -> Result<JsonRpcRequest, WebSocketError> {
        let mut params = serde_json::Map::new();
        params.insert(
            "instrument_name".to_string(),
            serde_json::Value::String(instrument_name.to_string()),
        );
        params.insert(
            "type".to_string(),
            serde_json::Value::String(order_type.to_string()),
        );

        if let Some(price) = price {
            check_finite("price", price)?;
            params.insert("price".to_string(), serde_json::to_value(price)?);
        }

        Ok(self.build_request(
            crate::constants::methods::PRIVATE_CLOSE_POSITION,
            Some(serde_json::Value::Object(params)),
        ))
    }

    /// Build a move_positions request
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency for the positions (BTC, ETH, etc.)
    /// * `source_uid` - Source subaccount ID
    /// * `target_uid` - Target subaccount ID
    /// * `trades` - List of positions to move
    ///
    /// # Returns
    ///
    /// A JSON-RPC request for moving positions between subaccounts
    ///
    /// # Errors
    ///
    /// Returns [`WebSocketError::Serialization`] if any `amount` or `price`
    /// value in `trades` is `NaN` or `Infinity`, which cannot be represented
    /// in JSON.
    pub fn build_move_positions_request(
        &mut self,
        currency: &str,
        source_uid: u64,
        target_uid: u64,
        trades: &[crate::model::MovePositionTrade],
    ) -> Result<JsonRpcRequest, WebSocketError> {
        for trade in trades {
            check_finite("trades[_].amount", trade.amount)?;
            check_finite_opt("trades[_].price", trade.price)?;
        }
        let trades_json = serde_json::to_value(trades)?;

        let params = serde_json::json!({
            "currency": currency,
            "source_uid": source_uid,
            "target_uid": target_uid,
            "trades": trades_json
        });

        Ok(self.build_request(
            crate::constants::methods::PRIVATE_MOVE_POSITIONS,
            Some(params),
        ))
    }
}
