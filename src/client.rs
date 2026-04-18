//! WebSocket client implementation for Deribit

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::model::SubscriptionChannel;
use crate::{
    callback::MessageHandler,
    config::WebSocketConfig,
    connection::Dispatcher,
    error::WebSocketError,
    message::request::RequestBuilder,
    model::{
        quote::*,
        subscription::SubscriptionManager,
        ws_types::{JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
    },
    session::WebSocketSession,
};

/// WebSocket client for Deribit
///
/// Owns a shared, optional [`Dispatcher`] that runs the send/receive loop
/// in a dedicated tokio task. All request/response multiplexing and
/// notification routing happens inside that task; this façade only
/// clones an `Arc<Dispatcher>` out of the slot and forwards calls to it.
#[derive(Debug)]
pub struct DeribitWebSocketClient {
    /// WebSocket configuration
    pub config: Arc<WebSocketConfig>,
    /// Shared slot holding the live dispatcher, if any. The slot's mutex
    /// is only held long enough to read/insert/remove the `Arc`, never
    /// across a `send_request` await.
    dispatcher: Arc<Mutex<Option<Arc<Dispatcher>>>>,
    /// WebSocket session
    pub session: Arc<WebSocketSession>,
    request_builder: Arc<Mutex<RequestBuilder>>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
    message_handler: Option<MessageHandler>,
}

impl DeribitWebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: &WebSocketConfig) -> Result<Self, WebSocketError> {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let session = Arc::new(WebSocketSession::new(
            config.clone(),
            Arc::clone(&subscription_manager),
        ));

