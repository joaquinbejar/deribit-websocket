//! Quote subscription example
//!
//! This example demonstrates how to subscribe to quote updates
//! for instruments, showing bid/ask prices and sizes.

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

    tracing::info!("🚀 Starting Quote Subscription Example");

    // Statistics tracking
    let quote_updates = Arc::new(Mutex::new(0u32));
    let quote_count_clone = quote_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config)?;

    // Set up message handler for quote data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("quote.")
            {
                let mut count = quote_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!("💱 Quote Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract quote information
                    if let Some(best_bid_price) = data.get("best_bid_price") {
                        tracing::info!("   📉 Best Bid Price: {}", best_bid_price);
                    }
                    if let Some(best_bid_amount) = data.get("best_bid_amount") {
                        tracing::info!("   📊 Best Bid Amount: {}", best_bid_amount);
                    }
                    if let Some(best_ask_price) = data.get("best_ask_price") {
                        tracing::info!("   📈 Best Ask Price: {}", best_ask_price);
                    }
                    if let Some(best_ask_amount) = data.get("best_ask_amount") {
                        tracing::info!("   📊 Best Ask Amount: {}", best_ask_amount);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        tracing::info!("   ⏰ Timestamp: {}", timestamp);
                    }

                    // Calculate spread if both prices available
                    if let (Some(bid), Some(ask)) = (
                        data.get("best_bid_price").and_then(|p| p.as_f64()),
                        data.get("best_ask_price").and_then(|p| p.as_f64()),
                    ) {
                        let spread = ask - bid;
                        let spread_pct = (spread / bid) * 100.0;
                        tracing::info!("   📏 Spread: {:.4} ({:.2}%)", spread, spread_pct);
                    }

                    // Extract instrument from channel name
                    let instrument = channel.strip_prefix("quote.").unwrap_or("unknown");
                    tracing::info!("   🎯 Instrument: {}", instrument);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing quote message: {}", error);
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

    // Subscribe to quote channels
    tracing::info!("📊 Subscribing to quotes...");
    let channels = vec![
        "quote.BTC-PERPETUAL".to_string(),
        "quote.ETH-PERPETUAL".to_string(),
        "quote.BTC-29MAR24-50000-C".to_string(), // Options example
        "quote.ETH-29MAR24-3000-P".to_string(),  // Options example
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to quotes"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for quote updates (15 seconds)...");
    tracing::info!("   - Quotes show best bid/ask prices and amounts");
    tracing::info!("   - Spread calculations are included");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *quote_updates.lock().unwrap();

    tracing::info!("\n📊 Quote Statistics:");
    tracing::info!("   💱 Total quote updates: {}", final_updates);

    if final_updates == 0 {
        tracing::info!("\n💡 Tips for quote updates:");
        tracing::info!("   - Quote updates show best bid/ask changes");
        tracing::info!("   - Updates occur when order book top levels change");
        tracing::info!("   - Perpetual instruments typically have the most activity");
    }

    tracing::info!("\n🎉 Quote subscription example completed!");
    Ok(())
}
