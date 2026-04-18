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
            loop {
                match stream.next().await {
                    Some(Ok(Message::Text(text))) => return Ok(text.to_string()),
                    Some(Ok(
                        Message::Binary(_)
                        | Message::Ping(_)
                        | Message::Pong(_)
                        | Message::Frame(_),
                    )) => {
                        // Skip non-text frames and continue draining the stream.
                        continue;
                    }
                    Some(Ok(Message::Close(_))) => {
                        self.stream = None;
                        return Err(WebSocketError::ConnectionClosed);
                    }
                    Some(Err(e)) => {
                        self.stream = None;
                        return Err(WebSocketError::ConnectionFailed(format!(
                            "Failed to receive message: {}",
                            e
                        )));
                    }
                    None => {
                        self.stream = None;
                        return Err(WebSocketError::ConnectionClosed);
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;

    /// Spawn a local WebSocket server that accepts a single connection and
    /// runs `send_frames` over the server sink. A concurrent read-drain task
    /// keeps the peer side from blocking on auto-pong back-pressure. Returns
    /// the bound address and a `JoinHandle` for the acceptor task.
    async fn spawn_mock_server<F, Fut>(send_frames: F) -> (SocketAddr, JoinHandle<()>)
    where
        F: FnOnce(
                futures_util::stream::SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
            ) -> Fut
            + Send
            + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind localhost ephemeral port");
        let addr = listener
            .local_addr()
            .expect("read local addr of bound listener");
        let handle = tokio::spawn(async move {
            let (socket, _peer) = match listener.accept().await {
                Ok(pair) => pair,
                Err(_) => return,
            };
            let ws = match accept_async(socket).await {
                Ok(ws) => ws,
                Err(_) => return,
            };
            let (sink, mut stream) = ws.split();
            // Drain anything the client sends (including auto-pongs from
            // tungstenite) so the client's write side never blocks on a
            // full socket buffer. The drain task exits when the client
            // disconnects at end of test.
            let drain = tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    if msg.is_err() {
                        break;
                    }
                }
            });
            send_frames(sink).await;
            let _ = drain.await;
        });
        (addr, handle)
    }

    fn ws_url(addr: SocketAddr) -> Url {
        Url::parse(&format!("ws://{}/", addr)).expect("valid ws url")
    }

    async fn connect_client(addr: SocketAddr) -> WebSocketConnection {
        let mut client = WebSocketConnection::new(ws_url(addr));
        client
            .connect()
            .await
            .expect("client connects to mock server");
        client
    }

    #[tokio::test]
    async fn test_receive_skips_ping_frames_then_returns_text() {
        let (addr, server) = spawn_mock_server(|mut sink| async move {
            for _ in 0..10_000 {
                if sink.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return;
                }
            }
            let _ = sink.send(Message::Text("payload".into())).await;
        })
        .await;

        let mut client = connect_client(addr).await;
        let received = client.receive().await.expect("receive returns the text");
        assert_eq!(received, "payload");
        drop(client);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_receive_skips_binary_frames_then_returns_text() {
        let (addr, server) = spawn_mock_server(|mut sink| async move {
            for _ in 0..100 {
                if sink
                    .send(Message::Binary(vec![1, 2, 3].into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            let _ = sink.send(Message::Text("payload".into())).await;
        })
        .await;

        let mut client = connect_client(addr).await;
        let received = client.receive().await.expect("receive returns the text");
        assert_eq!(received, "payload");
        drop(client);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_receive_skips_pong_frames_then_returns_text() {
        let (addr, server) = spawn_mock_server(|mut sink| async move {
            for _ in 0..100 {
                if sink.send(Message::Pong(Vec::new().into())).await.is_err() {
                    return;
                }
            }
            let _ = sink.send(Message::Text("payload".into())).await;
        })
        .await;

        let mut client = connect_client(addr).await;
        let received = client.receive().await.expect("receive returns the text");
        assert_eq!(received, "payload");
        drop(client);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_receive_returns_closed_on_close_frame() {
        let (addr, server) = spawn_mock_server(|mut sink| async move {
            let _ = sink.send(Message::Close(None)).await;
            let _ = sink.close().await;
        })
        .await;

        let mut client = connect_client(addr).await;
        let result = client.receive().await;
        assert!(
            matches!(result, Err(WebSocketError::ConnectionClosed)),
            "expected ConnectionClosed, got {:?}",
            result
        );
        assert!(
            !client.is_connected(),
            "stream should be cleared after close frame"
        );
        drop(client);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_receive_skips_mixed_non_text_frames() {
        let (addr, server) = spawn_mock_server(|mut sink| async move {
            for _ in 0..200 {
                if sink.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return;
                }
                if sink
                    .send(Message::Binary(vec![9, 9, 9].into()))
                    .await
                    .is_err()
                {
                    return;
                }
                if sink.send(Message::Pong(Vec::new().into())).await.is_err() {
                    return;
                }
            }
            let _ = sink.send(Message::Text("payload".into())).await;
        })
        .await;

        let mut client = connect_client(addr).await;
        let received = client.receive().await.expect("receive returns the text");
        assert_eq!(received, "payload");
        drop(client);
        server.await.expect("server task did not panic");
    }
}
