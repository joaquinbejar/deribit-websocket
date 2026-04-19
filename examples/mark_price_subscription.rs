//! Mark price subscription example
//!
//! This example demonstrates how to subscribe to mark price updates
//! for options instruments.

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

    tracing::info!("🚀 Starting Mark Price Subscription Example");

    // Statistics tracking
    let mark_price_updates = Arc::new(Mutex::new(0u32));
    let mark_count_clone = mark_price_updates.clone();

    // Create client configuration
    setup_logger();
    let mut client = DeribitWebSocketClient::default();

    // Set up message handler for mark price data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("markprice.options.")
            {
                let mut count = mark_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!("🎯 Mark Price Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract mark price information
                    if let Some(mark_price) = data.get("mark_price") {
                        tracing::info!("   💰 Mark Price: {}", mark_price);
                    }
                    if let Some(underlying_price) = data.get("underlying_price") {
                        tracing::info!("   📊 Underlying Price: {}", underlying_price);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        tracing::info!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(iv) = data.get("iv") {
                        tracing::info!("   📈 Implied Volatility: {}", iv);
                    }
                    if let Some(delta) = data.get("delta") {
                        tracing::info!("   🔺 Delta: {}", delta);
                    }
                    if let Some(gamma) = data.get("gamma") {
                        tracing::info!("   📐 Gamma: {}", gamma);
                    }
                    if let Some(theta) = data.get("theta") {
                        tracing::info!("   ⏳ Theta: {}", theta);
                    }
                    if let Some(vega) = data.get("vega") {
                        tracing::info!("   📊 Vega: {}", vega);
                    }

                    // Extract instrument from channel name
                    let instrument = channel
                        .strip_prefix("markprice.options.")
                        .unwrap_or("unknown");
                    tracing::info!("   🎯 Instrument: {}", instrument);
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing mark price message: {}", error);
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

    // Subscribe to mark price channels for options
    tracing::info!("📊 Subscribing to mark prices...");
    let channels = vec![
        "markprice.options.BTC-29MAR24-50000-C".to_string(),
        "markprice.options.BTC-29MAR24-60000-P".to_string(),
        "markprice.options.ETH-29MAR24-3000-C".to_string(),
        "markprice.options.ETH-29MAR24-4000-P".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to mark prices"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for mark price updates (15 seconds)...");
    tracing::info!("   - Mark prices include Greeks (Delta, Gamma, Theta, Vega)");
    tracing::info!("   - Used for options valuation and risk management");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 15 seconds
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *mark_price_updates.lock().unwrap();

    tracing::info!("\n📊 Mark Price Statistics:");
    tracing::info!("   🎯 Total mark price updates: {}", final_updates);

    if final_updates == 0 {
        tracing::info!("\n💡 Tips for mark price updates:");
        tracing::info!("   - Mark prices are calculated for options instruments");
        tracing::info!("   - Updates depend on underlying price movements and volatility");
        tracing::info!("   - Try subscribing to active options with upcoming expirations");
        tracing::info!("   - Check available options instruments first");
    }

    tracing::info!("\n🎉 Mark price subscription example completed!");
    Ok(())
}
