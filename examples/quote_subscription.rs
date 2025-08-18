//! Quote subscription example
//!
//! This example demonstrates how to subscribe to quote updates
//! for instruments, showing bid/ask prices and sizes.

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

    println!("🚀 Starting Quote Subscription Example");

    // Statistics tracking
    let quote_updates = Arc::new(Mutex::new(0u32));
    let quote_count_clone = quote_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for quote data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("quote.")
            {
                let mut count = quote_count_clone.lock().unwrap();
                *count += 1;

                println!("💱 Quote Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract quote information
                    if let Some(best_bid_price) = data.get("best_bid_price") {
                        println!("   📉 Best Bid Price: {}", best_bid_price);
                    }
                    if let Some(best_bid_amount) = data.get("best_bid_amount") {
                        println!("   📊 Best Bid Amount: {}", best_bid_amount);
                    }
                    if let Some(best_ask_price) = data.get("best_ask_price") {
                        println!("   📈 Best Ask Price: {}", best_ask_price);
                    }
                    if let Some(best_ask_amount) = data.get("best_ask_amount") {
                        println!("   📊 Best Ask Amount: {}", best_ask_amount);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        println!("   ⏰ Timestamp: {}", timestamp);
                    }

                    // Calculate spread if both prices available
                    if let (Some(bid), Some(ask)) = (
                        data.get("best_bid_price").and_then(|p| p.as_f64()),
                        data.get("best_ask_price").and_then(|p| p.as_f64()),
                    ) {
                        let spread = ask - bid;
                        let spread_pct = (spread / bid) * 100.0;
                        println!("   📏 Spread: {:.4} ({:.2}%)", spread, spread_pct);
                    }

                    // Extract instrument from channel name
                    let instrument = channel.strip_prefix("quote.").unwrap_or("unknown");
                    println!("   🎯 Instrument: {}", instrument);
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing quote message: {}", error);
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

    // Subscribe to quote channels
    println!("📊 Subscribing to quotes...");
    let channels = vec![
        "quote.BTC-PERPETUAL".to_string(),
        "quote.ETH-PERPETUAL".to_string(),
        "quote.BTC-29MAR24-50000-C".to_string(), // Options example
        "quote.ETH-29MAR24-3000-P".to_string(),  // Options example
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to quotes"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for quote updates (15 seconds)...");
    println!("   - Quotes show best bid/ask prices and amounts");
    println!("   - Spread calculations are included");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *quote_updates.lock().unwrap();

    println!("\n📊 Quote Statistics:");
    println!("   💱 Total quote updates: {}", final_updates);

    if final_updates == 0 {
        println!("\n💡 Tips for quote updates:");
        println!("   - Quote updates show best bid/ask changes");
        println!("   - Updates occur when order book top levels change");
        println!("   - Perpetual instruments typically have the most activity");
    }

    println!("\n🎉 Quote subscription example completed!");
    Ok(())
}
