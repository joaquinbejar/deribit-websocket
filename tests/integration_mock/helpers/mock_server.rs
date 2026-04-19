//! Local `tokio-tungstenite` WebSocket server used by the mock
//! integration tests.
//!
//! Each test spawns a server on `127.0.0.1:0`, hands the split
//! sink/stream to a scenario closure, and the scenario drives the
//! conversation (reads the client's request, replies with canned
//! frames, optionally closes the socket). The returned [`MockServer`]
//! aborts the background task on `Drop` so tests cannot leak server
//! tasks between runs.

use std::future::Future;
use std::net::SocketAddr;

use futures_util::StreamExt;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

/// Sink half of the mock server's WebSocket stream passed to scenarios.
pub(crate) type ScenarioSink = SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>;

/// Stream half of the mock server's WebSocket stream passed to scenarios.
pub(crate) type ScenarioStream = SplitStream<WebSocketStream<tokio::net::TcpStream>>;

/// Handle to a running mock server.
///
/// The background task is aborted on `Drop`, so tests that create a
/// [`MockServer`] must keep it alive for the duration of the assertions
/// they care about. The [`MockServer::ws_url`] helper returns the
/// `ws://127.0.0.1:<port>/` URL the client should dial.
pub(crate) struct MockServer {
    /// Address the listener is bound to.
    pub(crate) addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl MockServer {
    /// Return the URL a client should use to connect to this server.
    pub(crate) fn ws_url(&self) -> String {
        format!("ws://{}/", self.addr)
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Spawn a local WebSocket server that accepts a single connection and
/// runs `scenario` against the split sink/stream.
///
/// The listener binds `127.0.0.1:0` so the OS picks a free port,
/// making parallel test execution safe. Any error during `accept` or
/// the WebSocket handshake silently terminates the task: tests assert
/// through the client side, not the server side, so the server simply
/// goes away on errors.
pub(crate) async fn spawn_mock_server<F, Fut>(scenario: F) -> MockServer
where
    F: FnOnce(ScenarioSink, ScenarioStream) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
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
        let (sink, stream) = ws.split();
        scenario(sink, stream).await;
    });
    MockServer { addr, handle }
}

/// Spawn a local WebSocket server that accepts exactly two connections
/// in sequence, running `first` against the first accept and `second`
/// against the second.
///
/// Used by the reconnect test: the first scenario closes the socket,
/// the client then calls `connect()` again, and the listener accepts
/// the new handshake for the second scenario.
pub(crate) async fn spawn_mock_server_twice<F1, Fut1, F2, Fut2>(first: F1, second: F2) -> MockServer
where
    F1: FnOnce(ScenarioSink, ScenarioStream) -> Fut1 + Send + 'static,
    Fut1: Future<Output = ()> + Send,
    F2: FnOnce(ScenarioSink, ScenarioStream) -> Fut2 + Send + 'static,
    Fut2: Future<Output = ()> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind localhost ephemeral port");
    let addr = listener
        .local_addr()
        .expect("read local addr of bound listener");
    let handle = tokio::spawn(async move {
        // First accept.
        let (socket, _peer) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => return,
        };
        let ws = match accept_async(socket).await {
            Ok(ws) => ws,
            Err(_) => return,
        };
        let (sink, stream) = ws.split();
        first(sink, stream).await;

        // Second accept.
        let (socket, _peer) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => return,
        };
        let ws = match accept_async(socket).await {
            Ok(ws) => ws,
            Err(_) => return,
        };
        let (sink, stream) = ws.split();
        second(sink, stream).await;
    });
    MockServer { addr, handle }
}
