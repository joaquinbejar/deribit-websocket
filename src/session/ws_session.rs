//! WebSocket session management

use crate::config::WebSocketConfig;
use crate::model::ConnectionState;
use crate::model::subscription::SubscriptionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// WebSocket session manager
#[derive(Debug)]
pub struct WebSocketSession {
    config: Arc<WebSocketConfig>,
    state: Arc<Mutex<ConnectionState>>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl WebSocketSession {
    /// Create a new WebSocket session.
    ///
    /// Accepts anything convertible into `Arc<WebSocketConfig>`, which
    /// via Rust's blanket `impl<T> From<T> for Arc<T>` covers both an
    /// owned `WebSocketConfig` (wrapped once here) and an existing
    /// `Arc<WebSocketConfig>` shared with
    /// [`crate::client::DeribitWebSocketClient`] (zero extra copies).
    /// This keeps the constructor backward-compatible with pre-existing
    /// owned-value call sites while also giving the client a zero-copy
    /// path for its shared configuration.
    pub fn new(
        config: impl Into<Arc<WebSocketConfig>>,
        subscription_manager: Arc<Mutex<SubscriptionManager>>,
    ) -> Self {
        Self {
            config: config.into(),
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            subscription_manager,
        }
    }

    /// Get the current connection state
    pub async fn state(&self) -> ConnectionState {
        self.state.lock().await.clone()
    }

    /// Set the connection state
    pub async fn set_state(&self, new_state: ConnectionState) {
        *self.state.lock().await = new_state;
    }

    /// Get the configuration
    pub fn config(&self) -> &WebSocketConfig {
        &self.config
    }

    /// Get the subscription manager
    pub fn subscription_manager(&self) -> Arc<Mutex<SubscriptionManager>> {
        Arc::clone(&self.subscription_manager)
    }

    /// Check if session is connected
    pub async fn is_connected(&self) -> bool {
        matches!(
            *self.state.lock().await,
            ConnectionState::Connected | ConnectionState::Authenticated
        )
    }

    /// Check if session is authenticated
    pub async fn is_authenticated(&self) -> bool {
        matches!(*self.state.lock().await, ConnectionState::Authenticated)
    }

    /// Mark session as authenticated
    pub async fn mark_authenticated(&self) {
        self.set_state(ConnectionState::Authenticated).await;
    }

    /// Mark session as disconnected
    pub async fn mark_disconnected(&self) {
        self.set_state(ConnectionState::Disconnected).await;
        // Deactivate all subscriptions but preserve their entries so
        // `reactivate_subscriptions` can restore them on reconnect.
        self.subscription_manager.lock().await.deactivate_all();
    }

    /// Reactivate subscriptions after reconnection
    pub async fn reactivate_subscriptions(&self) {
        self.subscription_manager.lock().await.reactivate_all();
    }
}
