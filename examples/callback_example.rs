//! Example demonstrating callback-based message handling
//!
//! This example shows how to use the callback system where:
//! 1. Primary callback processes each message and returns Result<(), Error>
//! 2. Error callback handles failures from the primary callback

use deribit_websocket::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Initialize logging
    env_logger::init();

    println!("🚀 Starting Deribit WebSocket Callback Example");

    // Shared state for tracking processed messages
    let processed_count = Arc::new(Mutex::new(0u32));
    let error_count = Arc::new(Mutex::new(0u32));

    // Clone for use in callbacks
    let processed_count_clone = processed_count.clone();
    let error_count_clone = error_count.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up callback system
    client.set_message_handler(
        // Primary callback: processes each message
        move |message: &str| -> Result<(), WebSocketError> {
            // Try to parse the message as JSON
            match serde_json::from_str::<Value>(message) {
                Ok(json) => {
                    // Successfully parsed JSON
                    let mut count = processed_count_clone.lock().unwrap();
                    *count += 1;

                    // Extract message type if available
                    let msg_type = json
                        .get("method")
                        .or_else(|| json.get("result"))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    println!("✅ Processed message #{}: type={}", *count, msg_type);

                    // Simulate processing failure for demonstration (every 5th message)
                    if *count % 5 == 0 {
                        return Err(WebSocketError::InvalidMessage(format!(
                            "Simulated processing error for message #{}",
                            *count
                        )));
                    }

                    Ok(())
                }
                Err(e) => {
                    // Failed to parse JSON
                    Err(WebSocketError::InvalidMessage(format!(
                        "Failed to parse JSON: {}",
                        e
                    )))
                }
            }
        },
        // Error callback: handles failures from primary callback
        move |message: &str, error: &WebSocketError| {
            let mut count = error_count_clone.lock().unwrap();
            *count += 1;

            println!("❌ Error #{} processing message: {}", *count, error);
            println!(
                "   Message preview: {}",
                if message.len() > 100 {
                    format!("{}...", &message[..100])
                } else {
                    message.to_string()
                }
            );

            // Log error details or send to monitoring system
            eprintln!("ERROR LOG: {}", error);
        },
    );

    // Connect to server
    println!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    println!("✅ Connected successfully");

    // Subscribe to some channels to generate messages
    println!("📊 Subscribing to market data...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Subscribed to channels"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start the message processing loop
    println!("🔄 Starting message processing loop...");
    println!("   - Messages will be processed by the primary callback");
    println!("   - Errors will be handled by the error callback");
    println!("   - Processing will run for 10 seconds...");

    // Run the processing loop for a limited time
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 10 seconds
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Stop the processing (in a real application, you'd have a proper shutdown mechanism)
    processing_task.abort();

    // Display final statistics
    let final_processed = *processed_count.lock().unwrap();
    let final_errors = *error_count.lock().unwrap();

    println!("\n📊 Final Statistics:");
    println!("   ✅ Successfully processed messages: {}", final_processed);
    println!("   ❌ Messages with errors: {}", final_errors);
    println!(
        "   📈 Total messages handled: {}",
        final_processed + final_errors
    );

    println!("\n🎉 Callback example completed!");
    Ok(())
}

// Alternative example using the builder pattern
#[allow(dead_code)]
async fn example_with_builder() -> Result<(), Box<dyn std::error::Error>> {
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Create handler using builder pattern
    let handler = MessageHandlerBuilder::new()
        .with_message_callback(|message| {
            println!("Processing: {}", message);
            // Your message processing logic here
            Ok(())
        })
        .with_error_callback(|message, error| {
            eprintln!("Error processing message: {}", error);
            eprintln!("Message was: {}", message);
        })
        .build()?;

    client.set_message_handler_builder(handler);
    client.connect().await?;

    // Process messages
    client.start_message_processing_loop().await?;

    Ok(())
}
