//! Unit tests for trading module

use deribit_websocket::prelude::*;

#[test]
fn test_order_request_limit() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0);

    assert_eq!(request.instrument_name, "BTC-PERPETUAL");
    assert_eq!(request.amount, 100.0);
    assert_eq!(request.price, Some(50000.0));
    assert_eq!(request.order_type, Some(OrderType::Limit));
}

#[test]
fn test_order_request_market() {
    let request = OrderRequest::market("ETH-PERPETUAL".to_string(), 10.0);

    assert_eq!(request.instrument_name, "ETH-PERPETUAL");
    assert_eq!(request.amount, 10.0);
    assert_eq!(request.price, None);
    assert_eq!(request.order_type, Some(OrderType::Market));
}

#[test]
fn test_order_request_with_label() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
        .with_label("test_order".to_string());

    assert_eq!(request.label, Some("test_order".to_string()));
}

#[test]
fn test_order_request_with_time_in_force() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
        .with_time_in_force(TimeInForce::GoodTilCancelled);

    assert_eq!(request.time_in_force, Some(TimeInForce::GoodTilCancelled));
}

#[test]
fn test_order_request_with_post_only() {
    let request =
        OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0).with_post_only(true);

    assert_eq!(request.post_only, Some(true));
}

#[test]
fn test_order_request_with_reduce_only() {
    let request = OrderRequest::market("BTC-PERPETUAL".to_string(), 100.0).with_reduce_only(true);

    assert_eq!(request.reduce_only, Some(true));
}

#[test]
fn test_order_request_with_max_show() {
    let request =
        OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0).with_max_show(10.0);

    assert_eq!(request.max_show, Some(10.0));
}

#[test]
fn test_order_request_with_trigger() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
        .with_trigger(49000.0, Trigger::MarkPrice);

    assert_eq!(request.trigger_price, Some(49000.0));
    assert_eq!(request.trigger, Some(Trigger::MarkPrice));
}

#[test]
fn test_order_request_with_mmp() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0).with_mmp(true);

    assert_eq!(request.mmp, Some(true));
}

#[test]
fn test_order_request_chained_builders() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
        .with_label("my_order".to_string())
        .with_post_only(true)
        .with_time_in_force(TimeInForce::ImmediateOrCancel)
        .with_mmp(true);

    assert_eq!(request.instrument_name, "BTC-PERPETUAL");
    assert_eq!(request.amount, 100.0);
    assert_eq!(request.price, Some(50000.0));
    assert_eq!(request.label, Some("my_order".to_string()));
    assert_eq!(request.post_only, Some(true));
    assert_eq!(request.time_in_force, Some(TimeInForce::ImmediateOrCancel));
    assert_eq!(request.mmp, Some(true));
}

#[test]
fn test_edit_order_request() {
    let request = EditOrderRequest::new("order123".to_string(), 200.0);

    assert_eq!(request.order_id, "order123");
    assert_eq!(request.amount, 200.0);
    assert_eq!(request.price, None);
}

#[test]
fn test_edit_order_request_with_price() {
    let request = EditOrderRequest::new("order123".to_string(), 200.0).with_price(51000.0);

    assert_eq!(request.order_id, "order123");
    assert_eq!(request.amount, 200.0);
    assert_eq!(request.price, Some(51000.0));
}

#[test]
fn test_edit_order_request_with_post_only() {
    let request = EditOrderRequest::new("order123".to_string(), 200.0).with_post_only(true);

    assert_eq!(request.post_only, Some(true));
}

#[test]
fn test_edit_order_request_with_reduce_only() {
    let request = EditOrderRequest::new("order123".to_string(), 200.0).with_reduce_only(true);

    assert_eq!(request.reduce_only, Some(true));
}

#[test]
fn test_edit_order_request_chained() {
    let request = EditOrderRequest::new("order456".to_string(), 300.0)
        .with_price(52000.0)
        .with_post_only(true)
        .with_reduce_only(false);

    assert_eq!(request.order_id, "order456");
    assert_eq!(request.amount, 300.0);
    assert_eq!(request.price, Some(52000.0));
    assert_eq!(request.post_only, Some(true));
    assert_eq!(request.reduce_only, Some(false));
}

#[test]
fn test_order_type_as_str() {
    assert_eq!(OrderType::Limit.as_str(), "limit");
    assert_eq!(OrderType::Market.as_str(), "market");
    assert_eq!(OrderType::StopLimit.as_str(), "stop_limit");
    assert_eq!(OrderType::StopMarket.as_str(), "stop_market");
    assert_eq!(OrderType::TakeLimit.as_str(), "take_limit");
    assert_eq!(OrderType::TakeMarket.as_str(), "take_market");
    assert_eq!(OrderType::MarketLimit.as_str(), "market_limit");
    assert_eq!(OrderType::TrailingStop.as_str(), "trailing_stop");
}

