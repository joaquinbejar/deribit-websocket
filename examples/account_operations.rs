//! Account Operations Example
//!
//! This example demonstrates the account management features added in v0.2.0:
//! - `get_positions()` - Get all positions for a currency
//! - `get_account_summary()` - Get account summary with balance info
//! - `get_order_state()` - Get state of a specific order
//! - `get_order_history_by_currency()` - Get order history
//!
//! These features were added in issue #6.
//!
//! **NOTE**: This example requires authentication with valid API credentials.
//! Set the following environment variables:
//! - DERIBIT_CLIENT_ID
//! - DERIBIT_CLIENT_SECRET

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Account Operations Example");
    tracing::info!("Demonstrating: get_positions, get_account_summary, get_order_state");

    // Load credentials from environment
    dotenv::dotenv().ok();
    let client_id = env::var("DERIBIT_CLIENT_ID").expect("DERIBIT_CLIENT_ID must be set");
    let client_secret =
        env::var("DERIBIT_CLIENT_SECRET").expect("DERIBIT_CLIENT_SECRET must be set");

    // Create client configuration for testnet
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(std::time::Duration::from_secs(30))
        .with_max_reconnect_attempts(3);

    // Create the WebSocket client
    let client = DeribitWebSocketClient::new(&config)?;
    tracing::info!("✅ Client created successfully");

    // Connect to the server
    tracing::info!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    tracing::info!("✅ Connected to Deribit WebSocket");

    // ==========================================================================
    // Authenticate (required for account operations)
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔐 Authenticating...");

    match client.authenticate(&client_id, &client_secret).await {
        Ok(auth_response) => {
            tracing::info!("✅ Authentication successful!");
            tracing::info!("   Token type: {}", auth_response.token_type);
            tracing::info!("   Expires in: {} seconds", auth_response.expires_in);
        }
        Err(e) => {
            tracing::error!("❌ Authentication failed: {}", e);
            client.disconnect().await?;
            return Err(e.into());
        }
    }

    // ==========================================================================
    // 1. Get Account Summary
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("💰 Getting account summary for BTC...");

    match client.get_account_summary("BTC", Some(true)).await {
        Ok(summary) => {
            tracing::info!("✅ Account summary received!");
            tracing::info!("   Currency: {:?}", summary.currency);
            tracing::info!("   Balance: {:?} BTC", summary.balance);
            tracing::info!("   Equity: {:?} BTC", summary.equity);
            tracing::info!("   Available Funds: {:?} BTC", summary.available_funds);
            tracing::info!("   Margin Balance: {:?} BTC", summary.margin_balance);
            if let Some(pnl) = summary.total_pl {
                tracing::info!("   Total P&L: {} BTC", pnl);
            }
            if let Some(initial_margin) = summary.initial_margin {
                tracing::info!("   Initial Margin: {} BTC", initial_margin);
            }
            if let Some(maintenance_margin) = summary.maintenance_margin {
                tracing::info!("   Maintenance Margin: {} BTC", maintenance_margin);
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get account summary: {}", e);
        }
    }

    // Also get ETH account summary
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("💰 Getting account summary for ETH...");

    match client.get_account_summary("ETH", Some(true)).await {
        Ok(summary) => {
            tracing::info!("✅ Account summary received!");
            tracing::info!("   Currency: {:?}", summary.currency);
            tracing::info!("   Balance: {:?} ETH", summary.balance);
            tracing::info!("   Equity: {:?} ETH", summary.equity);
            tracing::info!("   Available Funds: {:?} ETH", summary.available_funds);
        }
        Err(e) => {
            tracing::error!("❌ Failed to get ETH account summary: {}", e);
        }
    }

    // ==========================================================================
    // 2. Get Positions
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📊 Getting positions for BTC...");

    match client.get_positions(Some("BTC"), None).await {
        Ok(positions) => {
            tracing::info!("✅ Positions received!");
            tracing::info!("   Total positions: {}", positions.len());

            for (i, position) in positions.iter().enumerate() {
                tracing::info!("   ────────────────────────────────────────────────────");
                tracing::info!("   Position #{}", i + 1);
                tracing::info!("      Instrument: {}", position.instrument_name);
                tracing::info!("      Direction: {:?}", position.direction);
                tracing::info!("      Size: {}", position.size);
                tracing::info!("      Avg Price: {}", position.average_price);
                if let Some(pnl) = position.floating_profit_loss {
                    tracing::info!("      Floating P&L: {}", pnl);
                }
                if let Some(mark_price) = position.mark_price {
                    tracing::info!("      Mark Price: {}", mark_price);
                }
            }

            if positions.is_empty() {
                tracing::info!("   No open positions for BTC");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get positions: {}", e);
        }
    }

    // ==========================================================================
    // 3. Get Order History
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📜 Getting order history for BTC...");

    match client
        .get_order_history_by_currency("BTC", None, Some(10))
        .await
    {
        Ok(orders) => {
            tracing::info!("✅ Order history received!");
            tracing::info!("   Orders found: {}", orders.len());

            for (i, order) in orders.iter().take(5).enumerate() {
                tracing::info!("   ────────────────────────────────────────────────────");
                tracing::info!("   Order #{}", i + 1);
                tracing::info!("      Order ID: {}", order.order_id);
                tracing::info!("      Instrument: {}", order.instrument_name);
                tracing::info!("      Direction: {:?}", order.direction);
                tracing::info!("      Amount: {}", order.amount);
                tracing::info!("      State: {}", order.order_state);
                if let Some(price) = order.price {
                    tracing::info!("      Price: {}", price);
                }
            }

            if orders.is_empty() {
                tracing::info!("   No order history for BTC");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get order history: {}", e);
        }
    }

    // ==========================================================================
    // Cleanup
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔌 Disconnecting...");
    client.disconnect().await?;
    tracing::info!("✅ Disconnected successfully");

    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("✅ Account operations example completed!");

    Ok(())
}
