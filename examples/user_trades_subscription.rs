//! User trades subscription example
//!
//! This example demonstrates how to subscribe to user trade updates
//! and process executed trades in real-time.
//!
//! Note: This requires valid authentication credentials.

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

    tracing::info!("🚀 Starting User Trades Subscription Example");

    // Statistics tracking
    let trade_count = Arc::new(Mutex::new(0u32));
    let total_pnl = Arc::new(Mutex::new(0.0f64));
    let trade_count_clone = trade_count.clone();
    let pnl_clone = total_pnl.clone();

    // Create client configuration
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config)?;

    // Set up message handler for user trade data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("user.trades.")
            {
                let mut count = trade_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!("🔄 User Trade #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract trade execution information
                    if let Some(trade_id) = data.get("trade_id") {
                        tracing::info!("   🆔 Trade ID: {}", trade_id);
                    }
                    if let Some(order_id) = data.get("order_id") {
                        tracing::info!("   📋 Order ID: {}", order_id);
                    }
                    if let Some(instrument) = data.get("instrument_name") {
                        tracing::info!("   🎯 Instrument: {}", instrument);
                    }
                    if let Some(direction) = data.get("direction") {
                        tracing::info!("   ➡️  Direction: {}", direction);
                    }
                    if let Some(amount) = data.get("amount") {
                        tracing::info!("   📏 Amount: {}", amount);
                    }
                    if let Some(price) = data.get("price") {
                        tracing::info!("   💰 Price: {}", price);
                    }
                    if let Some(fee) = data.get("fee") {
                        tracing::info!("   💸 Fee: {}", fee);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        tracing::info!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(profit_loss) = data.get("profit_loss").and_then(|p| p.as_f64()) {
                        let mut pnl = pnl_clone.lock().unwrap();
                        *pnl += profit_loss;
                        tracing::info!("   📈 P&L: {:.4}", profit_loss);
                    }
                    if let Some(mark_price) = data.get("mark_price") {
                        tracing::info!("   🎯 Mark Price: {}", mark_price);
                    }
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing user trade message: {}", error);
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

    // Authenticate
    tracing::info!("🔐 Authenticating...");
    match client
        .authenticate(&config.client_id.unwrap(), &config.client_secret.unwrap())
        .await
    {
        Ok(_) => tracing::info!("✅ Authentication successful"),
        Err(e) => {
            tracing::info!("❌ Authentication failed: {}", e);
            return Err(e.into());
        }
    }

    // Subscribe to user trades channels
    tracing::info!("📊 Subscribing to user trades...");
    let channels = vec![
        "user.trades.any.any.raw".to_string(),       // All user trades
        "user.trades.BTC-PERPETUAL.raw".to_string(), // BTC perpetual trades
        "user.trades.ETH-PERPETUAL.raw".to_string(), // ETH perpetual trades
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to user trades"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for trade executions (20 seconds)...");
    tracing::info!("   - Trade executions will be displayed with P&L");
    tracing::info!("   - Execute trades in another session to see updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_trades = *trade_count.lock().unwrap();
    let final_pnl = *total_pnl.lock().unwrap();

    tracing::info!("\n📊 User Trade Statistics:");
    tracing::info!("   🔄 Total trade executions: {}", final_trades);
    tracing::info!("   📈 Total P&L: {:.4}", final_pnl);

    if final_trades == 0 {
        tracing::info!("\n💡 Tips for user trade updates:");
        tracing::info!("   - User trades only occur when your orders are executed");
        tracing::info!("   - Try placing and executing orders in another session");
        tracing::info!("   - Make sure you're authenticated with valid credentials");
        tracing::info!("   - Trade updates include execution details and P&L calculations");
    }

    tracing::info!("\n🎉 User trades subscription example completed!");
    Ok(())
}
