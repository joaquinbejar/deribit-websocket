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
