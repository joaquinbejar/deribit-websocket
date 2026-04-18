//! Connection module for WebSocket client

pub mod dispatcher;
pub mod ws_connection;

pub use dispatcher::Dispatcher;
pub use ws_connection::*;
