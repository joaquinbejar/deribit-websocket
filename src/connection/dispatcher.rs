//! WebSocket dispatcher task.
//!
//! The dispatcher owns the split WebSocket stream inside a single tokio
//! task. It multiplexes outbound JSON-RPC requests and inbound frames:
//!
//! - Each `send_request` hands the dispatcher a [`JsonRpcRequest`] plus a
//!   `oneshot` responder. The dispatcher records the waiter keyed by the
//!   request `id`, serializes the request, and writes it to the sink.
//! - Every inbound text frame is inspected for a numeric `id` field. If a
//!   waiter exists for that id, the parsed [`JsonRpcResponse`] is routed
//!   to the waiter. Otherwise the raw frame text is forwarded on the
//!   notification channel.
//! - Non-text frames (binary, ping, pong, raw frame) are ignored. A
//!   `Close` frame or a stream error terminates the dispatcher loop;
//!   pending waiters are drained with [`WebSocketError::ConnectionClosed`].
//!
//! This replaces the previous single-threaded "send then receive the next
//! frame" pattern, which raced against server-pushed notifications and
//! could hand a notification back to a caller expecting its own response.

use std::collections::HashMap;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use url::Url;

use crate::error::WebSocketError;
use crate::model::ws_types::{JsonRpcRequest, JsonRpcResponse};

/// Commands accepted by the dispatcher task.
///
/// Public only within the crate — users drive the dispatcher through
/// [`Dispatcher::send_request`] / [`Dispatcher::shutdown`].
#[derive(Debug)]
enum DispatcherCommand {
    /// Send a pre-serialised JSON-RPC request and route the matching
    /// response back via the attached oneshot responder.
    ///
    /// `payload` is the wire-ready JSON text and `id` is the request id
    /// extracted from it; both are computed by the caller (see
    /// [`Dispatcher::send_request`]) so the dispatcher loop never needs
    /// to deserialise or re-serialise the request, and the caller does
    /// not have to clone the [`JsonRpcRequest`] just to surface
    /// id-validation or serialisation errors.
    SendRequest {
        /// The serialised JSON-RPC request, ready for `sink.send`.
        payload: String,
        /// JSON-RPC `id` of `payload`. Used for the duplicate-in-flight
        /// check and for cancel-on-timeout routing.
        id: u64,
        /// Channel used to deliver the response (or an error) to the caller.
        responder: oneshot::Sender<Result<JsonRpcResponse, WebSocketError>>,
    },
    /// Cancel a pending waiter by id. Sent from `send_request` after a
    /// timeout so the dispatcher does not hold a dangling sender for a
    /// caller that already gave up.
    CancelRequest {
        /// JSON-RPC request id whose waiter should be evicted.
        id: u64,
    },
    /// Stop the dispatcher loop. In-flight waiters are drained with
    /// [`WebSocketError::ConnectionClosed`].
    Shutdown,
}

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Handle to a running dispatcher task.
///
/// Owns the command channel, the inbound notification receiver, and the
/// task join handle. Dropping a `Dispatcher` without calling
/// [`Dispatcher::shutdown`] leaves the spawned task running until the
/// underlying stream closes — prefer an explicit shutdown.
#[derive(Debug)]
pub struct Dispatcher {
    cmd_tx: mpsc::Sender<DispatcherCommand>,
    notification_rx: Mutex<mpsc::Receiver<String>>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
    request_timeout: Duration,
}

