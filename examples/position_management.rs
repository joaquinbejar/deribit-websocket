//! Position Management Example
//!
//! This example demonstrates the position management features added in v0.2.0:
//! - `close_position()` - Close an existing position
//! - `move_positions()` - Move positions between subaccounts
//!
//! These features were added in issue #8.
//!
//! **NOTE**: This example requires authentication with valid API credentials.
//! Set the following environment variables:
//! - DERIBIT_CLIENT_ID
//! - DERIBIT_CLIENT_SECRET
//!
//! **WARNING**: This example may execute real trades on your account.
//! Use with caution on testnet only!

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;
use std::env;

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

    tracing::info!("🚀 Position Management Example");
    tracing::info!("Demonstrating: close_position, move_positions");
    tracing::info!("⚠️  WARNING: This example may execute real trades!");

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
    // Authenticate (required for position management)
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
    // 1. Check current positions before any operations
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📊 Checking current BTC positions...");

    match client.get_positions(Some("BTC"), None).await {
        Ok(positions) => {
            tracing::info!("   Found {} position(s)", positions.len());

            for position in &positions {
                if position.size != 0.0 {
                    tracing::info!("   ────────────────────────────────────────────────────");
                    tracing::info!("   Instrument: {}", position.instrument_name);
                    tracing::info!("   Direction: {:?}", position.direction);
                    tracing::info!("   Size: {}", position.size);
                    tracing::info!("   Avg Price: {}", position.average_price);
                }
            }

            // ==========================================================================
            // 2. Demonstrate close_position (only if there's a position to close)
            // ==========================================================================
            let open_positions: Vec<_> = positions.iter().filter(|p| p.size != 0.0).collect();

            if !open_positions.is_empty() {
                let position_to_close = &open_positions[0];
                tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                tracing::info!(
                    "📉 Closing position for {}...",
                    position_to_close.instrument_name
                );
                tracing::info!("   Current size: {}", position_to_close.size);

                // Note: In a real scenario, you would use:
                // client.close_position(&position_to_close.instrument_name, "market").await

                tracing::info!("   ⚠️ Skipping actual close to avoid unwanted trades");
                tracing::info!("   To close a position, use:");
                tracing::info!("      client.close_position(\"BTC-PERPETUAL\", \"market\").await");
            } else {
                tracing::info!("   No open positions to close");
                tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                tracing::info!("📉 close_position() usage example:");
                tracing::info!("   // Close with market order");
                tracing::info!("   client.close_position(\"BTC-PERPETUAL\", \"market\").await");
                tracing::info!("   ");
                tracing::info!("   // Close with limit order");
                tracing::info!("   client.close_position(\"BTC-PERPETUAL\", \"limit\").await");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get positions: {}", e);
        }
    }

    // ==========================================================================
    // 3. Demonstrate move_positions (requires subaccounts)
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔄 move_positions() usage example:");
    tracing::info!("   // Move positions between subaccounts");
    tracing::info!("   let trades = vec![");
    tracing::info!("       MovePositionTrade {{");
    tracing::info!("           instrument_name: \"BTC-PERPETUAL\".to_string(),");
    tracing::info!("           amount: 100.0,");
    tracing::info!("           price: 50000.0,");
    tracing::info!("       }}");
    tracing::info!("   ];");
    tracing::info!("   client.move_positions(");
    tracing::info!("       \"BTC\",              // currency");
    tracing::info!("       123,                 // source_uid");
    tracing::info!("       456,                 // target_uid");
    tracing::info!("       trades              // positions to move");
    tracing::info!("   ).await");

    tracing::info!("   ⚠️ Skipping actual move_positions to avoid unwanted transfers");

    // ==========================================================================
    // Cleanup
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔌 Disconnecting...");
    client.disconnect().await?;
    tracing::info!("✅ Disconnected successfully");

    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("✅ Position management example completed!");

    Ok(())
}
