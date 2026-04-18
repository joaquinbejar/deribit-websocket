//! Unit tests for position management module

use deribit_websocket::prelude::*;

// MovePositionTrade tests

#[test]
fn test_move_position_trade_new() {
    let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0);
    assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
    assert_eq!(trade.amount, 100.0);
    assert!(trade.price.is_none());
}

#[test]
fn test_move_position_trade_with_price() {
    let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0).with_price(50000.0);
    assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
    assert_eq!(trade.amount, 100.0);
    assert_eq!(trade.price, Some(50000.0));
}

#[test]
fn test_move_position_trade_chained() {
    let trade = MovePositionTrade::new("ETH-PERPETUAL", 50.0).with_price(3000.0);
    assert_eq!(trade.instrument_name, "ETH-PERPETUAL");
    assert_eq!(trade.amount, 50.0);
    assert_eq!(trade.price, Some(3000.0));
}

#[test]
fn test_move_position_trade_serialization() {
    let trade = MovePositionTrade::new("ETH-PERPETUAL", 50.0).with_price(3000.0);
    let json = serde_json::to_string(&trade).expect("serialize");
    assert!(json.contains("ETH-PERPETUAL"));
    assert!(json.contains("50"));
    assert!(json.contains("3000"));
}

#[test]
fn test_move_position_trade_without_price_serialization() {
    let trade = MovePositionTrade::new("BTC-PERPETUAL", 100.0);
    let json = serde_json::to_string(&trade).expect("serialize");
    assert!(json.contains("BTC-PERPETUAL"));
    assert!(json.contains("100"));
    // price should not be serialized when None
    assert!(!json.contains("price"));
}

#[test]
fn test_move_position_trade_deserialization() {
    let json = r#"{
        "instrument_name": "BTC-PERPETUAL",
        "amount": 100.0,
        "price": 45000.0
    }"#;

    let trade: MovePositionTrade = serde_json::from_str(json).expect("deserialize");
    assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
    assert_eq!(trade.amount, 100.0);
    assert_eq!(trade.price, Some(45000.0));
}

// MovePositionResult tests

#[test]
fn test_move_position_result_deserialization() {
    let json = r#"{
        "target_uid": 23,
        "source_uid": 3,
        "price": 35800.0,
        "instrument_name": "BTC-PERPETUAL",
        "direction": "buy",
        "amount": 110.0
    }"#;

    let result: MovePositionResult = serde_json::from_str(json).expect("deserialize");
    assert_eq!(result.target_uid, 23);
    assert_eq!(result.source_uid, 3);
    assert_eq!(result.price, 35800.0);
    assert_eq!(result.instrument_name, "BTC-PERPETUAL");
    assert_eq!(result.direction, "buy");
    assert_eq!(result.amount, 110.0);
}

#[test]
fn test_move_position_result_sell_direction() {
    let json = r#"{
        "target_uid": 10,
        "source_uid": 5,
        "price": 0.1223,
        "instrument_name": "BTC-28JAN22-32500-C",
        "direction": "sell",
        "amount": 0.1
    }"#;

    let result: MovePositionResult = serde_json::from_str(json).expect("deserialize");
    assert_eq!(result.direction, "sell");
    assert_eq!(result.instrument_name, "BTC-28JAN22-32500-C");
}

// ClosePositionResponse tests

#[test]
fn test_close_position_response_deserialization() {
    let json = r#"{
        "trades": [{
            "trade_seq": 1966068,
            "trade_id": "ETH-2696097",
            "timestamp": 1590486335742,
            "tick_direction": 0,
            "state": "filled",
            "reduce_only": true,
            "price": 202.8,
            "post_only": false,
            "order_type": "limit",
            "order_id": "ETH-584864807",
            "mark_price": 202.79,
            "liquidity": "T",
            "instrument_name": "ETH-PERPETUAL",
            "index_price": 202.86,
            "fee_currency": "ETH",
            "fee": 0.00007766,
            "direction": "sell",
            "amount": 21.0
        }],
        "order": {
            "time_in_force": "good_til_cancelled",
            "reduce_only": true,
            "price": 198.75,
            "post_only": false,
            "order_type": "limit",
            "order_state": "filled",
            "order_id": "ETH-584864807",
            "instrument_name": "ETH-PERPETUAL",
            "filled_amount": 21.0,
            "direction": "sell",
            "creation_timestamp": 1590486335742,
            "average_price": 202.8,
            "api": true,
            "amount": 21.0
        }
    }"#;

    let response: ClosePositionResponse = serde_json::from_str(json).expect("deserialize");
    assert_eq!(response.trades.len(), 1);
    assert!(response.order.is_some());

    let trade = &response.trades[0];
    assert_eq!(trade.trade_id, Some("ETH-2696097".to_string()));
    assert_eq!(trade.price, Some(202.8));
    assert_eq!(trade.direction, Some("sell".to_string()));
    assert_eq!(trade.state, Some("filled".to_string()));

    let order = response.order.as_ref().expect("order");
    assert_eq!(order.order_id, Some("ETH-584864807".to_string()));
    assert_eq!(order.order_state, Some("filled".to_string()));
    assert_eq!(order.reduce_only, Some(true));
}

