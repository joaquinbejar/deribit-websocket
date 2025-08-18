//! User trades subscription example
//!
//! This example demonstrates how to subscribe to user trade updates
//! and process executed trades in real-time.
//!
//! Note: This requires valid authentication credentials.

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

    println!("🚀 Starting User Trades Subscription Example");

    // Load environment variables
    dotenv::dotenv().ok();
    let client_id = std::env::var("DERIBIT_CLIENT_ID")
        .map_err(|_| "DERIBIT_CLIENT_ID environment variable not set")?;
    let client_secret = std::env::var("DERIBIT_CLIENT_SECRET")
        .map_err(|_| "DERIBIT_CLIENT_SECRET environment variable not set")?;

    // Statistics tracking
    let trade_count = Arc::new(Mutex::new(0u32));
    let total_pnl = Arc::new(Mutex::new(0.0f64));
    let trade_count_clone = trade_count.clone();
    let pnl_clone = total_pnl.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for user trade data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("user.trades.")
            {
                let mut count = trade_count_clone.lock().unwrap();
                *count += 1;

                println!("🔄 User Trade #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract trade execution information
                    if let Some(trade_id) = data.get("trade_id") {
                        println!("   🆔 Trade ID: {}", trade_id);
                    }
                    if let Some(order_id) = data.get("order_id") {
                        println!("   📋 Order ID: {}", order_id);
                    }
                    if let Some(instrument) = data.get("instrument_name") {
                        println!("   🎯 Instrument: {}", instrument);
                    }
                    if let Some(direction) = data.get("direction") {
                        println!("   ➡️  Direction: {}", direction);
                    }
                    if let Some(amount) = data.get("amount") {
                        println!("   📏 Amount: {}", amount);
                    }
                    if let Some(price) = data.get("price") {
                        println!("   💰 Price: {}", price);
                    }
                    if let Some(fee) = data.get("fee") {
                        println!("   💸 Fee: {}", fee);
                    }
                    if let Some(timestamp) = data.get("timestamp") {
                        println!("   ⏰ Timestamp: {}", timestamp);
                    }
                    if let Some(profit_loss) = data.get("profit_loss").and_then(|p| p.as_f64()) {
                        let mut pnl = pnl_clone.lock().unwrap();
                        *pnl += profit_loss;
                        println!("   📈 P&L: {:.4}", profit_loss);
                    }
                    if let Some(mark_price) = data.get("mark_price") {
                        println!("   🎯 Mark Price: {}", mark_price);
                    }
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing user trade message: {}", error);
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

    // Authenticate
    println!("🔐 Authenticating...");
    match client.authenticate(&client_id, &client_secret).await {
        Ok(_) => println!("✅ Authentication successful"),
        Err(e) => {
            println!("❌ Authentication failed: {}", e);
            return Err(e.into());
        }
    }

    // Subscribe to user trades channels
    println!("📊 Subscribing to user trades...");
    let channels = vec![
        "user.trades.any.any.raw".to_string(),       // All user trades
        "user.trades.BTC-PERPETUAL.raw".to_string(), // BTC perpetual trades
        "user.trades.ETH-PERPETUAL.raw".to_string(), // ETH perpetual trades
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to user trades"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for trade executions (20 seconds)...");
    println!("   - Trade executions will be displayed with P&L");
    println!("   - Execute trades in another session to see updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_trades = *trade_count.lock().unwrap();
    let final_pnl = *total_pnl.lock().unwrap();

    println!("\n📊 User Trade Statistics:");
    println!("   🔄 Total trade executions: {}", final_trades);
    println!("   📈 Total P&L: {:.4}", final_pnl);

    if final_trades == 0 {
        println!("\n💡 Tips for user trade updates:");
        println!("   - User trades only occur when your orders are executed");
        println!("   - Try placing and executing orders in another session");
        println!("   - Make sure you're authenticated with valid credentials");
        println!("   - Trade updates include execution details and P&L calculations");
    }

    println!("\n🎉 User trades subscription example completed!");
    Ok(())
}
