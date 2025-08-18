//! Unit tests for model module

use deribit_websocket::model::{SubscriptionChannel, ws_types::JsonRpcRequest};
use serde_json::json;

#[test]
fn test_subscription_channel_ticker() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());

    match channel {
        SubscriptionChannel::Ticker(instrument) => {
            assert_eq!(instrument, "BTC-PERPETUAL");
        }
        _ => panic!("Expected Ticker variant"),
    }
}

#[test]
fn test_subscription_channel_order_book() {
    let channel = SubscriptionChannel::OrderBook("ETH-PERPETUAL".to_string());

    match channel {
        SubscriptionChannel::OrderBook(instrument) => {
            assert_eq!(instrument, "ETH-PERPETUAL");
        }
        _ => panic!("Expected OrderBook variant"),
    }
}

#[test]
fn test_subscription_channel_trades() {
    let channel = SubscriptionChannel::Trades("BTC-PERPETUAL".to_string());

    match channel {
        SubscriptionChannel::Trades(instrument) => {
            assert_eq!(instrument, "BTC-PERPETUAL");
        }
        _ => panic!("Expected Trades variant"),
    }
}

#[test]
fn test_subscription_channel_user_orders() {
    let channel = SubscriptionChannel::UserOrders;

    match channel {
        SubscriptionChannel::UserOrders => {
            // Success
        }
        _ => panic!("Expected UserOrders variant"),
    }
}

#[test]
fn test_subscription_channel_user_trades() {
    let channel = SubscriptionChannel::UserTrades;

    match channel {
        SubscriptionChannel::UserTrades => {
            // Success
        }
        _ => panic!("Expected UserTrades variant"),
    }
}

#[test]
fn test_subscription_channel_chart_trades() {
    let channel = SubscriptionChannel::ChartTrades {
        instrument: "BTC-PERPETUAL".to_string(),
        resolution: "1".to_string(),
    };

    match channel {
        SubscriptionChannel::ChartTrades {
            instrument,
            resolution,
        } => {
            assert_eq!(instrument, "BTC-PERPETUAL");
            assert_eq!(resolution, "1");
        }
        _ => panic!("Expected ChartTrades variant"),
    }
}

#[test]
fn test_subscription_channel_user_changes() {
    let channel = SubscriptionChannel::UserChanges {
        instrument: "ETH-PERPETUAL".to_string(),
        interval: "raw".to_string(),
    };

    match channel {
        SubscriptionChannel::UserChanges {
            instrument,
            interval,
        } => {
            assert_eq!(instrument, "ETH-PERPETUAL");
            assert_eq!(interval, "raw");
        }
        _ => panic!("Expected UserChanges variant"),
    }
}

#[test]
fn test_subscription_channel_debug() {
    let channel = SubscriptionChannel::Ticker("BTC-PERPETUAL".to_string());
    let debug_str = format!("{:?}", channel);

    assert!(debug_str.contains("Ticker"));
    assert!(debug_str.contains("BTC-PERPETUAL"));
}

#[test]
fn test_subscription_channel_clone() {
    let original = SubscriptionChannel::OrderBook("BTC-PERPETUAL".to_string());
    let cloned = original.clone();

    match (original, cloned) {
        (SubscriptionChannel::OrderBook(orig), SubscriptionChannel::OrderBook(clone)) => {
            assert_eq!(orig, clone);
        }
        _ => panic!("Clone should preserve variant and data"),
    }
}

#[test]
fn test_json_rpc_request_creation() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "public/test".to_string(),
        params: Some(json!({})),
        id: json!(1),
    };

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "public/test");
    assert_eq!(request.id, json!(1));
    assert!(request.params.is_some());
}

#[test]
fn test_json_rpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "public/subscribe".to_string(),
        params: Some(json!({"channels": ["ticker.BTC-PERPETUAL"]})),
        id: json!(42),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "public/subscribe");
    assert_eq!(parsed["id"], 42);
    assert!(parsed["params"]["channels"].is_array());
}

#[test]
fn test_json_rpc_response_success() {
    use deribit_websocket::model::ws_types::{JsonRpcResponse, JsonRpcResult};

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        result: JsonRpcResult::Success {
            result: json!({"version": "1.2.26"}),
        },
    };

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, json!(1));

    match response.result {
        JsonRpcResult::Success { result } => {
            assert_eq!(result["version"], "1.2.26");
        }
        _ => panic!("Expected success result"),
    }
}

#[test]
fn test_json_rpc_response_with_error() {
    use deribit_websocket::model::ws_types::{JsonRpcError, JsonRpcResponse, JsonRpcResult};

    let error = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };

    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        result: JsonRpcResult::Error {
            error: error.clone(),
        },
    };

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, json!(1));

    match response.result {
        JsonRpcResult::Error { error } => {
            assert_eq!(error.code, -32600);
            assert_eq!(error.message, "Invalid Request");
        }
        _ => panic!("Expected error result"),
    }
}