#[test]
fn test_time_in_force_as_str() {
    assert_eq!(TimeInForce::GoodTilCancelled.as_str(), "good_til_cancelled");
    assert_eq!(TimeInForce::GoodTilDay.as_str(), "good_til_day");
    assert_eq!(TimeInForce::FillOrKill.as_str(), "fill_or_kill");
    assert_eq!(
        TimeInForce::ImmediateOrCancel.as_str(),
        "immediate_or_cancel"
    );
}

#[test]
fn test_order_type_serialization() {
    let order_type = OrderType::Limit;
    let json = serde_json::to_string(&order_type).expect("serialize");
    assert_eq!(json, "\"limit\"");

    let order_type = OrderType::StopLimit;
    let json = serde_json::to_string(&order_type).expect("serialize");
    assert_eq!(json, "\"stop_limit\"");
}

#[test]
fn test_order_type_deserialization() {
    let order_type: OrderType = serde_json::from_str("\"limit\"").expect("deserialize");
    assert_eq!(order_type, OrderType::Limit);

    let order_type: OrderType = serde_json::from_str("\"stop_market\"").expect("deserialize");
    assert_eq!(order_type, OrderType::StopMarket);
}

#[test]
fn test_time_in_force_serialization() {
    let tif = TimeInForce::GoodTilCancelled;
    let json = serde_json::to_string(&tif).expect("serialize");
    assert_eq!(json, "\"good_til_cancelled\"");

    let tif = TimeInForce::ImmediateOrCancel;
    let json = serde_json::to_string(&tif).expect("serialize");
    assert_eq!(json, "\"immediate_or_cancel\"");
}

#[test]
fn test_time_in_force_deserialization() {
    let tif: TimeInForce = serde_json::from_str("\"good_til_cancelled\"").expect("deserialize");
    assert_eq!(tif, TimeInForce::GoodTilCancelled);

    let tif: TimeInForce = serde_json::from_str("\"fill_or_kill\"").expect("deserialize");
    assert_eq!(tif, TimeInForce::FillOrKill);
}

#[test]
fn test_trigger_serialization() {
    let trigger = Trigger::IndexPrice;
    let json = serde_json::to_string(&trigger).expect("serialize");
    assert_eq!(json, "\"index_price\"");

    let trigger = Trigger::MarkPrice;
    let json = serde_json::to_string(&trigger).expect("serialize");
    assert_eq!(json, "\"mark_price\"");
}

#[test]
fn test_trigger_deserialization() {
    let trigger: Trigger = serde_json::from_str("\"index_price\"").expect("deserialize");
    assert_eq!(trigger, Trigger::IndexPrice);

    let trigger: Trigger = serde_json::from_str("\"last_price\"").expect("deserialize");
    assert_eq!(trigger, Trigger::LastPrice);
}

#[test]
fn test_order_request_serialization() {
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0)
        .with_label("test".to_string())
        .with_post_only(true);

    let json = serde_json::to_value(&request).expect("serialize");

    assert_eq!(json["instrument_name"], "BTC-PERPETUAL");
    assert_eq!(json["amount"], 100.0);
    assert_eq!(json["price"], 50000.0);
    assert_eq!(json["type"], "limit");
    assert_eq!(json["label"], "test");
    assert_eq!(json["post_only"], true);
}

#[test]
fn test_edit_order_request_serialization() {
    let request = EditOrderRequest::new("order123".to_string(), 200.0).with_price(51000.0);

    let json = serde_json::to_value(&request).expect("serialize");

    assert_eq!(json["order_id"], "order123");
    assert_eq!(json["amount"], 200.0);
    assert_eq!(json["price"], 51000.0);
}

#[test]
fn test_order_info_deserialization() {
    let json = r#"{
        "order_id": "123456",
        "instrument_name": "BTC-PERPETUAL",
        "direction": "buy",
        "amount": 100.0,
        "filled_amount": 50.0,
        "price": 50000.0,
        "average_price": 49950.0,
        "order_type": "limit",
        "order_state": "open",
        "label": "test",
        "creation_timestamp": 1609459200000,
        "last_update_timestamp": 1609459200000,
        "api": true,
        "web": false,
        "post_only": true,
        "reduce_only": false,
        "is_liquidation": false,
        "replaced": false,
        "mmp": false,
        "mmp_cancelled": false
    }"#;

    let order_info: OrderInfo = serde_json::from_str(json).expect("deserialize");

    assert_eq!(order_info.order_id, "123456");
    assert_eq!(order_info.instrument_name, "BTC-PERPETUAL");
    assert_eq!(order_info.direction, "buy");
    assert_eq!(order_info.amount, 100.0);
    assert_eq!(order_info.filled_amount, 50.0);
    assert_eq!(order_info.price, Some(50000.0));
    assert_eq!(order_info.order_state, "open");
    assert!(order_info.api);
    assert!(order_info.post_only);
}

