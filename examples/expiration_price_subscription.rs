//! Estimated expiration price subscription example
//!
//! This example demonstrates how to subscribe to estimated expiration price updates
//! for options and futures instruments.

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
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

    tracing::info!("🚀 Starting Estimated Expiration Price Subscription Example");

    // Statistics tracking
    let price_updates = Arc::new(Mutex::new(0u32));
    let price_count_clone = price_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for expiration price data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("estimated_expiration_price.")
            {
                let mut count = price_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!(
                    "⏰ Expiration Price Update #{}: Channel: {}",
                    *count,
                    channel
                );

                if let Some(data) = params.get("data") {
                    // Extract expiration price information
                    if let Some(estimated_price) = data.get("estimated_price") {
                        tracing::info!("   💰 Estimated Price: {}", estimated_price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        tracing::info!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(seconds_to_expiry) = data.get("seconds_to_expiry") {
                        tracing::info!("   ⏳ Seconds to Expiry: {}", seconds_to_expiry);
                    }

                    // Extract instrument from channel name
                    let instrument = channel
                        .strip_prefix("estimated_expiration_price.")
                        .unwrap_or("unknown");
                    tracing::info!("   🎯 Instrument: {}", instrument);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing expiration price message: {}", error);
            tracing::info!(
                "   Message preview: {}",
                if message.len() > 100 {
                    format!("{}...", &message[..100])
                } else {
                    message.to_string()
                }
            );
        },
    );

    // Connect to server
    tracing::info!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    tracing::info!("✅ Connected successfully");

    // Subscribe to estimated expiration price channels
    tracing::info!("📊 Subscribing to estimated expiration prices...");
    let channels = vec![
        "estimated_expiration_price.btc_usd".to_string(),
        "estimated_expiration_price.eth_usd".to_string(),
        // Add specific options if available
        "estimated_expiration_price.BTC-29MAR24".to_string(),
        "estimated_expiration_price.ETH-29MAR24".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to expiration prices"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for expiration price updates (15 seconds)...");
    tracing::info!("   - Estimated expiration prices are used for options settlement");
    tracing::info!("   - Updates show projected settlement prices");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *price_updates.lock().unwrap();

    tracing::info!("\n📊 Expiration Price Statistics:");
    tracing::info!("   ⏰ Total expiration price updates: {}", final_updates);

    if final_updates == 0 {
        tracing::info!("\n💡 Tips for expiration price updates:");
        tracing::info!("   - Expiration prices are calculated for instruments nearing expiry");
        tracing::info!("   - Updates are more frequent closer to expiration time");
        tracing::info!("   - Try subscribing to instruments with upcoming expirations");
    }

    tracing::info!("\n🎉 Estimated expiration price subscription example completed!");
    Ok(())
}
