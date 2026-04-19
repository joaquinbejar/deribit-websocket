//! Aggregate message builder and parser for JSON-RPC traffic.
//!
//! Combines the three stateless helpers in [`crate::message`] into a
//! single facade: [`RequestBuilder`] for *outbound* JSON-RPC requests,
//! [`ResponseHandler`] for *inbound* replies, and [`NotificationHandler`]
//! for server-initiated pushes. Typical usage is to hold one
//! [`MessageBuilder`] per connection and feed every frame read off the
//! wire to [`MessageBuilder::parse_message`], dispatching on the
//! returned [`MessageType`].

use crate::message::{NotificationHandler, RequestBuilder, ResponseHandler};
use crate::model::ws_types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Aggregate JSON-RPC message builder/parser.
///
/// Holds the three sub-components needed to drive a JSON-RPC session:
///
/// - [`RequestBuilder`] — produces outbound requests and assigns
///   monotonically-increasing ids; held mutably because it mutates
///   that id counter on every call.
/// - [`ResponseHandler`] — parses inbound replies keyed by id; fully
///   stateless, so a shared reference is enough.
/// - [`NotificationHandler`] — parses server-initiated pushes
///   (subscription events, heartbeats); also stateless.
///
/// One instance per WebSocket connection is the intended lifetime: the
/// id counter must not be shared across connections, and the handlers
/// hold no connection-specific state.
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
    /// Create a new aggregate message builder with a fresh id counter
    /// and stateless response/notification parsers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_builder: RequestBuilder::new(),
            response_handler: ResponseHandler::new(),
            notification_handler: NotificationHandler::new(),
        }
    }

    /// Borrow the request builder mutably.
    ///
    /// Mutability is required because every outbound request advances
    /// the builder's id counter. Prefer calling this from a single
    /// task: the `MessageBuilder` itself is not `Sync`-protected.
    pub fn request_builder(&mut self) -> &mut RequestBuilder {
        &mut self.request_builder
    }

    /// Borrow the response handler.
    ///
    /// Returned as an immutable reference because the handler is a pure
    /// JSON-to-DTO parser with no internal state.
    #[must_use]
    pub fn response_handler(&self) -> &ResponseHandler {
        &self.response_handler
    }

    /// Borrow the notification handler.
    ///
    /// Returned as an immutable reference because the handler is a pure
    /// JSON-to-DTO parser with no internal state.
    #[must_use]
    pub fn notification_handler(&self) -> &NotificationHandler {
        &self.notification_handler
    }

    /// Classify and parse a raw JSON-RPC frame.
    ///
    /// Attempts the response path first (JSON-RPC 2.0 responses always
    /// carry an `id`), then the notification path (notifications never
    /// have `id`). Outbound-only [`JsonRpcRequest`]s — the third
    /// [`MessageType`] variant — are produced by [`RequestBuilder`],
    /// not by this parser, so receiving a "request-shaped" frame from
    /// the server is treated as invalid data.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] of kind [`std::io::ErrorKind::InvalidData`]
    /// when `data` parses as neither a response nor a notification —
    /// typically because it is malformed JSON, lacks both an `id` and a
    /// `method`, or looks like an outbound request rather than an
    /// inbound frame.
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

/// Classification of a JSON-RPC 2.0 frame.
///
/// Emitted by [`MessageBuilder::parse_message`] to let callers dispatch
/// on the three fundamental flavours of traffic in a JSON-RPC session.
#[derive(Debug, Clone)]
pub enum MessageType {
    /// Outbound request frame — `method`, `params`, and a numeric `id`.
    ///
    /// Never produced by [`MessageBuilder::parse_message`]; reserved
    /// for code paths that construct requests via [`RequestBuilder`]
    /// and want to round-trip them through the same enum.
    Request(JsonRpcRequest),
    /// Inbound reply frame correlated with a previously-sent request
    /// via a shared numeric `id`. Carries either a `result` or an
    /// `error` payload (never both).
    Response(JsonRpcResponse),
    /// Server-initiated push frame. Has a `method` and `params` like a
    /// request but no `id`, so it expects no reply. Examples:
    /// subscription updates, heartbeats.
    Notification(JsonRpcNotification),
}