        let config = Arc::new(config.clone());
        Ok(Self {
            config,
            dispatcher: Arc::new(Mutex::new(None)),
            session,
            request_builder: Arc::new(Mutex::new(RequestBuilder::new())),
            subscription_manager,
            message_handler: None,
        })
    }

    /// Returns a handle to the shared subscription manager. The same
    /// handle is held by `self.session`, so all subscription state is
    /// observable from either side.
    #[must_use]
    pub fn subscription_manager(&self) -> Arc<Mutex<SubscriptionManager>> {
        Arc::clone(&self.subscription_manager)
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
    ///
    /// Spawns the dispatcher task that owns the WebSocket stream. If a
    /// previous dispatcher is still installed, it is shut down first.
    ///
    /// The slot lock is held across the entire shutdown + connect_async +
    /// install sequence so concurrent `connect()` calls are serialized.
    /// Without this, two callers could each see an empty slot, each spawn
    /// a dispatcher, and the loser's dispatcher task would leak. While a
    /// connect is in flight, other client operations that touch the slot
    /// (`send_request`, `disconnect`, `is_connected`) wait on the same
    /// mutex — the desired semantics.
    pub async fn connect(&self) -> Result<(), WebSocketError> {
        let mut guard = self.dispatcher.lock().await;
        if let Some(prev) = guard.take() {
            let _ = prev.shutdown().await;
        }
        let dispatcher = Dispatcher::connect(
            self.config.ws_url.clone(),
            self.config.request_timeout,
            self.config.notification_channel_capacity,
            self.config.dispatcher_command_capacity,
        )
        .await?;
        *guard = Some(Arc::new(dispatcher));
        Ok(())
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&self) -> Result<(), WebSocketError> {
        // Take the Arc out under the lock so the lock is not held across
        // the shutdown await.
        let dispatcher = {
            let mut guard = self.dispatcher.lock().await;
            guard.take()
        };
        if let Some(dispatcher) = dispatcher {
            dispatcher.shutdown().await?;
        }
        Ok(())
    }

    /// Check if connected (i.e., a dispatcher is currently installed).
    pub async fn is_connected(&self) -> bool {
        self.dispatcher.lock().await.is_some()
    }

    /// Authenticate with the server
    ///
    /// Authenticates the connection using API credentials and returns authentication
    /// details including access token and refresh token.
    ///
    /// # Arguments
    ///
    /// * `client_id` - API client ID
    /// * `client_secret` - API client secret
    ///
    /// # Returns
    ///
    /// Returns `AuthResponse` containing access token, token type, expiration, and scope
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or credentials are invalid
    pub async fn authenticate(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<crate::model::AuthResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_auth_request(client_id, client_secret)
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!(
                    "Failed to parse authentication response: {}",
                    e
                ))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Subscribe to channels
    ///
    /// Local subscription state is updated only after the server confirms
    /// the subscribe with a `JsonRpcResult::Success`. Transport failures
    /// and API errors leave the local view untouched. Server-side partial
    /// success (the response `result` array may be shorter than the
    /// requested channel list) is not yet reconciled — see follow-up
    /// issue.
    pub async fn subscribe(
        &self,
        channels: Vec<String>,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_subscribe_request(channels.clone())
        };

        let response = self.send_request(request).await?;

        if matches!(response.result, JsonRpcResult::Success { .. }) {
            let mut sub_manager = self.subscription_manager.lock().await;
            for channel in channels {
                let channel_type = self.parse_channel_type(&channel);
                let instrument = self.extract_instrument(&channel);
                sub_manager.add_subscription(channel, channel_type, instrument);
            }
        }

        Ok(response)
    }

    /// Unsubscribe from channels
    ///
    /// Local subscription state is updated only after the server confirms
    /// the unsubscribe with a `JsonRpcResult::Success`. Transport failures
    /// and API errors leave the local view untouched so the caller can
    /// retry without inconsistency.
    pub async fn unsubscribe(
        &self,
        channels: Vec<String>,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_unsubscribe_request(channels.clone())
        };

        let response = self.send_request(request).await?;

        if matches!(response.result, JsonRpcResult::Success { .. }) {
            let mut sub_manager = self.subscription_manager.lock().await;
            for channel in channels {
                sub_manager.remove_subscription(&channel);
            }
        }

        Ok(response)
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

        // Clear the local subscription manager only after the server
        // confirms success. On API error (e.g. not authenticated) we
        // preserve the local view so the caller can retry without
        // inconsistency.
        match response.result {
            JsonRpcResult::Success { result } => {
                self.subscription_manager.lock().await.clear();
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

        // Clear the local subscription manager only after the server
        // confirms success. On API error we preserve the local view so
        // the caller can retry without inconsistency.
        match response.result {
            JsonRpcResult::Success { result } => {
                self.subscription_manager.lock().await.clear();
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

    /// Send a JSON-RPC request and await the matching response.
    ///
    /// Forwards the request to the dispatcher, which serializes it,
    /// writes it to the WebSocket sink, and routes the response back by
    /// matching on the JSON-RPC `id` field. Notifications arriving
    /// between the request and the response do not affect this call and
    /// are routed to the notification channel instead.
    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        // Clone the Arc<Dispatcher> out under the short-lived slot lock,
        // then drop the guard before awaiting on the dispatcher. This
        // keeps the per-client mutex off the hot path so concurrent
        // send_request calls do not serialize against each other.
        let dispatcher = {
            let guard = self.dispatcher.lock().await;
            guard
                .as_ref()
                .map(Arc::clone)
                .ok_or(WebSocketError::ConnectionClosed)?
        };
        dispatcher.send_request(request).await
    }

    /// Receive the next notification (or unmatched frame) from the server.
    ///
    /// Returns [`WebSocketError::ConnectionClosed`] if the dispatcher is
    /// not running, or if its notification channel has been drained and
    /// closed.
    pub async fn receive_message(&self) -> Result<String, WebSocketError> {
        let dispatcher = {
            let guard = self.dispatcher.lock().await;
            guard
                .as_ref()
                .map(Arc::clone)
                .ok_or(WebSocketError::ConnectionClosed)?
        };
        dispatcher
            .next_notification()
            .await
            .ok_or(WebSocketError::ConnectionClosed)
    }

    /// Get active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        let sub_manager = self.subscription_manager.lock().await;
        sub_manager.get_all_channels()
    }

    /// Test connection
    ///
    /// Tests the WebSocket connection and returns API version information.
    ///
    /// # Returns
    ///
    /// Returns `TestResponse` containing the API version string
    ///
    /// # Errors
    ///
    /// Returns an error if the connection test fails
    pub async fn test_connection(&self) -> Result<crate::model::TestResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_test_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse test response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get server time
    ///
    /// Returns the current server timestamp in milliseconds since Unix epoch.
    ///
    /// # Returns
    ///
    /// Returns `u64` timestamp in milliseconds
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn get_time(&self) -> Result<u64, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_time_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => result.as_u64().ok_or_else(|| {
                WebSocketError::InvalidMessage(
                    "Expected u64 timestamp in get_time response".to_string(),
                )
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Enable heartbeat with specified interval
    ///
    /// The server will send a heartbeat message every `interval` seconds.
    /// If heartbeat is enabled, the server will also send `test_request` notifications
    /// which the client should respond to with `public/test` to keep the connection alive.
    ///
    /// # Arguments
    ///
    /// * `interval` - Heartbeat interval in seconds (10-3600)
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the interval is invalid
    pub async fn set_heartbeat(&self, interval: u64) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_set_heartbeat_request(interval)
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from set_heartbeat".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Disable heartbeat
    ///
    /// Stops the server from sending heartbeat messages and `test_request` notifications.
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn disable_heartbeat(&self) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_disable_heartbeat_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from disable_heartbeat".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Send client identification to the server
    ///
    /// This method identifies the client to the server with its name and version.
    /// It's recommended to call this after connecting to provide debugging information.
    ///
    /// # Arguments
    ///
    /// * `client_name` - Name of the client application
    /// * `client_version` - Version of the client application
    ///
    /// # Returns
    ///
    /// Returns `HelloResponse` containing the API version information
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn hello(
        &self,
        client_name: &str,
        client_version: &str,
    ) -> Result<crate::model::HelloResponse, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_hello_request(client_name, client_version)
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => serde_json::from_value(result).map_err(|e| {
                WebSocketError::InvalidMessage(format!("Failed to parse hello response: {}", e))
            }),
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Enable automatic order cancellation on disconnect
    ///
    /// When enabled, all open orders will be automatically cancelled if the WebSocket
    /// connection is lost. This is a safety feature to prevent unintended order
    /// execution when the client loses connectivity.
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or requires authentication
    pub async fn enable_cancel_on_disconnect(&self) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_enable_cancel_on_disconnect_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from enable_cancel_on_disconnect".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Disable automatic order cancellation on disconnect
    ///
    /// When disabled, orders will remain active even if the WebSocket connection
    /// is lost.
    ///
    /// # Returns
    ///
    /// Returns `"ok"` on success
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or requires authentication
    pub async fn disable_cancel_on_disconnect(&self) -> Result<String, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_disable_cancel_on_disconnect_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => {
                result.as_str().map(String::from).ok_or_else(|| {
                    WebSocketError::InvalidMessage(
                        "Expected string result from disable_cancel_on_disconnect".to_string(),
                    )
                })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
    }

    /// Get current cancel-on-disconnect status
    ///
    /// Returns whether automatic order cancellation on disconnect is currently enabled.
    ///
    /// # Returns
    ///
    /// Returns `true` if cancel-on-disconnect is enabled, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or requires authentication
    pub async fn get_cancel_on_disconnect(&self) -> Result<bool, WebSocketError> {
        let request = {
            let mut builder = self.request_builder.lock().await;
            builder.build_get_cancel_on_disconnect_request()
        };

        let response = self.send_request(request).await?;

        match response.result {
            JsonRpcResult::Success { result } => {
                // The result contains "enabled" field
                result
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        WebSocketError::InvalidMessage(
                            "Expected 'enabled' boolean in get_cancel_on_disconnect response"
                                .to_string(),
                        )
                    })
            }
            JsonRpcResult::Error { error } => {
                Err(WebSocketError::ApiError(error.code, error.message))
            }
        }
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
            builder.build_mass_quote_request(request)?
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
            builder.build_cancel_quotes_request(request)?
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
            builder.build_set_mmp_config_request(config)?
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
            builder.build_buy_request(&request)?
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
            builder.build_sell_request(&request)?
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
            builder.build_edit_request(&request)?
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
            builder.build_close_position_request(instrument_name, order_type, price)?
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
            builder.build_move_positions_request(currency, source_uid, target_uid, trades)?
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
        // `Default` cannot return `Result`; `Self::new` only fails on invalid
        // URL parsing which cannot happen for `WebSocketConfig::default()`.
        // Tracked separately for a fallible-only constructor redesign.
        #[allow(clippy::unwrap_used)]
        Self::new(&config).unwrap()
    }
}
