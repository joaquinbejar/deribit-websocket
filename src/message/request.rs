//! WebSocket request message handling

use crate::model::{quote::*, trading::*, ws_types::JsonRpcRequest};

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

    /// Build buy order request
    pub fn build_buy_request(&mut self, request: &OrderRequest) -> JsonRpcRequest {
        let mut params = serde_json::json!({
            "instrument_name": request.instrument_name,
            "amount": request.amount
        });

        if let Some(ref order_type) = request.order_type {
            params["type"] = serde_json::Value::String(order_type.as_str().to_string());
        }
        if let Some(price) = request.price {
            params["price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(ref label) = request.label {
            params["label"] = serde_json::Value::String(label.clone());
        }
        if let Some(ref tif) = request.time_in_force {
            params["time_in_force"] = serde_json::Value::String(tif.as_str().to_string());
        }
        if let Some(max_show) = request.max_show {
            params["max_show"] = serde_json::Value::Number(
                serde_json::Number::from_f64(max_show).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(post_only) = request.post_only {
            params["post_only"] = serde_json::Value::Bool(post_only);
        }
        if let Some(reduce_only) = request.reduce_only {
            params["reduce_only"] = serde_json::Value::Bool(reduce_only);
        }
        if let Some(trigger_price) = request.trigger_price {
            params["trigger_price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(trigger_price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(ref trigger) = request.trigger {
            let trigger_str = match trigger {
                Trigger::IndexPrice => "index_price",
                Trigger::MarkPrice => "mark_price",
                Trigger::LastPrice => "last_price",
            };
            params["trigger"] = serde_json::Value::String(trigger_str.to_string());
        }
        if let Some(ref advanced) = request.advanced {
            params["advanced"] = serde_json::Value::String(advanced.clone());
        }
        if let Some(mmp) = request.mmp {
            params["mmp"] = serde_json::Value::Bool(mmp);
        }
        if let Some(valid_until) = request.valid_until {
            params["valid_until"] =
                serde_json::Value::Number(serde_json::Number::from(valid_until));
        }

        self.build_request("private/buy", Some(params))
    }

    /// Build sell order request
    pub fn build_sell_request(&mut self, request: &OrderRequest) -> JsonRpcRequest {
        let mut params = serde_json::json!({
            "instrument_name": request.instrument_name,
            "amount": request.amount
        });

        if let Some(ref order_type) = request.order_type {
            params["type"] = serde_json::Value::String(order_type.as_str().to_string());
        }
        if let Some(price) = request.price {
            params["price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(ref label) = request.label {
            params["label"] = serde_json::Value::String(label.clone());
        }
        if let Some(ref tif) = request.time_in_force {
            params["time_in_force"] = serde_json::Value::String(tif.as_str().to_string());
        }
        if let Some(max_show) = request.max_show {
            params["max_show"] = serde_json::Value::Number(
                serde_json::Number::from_f64(max_show).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(post_only) = request.post_only {
            params["post_only"] = serde_json::Value::Bool(post_only);
        }
        if let Some(reduce_only) = request.reduce_only {
            params["reduce_only"] = serde_json::Value::Bool(reduce_only);
        }
        if let Some(trigger_price) = request.trigger_price {
            params["trigger_price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(trigger_price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(ref trigger) = request.trigger {
            let trigger_str = match trigger {
                Trigger::IndexPrice => "index_price",
                Trigger::MarkPrice => "mark_price",
                Trigger::LastPrice => "last_price",
            };
            params["trigger"] = serde_json::Value::String(trigger_str.to_string());
        }
        if let Some(ref advanced) = request.advanced {
            params["advanced"] = serde_json::Value::String(advanced.clone());
        }
        if let Some(mmp) = request.mmp {
            params["mmp"] = serde_json::Value::Bool(mmp);
        }
        if let Some(valid_until) = request.valid_until {
            params["valid_until"] =
                serde_json::Value::Number(serde_json::Number::from(valid_until));
        }

        self.build_request("private/sell", Some(params))
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
    pub fn build_edit_request(&mut self, request: &EditOrderRequest) -> JsonRpcRequest {
        let mut params = serde_json::json!({
            "order_id": request.order_id,
            "amount": request.amount
        });

        if let Some(price) = request.price {
            params["price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(post_only) = request.post_only {
            params["post_only"] = serde_json::Value::Bool(post_only);
        }
        if let Some(reduce_only) = request.reduce_only {
            params["reduce_only"] = serde_json::Value::Bool(reduce_only);
        }
        if let Some(ref advanced) = request.advanced {
            params["advanced"] = serde_json::Value::String(advanced.clone());
        }
        if let Some(trigger_price) = request.trigger_price {
            params["trigger_price"] = serde_json::Value::Number(
                serde_json::Number::from_f64(trigger_price).unwrap_or(serde_json::Number::from(0)),
            );
        }
        if let Some(mmp) = request.mmp {
            params["mmp"] = serde_json::Value::Bool(mmp);
        }
        if let Some(valid_until) = request.valid_until {
            params["valid_until"] =
                serde_json::Value::Number(serde_json::Number::from(valid_until));
        }

        self.build_request("private/edit", Some(params))
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
    pub fn build_close_position_request(
        &mut self,
        instrument_name: &str,
        order_type: &str,
        price: Option<f64>,
    ) -> JsonRpcRequest {
        let mut params = serde_json::Map::new();
        params.insert(
            "instrument_name".to_string(),
            serde_json::Value::String(instrument_name.to_string()),
        );
        params.insert(
            "type".to_string(),
            serde_json::Value::String(order_type.to_string()),
        );

        if let Some(price) = price
            && let Some(price_num) = serde_json::Number::from_f64(price)
        {
            params.insert("price".to_string(), serde_json::Value::Number(price_num));
        }

        self.build_request(
            crate::constants::methods::PRIVATE_CLOSE_POSITION,
            Some(serde_json::Value::Object(params)),
        )
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
    pub fn build_move_positions_request(
        &mut self,
        currency: &str,
        source_uid: u64,
        target_uid: u64,
        trades: &[crate::model::MovePositionTrade],
    ) -> JsonRpcRequest {
        let trades_json: Vec<serde_json::Value> = trades
            .iter()
            .map(|t| {
                let mut trade_obj = serde_json::Map::new();
                trade_obj.insert(
                    "instrument_name".to_string(),
                    serde_json::Value::String(t.instrument_name.clone()),
                );
                if let Some(amount_num) = serde_json::Number::from_f64(t.amount) {
                    trade_obj.insert("amount".to_string(), serde_json::Value::Number(amount_num));
                }
                if let Some(price) = t.price
                    && let Some(price_num) = serde_json::Number::from_f64(price)
                {
                    trade_obj.insert("price".to_string(), serde_json::Value::Number(price_num));
                }
                serde_json::Value::Object(trade_obj)
            })
            .collect();

        let params = serde_json::json!({
            "currency": currency,
            "source_uid": source_uid,
            "target_uid": target_uid,
            "trades": trades_json
        });

        self.build_request(
            crate::constants::methods::PRIVATE_MOVE_POSITIONS,
            Some(params),
        )
    }
}
