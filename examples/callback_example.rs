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
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Starting Deribit WebSocket Callback Example");

    // Shared state for tracking processed messages
    let processed_count = Arc::new(Mutex::new(0u32));
    let error_count = Arc::new(Mutex::new(0u32));

    // Clone for use in callbacks
    let processed_count_clone = processed_count.clone();
    let error_count_clone = error_count.clone();

    // Create client configuration
    setup_logger();
    let mut client = DeribitWebSocketClient::default();

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

                    tracing::info!("📨 Processing message #{}: {}", *count, msg_type);

                    // Simulate processing failure for demonstration (every 5th message)
                    if *count % 5 == 0 {
                        tracing::warn!("⚠️ Simulating processing error for message #{}", *count);
                        return Err(WebSocketError::InvalidMessage(format!(
                            "Simulated processing error for message #{}",
                            *count
                        )));
                    }

                    Ok(())
                }
                Err(e) => {
                    // Failed to parse JSON
                    tracing::error!("❌ Failed to parse JSON: {}", e);
                    Err(WebSocketError::InvalidMessage(format!(
                        "Failed to parse JSON: {}",
                        e
                    )))
                }
            }
        },
        // Error callback: handles failures from primary callback
        move |message: &str, error: &WebSocketError| {
            tracing::error!("🔥 Error callback triggered!");
            let mut count = error_count_clone.lock().unwrap();
            *count += 1;

            // Log error details
            let preview = if message.len() > 100 {
                format!("{}...", &message[..100])
            } else {
                message.to_string()
            };

            tracing::error!("   Message preview: {}", preview);

            // Log error details or send to monitoring system
            tracing::error!("   Error: {}", error);
        },
    );

    // Connect to server
    tracing::info!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    tracing::info!("✅ Client created successfully");

    // Subscribe to some channels to generate messages
    tracing::info!("📡 Subscribing to market data channels...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Subscribed to channels"),
        Err(e) => tracing::error!("❌ Subscription failed: {}", e),
    }

    // Start the message processing loop
    tracing::info!("🛑 Stopping message processing...");
    tracing::info!("   - Messages will be processed by the primary callback");
    tracing::info!("   - Errors will be handled by the error callback");
    tracing::info!("⏳ Processing messages for 15 seconds...");

    // Run the processing loop for a limited time
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 10 seconds
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Stop the processing (in a real application, you'd have a proper shutdown mechanism)
    processing_task.abort();

    // Display final statistics
    let final_processed = *processed_count.lock().unwrap();
    let final_errors = *error_count.lock().unwrap();

    tracing::info!("📊 Final Statistics:");
    tracing::info!("   💚 Successfully processed: {} messages", final_processed);
    tracing::info!("   🔴 Errors encountered: {} messages", final_errors);
    tracing::info!(
        "   📈 Total messages handled: {}",
        final_processed + final_errors
    );

    tracing::info!("🎉 Callback example completed successfully!");
    Ok(())
}
