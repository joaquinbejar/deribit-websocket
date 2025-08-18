//! User portfolio subscription example
//!
//! This example demonstrates how to subscribe to portfolio updates
//! and monitor account balance and position changes.
//!
//! Note: This requires valid authentication credentials.

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Starting User Portfolio Subscription Example");

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

                tracing::info!("💼 Portfolio Update #{}: Channel: {}", *count, channel);

                if let Some(data) = params.get("data") {
                    // Extract portfolio information
                    if let Some(currency) = data.get("currency") {
                        tracing::info!("   💱 Currency: {}", currency);
                    }
                    if let Some(equity) = data.get("equity") {
                        tracing::info!("   💰 Equity: {}", equity);
                    }
                    if let Some(balance) = data.get("balance") {
                        tracing::info!("   💳 Balance: {}", balance);
                    }
                    if let Some(available_funds) = data.get("available_funds") {
                        tracing::info!("   💵 Available Funds: {}", available_funds);
                    }
                    if let Some(margin_balance) = data.get("margin_balance") {
                        tracing::info!("   📊 Margin Balance: {}", margin_balance);
                    }
                    if let Some(initial_margin) = data.get("initial_margin") {
                        tracing::info!("   🔒 Initial Margin: {}", initial_margin);
                    }
                    if let Some(maintenance_margin) = data.get("maintenance_margin") {
                        tracing::info!("   ⚖️  Maintenance Margin: {}", maintenance_margin);
                    }
                    if let Some(total_pl) = data.get("total_pl") {
                        tracing::info!("   📈 Total P&L: {}", total_pl);
                    }
                    if let Some(session_rpl) = data.get("session_rpl") {
                        tracing::info!("   📊 Session RPL: {}", session_rpl);
                    }
                    if let Some(session_upl) = data.get("session_upl") {
                        tracing::info!("   📈 Session UPL: {}", session_upl);
                    }
                    if let Some(delta_total) = data.get("delta_total") {
                        tracing::info!("   🔺 Delta Total: {}", delta_total);
                    }
                }
            }
            Ok(())
        },
        |message: &str, error: &WebSocketError| {
            tracing::info!("❌ Error processing portfolio message: {}", error);
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

    // Subscribe to portfolio channels
    tracing::info!("📊 Subscribing to portfolio updates...");
    let channels = vec![
        "user.portfolio.any".to_string(), // All currencies
        "user.portfolio.BTC".to_string(), // BTC portfolio
        "user.portfolio.ETH".to_string(), // ETH portfolio
    ];

    match client.subscribe(channels).await {
        Ok(_) => tracing::info!("✅ Successfully subscribed to portfolio updates"),
        Err(e) => tracing::info!("❌ Subscription failed: {}", e),
    }

    // Start message processing
    tracing::info!("🎯 Listening for portfolio updates (20 seconds)...");
    tracing::info!("   - Portfolio changes will be displayed with balance details");
    tracing::info!("   - Trade or modify positions to see portfolio updates");

    // Run the processing loop
    let processing_task = tokio::spawn(async move { client.start_message_processing_loop().await });

    // Wait for 20 seconds
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    // Stop processing
    processing_task.abort();

    // Display final statistics
    let final_updates = *portfolio_updates.lock().unwrap();

    tracing::info!("\n📊 Portfolio Update Statistics:");
    tracing::info!("   💼 Total portfolio updates: {}", final_updates);

    if final_updates == 0 {
        tracing::info!("\n💡 Tips for portfolio updates:");
        tracing::info!("   - Portfolio updates occur when balance or positions change");
        tracing::info!("   - Try executing trades to trigger portfolio changes");
        tracing::info!("   - Make sure you're authenticated with valid credentials");
        tracing::info!("   - Updates include equity, balance, margin, and P&L changes");
    }

    tracing::info!("\n🎉 User portfolio subscription example completed!");
    Ok(())
}
