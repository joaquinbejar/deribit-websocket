//! Unit tests for message module

use deribit_websocket::message::notification::NotificationHandler;
use deribit_websocket::message::request::RequestBuilder;
use deribit_websocket::message::response::ResponseHandler;

#[test]
fn test_request_builder_creation() {
    let builder = RequestBuilder::new();

    // Test that builder can be created
    let debug_str = format!("{:?}", builder);
    assert!(debug_str.contains("RequestBuilder"));
}

#[test]
fn test_request_builder_auth_request() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_auth_request("test_client_id", "test_client_secret");

    assert_eq!(request.method, "public/auth");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());

    // Check that params contain the credentials
    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        assert_eq!(params_obj["grant_type"], "client_credentials");
        assert_eq!(params_obj["client_id"], "test_client_id");
        assert_eq!(params_obj["client_secret"], "test_client_secret");
    } else {
        panic!("Auth request should have params");
    }
}

#[test]
fn test_request_builder_subscribe_request() {
    let mut builder = RequestBuilder::new();
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.ETH-PERPETUAL.100ms".to_string(),
    ];
    let request = builder.build_subscribe_request(channels.clone());

    assert_eq!(request.method, "public/subscribe");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        let channels_array = params_obj["channels"].as_array().unwrap();
        assert_eq!(channels_array.len(), 2);
        assert_eq!(channels_array[0], "ticker.BTC-PERPETUAL");
        assert_eq!(channels_array[1], "book.ETH-PERPETUAL.100ms");
    } else {
        panic!("Subscribe request should have params");
    }
}

#[test]
fn test_request_builder_unsubscribe_request() {
    let mut builder = RequestBuilder::new();
    let channels = vec!["ticker.BTC-PERPETUAL".to_string()];
    let request = builder.build_unsubscribe_request(channels.clone());

    assert_eq!(request.method, "public/unsubscribe");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
}

#[test]
fn test_request_builder_test_request() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_test_request();

    assert_eq!(request.method, "public/test");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
}

#[test]
fn test_request_builder_get_time_request() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_time_request();

    assert_eq!(request.method, "public/get_time");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
}

#[test]
fn test_request_builder_incremental_ids() {
    let mut builder = RequestBuilder::new();

    let request1 = builder.build_test_request();
    let request2 = builder.build_test_request();
    let request3 = builder.build_test_request();

    assert!(request1.id.is_number());
    assert!(request2.id.is_number());
    assert!(request3.id.is_number());
    // IDs should be different
    assert_ne!(request1.id, request2.id);
    assert_ne!(request2.id, request3.id);
}

#[test]
fn test_response_handler_creation() {
    let handler = ResponseHandler::new();

    let debug_str = format!("{:?}", handler);
    assert!(debug_str.contains("ResponseHandler"));
}

#[test]
fn test_notification_handler_creation() {
    let handler = NotificationHandler::new();

    let debug_str = format!("{:?}", handler);
    assert!(debug_str.contains("NotificationHandler"));
}

#[test]
fn test_request_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_test_request();

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "public/test");
    assert!(parsed["id"].is_number());
}

#[test]
fn test_request_with_empty_channels() {
    let mut builder = RequestBuilder::new();
    let channels: Vec<String> = vec![];
    let request = builder.build_subscribe_request(channels);

    assert_eq!(request.method, "public/subscribe");

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        let channels_array = params_obj["channels"].as_array().unwrap();
        assert_eq!(channels_array.len(), 0);
    }
}

// =============================================================================
// Session management request tests (Issue #14)
// =============================================================================

#[test]
fn test_request_builder_set_heartbeat() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_set_heartbeat_request(30);

    assert_eq!(request.method, "public/set_heartbeat");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        assert_eq!(params_obj["interval"], 30);
    } else {
        panic!("set_heartbeat request should have params");
    }
}

#[test]
fn test_request_builder_set_heartbeat_custom_interval() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_set_heartbeat_request(60);

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        assert_eq!(params_obj["interval"], 60);
    } else {
        panic!("set_heartbeat request should have params");
    }
}

