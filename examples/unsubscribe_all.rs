//! Unsubscribe All Example
//!
//! This example demonstrates the subscription management features added in v0.2.0:
//! - `public_unsubscribe_all()` - Unsubscribe from all public channels
//! - `private_unsubscribe_all()` - Unsubscribe from all private channels (requires auth)
//!
//! These features were added in issue #9.
//!
//! **NOTE**: Private unsubscribe requires authentication.
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

    tracing::info!("🚀 Unsubscribe All Example");
    tracing::info!("Demonstrating: public_unsubscribe_all, private_unsubscribe_all");

    // Load credentials from environment (optional for public, required for private)
    dotenv::dotenv().ok();
    let client_id = env::var("DERIBIT_CLIENT_ID").ok();
    let client_secret = env::var("DERIBIT_CLIENT_SECRET").ok();

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
    // 1. Subscribe to multiple public channels
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📡 Subscribing to multiple public channels...");

    let public_channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "ticker.ETH-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
        "trades.BTC-PERPETUAL.raw".to_string(),
    ];

    match client.subscribe(public_channels.clone()).await {
        Ok(_) => {
            tracing::info!(
                "✅ Subscribed to {} public channels:",
                public_channels.len()
            );
            for channel in &public_channels {
                tracing::info!("   - {}", channel);
            }
        }
        Err(e) => {
            tracing::error!("❌ Subscription failed: {}", e);
        }
    }

    // Show current subscriptions
    let subs = client.get_subscriptions().await;
    tracing::info!("📋 Active subscriptions: {}", subs.len());

    // ==========================================================================
    // 2. Wait briefly to confirm subscriptions work
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("⏳ Waiting 2 seconds for subscriptions to be active...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    tracing::info!("   Subscriptions should now be receiving data");

    // ==========================================================================
    // 3. Unsubscribe from ALL public channels
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🗑️ Unsubscribing from ALL public channels...");

    match client.public_unsubscribe_all().await {
        Ok(response) => {
            tracing::info!("✅ public_unsubscribe_all() successful!");
            tracing::info!("   Response: {:?}", response);
        }
        Err(e) => {
            tracing::error!("❌ public_unsubscribe_all failed: {}", e);
        }
    }

    // Verify subscriptions are cleared
    let subs_after = client.get_subscriptions().await;
    tracing::info!(
        "📋 Active subscriptions after unsubscribe_all: {}",
        subs_after.len()
    );

    // ==========================================================================
    // 4. Private unsubscribe_all (if authenticated)
    // ==========================================================================
    if let (Some(id), Some(secret)) = (client_id, client_secret) {
        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        tracing::info!("🔐 Authenticating for private channels...");

        match client.authenticate(&id, &secret).await {
            Ok(auth_response) => {
                tracing::info!("✅ Authentication successful!");
                tracing::info!("   Token type: {}", auth_response.token_type);

                // Subscribe to private channels
                tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                tracing::info!("📡 Subscribing to private channels...");

                let private_channels = vec![
                    "user.orders.BTC-PERPETUAL.raw".to_string(),
                    "user.trades.BTC-PERPETUAL.raw".to_string(),
                ];

                match client.subscribe(private_channels.clone()).await {
                    Ok(_) => {
                        tracing::info!(
                            "✅ Subscribed to {} private channels",
                            private_channels.len()
                        );
                    }
                    Err(e) => {
                        tracing::error!("❌ Private subscription failed: {}", e);
                    }
                }

                // Unsubscribe from all private channels
                tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                tracing::info!("🗑️ Unsubscribing from ALL private channels...");

                match client.private_unsubscribe_all().await {
                    Ok(response) => {
                        tracing::info!("✅ private_unsubscribe_all() successful!");
                        tracing::info!("   Response: {:?}", response);
                    }
                    Err(e) => {
                        tracing::error!("❌ private_unsubscribe_all failed: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("❌ Authentication failed: {}", e);
            }
        }
    } else {
        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        tracing::info!("ℹ️ Skipping private_unsubscribe_all (no credentials)");
        tracing::info!("   Set DERIBIT_CLIENT_ID and DERIBIT_CLIENT_SECRET to test");
    }

    // ==========================================================================
    // Cleanup
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔌 Disconnecting...");
    client.disconnect().await?;
    tracing::info!("✅ Disconnected successfully");

    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("✅ Unsubscribe all example completed!");
    tracing::info!("");
    tracing::info!("📋 Summary:");
    tracing::info!("   - public_unsubscribe_all() - Clears all public subscriptions");
    tracing::info!("   - private_unsubscribe_all() - Clears all private subscriptions");
    tracing::info!("   Both methods are useful for cleanup or resetting subscriptions");

    Ok(())
}
