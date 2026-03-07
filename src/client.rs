//! WebSocket client implementation for Deribit

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::model::SubscriptionChannel;
use crate::{
    callback::MessageHandler,
    config::WebSocketConfig,
    connection::WebSocketConnection,
    error::WebSocketError,
    message::{
        notification::NotificationHandler, request::RequestBuilder, response::ResponseHandler,
    },
    model::{
        quote::*,
        subscription::SubscriptionManager,
        ws_types::{JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
    },
    session::WebSocketSession,
};

/// WebSocket client for Deribit
#[derive(Debug)]
pub struct DeribitWebSocketClient {
    /// WebSocket configuration
    pub config: Arc<WebSocketConfig>,
    connection: Arc<Mutex<WebSocketConnection>>,
    /// WebSocket session
    pub session: Arc<WebSocketSession>,
    request_builder: Arc<Mutex<RequestBuilder>>,
    #[allow(dead_code)]
    response_handler: Arc<ResponseHandler>,
    #[allow(dead_code)]
    notification_handler: Arc<NotificationHandler>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
    #[allow(dead_code)]
    message_sender: Option<mpsc::UnboundedSender<String>>,
    #[allow(dead_code)]
    message_receiver: Option<mpsc::UnboundedReceiver<String>>,
    message_handler: Option<MessageHandler>,
}

impl DeribitWebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: &WebSocketConfig) -> Result<Self, WebSocketError> {
        let connection = Arc::new(Mutex::new(WebSocketConnection::new(config.ws_url.clone())));
        let session = Arc::new(WebSocketSession::new(config.clone()));
        let (tx, rx) = mpsc::unbounded_channel();

