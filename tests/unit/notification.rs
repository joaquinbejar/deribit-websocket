//! Unit tests for message/notification.rs

use deribit_websocket::message::NotificationHandler;
use deribit_websocket::model::ws_types::JsonRpcNotification;

#[test]
fn test_notification_handler_new() {
    let handler = NotificationHandler::new();
    assert!(format!("{:?}", handler).contains("NotificationHandler"));
}

#[test]
fn test_notification_handler_default() {
    let handler = NotificationHandler::default();
    assert!(format!("{:?}", handler).contains("NotificationHandler"));
}

#[test]
fn test_notification_handler_clone() {
    let handler = NotificationHandler::new();
    let cloned = handler.clone();
    assert!(format!("{:?}", cloned).contains("NotificationHandler"));
}

#[test]
fn test_parse_notification_valid() {
    let handler = NotificationHandler::new();
    
    let json = r#"{
        "jsonrpc": "2.0",
        "method": "subscription",
        "params": {
            "channel": "ticker.BTC-PERPETUAL",
            "data": {"price": 50000}
        }
    }"#;
    
    let result = handler.parse_notification(json);
    assert!(result.is_ok());
    
    let notification = result.unwrap();
    assert_eq!(notification.method, "subscription");
}

#[test]
fn test_parse_notification_invalid() {
    let handler = NotificationHandler::new();
    
    let json = r#"not valid json"#;
    
    let result = handler.parse_notification(json);
    assert!(result.is_err());
}

#[test]
fn test_is_subscription_notification_true() {
    let handler = NotificationHandler::new();
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: None,
    };
    
    assert!(handler.is_subscription_notification(&notification));
}

#[test]
fn test_is_subscription_notification_false() {
    let handler = NotificationHandler::new();
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "heartbeat".to_string(),
        params: None,
    };
    
    assert!(!handler.is_subscription_notification(&notification));
}

#[test]
fn test_extract_channel_with_channel() {
    let handler = NotificationHandler::new();
    
    let mut params = serde_json::Map::new();
    params.insert("channel".to_string(), serde_json::json!("ticker.BTC-PERPETUAL"));
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: Some(serde_json::Value::Object(params)),
    };
    
    let channel = handler.extract_channel(&notification);
    assert_eq!(channel, Some("ticker.BTC-PERPETUAL".to_string()));
}

#[test]
fn test_extract_channel_no_params() {
    let handler = NotificationHandler::new();
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: None,
    };
    
    let channel = handler.extract_channel(&notification);
    assert!(channel.is_none());
}

#[test]
fn test_extract_channel_no_channel_field() {
    let handler = NotificationHandler::new();
    
    let mut params = serde_json::Map::new();
    params.insert("data".to_string(), serde_json::json!({"price": 50000}));
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: Some(serde_json::Value::Object(params)),
    };
    
    let channel = handler.extract_channel(&notification);
    assert!(channel.is_none());
}

#[test]
fn test_extract_data_with_data() {
    let handler = NotificationHandler::new();
    
    let mut params = serde_json::Map::new();
    params.insert("data".to_string(), serde_json::json!({"price": 50000}));
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: Some(serde_json::Value::Object(params)),
    };
    
    let data = handler.extract_data(&notification);
    assert!(data.is_some());
    assert_eq!(data.unwrap()["price"], 50000);
}

#[test]
fn test_extract_data_no_params() {
    let handler = NotificationHandler::new();
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: None,
    };
    
    let data = handler.extract_data(&notification);
    assert!(data.is_none());
}

#[test]
fn test_extract_data_no_data_field() {
    let handler = NotificationHandler::new();
    
    let mut params = serde_json::Map::new();
    params.insert("channel".to_string(), serde_json::json!("ticker.BTC-PERPETUAL"));
    
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "subscription".to_string(),
        params: Some(serde_json::Value::Object(params)),
    };
    
    let data = handler.extract_data(&notification);
    assert!(data.is_none());
}
