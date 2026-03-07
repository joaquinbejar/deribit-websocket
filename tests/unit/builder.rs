//! Unit tests for message/builder.rs

use deribit_websocket::message::{MessageBuilder, MessageType};

#[test]
fn test_message_builder_new() {
    let builder = MessageBuilder::new();
    // Verify builder is created successfully
    assert!(format!("{:?}", builder).contains("MessageBuilder"));
}

#[test]
fn test_message_builder_default() {
    let builder = MessageBuilder::default();
    assert!(format!("{:?}", builder).contains("MessageBuilder"));
}

#[test]
fn test_message_builder_request_builder() {
    let mut builder = MessageBuilder::new();
    let request_builder = builder.request_builder();
    // Verify we can access the request builder
    assert!(format!("{:?}", request_builder).contains("RequestBuilder"));
}

#[test]
fn test_message_builder_response_handler() {
    let builder = MessageBuilder::new();
    let response_handler = builder.response_handler();
    assert!(format!("{:?}", response_handler).contains("ResponseHandler"));
}

#[test]
fn test_message_builder_notification_handler() {
    let builder = MessageBuilder::new();
    let notification_handler = builder.notification_handler();
    assert!(format!("{:?}", notification_handler).contains("NotificationHandler"));
}

#[test]
fn test_parse_message_response() {
    let builder = MessageBuilder::new();
    
    let response_json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"version": "1.2.26"}
    }"#;
    
    let result = builder.parse_message(response_json);
    assert!(result.is_ok());
    
    if let Ok(MessageType::Response(response)) = result {
        assert_eq!(response.id.as_u64().unwrap(), 1);
    } else {
        panic!("Expected Response message type");
    }
}

#[test]
fn test_parse_message_notification() {
    let builder = MessageBuilder::new();
    
    let notification_json = r#"{
        "jsonrpc": "2.0",
        "method": "subscription",
        "params": {
            "channel": "ticker.BTC-PERPETUAL",
            "data": {"price": 50000}
        }
    }"#;
    
    let result = builder.parse_message(notification_json);
    assert!(result.is_ok());
    
    if let Ok(MessageType::Notification(notification)) = result {
        assert_eq!(notification.method, "subscription");
    } else {
        panic!("Expected Notification message type");
    }
}

#[test]
fn test_parse_message_invalid() {
    let builder = MessageBuilder::new();
    
    let invalid_json = r#"not valid json"#;
    
    let result = builder.parse_message(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_message_type_debug() {
    let builder = MessageBuilder::new();
    
    let response_json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"version": "1.2.26"}
    }"#;
    
    if let Ok(msg_type) = builder.parse_message(response_json) {
        let debug_str = format!("{:?}", msg_type);
        assert!(debug_str.contains("Response"));
    }
}

#[test]
fn test_message_type_clone() {
    let builder = MessageBuilder::new();
    
    let response_json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"version": "1.2.26"}
    }"#;
    
    if let Ok(msg_type) = builder.parse_message(response_json) {
        let cloned = msg_type.clone();
        assert!(format!("{:?}", cloned).contains("Response"));
    }
}
