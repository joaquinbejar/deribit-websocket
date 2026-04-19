//! Cancel-on-Disconnect Example
//!
//! This example demonstrates the cancel-on-disconnect features added in v0.2.0:
//! - `enable_cancel_on_disconnect()` - Enable automatic order cancellation on disconnect
//! - `disable_cancel_on_disconnect()` - Disable automatic order cancellation
//! - `get_cancel_on_disconnect()` - Get current cancel-on-disconnect status
//!
//! These features were added in issue #15.
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
    deribit_websocket::install_default_crypto_provider()
        .map_err(|e| format!("Failed to install crypto provider: {e}"))?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Cancel-on-Disconnect Example");
    tracing::info!("Demonstrating: enable, disable, get cancel-on-disconnect status");

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
    // Authenticate (required for cancel-on-disconnect)
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
    // 1. Get current cancel-on-disconnect status
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔍 Getting current cancel-on-disconnect status...");

    match client.get_cancel_on_disconnect().await {
        Ok(enabled) => {
            tracing::info!("✅ Cancel-on-disconnect status:");
            tracing::info!("   Enabled: {}", enabled);
            if enabled {
                tracing::info!("   📋 Orders WILL be cancelled on disconnect");
            } else {
                tracing::info!("   📋 Orders will NOT be cancelled on disconnect");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get status: {}", e);
        }
    }

    // ==========================================================================
    // 2. Enable cancel-on-disconnect
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🛡️ Enabling cancel-on-disconnect...");

    match client.enable_cancel_on_disconnect().await {
        Ok(result) => {
            tracing::info!("✅ Cancel-on-disconnect enabled: {}", result);
            tracing::info!("   All open orders will be automatically cancelled");
            tracing::info!("   if the WebSocket connection is lost");
        }
        Err(e) => {
            tracing::error!("❌ Failed to enable: {}", e);
        }
    }

    // ==========================================================================
    // 3. Verify the status changed
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔍 Verifying status after enable...");

    match client.get_cancel_on_disconnect().await {
        Ok(enabled) => {
            tracing::info!("   Enabled: {}", enabled);
            if enabled {
                tracing::info!("   ✅ Status correctly shows ENABLED");
            } else {
                tracing::warn!("   ⚠️ Status still shows disabled");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get status: {}", e);
        }
    }

    // ==========================================================================
    // 4. Disable cancel-on-disconnect
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔓 Disabling cancel-on-disconnect...");

    match client.disable_cancel_on_disconnect().await {
        Ok(result) => {
            tracing::info!("✅ Cancel-on-disconnect disabled: {}", result);
            tracing::info!("   Orders will remain active even if connection is lost");
        }
        Err(e) => {
            tracing::error!("❌ Failed to disable: {}", e);
        }
    }

    // ==========================================================================
    // 5. Final status check
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔍 Final status check...");

    match client.get_cancel_on_disconnect().await {
        Ok(enabled) => {
            tracing::info!("   Enabled: {}", enabled);
            if !enabled {
                tracing::info!("   ✅ Status correctly shows DISABLED");
            } else {
                tracing::warn!("   ⚠️ Status still shows enabled");
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to get status: {}", e);
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
    tracing::info!("✅ Cancel-on-disconnect example completed!");
    tracing::info!("");
    tracing::info!("📋 Summary:");
    tracing::info!("   - enable_cancel_on_disconnect() - Enables safety feature");
    tracing::info!("   - disable_cancel_on_disconnect() - Disables safety feature");
    tracing::info!("   - get_cancel_on_disconnect() - Returns current status (bool)");

    Ok(())
}