#[test]
fn test_order_response_deserialization() {
    let json = r#"{
        "order": {
            "order_id": "123456",
            "instrument_name": "BTC-PERPETUAL",
            "direction": "buy",
            "amount": 100.0,
            "filled_amount": 0.0,
            "price": 50000.0,
            "order_type": "limit",
            "order_state": "open",
            "label": "",
            "creation_timestamp": 1609459200000,
            "last_update_timestamp": 1609459200000,
            "api": true,
            "web": false,
            "post_only": false,
            "reduce_only": false,
            "is_liquidation": false,
            "replaced": false,
            "mmp": false,
            "mmp_cancelled": false
        },
        "trades": []
    }"#;

    let response: OrderResponse = serde_json::from_str(json).expect("deserialize");

    assert_eq!(response.order.order_id, "123456");
    assert_eq!(response.order.instrument_name, "BTC-PERPETUAL");
    assert!(response.trades.is_empty());
}

#[test]
fn test_trade_execution_deserialization() {
    let json = r#"{
        "trade_id": "trade123",
        "instrument_name": "BTC-PERPETUAL",
        "direction": "buy",
        "amount": 50.0,
        "price": 49950.0,
        "fee": 0.0001,
        "fee_currency": "BTC",
        "order_id": "order123",
        "order_type": "limit",
        "timestamp": 1609459200000,
        "liquidity": "maker",
        "index_price": 50000.0,
        "mark_price": 49980.0
    }"#;

    let trade: TradeExecution = serde_json::from_str(json).expect("deserialize");

    assert_eq!(trade.trade_id, "trade123");
    assert_eq!(trade.instrument_name, "BTC-PERPETUAL");
    assert_eq!(trade.direction, "buy");
    assert_eq!(trade.amount, 50.0);
    assert_eq!(trade.price, 49950.0);
    assert_eq!(trade.fee, 0.0001);
    assert_eq!(trade.liquidity, Some("maker".to_string()));
}

#[test]
fn test_request_builder_buy() {
    let mut builder = RequestBuilder::new();
    let order_request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 100.0, 50000.0);
    let request = builder
        .build_buy_request(&order_request)
        .expect("build request");

    assert_eq!(request.method, "private/buy");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["instrument_name"], "BTC-PERPETUAL");
    assert_eq!(params["amount"], 100.0);
}

#[test]
fn test_request_builder_sell() {
    let mut builder = RequestBuilder::new();
    let order_request = OrderRequest::limit("ETH-PERPETUAL".to_string(), 10.0, 3000.0);
    let request = builder
        .build_sell_request(&order_request)
        .expect("build request");

    assert_eq!(request.method, "private/sell");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["instrument_name"], "ETH-PERPETUAL");
    assert_eq!(params["amount"], 10.0);
}

#[test]
fn test_request_builder_cancel() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_cancel_request("order123");

    assert_eq!(request.method, "private/cancel");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["order_id"], "order123");
}

#[test]
fn test_request_builder_cancel_all() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_cancel_all_request();

    assert_eq!(request.method, "private/cancel_all");
}

#[test]
fn test_request_builder_cancel_all_by_currency() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_cancel_all_by_currency_request("BTC");

    assert_eq!(request.method, "private/cancel_all_by_currency");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "BTC");
}

#[test]
fn test_request_builder_cancel_all_by_instrument() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_cancel_all_by_instrument_request("BTC-PERPETUAL");

    assert_eq!(request.method, "private/cancel_all_by_instrument");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["instrument_name"], "BTC-PERPETUAL");
}

#[test]
fn test_request_builder_edit() {
    let mut builder = RequestBuilder::new();
    let edit_request = EditOrderRequest::new("order123".to_string(), 200.0).with_price(51000.0);
    let request = builder
        .build_edit_request(&edit_request)
        .expect("build request");

    assert_eq!(request.method, "private/edit");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["order_id"], "order123");
    assert_eq!(params["amount"], 200.0);
}

#[test]
fn test_request_builder_incremental_ids() {
    let mut builder = RequestBuilder::new();

    let r1 = builder.build_cancel_all_request();
    let r2 = builder.build_cancel_all_request();
    let r3 = builder.build_cancel_all_request();

    assert_eq!(r1.id, serde_json::json!(1));
    assert_eq!(r2.id, serde_json::json!(2));
    assert_eq!(r3.id, serde_json::json!(3));
}
