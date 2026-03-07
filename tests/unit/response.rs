//! Unit tests for message/response.rs

use deribit_websocket::message::ResponseHandler;
use deribit_websocket::model::ws_types::{JsonRpcError, JsonRpcResponse, JsonRpcResult};

#[test]
fn test_response_handler_new() {
    let handler = ResponseHandler::new();
    assert!(format!("{:?}", handler).contains("ResponseHandler"));
}

#[test]
fn test_response_handler_default() {
    let handler = ResponseHandler::default();
    assert!(format!("{:?}", handler).contains("ResponseHandler"));
}

#[test]
fn test_response_handler_clone() {
    let handler = ResponseHandler::new();
    let cloned = handler.clone();
    assert!(format!("{:?}", cloned).contains("ResponseHandler"));
}

#[test]
fn test_parse_response_success() {
    let handler = ResponseHandler::new();
    
    let json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"version": "1.2.26"}
    }"#;
    
    let result = handler.parse_response(json);
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.id.as_u64().unwrap(), 1);
}

#[test]
fn test_parse_response_error() {
    let handler = ResponseHandler::new();
    
    let json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32600,
            "message": "Invalid Request"
        }
    }"#;
    
    let result = handler.parse_response(json);
    assert!(result.is_ok());
}

#[test]
fn test_parse_response_invalid() {
    let handler = ResponseHandler::new();
    
    let json = r#"not valid json"#;
    
    let result = handler.parse_response(json);
    assert!(result.is_err());
}

#[test]
fn test_is_success_true() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Success {
            result: serde_json::json!({"version": "1.2.26"}),
        },
    };
    
    assert!(handler.is_success(&response));
}

#[test]
fn test_is_success_false() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Error {
            error: JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            },
        },
    };
    
    assert!(!handler.is_success(&response));
}

#[test]
fn test_extract_result_success() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Success {
            result: serde_json::json!({"version": "1.2.26"}),
        },
    };
    
    let result = handler.extract_result(&response);
    assert!(result.is_some());
    assert_eq!(result.unwrap()["version"], "1.2.26");
}

#[test]
fn test_extract_result_error() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Error {
            error: JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            },
        },
    };
    
    let result = handler.extract_result(&response);
    assert!(result.is_none());
}

#[test]
fn test_extract_error_success() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Success {
            result: serde_json::json!({"version": "1.2.26"}),
        },
    };
    
    let error = handler.extract_error(&response);
    assert!(error.is_none());
}

#[test]
fn test_extract_error_error() {
    let handler = ResponseHandler::new();
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        result: JsonRpcResult::Error {
            error: JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            },
        },
    };
    
    let error = handler.extract_error(&response);
    assert!(error.is_some());
    assert_eq!(error.unwrap().code, -32600);
    assert_eq!(error.unwrap().message, "Invalid Request");
}