#[test]
fn test_request_builder_disable_heartbeat() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_disable_heartbeat_request();

    assert_eq!(request.method, "public/disable_heartbeat");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_hello() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_hello_request("test-client", "1.0.0");

    assert_eq!(request.method, "public/hello");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        assert_eq!(params_obj["client_name"], "test-client");
        assert_eq!(params_obj["client_version"], "1.0.0");
    } else {
        panic!("hello request should have params");
    }
}

#[test]
fn test_request_builder_hello_custom_values() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_hello_request("deribit-websocket", "0.2.0");

    if let Some(params) = request.params {
        let params_obj = params.as_object().unwrap();
        assert_eq!(params_obj["client_name"], "deribit-websocket");
        assert_eq!(params_obj["client_version"], "0.2.0");
    } else {
        panic!("hello request should have params");
    }
}

#[test]
fn test_request_builder_set_heartbeat_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_set_heartbeat_request(30);

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "public/set_heartbeat");
    assert!(parsed["id"].is_number());
    assert_eq!(parsed["params"]["interval"], 30);
}

#[test]
fn test_request_builder_hello_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_hello_request("my-app", "2.0.0");

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "public/hello");
    assert!(parsed["id"].is_number());
    assert_eq!(parsed["params"]["client_name"], "my-app");
    assert_eq!(parsed["params"]["client_version"], "2.0.0");
}

#[test]
fn test_request_builder_incremental_ids_session_methods() {
    let mut builder = RequestBuilder::new();

    let req1 = builder.build_set_heartbeat_request(30);
    let req2 = builder.build_disable_heartbeat_request();
    let req3 = builder.build_hello_request("test", "1.0");

    let id1 = req1.id.as_u64().unwrap();
    let id2 = req2.id.as_u64().unwrap();
    let id3 = req3.id.as_u64().unwrap();

    assert_eq!(id2, id1 + 1);
    assert_eq!(id3, id2 + 1);
}

// =============================================================================
// Cancel-on-disconnect request tests (Issue #15)
// =============================================================================

#[test]
fn test_request_builder_enable_cancel_on_disconnect() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_enable_cancel_on_disconnect_request();

    assert_eq!(request.method, "private/enable_cancel_on_disconnect");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_disable_cancel_on_disconnect() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_disable_cancel_on_disconnect_request();

    assert_eq!(request.method, "private/disable_cancel_on_disconnect");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_get_cancel_on_disconnect() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_cancel_on_disconnect_request();

    assert_eq!(request.method, "private/get_cancel_on_disconnect");
    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_number());
    assert!(request.params.is_some());
}

#[test]
fn test_request_builder_enable_cancel_on_disconnect_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_enable_cancel_on_disconnect_request();

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "private/enable_cancel_on_disconnect");
    assert!(parsed["id"].is_number());
}

#[test]
fn test_request_builder_disable_cancel_on_disconnect_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_disable_cancel_on_disconnect_request();

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "private/disable_cancel_on_disconnect");
    assert!(parsed["id"].is_number());
}

#[test]
fn test_request_builder_get_cancel_on_disconnect_serialization() {
    let mut builder = RequestBuilder::new();
    let request = builder.build_get_cancel_on_disconnect_request();

    let serialized = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "private/get_cancel_on_disconnect");
    assert!(parsed["id"].is_number());
}

#[test]
fn test_request_builder_incremental_ids_cancel_on_disconnect() {
    let mut builder = RequestBuilder::new();

    let req1 = builder.build_enable_cancel_on_disconnect_request();
    let req2 = builder.build_disable_cancel_on_disconnect_request();
    let req3 = builder.build_get_cancel_on_disconnect_request();

    let id1 = req1.id.as_u64().unwrap();
    let id2 = req2.id.as_u64().unwrap();
    let id3 = req3.id.as_u64().unwrap();

    assert_eq!(id2, id1 + 1);
    assert_eq!(id3, id2 + 1);
}

// =============================================================================
// Typed response deserialization tests (Issue #16)
// =============================================================================

