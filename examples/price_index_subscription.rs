//! Price index subscription example
//!
//! This example demonstrates how to subscribe to Deribit price index updates
//! for various cryptocurrencies.

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

    println!("🚀 Starting Price Index Subscription Example");

    // Statistics tracking
    let index_updates = Arc::new(Mutex::new(0u32));
    let index_count_clone = index_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for price index data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("deribit_price_index.")
            {
                let mut count = index_count_clone.lock().unwrap();
                *count += 1;

                println!("📊 Price Index Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract price index information
                    if let Some(price) = data.get("price") {
                        println!("   💰 Index Price: {}", price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        println!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(index_name) = data.get("index_name") {
                        println!("   🏷️  Index Name: {}", index_name);
                    }

                    // Extract currency from channel name
                    let currency = channel
                        .strip_prefix("deribit_price_index.")
                        .and_then(|s| s.strip_suffix("_usd"))
                        .unwrap_or("unknown");
                    println!("   💱 Currency: {}", currency.to_uppercase());
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing price index message: {}", error);
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

    // Subscribe to price index channels
    println!("📊 Subscribing to price indices...");
    let channels = vec![
        "deribit_price_index.btc_usd".to_string(),
        "deribit_price_index.eth_usd".to_string(),
        "deribit_price_index.sol_usd".to_string(),
        "deribit_price_index.usdc_usd".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to price indices"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for price index updates (15 seconds)...");
    println!("   - Price index updates show reference prices for each currency");
    println!("   - These are used for mark price calculations and settlements");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *index_updates.lock().unwrap();

    println!("\n📊 Price Index Statistics:");
    println!("   📈 Total index updates: {}", final_updates);

    if final_updates == 0 {
        println!("\n💡 Tips for price index updates:");
        println!("   - Price indices update regularly based on external exchanges");
        println!("   - Updates may be less frequent during low volatility periods");
        println!("   - BTC and ETH indices typically have the most frequent updates");
    }

    println!("\n🎉 Price index subscription example completed!");
    Ok(())
}
