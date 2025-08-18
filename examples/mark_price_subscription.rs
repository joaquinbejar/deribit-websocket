//! Mark price subscription example
//!
//! This example demonstrates how to subscribe to mark price updates
//! for options instruments.

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

    println!("🚀 Starting Mark Price Subscription Example");

    // Statistics tracking
    let mark_price_updates = Arc::new(Mutex::new(0u32));
    let mark_count_clone = mark_price_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for mark price data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("markprice.options.")
            {
                let mut count = mark_count_clone.lock().unwrap();
                *count += 1;

                println!("🎯 Mark Price Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract mark price information
                    if let Some(mark_price) = data.get("mark_price") {
                        println!("   💰 Mark Price: {}", mark_price);
                    }
                    if let Some(underlying_price) = data.get("underlying_price") {
                        println!("   📊 Underlying Price: {}", underlying_price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        println!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(iv) = data.get("iv") {
                        println!("   📈 Implied Volatility: {}", iv);
                    }
                    if let Some(delta) = data.get("delta") {
                        println!("   🔺 Delta: {}", delta);
                    }
                    if let Some(gamma) = data.get("gamma") {
                        println!("   📐 Gamma: {}", gamma);
                    }
                    if let Some(theta) = data.get("theta") {
                        println!("   ⏳ Theta: {}", theta);
                    }
                    if let Some(vega) = data.get("vega") {
                        println!("   📊 Vega: {}", vega);
                    }

                    // Extract instrument from channel name
                    let instrument = channel
                        .strip_prefix("markprice.options.")
                        .unwrap_or("unknown");
                    println!("   🎯 Instrument: {}", instrument);
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing mark price message: {}", error);
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

    // Subscribe to mark price channels for options
    println!("📊 Subscribing to mark prices...");
    let channels = vec![
        "markprice.options.BTC-29MAR24-50000-C".to_string(),
        "markprice.options.BTC-29MAR24-60000-P".to_string(),
        "markprice.options.ETH-29MAR24-3000-C".to_string(),
        "markprice.options.ETH-29MAR24-4000-P".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to mark prices"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for mark price updates (15 seconds)...");
    println!("   - Mark prices include Greeks (Delta, Gamma, Theta, Vega)");
    println!("   - Used for options valuation and risk management");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *mark_price_updates.lock().unwrap();

    println!("\n📊 Mark Price Statistics:");
    println!("   🎯 Total mark price updates: {}", final_updates);

    if final_updates == 0 {
        println!("\n💡 Tips for mark price updates:");
        println!("   - Mark prices are calculated for options instruments");
        println!("   - Updates depend on underlying price movements and volatility");
        println!("   - Try subscribing to active options with upcoming expirations");
        println!("   - Check available options instruments first");
    }

    println!("\n🎉 Mark price subscription example completed!");
    Ok(())
}
