//! User orders subscription example
//!
//! This example demonstrates how to subscribe to user order updates
//! and process order state changes in real-time.
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

    println!("🚀 Starting User Orders Subscription Example");

    // Load environment variables
    dotenv::dotenv().ok();
    let client_id = std::env::var("DERIBIT_CLIENT_ID")
        .map_err(|_| "DERIBIT_CLIENT_ID environment variable not set")?;
    let client_secret = std::env::var("DERIBIT_CLIENT_SECRET")
        .map_err(|_| "DERIBIT_CLIENT_SECRET environment variable not set")?;

    // Statistics tracking
    let order_updates = Arc::new(Mutex::new(0u32));
    let order_count_clone = order_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for user order data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("user.orders.")
            {
                let mut count = order_count_clone.lock().unwrap();
                *count += 1;

                println!("📋 Order Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract order information
                    if let Some(order_id) = data.get("order_id") {
                        println!("   🆔 Order ID: {}", order_id);
                    }
                    if let Some(order_state) = data.get("order_state") {
                        println!("   📊 State: {}", order_state);
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
                    if let Some(filled_amount) = data.get("filled_amount") {
                        println!("   ✅ Filled: {}", filled_amount);
                    }
                    if let Some(creation_timestamp) = data.get("creation_timestamp") {
                        println!("   ⏰ Created: {}", creation_timestamp);
                    }
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing order message: {}", error);
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

    // Subscribe to user orders channels
    println!("📊 Subscribing to user orders...");
    let channels = vec![
        "user.orders.any.any.raw".to_string(),       // All orders
        "user.orders.BTC-PERPETUAL.raw".to_string(), // BTC perpetual orders
        "user.orders.ETH-PERPETUAL.raw".to_string(), // ETH perpetual orders
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to user orders"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for order updates (20 seconds)...");
    println!("   - Order state changes will be displayed");
    println!("   - Place/modify/cancel orders in another session to see updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_orders = *order_updates.lock().unwrap();

    println!("\n📊 Order Update Statistics:");
    println!("   📋 Total order updates: {}", final_orders);

    if final_orders == 0 {
        println!("\n💡 Tips for order updates:");
        println!("   - Order updates only occur when you have active orders");
        println!("   - Try placing a test order in another session");
        println!("   - Order updates include: placed, filled, cancelled, modified");
        println!("   - Make sure you're authenticated with valid credentials");
    }

    println!("\n🎉 User orders subscription example completed!");
    Ok(())
}
