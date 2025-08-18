//! Advanced subscription example demonstrating chart data and user changes
//!
//! This example shows how to subscribe to the newly implemented channels:
//! - chart.trades.{instrument}.{resolution} for chart data
//! - user.changes.{instrument}.{interval} for position changes

use deribit_websocket::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    println!("🚀 Starting advanced subscriptions example...");

    // Create WebSocket client for testnet
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Statistics tracking
    let chart_messages = Arc::new(Mutex::new(0u32));
    let user_changes_messages = Arc::new(Mutex::new(0u32));
    let other_messages = Arc::new(Mutex::new(0u32));

    let chart_count = chart_messages.clone();
    let changes_count = user_changes_messages.clone();
    let other_count = other_messages.clone();

    // Set up message handler with channel-specific processing
    client.set_message_handler(
        move |message: &str| {
            match serde_json::from_str::<serde_json::Value>(message) {
                Ok(json) => {
                    // Check if this is a subscription notification
                    if let Some(method) = json.get("method")
                        && method == "subscription"
                        && let Some(params) = json.get("params")
                        && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                    {
                        // Process different channel types
                        if channel.starts_with("chart.trades.") {
                            let mut count = chart_count.lock().unwrap();
                            *count += 1;

                            // Extract chart data
                            if let Some(data) = params.get("data") {
                                println!("📊 Chart Data #{}: Channel: {}", *count, channel);
                                if let Some(trades) = data.as_array() {
                                    println!("   📈 Received {} trade points", trades.len());
                                    // Process first trade point as example
                                    if let Some(first_trade) = trades.first()
                                        && let (Some(timestamp), Some(price), Some(amount)) = (
                                            first_trade.get("timestamp"),
                                            first_trade.get("price"),
                                            first_trade.get("amount"),
                                        )
                                    {
                                        println!(
                                            "   💰 Sample: Price: {}, Amount: {}, Time: {}",
                                            price, amount, timestamp
                                        );
                                    }
                                }
                            }
                        } else if channel.starts_with("user.changes.") {
                            let mut count = changes_count.lock().unwrap();
                            *count += 1;

                            // Extract position change data
                            if let Some(data) = params.get("data") {
                                println!("🔄 Position Change #{}: Channel: {}", *count, channel);

                                // Extract key position information
                                if let Some(instrument) = data.get("instrument_name") {
                                    println!("   📍 Instrument: {}", instrument);
                                }
                                if let Some(size) = data.get("size") {
                                    println!("   📏 Position Size: {}", size);
                                }
                                if let Some(direction) = data.get("direction") {
                                    println!("   ➡️  Direction: {}", direction);
                                }
                                if let Some(mark_price) = data.get("mark_price") {
                                    println!("   💲 Mark Price: {}", mark_price);
                                }
                            }
                        } else {
                            let mut count = other_count.lock().unwrap();
                            *count += 1;
                            println!("📨 Other subscription #{}: {}", *count, channel);
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(WebSocketError::InvalidMessage(format!(
                    "Failed to parse JSON: {}",
                    e
                ))),
            }
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing message: {}", error);
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
    println!("✅ Connected successfully!");

    // Subscribe to advanced channels
    println!("📡 Subscribing to advanced channels...");

    let channels = vec![
        // Chart data subscriptions for different resolutions
        "chart.trades.BTC-PERPETUAL.1".to_string(), // 1-minute chart data
        "chart.trades.BTC-PERPETUAL.5".to_string(), // 5-minute chart data
        "chart.trades.ETH-PERPETUAL.1".to_string(), // ETH 1-minute chart data
        // User position changes (requires authentication)
        "user.changes.BTC-PERPETUAL.raw".to_string(), // BTC position changes
        "user.changes.ETH-PERPETUAL.raw".to_string(), // ETH position changes
        // Standard subscriptions for comparison
        "ticker.BTC-PERPETUAL".to_string(),
        "ticker.ETH-PERPETUAL".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to all channels"),
        Err(e) => {
            println!("❌ Subscription failed: {}", e);
            println!("💡 Note: User channels require authentication");
        }
    }

    // Start message processing
    println!("🎯 Starting message processing...");
    println!("   - Chart data will show trade aggregations");
    println!("   - User changes will show position updates");
    println!("   - Processing will run for 15 seconds...");

    // Run the processing loop for a limited time
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing (in a real application, you'd handle this more gracefully)
    processing_task.abort();

    // Display final statistics
    let final_chart = *chart_messages.lock().unwrap();
    let final_changes = *user_changes_messages.lock().unwrap();
    let final_other = *other_messages.lock().unwrap();

    println!("\n📊 Final Statistics:");
    println!("   📈 Chart data messages: {}", final_chart);
    println!("   🔄 Position change messages: {}", final_changes);
    println!("   📨 Other subscription messages: {}", final_other);
    println!(
        "   📈 Total messages processed: {}",
        final_chart + final_changes + final_other
    );

    if final_chart == 0 {
        println!("\n💡 Tips for chart data:");
        println!("   - Chart data may be sparse during low activity periods");
        println!("   - Try different resolution values (1, 5, 15, 60, etc.)");
        println!("   - Chart data aggregates trades over the specified interval");
    }

    if final_changes == 0 {
        println!("\n💡 Tips for user changes:");
        println!("   - User changes require valid authentication");
        println!("   - Position changes only occur when you have active positions");
        println!("   - Try placing a small test order to generate position changes");
    }

    println!("\n🎉 Advanced subscriptions example completed!");
    println!("📚 Channel formats implemented:");
    println!("   📊 chart.trades.{{instrument}}.{{resolution}}");
    println!("   🔄 user.changes.{{instrument}}.{{interval}}");

    Ok(())
}