use deribit_websocket::model::{AuthResponse, HelloResponse, TestResponse};

#[test]
fn test_auth_response_deserialization() {
    let json = r#"{
        "access_token": "test_access_token_12345",
        "token_type": "bearer",
        "expires_in": 3600,
        "refresh_token": "test_refresh_token_67890",
        "scope": "session:name read write"
    }"#;

    let response: AuthResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.access_token, "test_access_token_12345");
    assert_eq!(response.token_type, "bearer");
    assert_eq!(response.expires_in, 3600);
    assert_eq!(response.refresh_token, "test_refresh_token_67890");
    assert_eq!(response.scope, "session:name read write");
}

#[test]
fn test_auth_response_serialization() {
    let response = AuthResponse {
        access_token: "access123".to_string(),
        token_type: "bearer".to_string(),
        expires_in: 7200,
        refresh_token: "refresh456".to_string(),
        scope: "trade:read_write".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["access_token"], "access123");
    assert_eq!(parsed["token_type"], "bearer");
    assert_eq!(parsed["expires_in"], 7200);
    assert_eq!(parsed["refresh_token"], "refresh456");
    assert_eq!(parsed["scope"], "trade:read_write");
}

#[test]
fn test_hello_response_deserialization() {
    let json = r#"{"version": "2.1.0"}"#;

    let response: HelloResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.version, "2.1.0");
}

#[test]
fn test_hello_response_serialization() {
    let response = HelloResponse {
        version: "2.0.0".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["version"], "2.0.0");
}

#[test]
fn test_test_response_deserialization() {
    let json = r#"{"version": "1.2.26"}"#;

    let response: TestResponse = serde_json::from_str(json).unwrap();

    assert_eq!(response.version, "1.2.26");
}

#[test]
fn test_test_response_serialization() {
    let response = TestResponse {
        version: "1.2.30".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["version"], "1.2.30");
}

#[test]
fn test_auth_response_equality() {
    let response1 = AuthResponse {
        access_token: "token1".to_string(),
        token_type: "bearer".to_string(),
        expires_in: 3600,
        refresh_token: "refresh1".to_string(),
        scope: "read".to_string(),
    };

    let response2 = AuthResponse {
        access_token: "token1".to_string(),
        token_type: "bearer".to_string(),
        expires_in: 3600,
        refresh_token: "refresh1".to_string(),
        scope: "read".to_string(),
    };

    let response3 = AuthResponse {
        access_token: "token2".to_string(),
        token_type: "bearer".to_string(),
        expires_in: 3600,
        refresh_token: "refresh1".to_string(),
        scope: "read".to_string(),
    };

    assert_eq!(response1, response2);
    assert_ne!(response1, response3);
}

#[test]
fn test_hello_response_equality() {
    let response1 = HelloResponse {
        version: "2.0.0".to_string(),
    };
    let response2 = HelloResponse {
        version: "2.0.0".to_string(),
    };
    let response3 = HelloResponse {
        version: "2.1.0".to_string(),
    };

    assert_eq!(response1, response2);
    assert_ne!(response1, response3);
}

#[test]
fn test_test_response_equality() {
    let response1 = TestResponse {
        version: "1.2.26".to_string(),
    };
    let response2 = TestResponse {
        version: "1.2.26".to_string(),
    };
    let response3 = TestResponse {
        version: "1.2.30".to_string(),
    };

    assert_eq!(response1, response2);
    assert_ne!(response1, response3);
}

// =============================================================================
// Serialization error propagation tests (Issue #46)
//
// Every request builder that accepts untrusted `f64` input must surface a
// `WebSocketError::Serialization` instead of panicking when the input cannot
// be encoded as JSON (NaN / Infinity). These tests lock in that contract.
// =============================================================================

use deribit_websocket::error::WebSocketError;
use deribit_websocket::model::{
    CancelQuotesRequest, EditOrderRequest, MassQuoteRequest, MmpGroupConfig, MovePositionTrade,
    OrderRequest, Quote,
};