        let config = Arc::new(config.clone());
        Ok(Self {
            config,
            connection,
            session,
            request_builder: Arc::new(Mutex::new(RequestBuilder::new())),
            response_handler: Arc::new(ResponseHandler::new()),
            notification_handler: Arc::new(NotificationHandler::new()),
            subscription_manager: Arc::new(Mutex::new(SubscriptionManager::new())),
            message_sender: Some(tx),
            message_receiver: Some(rx),
            message_handler: None,
        })
    }

    /// Create a new WebSocket client with default configuration
    pub fn new_with_url(ws_url: String) -> Result<Self, WebSocketError> {
        let config = WebSocketConfig::with_url(&ws_url)
            .map_err(|e| WebSocketError::ConnectionFailed(format!("Invalid URL: {}", e)))?;
        Self::new(&config)
    }

    /// Create a new WebSocket client for testnet
    pub fn new_testnet() -> Result<Self, WebSocketError> {
        Self::new_with_url("wss://test.deribit.com/ws/api/v2".to_string())
    }

    /// Create a new WebSocket client for production
    pub fn new_production() -> Result<Self, WebSocketError> {
        Self::new_with_url("wss://www.deribit.com/ws/api/v2".to_string())
    }

    /// Connect to the WebSocket server
    pub async fn connect(&self) -> Result<(), WebSocketError> {
        let mut connection = self.connection.lock().await;
        connection.connect().await
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&self) -> Result<(), WebSocketError> {
        let mut connection = self.connection.lock().await;
        connection.disconnect().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let connection = self.connection.lock().await;
        connection.is_connected()
    }

    /// Authenticate with the server
    pub async fn authenticate(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_auth_request(client_id, client_secret)
        };

        self.send_request(request).await
    }

    /// Subscribe to channels
    pub async fn subscribe(
        &self,
        channels: Vec<String>,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_subscribe_request(channels.clone())
        };

        // Update subscription manager
        let mut sub_manager = self.subscription_manager.lock().await;
        for channel in channels {
            let channel_type = self.parse_channel_type(&channel);
            let instrument = self.extract_instrument(&channel);
            sub_manager.add_subscription(channel, channel_type, instrument);
        }

        self.send_request(request).await
    }

    /// Unsubscribe from channels
    pub async fn unsubscribe(
        &self,
        channels: Vec<String>,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_unsubscribe_request(channels.clone())
        };

        // Update subscription manager
        let mut sub_manager = self.subscription_manager.lock().await;
        for channel in channels {
            sub_manager.remove_subscription(&channel);
        }

        self.send_request(request).await
    }

    /// Unsubscribe from all public channels
    ///
    /// Unsubscribes from all public channels subscribed so far and clears
    /// the local subscription manager state.
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn public_unsubscribe_all(&self) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_public_unsubscribe_all_request()
        };

        let response = self.send_request(request).await?;

        // Clear subscription manager
        let mut sub_manager = self.subscription_manager.lock().await;
        sub_manager.clear();

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from unsubscribe_all".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Unsubscribe from all private channels
    ///
    /// Unsubscribes from all private channels subscribed so far and clears
    /// the local subscription manager state. Requires authentication.
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn private_unsubscribe_all(&self) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_private_unsubscribe_all_request()
        };

        let response = self.send_request(request).await?;

        // Clear subscription manager
        let mut sub_manager = self.subscription_manager.lock().await;
        sub_manager.clear();

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from unsubscribe_all".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Send a JSON-RPC request
    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let message = serde_json::to_string(&request).map_err(|e| {
            WebSocketError::InvalidMessage(format!("Failed to serialize request: {}", e))
        })?;

        let mut connection = self.connection.lock().await;
        connection.send(message).await?;

        // Wait for response (simplified - in real implementation would match by ID)
        let response_text = connection.receive().await?;

        // Try to parse as JSON-RPC response first, then handle notifications
        let response: JsonRpcResponse = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                // Check if this might be a notification (missing id field)
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&response_text)
                    && json_val.get("method").is_some()
                    && json_val.get("id").is_none()
                {
                    // This is a notification, create a synthetic response
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: serde_json::Value::Null,
                        result: crate::model::JsonRpcResult::Success { result: json_val },
                    });
                }
                return Err(WebSocketError::InvalidMessage(format!(
                    "Failed to parse response: {}",
                    e
                )));
            }
        };

        Ok(response)
    }

    /// Send a raw message
    pub async fn send_message(&self, message: String) -> Result<(), WebSocketError> {
        let mut connection = self.connection.lock().await;
        connection.send(message).await
    }

    /// Receive a message
    pub async fn receive_message(&self) -> Result<String, WebSocketError> {
        let mut connection = self.connection.lock().await;
        connection.receive().await
    }

    /// Get active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        let sub_manager = self.subscription_manager.lock().await;
        sub_manager.get_all_channels()
    }

    /// Test connection
    pub async fn test_connection(&self) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_test_request()
        };

        self.send_request(request).await
    }

    /// Get server time
    pub async fn get_time(&self) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_time_request()
        };

        self.send_request(request).await
    }

    /// Place mass quotes
    pub async fn mass_quote(
        &self,
        request: MassQuoteRequest,
    ) -> Result<MassQuoteResult, WebSocketError> {
        // Validate the request first
        request.validate().map_err(WebSocketError::InvalidMessage)?;

        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_mass_quote_request(request)
        };

        let response = self.send_request(json_request).await?;

        // Parse the response using WsResponse structure
        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse mass quote response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Cancel quotes
    pub async fn cancel_quotes(
        &self,
        request: CancelQuotesRequest,
    ) -> Result<CancelQuotesResponse, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_cancel_quotes_request(request)
        };

        let response = self.send_request(json_request).await?;

        // Parse the response using JsonRpcResult structure
        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse cancel quotes response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Set MMP group configuration
    pub async fn set_mmp_config(&self, config: MmpGroupConfig) -> Result<(), WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_set_mmp_config_request(config)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { .. } => Ok(()),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get MMP group configuration
    pub async fn get_mmp_config(
        &self,
        mmp_group: Option<String>,
    ) -> Result<Vec<MmpGroupConfig>, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_mmp_config_request(mmp_group)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse MMP config response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Reset MMP group
    pub async fn reset_mmp(&self, mmp_group: Option<String>) -> Result<(), WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_reset_mmp_request(mmp_group)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { .. } => Ok(()),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get open orders (including quotes)
    pub async fn get_open_orders(
        &self,
        currency: Option<String>,
        kind: Option<String>,
        type_filter: Option<String>,
    ) -> Result<Vec<QuoteInfo>, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_open_orders_request(currency, kind, type_filter)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse open orders response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Place a buy order
    ///
    /// # Arguments
    ///
    /// * `request` - The order request parameters
    ///
    /// # Returns
    ///
    /// Returns `OrderResponse` containing order info and any immediate trades
    pub async fn buy(
        &self,
        request: crate::model::trading::OrderRequest,
    ) -> Result<crate::model::trading::OrderResponse, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_buy_request(&request)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse buy response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Place a sell order
    ///
    /// # Arguments
    ///
    /// * `request` - The order request parameters
    ///
    /// # Returns
    ///
    /// Returns `OrderResponse` containing order info and any immediate trades
    pub async fn sell(
        &self,
        request: crate::model::trading::OrderRequest,
    ) -> Result<crate::model::trading::OrderResponse, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_sell_request(&request)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse sell response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Cancel an order by ID
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to cancel
    ///
    /// # Returns
    ///
    /// Returns `OrderInfo` for the cancelled order
    pub async fn cancel(
        &self,
        order_id: &str,
    ) -> Result<crate::model::trading::OrderInfo, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_cancel_request(order_id)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse cancel response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Cancel all orders
    ///
    /// # Returns
    ///
    /// Returns the number of orders cancelled
    pub async fn cancel_all(&self) -> Result<u32, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_cancel_all_request()
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse cancel_all response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Cancel all orders by currency
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency to cancel orders for (e.g., "BTC", "ETH")
    ///
    /// # Returns
    ///
    /// Returns the number of orders cancelled
    pub async fn cancel_all_by_currency(&self, currency: &str) -> Result<u32, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_cancel_all_by_currency_request(currency)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse cancel_all_by_currency response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Cancel all orders by instrument
    ///
    /// # Arguments
    ///
    /// * `instrument_name` - Instrument name to cancel orders for (e.g., "BTC-PERPETUAL")
    ///
    /// # Returns
    ///
    /// Returns the number of orders cancelled
    pub async fn cancel_all_by_instrument(
        &self,
        instrument_name: &str,
    ) -> Result<u32, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_cancel_all_by_instrument_request(instrument_name)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse cancel_all_by_instrument response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Edit an existing order
    ///
    /// # Arguments
    ///
    /// * `request` - The edit order request parameters
    ///
    /// # Returns
    ///
    /// Returns `OrderResponse` containing updated order info and any trades
    pub async fn edit(
        &self,
        request: crate::model::trading::EditOrderRequest,
    ) -> Result<crate::model::trading::OrderResponse, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_edit_request(&request)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse edit response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    // Account methods

    /// Get positions for the specified currency and kind
    ///
    /// Retrieves user positions filtered by currency and/or instrument kind.
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency filter (BTC, ETH, USDC, etc.) - optional
    /// * `kind` - Kind filter (future, option, spot, etc.) - optional
    ///
    /// # Returns
    ///
    /// A vector of positions matching the filter criteria
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_positions(
        &self,
        currency: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<crate::model::Position>, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_positions_request(currency, kind)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse positions response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get account summary for the specified currency
    ///
    /// Retrieves account summary information including balance, margin, and other account details.
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency to get summary for (BTC, ETH, USDC, etc.)
    /// * `extended` - Whether to include extended information
    ///
    /// # Returns
    ///
    /// Account summary for the specified currency
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_account_summary(
        &self,
        currency: &str,
        extended: Option<bool>,
    ) -> Result<crate::model::AccountSummary, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_account_summary_request(currency, extended)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse account summary response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get the state of an order
    ///
    /// Retrieves detailed information about a specific order.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to get state for
    ///
    /// # Returns
    ///
    /// Order information for the specified order
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_order_state(
        &self,
        order_id: &str,
    ) -> Result<crate::model::OrderInfo, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_order_state_request(order_id)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse order state response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get order history by currency
    ///
    /// Retrieves historical orders for the specified currency.
    ///
    /// # Arguments
    ///
    /// * `currency` - Currency to get order history for
    /// * `kind` - Kind filter (future, option, spot, etc.) - optional
    /// * `count` - Number of items to return - optional
    ///
    /// # Returns
    ///
    /// A vector of historical orders matching the filter criteria
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn get_order_history_by_currency(
        &self,
        currency: &str,
        kind: Option<&str>,
        count: Option<u32>,
    ) -> Result<Vec<crate::model::OrderInfo>, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_order_history_by_currency_request(currency, kind, count)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse order history response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    // Position management methods

    /// Close an existing position
    ///
    /// Places a reduce-only order to close an existing position.
    ///
    /// # Arguments
    ///
    /// * `instrument_name` - The instrument to close position for
    /// * `order_type` - Order type: "limit" or "market"
    /// * `price` - Price for limit orders (required if order_type is "limit")
    ///
    /// # Returns
    ///
    /// Response containing the order and any trades executed
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn close_position(
        &self,
        instrument_name: &str,
        order_type: &str,
        price: Option<f64>,
    ) -> Result<crate::model::ClosePositionResponse, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_close_position_request(instrument_name, order_type, price)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse close position response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Move positions between subaccounts
    ///
    /// Transfers positions from one subaccount to another within the same main account.
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
    /// A vector of results for each position moved
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be parsed
    pub async fn move_positions(
        &self,
        currency: &str,
        source_uid: u64,
        target_uid: u64,
        trades: &[crate::model::MovePositionTrade],
    ) -> Result<Vec<crate::model::MovePositionResult>, WebSocketError> {
        let json_request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_move_positions_request(currency, source_uid, target_uid, trades)
        };

        let response = self.send_request(json_request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse move positions response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Set message handler with callbacks
    /// The message_callback processes each incoming message and returns Result<(), Error>
    /// The error_callback is called only when message_callback returns an error
    pub fn set_message_handler<F, E>(&mut self, message_callback: F, error_callback: E)
    where
        F: Fn(&str) -> Result<(), WebSocketError> + Send + Sync + 'static,
        E: Fn(&str, &WebSocketError) + Send + Sync + 'static,
    {
        self.message_handler = Some(MessageHandler::new(message_callback, error_callback));
    }

    /// Set message handler using builder pattern
    pub fn set_message_handler_builder(&mut self, handler: MessageHandler) {
        self.message_handler = Some(handler);
    }

    /// Remove the current message handler
    pub fn clear_message_handler(&mut self) {
        self.message_handler = None;
    }

    /// Check if message handler is set
    pub fn has_message_handler(&self) -> bool {
        self.message_handler.is_some()
    }

    /// Receive and process a message using the registered callbacks
    /// This method will:
    /// 1. Receive a message from the WebSocket
    /// 2. Call the primary callback with the message
    /// 3. If primary callback returns error, call error callback with message and error
    pub async fn receive_and_process_message(&self) -> Result<(), WebSocketError> {
        let message = self.receive_message().await?;

        if let Some(handler) = &self.message_handler {
            handler.handle_message(&message);
        }

        Ok(())
    }

    /// Start message processing loop with callbacks
    /// This will continuously receive messages and process them using the registered callbacks
    /// The loop will continue until an error occurs or the connection is closed
    pub async fn start_message_processing_loop(&self) -> Result<(), WebSocketError> {
        if self.message_handler.is_none() {
            return Err(WebSocketError::InvalidMessage(
                "No message handler set. Use set_message_handler() first.".to_string(),
            ));
        }

        loop {
            match self.receive_and_process_message().await {
                Ok(()) => {
                    // Message processed successfully, continue
                }
                Err(WebSocketError::ConnectionClosed) => {
                    // Connection closed, exit loop gracefully
                    break;
                }
                Err(e) => {
                    // Other error occurred, propagate it
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    // Helper methods

    /// Parse a channel string into a `SubscriptionChannel` variant
    ///
    /// Uses `SubscriptionChannel::from_string()` to properly detect all channel types.
    /// Unknown channels are returned as `SubscriptionChannel::Unknown(String)`.
    fn parse_channel_type(&self, channel: &str) -> SubscriptionChannel {
        SubscriptionChannel::from_string(channel)
    }

    fn extract_instrument(&self, channel: &str) -> Option<String> {
        let parts: Vec<&str> = channel.split('.').collect();
        match parts.as_slice() {
            ["ticker", instrument] | ["ticker", instrument, _] => Some(instrument.to_string()),
            ["book", instrument, ..] => Some(instrument.to_string()),
            ["trades", instrument, ..] => Some(instrument.to_string()),
            ["chart", "trades", instrument, _] => Some(instrument.to_string()),
            ["user", "changes", instrument, _] => Some(instrument.to_string()),
            ["estimated_expiration_price", instrument] => Some(instrument.to_string()),
            ["markprice", "options", instrument] => Some(instrument.to_string()),
            ["perpetual", instrument, _] => Some(instrument.to_string()),
            ["quote", instrument] => Some(instrument.to_string()),
            ["incremental_ticker", instrument] => Some(instrument.to_string()),
            ["deribit_price_index", index_name]
            | ["deribit_price_ranking", index_name]
            | ["deribit_price_statistics", index_name]
            | ["deribit_volatility_index", index_name] => Some(index_name.to_string()),
            ["instrument", "state", _kind, currency] => Some(currency.to_string()),
            ["block_rfq", "trades", currency] => Some(currency.to_string()),
            ["block_trade_confirmations", currency] => Some(currency.to_string()),
            ["user", "mmp_trigger", index_name] => Some(index_name.to_string()),
            ["platform_state"]
            | ["platform_state", "public_methods_state"]
            | ["block_trade_confirmations"]
            | ["user", "access_log"]
            | ["user", "lock"] => None,
            _ => None,
        }
    }
}

impl Default for DeribitWebSocketClient {
    fn default() -> Self {
        let config = WebSocketConfig::default();
        Self::new(&config).unwrap()
    }
}
