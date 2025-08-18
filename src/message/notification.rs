//! WebSocket notification message handling

use crate::model::ws_types::JsonRpcNotification;

/// Notification handler for WebSocket messages
#[derive(Debug, Clone)]
pub struct NotificationHandler;

impl NotificationHandler {
    /// Create a new notification handler
    pub fn new() -> Self {
        Self
    }

    /// Parse a JSON-RPC notification
    pub fn parse_notification(&self, data: &str) -> Result<JsonRpcNotification, serde_json::Error> {
        serde_json::from_str(data)
    }

    /// Check if this is a subscription notification
    pub fn is_subscription_notification(&self, notification: &JsonRpcNotification) -> bool {
        notification.method.starts_with("subscription")
    }

    /// Extract channel from subscription notification
    pub fn extract_channel(&self, notification: &JsonRpcNotification) -> Option<String> {
        if let Some(params) = &notification.params
            && let Some(channel) = params.get("channel")
        {
            return channel.as_str().map(|s| s.to_string());
        }
        None
    }

    /// Extract data from subscription notification
    pub fn extract_data(&self, notification: &JsonRpcNotification) -> Option<serde_json::Value> {
        if let Some(params) = &notification.params
            && let Some(data) = params.get("data")
        {
            return Some(data.clone());
        }
        None
    }
}

impl Default for NotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}
