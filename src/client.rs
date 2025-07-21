//! WebSocket client implementation for Deribit

/// WebSocket client for Deribit
#[derive(Debug)]
pub struct DeribitWebSocketClient {
    /// WebSocket URL
    pub ws_url: String,
}

impl DeribitWebSocketClient {
    /// Create a new WebSocket client
    pub fn new(test_net: bool) -> Self {
        let ws_url = if test_net {
            "wss://test.deribit.com/ws/api/v2".to_string()
        } else {
            "wss://www.deribit.com/ws/api/v2".to_string()
        };

        Self { ws_url }
    }
}
