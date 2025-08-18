//! WebSocket connection management

use crate::error::WebSocketError;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use url::Url;

/// WebSocket connection wrapper
#[derive(Debug)]
pub struct WebSocketConnection {
    url: Url,
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new(url: Url) -> Self {
        Self { url, stream: None }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<(), WebSocketError> {
        match connect_async(self.url.as_str()).await {
            Ok((stream, _response)) => {
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(WebSocketError::ConnectionFailed(format!(
                "Failed to connect: {}",
                e
            ))),
        }
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        self.stream = None;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Send a message
    pub async fn send(&mut self, message: String) -> Result<(), WebSocketError> {
        if let Some(stream) = &mut self.stream {
            match stream.send(Message::Text(message.into())).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.stream = None;
                    Err(WebSocketError::ConnectionFailed(format!(
                        "Failed to send message: {}",
                        e
                    )))
                }
            }
        } else {
            Err(WebSocketError::ConnectionClosed)
        }
    }

    /// Receive a message
    pub async fn receive(&mut self) -> Result<String, WebSocketError> {
        if let Some(stream) = &mut self.stream {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => Ok(text.to_string()),
                Some(Ok(Message::Close(_))) => {
                    self.stream = None;
                    Err(WebSocketError::ConnectionClosed)
                }
                Some(Ok(_)) => {
                    // Skip non-text messages (binary, ping, pong) - try again
                    Box::pin(self.receive()).await
                }
                Some(Err(e)) => {
                    self.stream = None;
                    Err(WebSocketError::ConnectionFailed(format!(
                        "Failed to receive message: {}",
                        e
                    )))
                }
                None => {
                    self.stream = None;
                    Err(WebSocketError::ConnectionClosed)
                }
            }
        } else {
            Err(WebSocketError::ConnectionClosed)
        }
    }

    /// Get the connection URL
    pub fn url(&self) -> &Url {
        &self.url
    }
}