impl Dispatcher {
    /// Connect to `url`, split the resulting stream, and spawn the
    /// dispatcher task that services JSON-RPC requests and forwards
    /// notifications.
    ///
    /// The WebSocket handshake (`connect_async`) is bounded by
    /// `connection_timeout`: a stalled handshake (TCP + TLS + HTTP
    /// upgrade) fails fast with [`WebSocketError::Timeout`] instead of
    /// hanging indefinitely.
    ///
    /// # Arguments
    ///
    /// - `url` — WebSocket URL to connect to.
    /// - `connection_timeout` — upper bound on the `connect_async`
    ///   handshake (TCP + TLS + HTTP upgrade). Exceeded means the peer
    ///   never completed the upgrade.
    /// - `request_timeout` — upper bound for each `send_request` call.
    /// - `notification_capacity` — depth of the bounded notifications
    ///   channel. Strategy A (await-send) backpressure: if the consumer
    ///   falls behind, the dispatcher task blocks on the channel send,
    ///   which stops it polling the WebSocket stream, which fills the
    ///   TCP recv buffer, which makes the server throttle. No frames are
    ///   dropped due to backpressure; if the notification receiver has
    ///   been closed (for example after shutdown or disconnect), the
    ///   affected frames are discarded. Every full-channel event emits
    ///   a `tracing::warn!` so slow consumers are visible in logs.
    /// - `cmd_capacity` — depth of the outbound command channel. Same
    ///   Strategy A (await-send) applies: callers of `send_request` /
    ///   `shutdown` block when the dispatcher has not drained prior
    ///   commands yet.
    ///
    /// # Errors
    ///
    /// - [`WebSocketError::Timeout`] if the handshake does not complete
    ///   within `connection_timeout`.
    /// - [`WebSocketError::ConnectionFailed`] if the underlying
    ///   `connect_async` handshake returns an error (DNS, TCP refused,
    ///   TLS failure, bad upgrade response, etc.).
    pub async fn connect(
        url: Url,
        connection_timeout: Duration,
        request_timeout: Duration,
        notification_capacity: usize,
        cmd_capacity: usize,
    ) -> Result<Self, WebSocketError> {
        let handshake = tokio::time::timeout(connection_timeout, connect_async(url.as_str()))
            .await
            .map_err(|_| {
                WebSocketError::Timeout(format!(
                    "connection_timeout {:?} elapsed during handshake",
                    connection_timeout
                ))
            })?;
        let (stream, _response) = handshake
            .map_err(|e| WebSocketError::ConnectionFailed(format!("Failed to connect: {}", e)))?;
        let (sink, stream) = stream.split();
        let (cmd_tx, cmd_rx) = mpsc::channel::<DispatcherCommand>(cmd_capacity);
        let (notif_tx, notif_rx) = mpsc::channel::<String>(notification_capacity);
        let join_handle = tokio::spawn(run_dispatcher(sink, stream, cmd_rx, notif_tx));
        Ok(Self {
            cmd_tx,
            notification_rx: Mutex::new(notif_rx),
            join_handle: Mutex::new(Some(join_handle)),
            request_timeout,
        })
    }

