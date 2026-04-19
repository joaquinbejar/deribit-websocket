//! Real-time trades subscription example
//!
//! This example demonstrates how to subscribe to real-time trade data
//! and process individual trade events as they occur.

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Starting Real-time Trades Subscription Example");

    // Statistics tracking
    let trade_count = Arc::new(Mutex::new(0u32));
    let total_volume = Arc::new(Mutex::new(0.0f64));
    let trade_count_clone = trade_count.clone();
    let volume_clone = total_volume.clone();

    // Create client configuration
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config)?;

    // Set up message handler for trade data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
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

                        tracing::info!(
                            "💰 Trade #{}: {} {} @ {} (ID: {}, Time: {})",
                            *count,
                            direction,
                            amount,
                            price,
                            trade_id,
                            timestamp
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

                    tracing::info!("💰 Trade #{}: {} {} @ {}", *count, direction, amount, price);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::error!("❌ Error processing trade message: {}", error);
            tracing::error!(
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

    // Subscribe to trades channels
    tracing::info!("📊 Subscribing to real-time trades...");
    let channels = vec![
        "trades.BTC-PERPETUAL.raw".to_string(),
        "trades.ETH-PERPETUAL.raw".to_string(),
        "trades.BTC-29MAR24.raw".to_string(), // Options example
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to trades channels"),
        Err(e) => tracing::error!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for real-time trades (15 seconds)...");
    tracing::info!("   - Each trade will show direction, amount, and price");
    tracing::info!("   - Volume statistics will be tracked");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_trades = *trade_count.lock().unwrap();
    let final_volume = *total_volume.lock().unwrap();

    tracing::info!("\n📊 Trade Statistics:");
    tracing::info!("   💰 Total trades processed: {}", final_trades);
    tracing::info!("   📈 Total volume: {:.4}", final_volume);
    if final_trades > 0 {
        tracing::info!(
            "   📊 Average trade size: {:.4}",
            final_volume / final_trades as f64
        );
    }

    if final_trades == 0 {
        tracing::info!("\n💡 Tips for trade data:");
        tracing::info!("   - Trade data depends on market activity");
        tracing::info!("   - Try during active trading hours");
        tracing::info!("   - BTC-PERPETUAL usually has the most activity");
    }

    tracing::info!("\n🎉 Real-time trades example completed!");
    Ok(())
}