#[test]
fn test_close_position_response_empty_trades() {
    let json = r#"{
        "trades": [],
        "order": {
            "order_id": "BTC-123",
            "order_state": "open"
        }
    }"#;

    let response: ClosePositionResponse = serde_json::from_str(json).expect("deserialize");
    assert!(response.trades.is_empty());
    assert!(response.order.is_some());
}

#[test]
fn test_close_position_response_no_order() {
    let json = r#"{
        "trades": []
    }"#;

    let response: ClosePositionResponse = serde_json::from_str(json).expect("deserialize");
    assert!(response.trades.is_empty());
    assert!(response.order.is_none());
}

// CloseTrade tests

#[test]
fn test_close_trade_deserialization() {
    let json = r#"{
        "trade_seq": 12345,
        "trade_id": "BTC-123456",
        "price": 50000.0,
        "amount": 1.0,
        "direction": "buy"
    }"#;

    let trade: CloseTrade = serde_json::from_str(json).expect("deserialize");
    assert_eq!(trade.trade_seq, Some(12345));
    assert_eq!(trade.trade_id, Some("BTC-123456".to_string()));
    assert_eq!(trade.price, Some(50000.0));
    assert_eq!(trade.amount, Some(1.0));
    assert_eq!(trade.direction, Some("buy".to_string()));
}

#[test]
fn test_close_trade_minimal() {
    let json = r#"{}"#;

    let trade: CloseTrade = serde_json::from_str(json).expect("deserialize");
    assert!(trade.trade_seq.is_none());
    assert!(trade.trade_id.is_none());
    assert!(trade.price.is_none());
}

// CloseOrder tests

#[test]
fn test_close_order_deserialization() {
    let json = r#"{
        "order_id": "BTC-123",
        "order_state": "open",
        "order_type": "limit",
        "price": 45000.0,
        "amount": 100.0,
        "direction": "sell"
    }"#;

    let order: CloseOrder = serde_json::from_str(json).expect("deserialize");
    assert_eq!(order.order_id, Some("BTC-123".to_string()));
    assert_eq!(order.order_state, Some("open".to_string()));
    assert_eq!(order.order_type, Some("limit".to_string()));
    assert_eq!(order.price, Some(45000.0));
    assert_eq!(order.amount, Some(100.0));
    assert_eq!(order.direction, Some("sell".to_string()));
}

#[test]
fn test_close_order_with_optional_fields() {
    let json = r#"{
        "order_id": "ETH-456",
        "order_state": "filled",
        "reduce_only": true,
        "post_only": false,
        "is_liquidation": false,
        "api": true,
        "web": false
    }"#;

    let order: CloseOrder = serde_json::from_str(json).expect("deserialize");
    assert_eq!(order.reduce_only, Some(true));
    assert_eq!(order.post_only, Some(false));
    assert_eq!(order.is_liquidation, Some(false));
    assert_eq!(order.api, Some(true));
    assert_eq!(order.web, Some(false));
}

// Request builder tests

#[test]
fn test_request_builder_close_position_market() {
    let mut builder = RequestBuilder::new();
    let request = builder
        .build_close_position_request("BTC-PERPETUAL", "market", None)
        .expect("build request");

    assert_eq!(request.method, "private/close_position");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["instrument_name"], "BTC-PERPETUAL");
    assert_eq!(params["type"], "market");
    assert!(params.get("price").is_none());
}

#[test]
fn test_request_builder_close_position_limit() {
    let mut builder = RequestBuilder::new();
    let request = builder
        .build_close_position_request("ETH-PERPETUAL", "limit", Some(3000.0))
        .expect("build request");

    assert_eq!(request.method, "private/close_position");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["instrument_name"], "ETH-PERPETUAL");
    assert_eq!(params["type"], "limit");
    assert_eq!(params["price"], 3000.0);
}

#[test]
fn test_request_builder_move_positions() {
    let mut builder = RequestBuilder::new();
    let trades = vec![
        MovePositionTrade::new("BTC-PERPETUAL", 100.0).with_price(50000.0),
        MovePositionTrade::new("ETH-PERPETUAL", 50.0),
    ];
    let request = builder
        .build_move_positions_request("BTC", 3, 23, &trades)
        .expect("build request");

    assert_eq!(request.method, "private/move_positions");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "BTC");
    assert_eq!(params["source_uid"], 3);
    assert_eq!(params["target_uid"], 23);

    let trades_arr = params["trades"].as_array().expect("trades array");
    assert_eq!(trades_arr.len(), 2);
}

#[test]
fn test_request_builder_move_positions_single() {
    let mut builder = RequestBuilder::new();
    let trades = vec![MovePositionTrade::new("BTC-PERPETUAL", 110.0).with_price(35800.0)];
    let request = builder
        .build_move_positions_request("BTC", 5, 10, &trades)
        .expect("build request");

    assert_eq!(request.method, "private/move_positions");
    let params = request.params.expect("params");
    assert_eq!(params["source_uid"], 5);
    assert_eq!(params["target_uid"], 10);
}

#[test]
fn test_request_builder_incremental_ids() {
    let mut builder = RequestBuilder::new();

    let r1 = builder
        .build_close_position_request("BTC-PERPETUAL", "market", None)
        .expect("build request 1");
    let r2 = builder
        .build_close_position_request("ETH-PERPETUAL", "limit", Some(3000.0))
        .expect("build request 2");

    assert_eq!(r1.id, serde_json::json!(1));
    assert_eq!(r2.id, serde_json::json!(2));
}
