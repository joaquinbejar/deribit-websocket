//! Unit tests for account module

use deribit_websocket::prelude::*;

// Position tests

#[test]
fn test_position_deserialization() {
    let json = r#"{
        "average_price": 50000.0,
        "direction": "buy",
        "instrument_name": "BTC-PERPETUAL",
        "size": 100.0,
        "floating_profit_loss": 50.0,
        "mark_price": 50050.0,
        "leverage": 10
    }"#;

    let position: Position = serde_json::from_str(json).expect("deserialize");
    assert_eq!(position.instrument_name, "BTC-PERPETUAL");
    assert_eq!(position.size, 100.0);
    assert_eq!(position.average_price, 50000.0);
    assert_eq!(position.direction, Direction::Buy);
    assert_eq!(position.floating_profit_loss, Some(50.0));
    assert_eq!(position.mark_price, Some(50050.0));
    assert_eq!(position.leverage, Some(10));
}

#[test]
fn test_position_deserialization_sell() {
    let json = r#"{
        "average_price": 3000.0,
        "direction": "sell",
        "instrument_name": "ETH-PERPETUAL",
        "size": 10.0
    }"#;

    let position: Position = serde_json::from_str(json).expect("deserialize");
    assert_eq!(position.instrument_name, "ETH-PERPETUAL");
    assert_eq!(position.direction, Direction::Sell);
}

#[test]
fn test_position_deserialization_zero() {
    let json = r#"{
        "average_price": 0.0,
        "direction": "zero",
        "instrument_name": "BTC-PERPETUAL",
        "size": 0.0
    }"#;

    let position: Position = serde_json::from_str(json).expect("deserialize");
    assert_eq!(position.direction, Direction::Zero);
    assert_eq!(position.size, 0.0);
}

#[test]
fn test_position_with_optional_fields() {
    let json = r#"{
        "average_price": 45000.0,
        "average_price_usd": 45000.0,
        "delta": 1.0,
        "direction": "buy",
        "estimated_liquidation_price": 35000.0,
        "floating_profit_loss": 500.0,
        "floating_profit_loss_usd": 500.0,
        "gamma": 0.001,
        "index_price": 45100.0,
        "initial_margin": 0.1,
        "instrument_name": "BTC-PERPETUAL",
        "interest_value": 0.0,
        "kind": "future",
        "leverage": 10,
        "maintenance_margin": 0.05,
        "mark_price": 45050.0,
        "open_orders_margin": 0.0,
        "realized_funding": 0.0,
        "realized_profit_loss": 100.0,
        "settlement_price": 45000.0,
        "size": 1000.0,
        "size_currency": 0.022,
        "theta": 0.0,
        "total_profit_loss": 600.0,
        "vega": 0.0
    }"#;

    let position: Position = serde_json::from_str(json).expect("deserialize");
    assert_eq!(position.instrument_name, "BTC-PERPETUAL");
    assert_eq!(position.kind, Some("future".to_string()));
    assert_eq!(position.leverage, Some(10));
    assert_eq!(position.delta, Some(1.0));
}

// Direction tests

#[test]
fn test_direction_as_str() {
    assert_eq!(Direction::Buy.as_str(), "buy");
    assert_eq!(Direction::Sell.as_str(), "sell");
    assert_eq!(Direction::Zero.as_str(), "zero");
}

#[test]
fn test_direction_serialization() {
    let dir = Direction::Buy;
    let json = serde_json::to_string(&dir).expect("serialize");
    assert_eq!(json, "\"buy\"");

    let dir = Direction::Sell;
    let json = serde_json::to_string(&dir).expect("serialize");
    assert_eq!(json, "\"sell\"");
}

#[test]
fn test_direction_deserialization() {
    let dir: Direction = serde_json::from_str("\"buy\"").expect("deserialize");
    assert_eq!(dir, Direction::Buy);

    let dir: Direction = serde_json::from_str("\"sell\"").expect("deserialize");
    assert_eq!(dir, Direction::Sell);

    let dir: Direction = serde_json::from_str("\"zero\"").expect("deserialize");
    assert_eq!(dir, Direction::Zero);
}

// CurrencySummary tests

#[test]
fn test_currency_summary_deserialization() {
    let json = r#"{
        "currency": "BTC",
        "balance": 1.5,
        "equity": 1.6,
        "available_funds": 1.0,
        "margin_balance": 1.5,
        "maintenance_margin": 0.1,
        "initial_margin": 0.2
    }"#;

    let summary: CurrencySummary = serde_json::from_str(json).expect("deserialize");
    assert_eq!(summary.currency, "BTC");
    assert_eq!(summary.balance, 1.5);
    assert_eq!(summary.equity, 1.6);
    assert_eq!(summary.available_funds, 1.0);
    assert_eq!(summary.margin_balance, 1.5);
    assert_eq!(summary.maintenance_margin, 0.1);
    assert_eq!(summary.initial_margin, 0.2);
}

