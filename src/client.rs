//! WebSocket client implementation for Deribit

use async_trait::async_trait;
use deribit_base::{DeribitClient, DeribitConfig, DeribitError, DeribitResult, DeribitUrls};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket client for Deribit
#[allow(dead_code)]
pub struct DeribitWebSocketClient {
    config: DeribitConfig,
    ws_url: String,
    ws_stream: Option<WsStream>,
    connected: bool,
}

impl DeribitWebSocketClient {
    /// Create a new WebSocket client
    pub fn new(config: DeribitConfig) -> Self {
        let ws_url = DeribitUrls::get_ws_url(config.test_net).to_string();

        Self {
            config,
            ws_url,
            ws_stream: None,
            connected: false,
        }
    }

    /// Send a message to the WebSocket
    pub async fn send_message(&mut self, message: Value) -> DeribitResult<()> {
        if let Some(ws_stream) = &mut self.ws_stream {
            let text = serde_json::to_string(&message).map_err(|e| {
                DeribitError::Serialization(format!("Failed to serialize message: {e}"))
            })?;
            let msg = Message::Text(text.into());

            ws_stream
                .send(msg)
                .await
                .map_err(|e| DeribitError::Connection(format!("Failed to send message: {e}")))?;

            Ok(())
        } else {
            Err(DeribitError::Connection("Not connected".to_string()))
        }
    }

    /// Receive a message from the WebSocket
    pub async fn receive_message(&mut self) -> DeribitResult<Option<Value>> {
        if let Some(ws_stream) = &mut self.ws_stream {
            match ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let value: Value = serde_json::from_str(&text).map_err(|e| {
                        DeribitError::Serialization(format!("Failed to parse message: {e}"))
                    })?;
                    Ok(Some(value))
                }
                Some(Ok(Message::Close(_))) => {
                    self.connected = false;
                    Ok(None)
                }
                Some(Err(e)) => Err(DeribitError::Connection(format!("WebSocket error: {e}"))),
                None => {
                    self.connected = false;
                    Ok(None)
                }
                _ => Ok(None), // Ignore other message types
            }
        } else {
            Err(DeribitError::Connection("Not connected".to_string()))
        }
    }
}

#[async_trait]
impl DeribitClient for DeribitWebSocketClient {
    type Error = DeribitError;

    async fn connect(&mut self) -> Result<(), Self::Error> {
        let (ws_stream, _) = connect_async(&self.ws_url).await.map_err(|e| {
            DeribitError::Connection(format!("Failed to connect to WebSocket: {e}"))
        })?;

        self.ws_stream = Some(ws_stream);
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Self::Error> {
        if let Some(mut ws_stream) = self.ws_stream.take() {
            let _ = ws_stream.close(None).await;
        }
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
