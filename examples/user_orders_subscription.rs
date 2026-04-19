//! User orders subscription example
//!
//! This example demonstrates how to subscribe to user order updates
//! and process order state changes in real-time.
//!
//! Note: This requires valid authentication credentials.

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

    tracing::info!("🚀 Starting User Orders Subscription Example");

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
    let config = WebSocketConfig::default();
    let mut client = DeribitWebSocketClient::new(&config)?;

    // Set up message handler for user order data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("user.orders.")
            {
                let mut count = order_count_clone.lock().unwrap();
                *count += 1;

                tracing::info!("📋 Order Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract order information
                    if let Some(order_id) = data.get("order_id") {
                        tracing::info!("   🆔 Order ID: {}", order_id);
                    }
                    if let Some(order_state) = data.get("order_state") {
                        tracing::info!("   📊 State: {}", order_state);
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
                    if let Some(filled_amount) = data.get("filled_amount") {
                        tracing::info!("   ✅ Filled: {}", filled_amount);
                    }
                    if let Some(creation_timestamp) = data.get("creation_timestamp") {
                        tracing::info!("   ⏰ Created: {}", creation_timestamp);
                    }
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing order message: {}", error);
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
    match client.authenticate(&client_id, &client_secret).await {
        Ok(_) => tracing::info!("✅ Authentication successful"),
        Err(e) => {
            tracing::info!("❌ Authentication failed: {}", e);
            return Err(e.into());
        }
    }

    // Subscribe to user orders channels
    tracing::info!("📊 Subscribing to user orders...");
    let channels = vec![
        "user.orders.any.any.raw".to_string(),       // All orders
        "user.orders.BTC-PERPETUAL.raw".to_string(), // BTC perpetual orders
        "user.orders.ETH-PERPETUAL.raw".to_string(), // ETH perpetual orders
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to user orders"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for order updates (20 seconds)...");
    tracing::info!("   - Order state changes will be displayed");
    tracing::info!("   - Place/modify/cancel orders in another session to see updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_orders = *order_updates.lock().unwrap();

    tracing::info!("\n📊 Order Update Statistics:");
    tracing::info!("   📋 Total order updates: {}", final_orders);

    if final_orders == 0 {
        tracing::info!("\n💡 Tips for order updates:");
        tracing::info!("   - Order updates only occur when you have active orders");
        tracing::info!("   - Try placing a test order in another session");
        tracing::info!("   - Order updates include: placed, filled, cancelled, modified");
        tracing::info!("   - Make sure you're authenticated with valid credentials");
    }

    tracing::info!("\n🎉 User orders subscription example completed!");
    Ok(())
}
