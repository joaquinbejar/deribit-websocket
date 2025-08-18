//! User portfolio subscription example
//!
//! This example demonstrates how to subscribe to portfolio updates
//! and monitor account balance and position changes.
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

    println!("🚀 Starting User Portfolio Subscription Example");

    // Load environment variables
    dotenv::dotenv().ok();
    let client_id = std::env::var("DERIBIT_CLIENT_ID")
        .map_err(|_| "DERIBIT_CLIENT_ID environment variable not set")?;
    let client_secret = std::env::var("DERIBIT_CLIENT_SECRET")
        .map_err(|_| "DERIBIT_CLIENT_SECRET environment variable not set")?;

    // Statistics tracking
    let portfolio_updates = Arc::new(Mutex::new(0u32));
    let portfolio_count_clone = portfolio_updates.clone();

    // Create client configuration
    let config = WebSocketConfig::testnet();
    let mut client = DeribitWebSocketClient::new(config)?;

    // Set up message handler for portfolio data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                && channel.starts_with("user.portfolio.")
            {
                let mut count = portfolio_count_clone.lock().unwrap();
                *count += 1;

                println!("💼 Portfolio Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract portfolio information
                    if let Some(currency) = data.get("currency") {
                        println!("   💱 Currency: {}", currency);
                    }
                    if let Some(equity) = data.get("equity") {
                        println!("   💰 Equity: {}", equity);
                    }
                    if let Some(balance) = data.get("balance") {
                        println!("   💳 Balance: {}", balance);
                    }
                    if let Some(available_funds) = data.get("available_funds") {
                        println!("   💵 Available Funds: {}", available_funds);
                    }
                    if let Some(margin_balance) = data.get("margin_balance") {
                        println!("   📊 Margin Balance: {}", margin_balance);
                    }
                    if let Some(initial_margin) = data.get("initial_margin") {
                        println!("   🔒 Initial Margin: {}", initial_margin);
                    }
                    if let Some(maintenance_margin) = data.get("maintenance_margin") {
                        println!("   ⚖️  Maintenance Margin: {}", maintenance_margin);
                    }
                    if let Some(total_pl) = data.get("total_pl") {
                        println!("   📈 Total P&L: {}", total_pl);
                    }
                    if let Some(session_rpl) = data.get("session_rpl") {
                        println!("   📊 Session RPL: {}", session_rpl);
                    }
                    if let Some(session_upl) = data.get("session_upl") {
                        println!("   📈 Session UPL: {}", session_upl);
                    }
                    if let Some(delta_total) = data.get("delta_total") {
                        println!("   🔺 Delta Total: {}", delta_total);
                    }
                }
                println!(); // Empty line for readability
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            println!("❌ Error processing portfolio message: {}", error);
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

    // Subscribe to portfolio channels
    println!("📊 Subscribing to portfolio updates...");
    let channels = vec![
        "user.portfolio.any".to_string(), // All currencies
        "user.portfolio.BTC".to_string(), // BTC portfolio
        "user.portfolio.ETH".to_string(), // ETH portfolio
    ];

    match client.subscribe(channels).await {
        Ok(_) => println!("✅ Successfully subscribed to portfolio updates"),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    println!("🎯 Listening for portfolio updates (20 seconds)...");
    println!("   - Portfolio changes will be displayed with balance details");
    println!("   - Trade or modify positions to see portfolio updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *portfolio_updates.lock().unwrap();

    println!("\n📊 Portfolio Update Statistics:");
    println!("   💼 Total portfolio updates: {}", final_updates);

    if final_updates == 0 {
        println!("\n💡 Tips for portfolio updates:");
        println!("   - Portfolio updates occur when balance or positions change");
        println!("   - Try executing trades to trigger portfolio changes");
        println!("   - Make sure you're authenticated with valid credentials");
        println!("   - Updates include equity, balance, margin, and P&L changes");
    }

    println!("\n🎉 User portfolio subscription example completed!");
    Ok(())
}
