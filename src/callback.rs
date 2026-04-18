//! Callback system for message handling

use crate::error::WebSocketError;
use std::sync::Arc;

/// Primary message processing callback
/// Takes a message and returns Result<(), Error> for processing
pub type MessageCallback = Arc<dyn Fn(&str) -> Result<(), WebSocketError> + Send + Sync>;

/// Error handling callback
/// Takes the original message and the error from the primary callback
/// Called only when the primary callback returns an error
pub type ErrorCallback = Arc<dyn Fn(&str, &WebSocketError) + Send + Sync>;

/// Message handler that combines both callbacks
#[derive(Clone)]
pub struct MessageHandler {
    /// Primary callback for processing messages
    pub message_callback: MessageCallback,
    /// Error callback for handling processing failures
    pub error_callback: ErrorCallback,
}

impl MessageHandler {
    /// Create a new message handler with both callbacks
    pub fn new<F, E>(message_callback: F, error_callback: E) -> Self
    where
        F: Fn(&str) -> Result<(), WebSocketError> + Send + Sync + 'static,
        E: Fn(&str, &WebSocketError) + Send + Sync + 'static,
    {
        Self {
            message_callback: Arc::new(message_callback),
            error_callback: Arc::new(error_callback),
        }
    }

    /// Process a message using the callback system
    /// 1. Calls the primary callback with the message
    /// 2. If primary callback returns error, calls error callback with message and error
    pub fn handle_message(&self, message: &str) {
        match (self.message_callback)(message) {
            Ok(()) => {
                // Message processed successfully, no further action needed
            }
            Err(error) => {
                // Primary callback failed, call error callback
                (self.error_callback)(message, &error);
            }
        }
    }
}

impl std::fmt::Debug for MessageHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageHandler")
            .field("message_callback", &"<callback function>")
            .field("error_callback", &"<error callback function>")
            .finish()
    }
}

/// Builder for creating message handlers with fluent API
pub struct MessageHandlerBuilder {
    message_callback: Option<MessageCallback>,
    error_callback: Option<ErrorCallback>,
}

impl Default for MessageHandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageHandlerBuilder {
    /// Create a new message handler builder
    pub fn new() -> Self {
        Self {
            message_callback: None,
            error_callback: None,
        }
    }

    /// Set the primary message processing callback
    pub fn with_message_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Result<(), WebSocketError> + Send + Sync + 'static,
    {
        self.message_callback = Some(Arc::new(callback));
        self
    }

    /// Set the error handling callback
    pub fn with_error_callback<E>(mut self, callback: E) -> Self
    where
        E: Fn(&str, &WebSocketError) + Send + Sync + 'static,
    {
        self.error_callback = Some(Arc::new(callback));
        self
    }

    /// Build the message handler
    /// Returns an error if either callback is missing
    pub fn build(self) -> Result<MessageHandler, WebSocketError> {
        let message_callback = self.message_callback.ok_or_else(|| {
            WebSocketError::InvalidMessage("Message callback is required".to_string())
        })?;

        let error_callback = self.error_callback.ok_or_else(|| {
            WebSocketError::InvalidMessage("Error callback is required".to_string())
        })?;

        Ok(MessageHandler {
            message_callback,
            error_callback,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_message_handler_success() {
        let handler = MessageHandler::new(
            |_message| Ok(()),
            |_message, _error| {
                panic!("Error callback should not be called on success");
            },
        );

        // This should not panic
        handler.handle_message("test message");
    }

    #[test]
    fn test_message_handler_error() {
        use std::sync::{Arc, Mutex};

        let error_called = Arc::new(Mutex::new(false));
        let error_called_clone = error_called.clone();

        let handler = MessageHandler::new(
            |_message| Err(WebSocketError::InvalidMessage("Test error".to_string())),
            move |_message, _error| {
                *error_called_clone.lock().unwrap() = true;
            },
        );

        handler.handle_message("test message");
        assert!(*error_called.lock().unwrap());
    }

    #[test]
    fn test_message_handler_builder() {
        let handler = MessageHandlerBuilder::new()
            .with_message_callback(|_| Ok(()))
            .with_error_callback(|_, _| {})
            .build()
            .unwrap();

        handler.handle_message("test");
    }
}
