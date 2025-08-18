//! Message builder utilities for WebSocket client

use crate::message::{NotificationHandler, RequestBuilder, ResponseHandler};
use crate::model::ws_types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Main message builder for WebSocket operations
#[derive(Debug)]
pub struct MessageBuilder {
    request_builder: RequestBuilder,
    response_handler: ResponseHandler,
    notification_handler: NotificationHandler,
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            request_builder: RequestBuilder::new(),
            response_handler: ResponseHandler::new(),
            notification_handler: NotificationHandler::new(),
        }
    }

    /// Get mutable reference to request builder
    pub fn request_builder(&mut self) -> &mut RequestBuilder {
        &mut self.request_builder
    }

    /// Get reference to response handler
    pub fn response_handler(&self) -> &ResponseHandler {
        &self.response_handler
    }

    /// Get reference to notification handler
    pub fn notification_handler(&self) -> &NotificationHandler {
        &self.notification_handler
    }

    /// Parse incoming message and determine type
    pub fn parse_message(&self, data: &str) -> Result<MessageType, serde_json::Error> {
        // Try to parse as response first (has 'id' field)
        if let Ok(response) = self.response_handler.parse_response(data) {
            return Ok(MessageType::Response(response));
        }

        // Try to parse as notification (no 'id' field)
        if let Ok(notification) = self.notification_handler.parse_notification(data) {
            return Ok(MessageType::Notification(notification));
        }

        // If neither works, return error
        Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unable to parse message",
        )))
    }
}

/// Message type enumeration
#[derive(Debug, Clone)]
pub enum MessageType {
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC response message
    Response(JsonRpcResponse),
    /// JSON-RPC notification message
    Notification(JsonRpcNotification),
}
