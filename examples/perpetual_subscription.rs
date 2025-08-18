//! Perpetual subscription example
//!
//! This example demonstrates how to subscribe to perpetual instrument updates
//! including funding rates and perpetual-specific data.

use deribit_websocket::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Initialize logging
    env_logger::init();

    println!("🚀 Starting Perpetual Subscription Example");

    // Statistics tracking
    let perpetual_updates = Arc::new(Mutex::new(0u32));
    let perpetual_count_clone = perpetual_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for perpetual data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("perpetual.")
                && channel.ends_with(".raw")
            {
                let mut count = perpetual_count_clone.lock().unwrap();
                *count += 1;

                println!("🔄 Perpetual Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract perpetual information
                    if let Some(funding_8h) = data.get("funding_8h") {
                        println!("   💰 Funding 8h: {}", funding_8h);
                    }
                    if let Some(funding_rate) = data.get("funding_rate") {
                        println!("   📊 Funding Rate: {}", funding_rate);
                    }
                    if let Some(index_price) = data.get("index_price") {
                        println!("   📈 Index Price: {}", index_price);
                    }
                    if let Some(mark_price) = data.get("mark_price") {
                        println!("   🎯 Mark Price: {}", mark_price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        println!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(interest_rate) = data.get("interest_rate") {
                        println!("   💹 Interest Rate: {}", interest_rate);
                    }

                    // Extract instrument from channel name
                    let instrument = channel
                        .strip_prefix("perpetual.")
                        .and_then(|s| s.strip_suffix(".raw"))
                        .unwrap_or("unknown");
                    println!("   🎯 Instrument: {}", instrument);
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing perpetual message: {}", error);
            println!(
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
    println!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    println!("✅ Connected successfully");

    // Subscribe to perpetual channels
    println!("📊 Subscribing to perpetual updates...");
    let channels = vec![
        "perpetual.BTC-PERPETUAL.raw".to_string(),
        "perpetual.ETH-PERPETUAL.raw".to_string(),
        "perpetual.SOL-PERPETUAL.raw".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to perpetual updates"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for perpetual updates (15 seconds)...");
    println!("   - Perpetual updates include funding rates and mark prices");
    println!("   - Funding rates are crucial for perpetual contract trading");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *perpetual_updates.lock().unwrap();

    println!("\n📊 Perpetual Statistics:");
    println!("   🔄 Total perpetual updates: {}", final_updates);

    if final_updates == 0 {
        println!("\n💡 Tips for perpetual updates:");
        println!("   - Perpetual updates include funding rate changes");
        println!("   - Updates occur regularly, especially around funding times");
        println!("   - BTC-PERPETUAL and ETH-PERPETUAL are the most active");
    }

    println!("\n🎉 Perpetual subscription example completed!");
    Ok(())
}
