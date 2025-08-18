//! Real-time trades subscription example
//!
//! This example demonstrates how to subscribe to real-time trade data
//! and process individual trade events as they occur.

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

    println!("🚀 Starting Real-time Trades Subscription Example");

    // Statistics tracking
    let trade_count = Arc::new(Mutex::new(0u32));
    let total_volume = Arc::new(Mutex::new(0.0f64));
    let trade_count_clone = trade_count.clone();
    let volume_clone = total_volume.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for trade data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("trades.")
                && let Some(data) = params.get("data")
            {
                if let Some(trades) = data.as_array() {
                    for trade in trades {
                        let mut count = trade_count_clone.lock().unwrap();
                        let mut volume = volume_clone.lock().unwrap();
                        *count += 1;

                        // Extract trade information
                        let price = trade.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                        let amount = trade.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
                        let direction = trade
                            .get("direction")
                            .and_then(|d| d.as_str())
                            .unwrap_or("unknown");
                        let timestamp =
                            trade.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0);
                        let trade_id = trade
                            .get("trade_id")
                            .and_then(|id| id.as_str())
                            .unwrap_or("unknown");

                        *volume += amount;

                        println!(
                            "💰 Trade #{}: {} {} @ {} (ID: {}, Time: {})",
                            *count, direction, amount, price, trade_id, timestamp
                        );
                    }
                } else if let Some(trade) = data.as_object() {
                    // Single trade object
                    let mut count = trade_count_clone.lock().unwrap();
                    let mut volume = volume_clone.lock().unwrap();
                    *count += 1;

                    let price = trade.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                    let amount = trade.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
                    let direction = trade
                        .get("direction")
                        .and_then(|d| d.as_str())
                        .unwrap_or("unknown");

                    *volume += amount;

                    println!("💰 Trade #{}: {} {} @ {}", *count, direction, amount, price);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing trade message: {}", error);
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

    // Subscribe to trades channels
    println!("📊 Subscribing to real-time trades...");
    let channels = vec![
        "trades.BTC-PERPETUAL.raw".to_string(),
        "trades.ETH-PERPETUAL.raw".to_string(),
        "trades.BTC-29MAR24.raw".to_string(), // Options example
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to trades channels"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for real-time trades (15 seconds)...");
    println!("   - Each trade will show direction, amount, and price");
    println!("   - Volume statistics will be tracked");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_trades = *trade_count.lock().unwrap();
    let final_volume = *total_volume.lock().unwrap();

    println!("\n📊 Trade Statistics:");
    println!("   💰 Total trades processed: {}", final_trades);
    println!("   📈 Total volume: {:.4}", final_volume);
    if final_trades > 0 {
        println!(
            "   📊 Average trade size: {:.4}",
            final_volume / final_trades as f64
        );
    }

    if final_trades == 0 {
        println!("\n💡 Tips for trade data:");
        println!("   - Trade data depends on market activity");
        println!("   - Try during active trading hours");
        println!("   - BTC-PERPETUAL usually has the most activity");
    }

    println!("\n🎉 Real-time trades example completed!");
    Ok(())
}