    /// Submit a JSON-RPC request and wait for its matching response.
    ///
    /// The request is enqueued on the dispatcher command channel; the
    /// dispatcher writes it to the sink and records a waiter keyed by the
    /// numeric `id`. When a frame with the same id arrives, the parsed
    /// response is delivered back through this call. The configured
    /// `request_timeout` covers the entire flow — enqueue, write, and the
    /// wait for the response — not just the wait phase.
    ///
    /// On timeout the call sends a `CancelRequest` command so the
    /// dispatcher evicts the now-orphaned waiter; this prevents unbounded
    /// growth of the waiter map under repeated timeouts.
    ///
    /// # Errors
    ///
    /// - [`WebSocketError::DispatcherDead`] if the dispatcher task has
    ///   stopped or its responder is dropped before it can reply.
    /// - [`WebSocketError::Timeout`] if the deadline elapses before a
    ///   response arrives. Includes time spent on the command channel.
    /// - [`WebSocketError::InvalidMessage`] if the request `id` is not a
    ///   `u64`, the request `id` is already in flight, or the response
    ///   payload cannot be parsed.
    /// - [`WebSocketError::Serialization`] if the request cannot be
    ///   serialised to JSON (for example a non-finite `f64` in `params`).
    /// - [`WebSocketError::ConnectionFailed`] if the sink reports an
    ///   error while writing.
    /// - [`WebSocketError::ConnectionClosed`] if the stream closed while
    ///   the waiter was pending.
    ///
    /// `request` is borrowed: serialisation happens up front in the
    /// caller's task, so the caller retains ownership and does not need
    /// to clone the (potentially large) `params` for error-context
    /// capture on the success path.
    pub async fn send_request(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse, WebSocketError> {
        // id-validation and serialisation are done synchronously here
        // (before the deadline starts) so failures surface without
        // round-tripping through the dispatcher task.
        let id = request
            .id
            .as_u64()
            .ok_or_else(|| WebSocketError::InvalidMessage("request id must be u64".to_string()))?;
        let payload = serde_json::to_string(request)?;
        let (responder, waiter) = oneshot::channel();
        let cmd = DispatcherCommand::SendRequest {
            payload,
            id,
            responder,
        };
        let outcome = tokio::time::timeout(self.request_timeout, async {
            self.cmd_tx
                .send(cmd)
                .await
                .map_err(|_| WebSocketError::DispatcherDead)?;
            waiter.await.map_err(|_| WebSocketError::DispatcherDead)?
        })
        .await;
        match outcome {
            Ok(result) => result,
            Err(_elapsed) => {
                let _ = self
                    .cmd_tx
                    .send(DispatcherCommand::CancelRequest { id })
                    .await;
                Err(WebSocketError::Timeout(format!(
                    "request_timeout {:?} elapsed",
                    self.request_timeout
                )))
            }
        }
    }

    /// Await the next notification (or unmatched frame) from the server.
    ///
    /// Returns `None` once the dispatcher exits and drains the
    /// notification channel.
    pub async fn next_notification(&self) -> Option<String> {
        self.notification_rx.lock().await.recv().await
    }

    /// Signal the dispatcher to stop and await its task handle.
    ///
    /// After this call, `send_request` will return
    /// [`WebSocketError::DispatcherDead`] and `next_notification` will
    /// drain the remaining buffered frames and then return `None`.
    ///
    /// # Errors
    ///
    /// This method currently never returns an error — any failure in the
    /// shutdown send or the task join is logged and swallowed.
    pub async fn shutdown(&self) -> Result<(), WebSocketError> {
        // Best-effort: if the channel is already closed the task has
        // already exited.
        let _ = self.cmd_tx.send(DispatcherCommand::Shutdown).await;
        if let Some(handle) = self.join_handle.lock().await.take()
            && let Err(e) = handle.await
        {
            tracing::warn!(error = %e, "dispatcher task join failed");
        }
        Ok(())
    }
}

/// Core dispatcher loop. Multiplexes outbound commands and inbound frames.
async fn run_dispatcher(
    mut sink: WsSink,
    mut stream: WsStream,
    mut cmd_rx: mpsc::Receiver<DispatcherCommand>,
    notif_tx: mpsc::Sender<String>,
) {
    let mut waiters: HashMap<u64, oneshot::Sender<Result<JsonRpcResponse, WebSocketError>>> =
        HashMap::new();

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(DispatcherCommand::SendRequest { payload, id, responder }) => {
                        // id-validation and serialisation already happened
                        // in `Dispatcher::send_request` (caller's task); the
                        // only check that has to live in the dispatcher
                        // task is duplicate-in-flight, because only this
                        // task owns the `waiters` map.
                        //
                        // Reject duplicate in-flight ids — silent overwrite
                        // would orphan the existing waiter and could
                        // mis-route the original response.
                        if waiters.contains_key(&id) {
                            let _ = responder.send(Err(WebSocketError::InvalidMessage(
                                format!("duplicate in-flight request id {}", id),
                            )));
                            continue;
                        }
                        // Register the waiter BEFORE writing so a fast
                        // server reply can find it.
                        waiters.insert(id, responder);
                        if let Err(e) = sink.send(Message::Text(payload.into())).await {
                            if let Some(r) = waiters.remove(&id) {
                                let _ = r.send(Err(WebSocketError::ConnectionFailed(
                                    format!("sink send: {}", e),
                                )));
                            }
                            tracing::warn!(error = %e, "sink send failed; dispatcher exiting");
                            break;
                        }
                    }
                    Some(DispatcherCommand::CancelRequest { id }) => {
                        // Drop the orphaned waiter. If a response races in
                        // before this command, the response path already
                        // removed it and this is a no-op.
                        let _ = waiters.remove(&id);
                    }
                    Some(DispatcherCommand::Shutdown) | None => {
                        tracing::debug!("dispatcher shutdown requested");
                        break;
                    }
                }
            }
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Text(t))) => {
                        let text: String = t.to_string();
                        // Inspect the id field without a full parse; a
                        // missing/non-numeric id means notification or
                        // unmatched frame.
                        let id_opt = serde_json::from_str::<serde_json::Value>(&text)
                            .ok()
                            .and_then(|v| v.get("id").and_then(|i| i.as_u64()));
                        if let Some(id) = id_opt
                            && let Some(responder) = waiters.remove(&id)
                        {
                            let resp_res = serde_json::from_str::<JsonRpcResponse>(&text)
                                .map_err(|e| WebSocketError::InvalidMessage(
                                    format!("response parse: {}", e),
                                ));
                            let _ = responder.send(resp_res);
                            continue;
                        }
                        // Notification or unmatched id — forward raw text
                        // on the notification channel. Strategy A
                        // backpressure: `try_send` first so the common
                        // fast path avoids allocating a wake; on `Full`
                        // emit a `tracing::warn!` and then block on
                        // `send().await` so the slow consumer applies
                        // backpressure to the dispatcher → TCP buffer →
                        // server. No frames are dropped due to
                        // backpressure; the only drop path is the
                        // `Closed` branch below, reached after the
                        // receiver has been dropped.
                        match notif_tx.try_send(text) {
                            Ok(()) => {}
                            Err(mpsc::error::TrySendError::Full(text)) => {
                                tracing::warn!(
                                    capacity = notif_tx.max_capacity(),
                                    "notification channel full; applying backpressure"
                                );
                                if notif_tx.send(text).await.is_err() {
                                    tracing::trace!(
                                        "notification channel closed; dropping frame"
                                    );
                                }
                            }
                            Err(mpsc::error::TrySendError::Closed(_)) => {
                                tracing::trace!(
                                    "notification channel closed; dropping frame"
                                );
                            }
                        }
                    }
                    Some(Ok(
                        Message::Binary(_)
                        | Message::Ping(_)
                        | Message::Pong(_)
                        | Message::Frame(_),
                    )) => continue,
                    Some(Ok(Message::Close(_))) => {
                        tracing::debug!("received close frame; dispatcher exiting");
                        break;
                    }
                    None => {
                        tracing::debug!("stream ended; dispatcher exiting");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "websocket stream error; dispatcher exiting");
                        break;
                    }
                }
            }
        }
    }

    // Drain pending waiters so callers receive a deterministic error.
    for (_, responder) in waiters.drain() {
        let _ = responder.send(Err(WebSocketError::ConnectionClosed));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex as TokioMutex;
    use tokio::task::JoinHandle;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;

    /// Spawn a local WebSocket server that accepts a single connection and
    /// runs the supplied scenario against the split sink/stream. Returns
    /// the bound address and the acceptor `JoinHandle`.
    async fn spawn_mock_server<F, Fut>(scenario: F) -> (SocketAddr, JoinHandle<()>)
    where
        F: FnOnce(
                futures_util::stream::SplitSink<
                    tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
                    Message,
                >,
                futures_util::stream::SplitStream<
                    tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
                >,
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
            let (sink, stream) = ws.split();
            scenario(sink, stream).await;
        });
        (addr, handle)
    }

    fn ws_url(addr: SocketAddr) -> Url {
        Url::parse(&format!("ws://{}/", addr)).expect("valid ws url")
    }

    fn make_request(id: u64, method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(id)),
            method: method.to_string(),
            params: None,
        }
    }

    /// Spawn a TCP listener that accepts one connection at the raw TCP
    /// layer and then holds the socket without performing the WebSocket
    /// upgrade. From the client's point of view the TCP connect
    /// succeeds but the HTTP/WebSocket handshake never completes, which
    /// is the stall mode `connection_timeout` must defend against.
    ///
    /// The returned [`JoinHandle`] owns the socket for the lifetime of
    /// the test and should be aborted when the test is done.
    async fn spawn_stalled_handshake_listener() -> (SocketAddr, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind localhost ephemeral port");
        let addr = listener
            .local_addr()
            .expect("read local addr of bound listener");
        let handle = tokio::spawn(async move {
            if let Ok((socket, _peer)) = listener.accept().await {
                // Hold the socket well past any realistic test deadline.
                // The test aborts this task before it ever wakes up.
                tokio::time::sleep(Duration::from_secs(60)).await;
                drop(socket);
            }
        });
        (addr, handle)
    }

    /// Regression test for issue #50: if the peer accepts the TCP
    /// connection but never completes the WebSocket upgrade, `connect`
    /// must fail fast with `WebSocketError::Timeout` within the
    /// configured `connection_timeout`.
    #[tokio::test]
    async fn test_connect_times_out_when_handshake_stalls() {
        let (addr, server) = spawn_stalled_handshake_listener().await;
        let url = ws_url(addr);

        let start = std::time::Instant::now();
        let result = Dispatcher::connect(
            url,
            Duration::from_millis(100),
            Duration::from_secs(5),
            16,
            16,
        )
        .await;
        let elapsed = start.elapsed();

        assert!(
            matches!(result, Err(WebSocketError::Timeout(_))),
            "stalled handshake must surface WebSocketError::Timeout, got {:?}",
            result
        );
        // 5x headroom over the 100ms deadline to tolerate CI scheduler jitter.
        assert!(
            elapsed < Duration::from_millis(500),
            "connect must return within 500ms of a 100ms connection_timeout, took {:?}",
            elapsed
        );
        if let Err(WebSocketError::Timeout(msg)) = result {
            assert!(
                msg.contains("handshake"),
                "error message should mention the handshake, got: {}",
                msg
            );
            assert!(
                msg.contains("connection_timeout"),
                "error message should name the config field, got: {}",
                msg
            );
        }

        server.abort();
    }

    #[tokio::test]
    async fn test_dispatch_matches_single_request_response_by_id() {
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            // Read the single request, echo back with matching id.
            if let Some(Ok(Message::Text(t))) = stream.next().await {
                let v: serde_json::Value = serde_json::from_str(&t).expect("server parses request");
                let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {}
                });
                let _ = sink.send(Message::Text(resp.to_string().into())).await;
            }
            // Keep the sink alive briefly so the client can read.
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_secs(5),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let req = make_request(42, "public/test");
        let response = dispatcher
            .send_request(&req)
            .await
            .expect("response arrives");
        assert_eq!(response.id, serde_json::Value::Number(42.into()));
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_routes_notifications_to_notification_channel() {
        let (addr, server) = spawn_mock_server(|mut sink, _stream| async move {
            let notif = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "subscription",
                "params": { "channel": "ticker.BTC-PERPETUAL", "data": {} }
            });
            let _ = sink.send(Message::Text(notif.to_string().into())).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_secs(5),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let text = tokio::time::timeout(Duration::from_secs(2), dispatcher.next_notification())
            .await
            .expect("notification arrives within timeout")
            .expect("notification channel still open");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parses as JSON");
        assert_eq!(
            v.get("method").and_then(|m| m.as_str()),
            Some("subscription")
        );
        assert!(v.get("id").is_none(), "notifications carry no id");
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_concurrent_requests_each_get_their_response() {
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            let mut ids: Vec<u64> = Vec::new();
            for _ in 0..3 {
                if let Some(Ok(Message::Text(t))) = stream.next().await {
                    let v: serde_json::Value =
                        serde_json::from_str(&t).expect("server parses request");
                    if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                        ids.push(id);
                    }
                }
            }
            // Reply in reverse order.
            ids.reverse();
            for id in ids {
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "echo": id }
                });
                let _ = sink.send(Message::Text(resp.to_string().into())).await;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
        .await;

        let dispatcher = Arc::new(
            Dispatcher::connect(
                ws_url(addr),
                Duration::from_secs(5),
                Duration::from_secs(5),
                16,
                16,
            )
            .await
            .expect("dispatcher connects"),
        );

        let mut handles = Vec::new();
        for id in [10u64, 11, 12] {
            let d = Arc::clone(&dispatcher);
            handles.push(tokio::spawn(async move {
                let req = make_request(id, "public/test");
                d.send_request(&req).await
            }));
        }
        for (expected_id, handle) in [10u64, 11, 12].into_iter().zip(handles) {
            let response = handle
                .await
                .expect("task did not panic")
                .expect("response arrives");
            assert_eq!(
                response.id,
                serde_json::Value::Number(expected_id.into()),
                "id mismatch"
            );
        }
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_concurrent_requests_under_notification_flood() {
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            let mut ids: Vec<u64> = Vec::new();
            for _ in 0..3 {
                if let Some(Ok(Message::Text(t))) = stream.next().await {
                    let v: serde_json::Value =
                        serde_json::from_str(&t).expect("server parses request");
                    if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                        ids.push(id);
                    }
                }
            }
            // Flood with notifications.
            for n in 0..100u32 {
                let notif = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "subscription",
                    "params": { "seq": n }
                });
                if sink
                    .send(Message::Text(notif.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            // Now respond out of order.
            ids.reverse();
            for id in ids {
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "echo": id }
                });
                let _ = sink.send(Message::Text(resp.to_string().into())).await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        // Use a generous notification buffer so we don't stall on the 100
        // burst while the consumer is still spinning up.
        let dispatcher = Arc::new(
            Dispatcher::connect(
                ws_url(addr),
                Duration::from_secs(5),
                Duration::from_secs(5),
                512,
                16,
            )
            .await
            .expect("dispatcher connects"),
        );

        // Drain notifications concurrently.
        let notif_count = Arc::new(TokioMutex::new(Vec::<String>::new()));
        let drainer_count = Arc::clone(&notif_count);
        let drainer = {
            let d = Arc::clone(&dispatcher);
            tokio::spawn(async move {
                // Drain up to 200 notifications or stop when channel closes.
                for _ in 0..200 {
                    match tokio::time::timeout(Duration::from_millis(500), d.next_notification())
                        .await
                    {
                        Ok(Some(frame)) => drainer_count.lock().await.push(frame),
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            })
        };

        let mut handles = Vec::new();
        for id in [100u64, 101, 102] {
            let d = Arc::clone(&dispatcher);
            handles.push(tokio::spawn(async move {
                let req = make_request(id, "public/test");
                d.send_request(&req).await
            }));
        }
        for (expected_id, handle) in [100u64, 101, 102].into_iter().zip(handles) {
            let response = handle
                .await
                .expect("task did not panic")
                .expect("response arrives under flood");
            assert_eq!(
                response.id,
                serde_json::Value::Number(expected_id.into()),
                "id mismatch under flood"
            );
        }

        // Give the drainer a chance to see the rest of the flood.
        drainer.await.expect("drainer did not panic");
        let frames = notif_count.lock().await;
        assert!(
            frames.len() >= 100,
            "expected at least 100 notifications, got {}",
            frames.len()
        );
        for frame in frames.iter() {
            let v: serde_json::Value = serde_json::from_str(frame).expect("notification is JSON");
            assert!(
                v.get("id").is_none(),
                "notifications must not carry an id; got: {}",
                frame
            );
        }
        drop(frames);
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_request_timeout() {
        let (addr, server) = spawn_mock_server(|_sink, mut stream| async move {
            // Read the request, do NOT respond. Hold the sink open until
            // the test drops the client.
            let _ = stream.next().await;
            // Keep the task alive a bit longer than the client's timeout.
            tokio::time::sleep(Duration::from_millis(600)).await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_millis(200),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let start = std::time::Instant::now();
        let req = make_request(7, "public/test");
        let result = dispatcher.send_request(&req).await;
        let elapsed = start.elapsed();
        assert!(
            matches!(result, Err(WebSocketError::Timeout(_))),
            "expected Timeout, got {:?}",
            result
        );
        assert!(
            elapsed < Duration::from_millis(400),
            "timeout should fire near 200ms, elapsed = {:?}",
            elapsed
        );
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_pending_waiters_drained_on_disconnect() {
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            // Read the request, then close the connection.
            let _ = stream.next().await;
            let _ = sink.send(Message::Close(None)).await;
            let _ = sink.close().await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_secs(5),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let req = make_request(99, "public/test");
        let result = dispatcher.send_request(&req).await;
        assert!(
            matches!(result, Err(WebSocketError::ConnectionClosed)),
            "expected ConnectionClosed after server close, got {:?}",
            result
        );
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_timeout_evicts_waiter_and_late_response_routed_to_notifications() {
        // After a request times out, the dispatcher must evict the waiter
        // so the map does not grow without bound. A late-arriving response
        // for the timed-out id then has no waiter and should land on the
        // notification channel like any other unmatched frame.
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            let req = match stream.next().await {
                Some(Ok(Message::Text(t))) => t.to_string(),
                _ => return,
            };
            let v: serde_json::Value = match serde_json::from_str(&req) {
                Ok(v) => v,
                Err(_) => return,
            };
            let id = match v.get("id").and_then(|i| i.as_u64()) {
                Some(id) => id,
                None => return,
            };
            // Wait long enough for the client to time out and cancel.
            tokio::time::sleep(Duration::from_millis(300)).await;
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "late": true }
            });
            let _ = sink.send(Message::Text(resp.to_string().into())).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_millis(100),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let req = make_request(7, "public/test");
        let result = dispatcher.send_request(&req).await;
        assert!(
            matches!(result, Err(WebSocketError::Timeout(_))),
            "expected Timeout, got {:?}",
            result
        );

        // Late response for id=7 should now arrive on the notification
        // channel, proving the waiter was evicted (otherwise it would have
        // been routed to the dropped oneshot and silently lost).
        let text = tokio::time::timeout(Duration::from_secs(2), dispatcher.next_notification())
            .await
            .expect("late response forwarded within timeout")
            .expect("notification channel still open");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parses as JSON");
        assert_eq!(v.get("id").and_then(|i| i.as_u64()), Some(7));
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_duplicate_in_flight_id_rejected() {
        // Two requests with the same id must not silently overwrite each
        // other. The second one is rejected with InvalidMessage; the first
        // continues to wait for its response. The server intentionally
        // delays its reply so the first waiter is still in the map when
        // the duplicate fires.
        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            let req = match stream.next().await {
                Some(Ok(Message::Text(t))) => t.to_string(),
                _ => return,
            };
            // Hold the reply long enough for the test to fire the
            // duplicate while the first waiter is still registered.
            tokio::time::sleep(Duration::from_millis(300)).await;
            let v: serde_json::Value = match serde_json::from_str(&req) {
                Ok(v) => v,
                Err(_) => return,
            };
            if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "ok": true }
                });
                let _ = sink.send(Message::Text(resp.to_string().into())).await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        let dispatcher = Arc::new(
            Dispatcher::connect(
                ws_url(addr),
                Duration::from_secs(5),
                Duration::from_secs(5),
                16,
                16,
            )
            .await
            .expect("dispatcher connects"),
        );

        // Spawn the first request; it parks until the server replies.
        let first = {
            let d = Arc::clone(&dispatcher);
            tokio::spawn(async move {
                let req = make_request(42, "public/test");
                d.send_request(&req).await
            })
        };

        // Briefly yield so the dispatcher registers the waiter for id=42.
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Second request with the same id must be rejected immediately.
        let dup_req = make_request(42, "public/test");
        let dup = dispatcher.send_request(&dup_req).await;
        assert!(
            matches!(dup, Err(WebSocketError::InvalidMessage(ref m)) if m.contains("duplicate")),
            "duplicate id must be rejected with InvalidMessage, got {:?}",
            dup
        );

        // The original request still completes successfully.
        let response = first
            .await
            .expect("first task did not panic")
            .expect("first request completes despite duplicate rejection");
        assert_eq!(
            response.id,
            serde_json::Value::Number(42u64.into()),
            "first request must still get its response"
        );
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test]
    async fn test_dispatch_unmatched_id_forwarded_to_notifications() {
        let (addr, server) = spawn_mock_server(|mut sink, _stream| async move {
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 999,
                "result": {}
            });
            let _ = sink.send(Message::Text(resp.to_string().into())).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_secs(5),
            16,
            16,
        )
        .await
        .expect("dispatcher connects");
        let text = tokio::time::timeout(Duration::from_secs(2), dispatcher.next_notification())
            .await
            .expect("unmatched id arrives within timeout")
            .expect("notification channel still open");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parses as JSON");
        assert_eq!(v.get("id").and_then(|i| i.as_u64()), Some(999));
        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_dispatch_concurrent_requests_run_in_parallel() {
        // Acceptance criterion for issue #44 — N concurrent requests must
        // complete in roughly one server-side hold time, not N times it.
        // The server reads N requests, holds for HOLD, then replies all
        // at once. If the dispatcher serialized requests, total wall time
        // would be approximately N * HOLD; in parallel it is roughly HOLD
        // plus scheduling overhead.
        const N: usize = 20;
        const HOLD: Duration = Duration::from_millis(200);

        let (addr, server) = spawn_mock_server(|mut sink, mut stream| async move {
            let mut ids: Vec<u64> = Vec::with_capacity(N);
            // Read all requests first so the test cannot mistake ordered
            // request/response RTTs for parallelism.
            while ids.len() < N {
                match stream.next().await {
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t)
                            && let Some(id) = v.get("id").and_then(|i| i.as_u64())
                        {
                            ids.push(id);
                        }
                    }
                    _ => return,
                }
            }
            tokio::time::sleep(HOLD).await;
            for id in ids {
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "echo": id }
                });
                if sink
                    .send(Message::Text(resp.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
        .await;

        let dispatcher = Arc::new(
            Dispatcher::connect(
                ws_url(addr),
                Duration::from_secs(5),
                Duration::from_secs(5),
                64,
                64,
            )
            .await
            .expect("dispatcher connects"),
        );

        let start = std::time::Instant::now();
        let mut handles = Vec::with_capacity(N);
        for i in 0..N {
            let d = Arc::clone(&dispatcher);
            let id = 1000u64 + i as u64;
            handles.push(tokio::spawn(async move {
                let req = make_request(id, "public/test");
                d.send_request(&req).await
            }));
        }
        for handle in handles {
            handle
                .await
                .expect("task did not panic")
                .expect("response arrives");
        }
        let elapsed = start.elapsed();

        // Serial would be N * HOLD = 4000ms. Parallel is roughly HOLD plus
        // scheduling overhead. Use a generous bound (3 * HOLD = 600ms)
        // so the test is not flaky on slow CI hosts.
        let serial_lower_bound = HOLD * (N as u32);
        let parallel_upper_bound = HOLD * 3;
        assert!(
            elapsed < parallel_upper_bound,
            "concurrent requests took {:?}; serial would be {:?}, parallel bound is {:?}",
            elapsed,
            serial_lower_bound,
            parallel_upper_bound
        );

        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }

    /// Regression test for issue #51: the notification channel is
    /// bounded (Strategy A, await-send) and preserves FIFO order under
    /// backpressure.
    ///
    /// Setup: the mock server sends 4 notifications with increasing
    /// sequence numbers, then holds the socket open. The dispatcher is
    /// created with `notification_capacity = 1`, so frames 2–4 must
    /// force the dispatcher task to await on `notif_tx.send` (the full
    /// path exercised by the `try_send` observability branch). The test
    /// then sleeps long enough for all frames to traverse the pipeline
    /// (buffered, awaiting, or on the wire), drains the receiver, and
    /// asserts every frame arrives exactly once and in order.
    ///
    /// Guards against: silent frame drops under backpressure and
    /// channel reordering when the producer has to await on a full
    /// channel between frames.
    #[tokio::test]
    async fn test_notification_channel_is_bounded_and_preserves_order() {
        const FRAME_COUNT: u64 = 4;
        let (addr, server) = spawn_mock_server(|mut sink, _stream| async move {
            for seq in 0..FRAME_COUNT {
                let notif = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "subscription",
                    "params": { "seq": seq }
                });
                if sink
                    .send(Message::Text(notif.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            // Hold the socket open so the dispatcher stream does not
            // close before the consumer finishes draining.
            tokio::time::sleep(Duration::from_millis(200)).await;
        })
        .await;

        // Capacity 1 forces every frame past the first to exercise the
        // `try_send -> Full -> warn! -> send().await` path.
        let dispatcher = Dispatcher::connect(
            ws_url(addr),
            Duration::from_secs(5),
            Duration::from_secs(5),
            1,
            16,
        )
        .await
        .expect("dispatcher connects");

        // Let the dispatcher pull all frames off the wire. Frame 1 lands
        // in the channel, frames 2..=4 block the dispatcher on the
        // bounded send until the consumer drains.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut received: Vec<u64> = Vec::with_capacity(FRAME_COUNT as usize);
        for _ in 0..FRAME_COUNT {
            let text = tokio::time::timeout(Duration::from_secs(2), dispatcher.next_notification())
                .await
                .expect("notification arrives within timeout")
                .expect("notification channel still open");
            let v: serde_json::Value = serde_json::from_str(&text).expect("frame is JSON");
            let seq = v
                .get("params")
                .and_then(|p| p.get("seq"))
                .and_then(|s| s.as_u64())
                .expect("seq field present");
            received.push(seq);
        }

        let expected: Vec<u64> = (0..FRAME_COUNT).collect();
        assert_eq!(
            received, expected,
            "bounded notification channel must preserve FIFO order under backpressure"
        );

        dispatcher.shutdown().await.expect("dispatcher shuts down");
        drop(dispatcher);
        server.await.expect("server task did not panic");
    }
}
