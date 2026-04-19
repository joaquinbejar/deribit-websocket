//! Perpetual subscription example
//!
//! This example demonstrates how to subscribe to perpetual instrument updates
//! including funding rates and perpetual-specific data.

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()
        .map_err(|e| format!("Failed to install crypto provider: {e}"))?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Starting Perpetual Subscription Example");

    // Statistics tracking
    let perpetual_updates = Arc::new(Mutex::new(0u32));
    let perpetual_count_clone = perpetual_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config)?;

    // Set up message handler for perpetual data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("perpetual.")
                && channel.ends_with(".raw")
            {
                let mut count = perpetual_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!("🔄 Perpetual Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract perpetual information
                    if let Some(funding_8h) = data.get("funding_8h") {
                        tracing::info!("   💰 Funding 8h: {}", funding_8h);
                    }
                    if let Some(funding_rate) = data.get("funding_rate") {
                        tracing::info!("   📊 Funding Rate: {}", funding_rate);
                    }
                    if let Some(index_price) = data.get("index_price") {
                        tracing::info!("   📈 Index Price: {}", index_price);
                    }
                    if let Some(mark_price) = data.get("mark_price") {
                        tracing::info!("   🎯 Mark Price: {}", mark_price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        tracing::info!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(interest_rate) = data.get("interest_rate") {
                        tracing::info!("   💹 Interest Rate: {}", interest_rate);
                    }

                    // Extract instrument from channel name
                    let instrument = channel
                        .strip_prefix("perpetual.")
                        .and_then(|s| s.strip_suffix(".raw"))
                        .unwrap_or("unknown");
                    tracing::info!("   🎯 Instrument: {}", instrument);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing perpetual message: {}", error);
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

    // Subscribe to perpetual channels
    tracing::info!("📊 Subscribing to perpetual updates...");
    let channels = vec![
        "perpetual.BTC-PERPETUAL.raw".to_string(),
        "perpetual.ETH-PERPETUAL.raw".to_string(),
        "perpetual.SOL-PERPETUAL.raw".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to perpetual updates"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for perpetual updates (15 seconds)...");
    tracing::info!("   - Perpetual updates include funding rates and mark prices");
    tracing::info!("   - Funding rates are crucial for perpetual contract trading");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *perpetual_updates.lock().unwrap();

    tracing::info!("\n📊 Perpetual Statistics:");
    tracing::info!("   🔄 Total perpetual updates: {}", final_updates);

    if final_updates == 0 {
        tracing::info!("\n💡 Tips for perpetual updates:");
        tracing::info!("   - Perpetual updates include funding rate changes");
        tracing::info!("   - Updates occur regularly, especially around funding times");
        tracing::info!("   - BTC-PERPETUAL and ETH-PERPETUAL are the most active");
    }

    tracing::info!("\n🎉 Perpetual subscription example completed!");
    Ok(())
}