#[test]
fn test_currency_summary_with_optional_fields() {
    let json = r#"{
        "currency": "ETH",
        "balance": 10.0,
        "equity": 10.5,
        "available_funds": 8.0,
        "margin_balance": 10.0,
        "maintenance_margin": 0.5,
        "initial_margin": 1.0,
        "total_pl": 0.5,
        "session_rpl": 0.2,
        "session_upl": 0.3,
        "delta_total": 5.0,
        "futures_pl": 0.4,
        "options_pl": 0.1
    }"#;

    let summary: CurrencySummary = serde_json::from_str(json).expect("deserialize");
    assert_eq!(summary.currency, "ETH");
    assert_eq!(summary.total_pl, Some(0.5));
    assert_eq!(summary.session_rpl, Some(0.2));
    assert_eq!(summary.delta_total, Some(5.0));
}

// AccountSummary tests

#[test]
fn test_account_summary_deserialization_single_currency() {
    let json = r#"{
        "currency": "BTC",
        "balance": 1.5,
        "equity": 1.6,
        "available_funds": 1.0,
        "margin_balance": 1.5,
        "initial_margin": 0.2,
        "maintenance_margin": 0.1,
        "delta_total": 0.5
    }"#;

    let summary: AccountSummary = serde_json::from_str(json).expect("deserialize");
    assert_eq!(summary.currency, Some("BTC".to_string()));
    assert_eq!(summary.balance, Some(1.5));
    assert_eq!(summary.equity, Some(1.6));
    assert_eq!(summary.delta_total, Some(0.5));
}

#[test]
fn test_account_summary_deserialization_with_metadata() {
    let json = r#"{
        "id": 12345,
        "email": "user@example.com",
        "username": "testuser",
        "mmp_enabled": true,
        "currency": "BTC",
        "balance": 2.0
    }"#;

    let summary: AccountSummary = serde_json::from_str(json).expect("deserialize");
    assert_eq!(summary.id, Some(12345));
    assert_eq!(summary.email, Some("user@example.com".to_string()));
    assert_eq!(summary.username, Some("testuser".to_string()));
    assert_eq!(summary.mmp_enabled, Some(true));
}

// Request builder tests

#[test]
fn test_request_builder_get_positions() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_positions_request(None, None);

    assert_eq!(request.method, "private/get_positions");
}

#[test]
fn test_request_builder_get_positions_with_currency() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_positions_request(Some("BTC"), None);

    assert_eq!(request.method, "private/get_positions");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "BTC");
}

#[test]
fn test_request_builder_get_positions_with_kind() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_positions_request(None, Some("future"));

    assert_eq!(request.method, "private/get_positions");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["kind"], "future");
}

#[test]
fn test_request_builder_get_positions_with_both() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_positions_request(Some("ETH"), Some("option"));

    assert_eq!(request.method, "private/get_positions");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "ETH");
    assert_eq!(params["kind"], "option");
}

#[test]
fn test_request_builder_get_account_summary() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_account_summary_request("BTC", None);

    assert_eq!(request.method, "private/get_account_summary");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "BTC");
}

#[test]
fn test_request_builder_get_account_summary_extended() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_account_summary_request("ETH", Some(true));

    assert_eq!(request.method, "private/get_account_summary");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "ETH");
    assert_eq!(params["extended"], true);
}

#[test]
fn test_request_builder_get_order_state() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_order_state_request("order123");

    assert_eq!(request.method, "private/get_order_state");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["order_id"], "order123");
}

#[test]
fn test_request_builder_get_order_history_by_currency() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_order_history_by_currency_request("BTC", None, None);

    assert_eq!(request.method, "private/get_order_history_by_currency");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "BTC");
}

#[test]
fn test_request_builder_get_order_history_with_filters() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_order_history_by_currency_request("ETH", Some("future"), Some(50));

    assert_eq!(request.method, "private/get_order_history_by_currency");
    assert!(request.params.is_some());

    let params = request.params.expect("params");
    assert_eq!(params["currency"], "ETH");
    assert_eq!(params["kind"], "future");
    assert_eq!(params["count"], 50);
}

#[test]
fn test_request_builder_incremental_ids() {
    let mut builder = RequestBuilder::new();

    let r1 = builder.build_get_positions_request(None, None);
    let r2 = builder.build_get_account_summary_request("BTC", None);
    let r3 = builder.build_get_order_state_request("order1");

    assert_eq!(r1.id, serde_json::json!(1));
    assert_eq!(r2.id, serde_json::json!(2));
    assert_eq!(r3.id, serde_json::json!(3));
}
