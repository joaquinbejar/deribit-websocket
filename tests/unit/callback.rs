//! Unit tests for callback module

use deribit_websocket::callback::{MessageHandler, MessageHandlerBuilder};
use deribit_websocket::error::WebSocketError;
use std::sync::{Arc, Mutex};

#[test]
fn test_message_handler_creation() {
    let handler = MessageHandler::new(|_message| Ok(()), |_message, _error| {});

    // handle_message returns () not Option, so we just test it doesn't panic
    handler.handle_message("test");
}

#[test]
fn test_message_handler_success() {
    let processed = Arc::new(Mutex::new(false));
    let processed_clone = processed.clone();

    let handler = MessageHandler::new(
        move |_message| {
            *processed_clone.lock().unwrap() = true;
            Ok(())
        },
        |_message, _error| {},
    );

    handler.handle_message("test message");
    assert!(*processed.lock().unwrap());
}

#[test]
fn test_message_handler_error() {
    let error_called = Arc::new(Mutex::new(false));
    let error_clone = error_called.clone();

    let handler = MessageHandler::new(
        |_message| Err(WebSocketError::InvalidMessage("test error".to_string())),
        move |_message, _error| {
            *error_clone.lock().unwrap() = true;
        },
    );

    handler.handle_message("test message");
    assert!(*error_called.lock().unwrap());
}

#[test]
fn test_message_handler_builder() {
    let processed = Arc::new(Mutex::new(0u32));
    let processed_clone = processed.clone();

    let handler = MessageHandlerBuilder::new()
        .with_message_callback(move |_message| {
            *processed_clone.lock().unwrap() += 1;
            Ok(())
        })
        .with_error_callback(|_message, _error| {})
        .build()
        .expect("Failed to build handler");

    handler.handle_message("message 1");
    handler.handle_message("message 2");

    assert_eq!(*processed.lock().unwrap(), 2);
}

#[test]
fn test_message_handler_builder_missing_callback() {
    let result = MessageHandlerBuilder::new()
        .with_error_callback(|_message, _error| {})
        .build();

    assert!(result.is_err());
}

#[test]
fn test_message_handler_builder_missing_error_callback() {
    let result = MessageHandlerBuilder::new()
        .with_message_callback(|_message| Ok(()))
        .build();

    assert!(result.is_err());
}

#[test]
fn test_message_handler_with_complex_processing() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let errors_clone = errors.clone();

    let handler = MessageHandler::new(
        move |message: &str| {
            messages_clone.lock().unwrap().push(message.to_string());
            if message.contains("error") {
                Err(WebSocketError::InvalidMessage(
                    "Simulated error".to_string(),
                ))
            } else {
                Ok(())
            }
        },
        move |message: &str, error: &WebSocketError| {
            errors_clone
                .lock()
                .unwrap()
                .push((message.to_string(), error.to_string()));
        },
    );

    handler.handle_message("normal message");
    handler.handle_message("error message");
    handler.handle_message("another normal message");

    let messages = messages.lock().unwrap();
    let errors = errors.lock().unwrap();

    assert_eq!(messages.len(), 3);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].0.contains("error message"));
}