#[test]
fn test_build_mass_quote_request_nan_price_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let quotes = vec![Quote::buy("BTC-PERPETUAL".to_string(), 1.0, f64::NAN)];
    let request = MassQuoteRequest::new("btc_group".to_string(), quotes);

    let result = builder.build_mass_quote_request(request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN price must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_mass_quote_request_infinity_amount_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let quotes = vec![Quote::buy(
        "BTC-PERPETUAL".to_string(),
        f64::INFINITY,
        50_000.0,
    )];
    let request = MassQuoteRequest::new("btc_group".to_string(), quotes);

    let result = builder.build_mass_quote_request(request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "Infinity amount must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_mass_quote_request_happy_path_returns_ok() {
    let mut builder = RequestBuilder::new();
    let quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 1.0, 50_000.0),
        Quote::sell("BTC-PERPETUAL".to_string(), 1.0, 51_000.0),
    ];
    let request = MassQuoteRequest::new("btc_group".to_string(), quotes);

    let rpc = builder
        .build_mass_quote_request(request)
        .expect("valid request should build successfully");

    assert_eq!(rpc.method, "private/mass_quote");
    assert_eq!(rpc.jsonrpc, "2.0");
    assert!(rpc.params.is_some());
}

#[test]
fn test_build_cancel_quotes_request_nan_delta_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let request = CancelQuotesRequest::by_delta_range(f64::NAN, 1.0);

    let result = builder.build_cancel_quotes_request(request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN delta must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_buy_request_nan_price_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 1.0, f64::NAN);

    let result = builder.build_buy_request(&request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN price must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_sell_request_infinity_max_show_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 1.0, 50_000.0)
        .with_max_show(f64::INFINITY);

    let result = builder.build_sell_request(&request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "Infinity max_show must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_edit_request_nan_trigger_price_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    // `EditOrderRequest` only exposes a `with_price` builder, so construct via
    // struct literal to place the NaN on the intended field.
    let request = EditOrderRequest {
        order_id: "order-1".to_string(),
        amount: 1.0,
        price: Some(50_000.0),
        post_only: None,
        reduce_only: None,
        advanced: None,
        trigger_price: Some(f64::NAN),
        mmp: None,
        valid_until: None,
    };

    let result = builder.build_edit_request(&request);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN trigger_price must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_close_position_request_nan_price_returns_serialization_error() {
    let mut builder = RequestBuilder::new();

    let result = builder.build_close_position_request("BTC-PERPETUAL", "limit", Some(f64::NAN));

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN close_position price must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_move_positions_request_nan_amount_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let trades = vec![MovePositionTrade::new("BTC-PERPETUAL", f64::NAN)];

    let result = builder.build_move_positions_request("BTC", 1, 2, &trades);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN move_positions amount must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_set_mmp_config_request_nan_quantity_limit_returns_serialization_error() {
    // `MmpGroupConfig::new` compares magnitudes with `>=` and `>`, both of
    // which return false for NaN. That lets NaN slip past validation and into
    // the wire format as a silent `null`. The builder must catch it.
    let mut builder = RequestBuilder::new();
    let config = MmpGroupConfig {
        mmp_group: "btc_mm".to_string(),
        quantity_limit: f64::NAN,
        delta_limit: 1.0,
        interval: 1_000,
        frozen_time: 5_000,
        enabled: true,
    };

    let result = builder.build_set_mmp_config_request(config);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "NaN quantity_limit must propagate as Serialization error, got {result:?}"
    );
}

#[test]
fn test_build_set_mmp_config_request_infinity_delta_limit_returns_serialization_error() {
    let mut builder = RequestBuilder::new();
    let config = MmpGroupConfig {
        mmp_group: "btc_mm".to_string(),
        quantity_limit: 10.0,
        delta_limit: f64::INFINITY,
        interval: 1_000,
        frozen_time: 5_000,
        enabled: true,
    };

    let result = builder.build_set_mmp_config_request(config);

    assert!(
        matches!(result, Err(WebSocketError::Serialization(_))),
        "Infinity delta_limit must propagate as Serialization error, got {result:?}"
    );
}
