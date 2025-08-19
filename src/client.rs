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

    fn parse_channel_type(&self, channel: &str) -> SubscriptionChannel {
        if channel.starts_with("ticker") {
            SubscriptionChannel::Ticker(self.extract_instrument(channel).unwrap_or_default())
        } else if channel.starts_with("book") {
            SubscriptionChannel::OrderBook(self.extract_instrument(channel).unwrap_or_default())
        } else if channel.starts_with("trades") {
            SubscriptionChannel::Trades(self.extract_instrument(channel).unwrap_or_default())
        } else if channel == "user.orders" {
            SubscriptionChannel::UserOrders
        } else if channel == "user.trades" {
            SubscriptionChannel::UserTrades
        } else {
            SubscriptionChannel::Ticker(String::new()) // Default fallback
        }
    }

    fn extract_instrument(&self, channel: &str) -> Option<String> {
        channel
            .find('.')
            .map(|dot_pos| channel[dot_pos + 1..].to_string())
    }
}

impl Default for DeribitWebSocketClient {
    fn default() -> Self {
        let config = WebSocketConfig::default();
        Self::new(&config).unwrap()
    }
}
