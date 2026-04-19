//! Session Management Example
//!
//! This example demonstrates the session management features added in v0.2.0:
//! - `hello()` - Client identification with typed HelloResponse
//! - `test_connection()` - Connection test with typed TestResponse
//! - `get_time()` - Server time as u64 timestamp
//! - `set_heartbeat()` - Enable heartbeat with interval
//! - `disable_heartbeat()` - Disable heartbeat
//!
//! These features were added in issues #14 and #16.

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;

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

    tracing::info!("🚀 Session Management Example");
    tracing::info!("Demonstrating: hello, test_connection, get_time, heartbeat");

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
    // 1. Test connection with typed TestResponse
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📋 Testing connection (typed TestResponse)...");

    match client.test_connection().await {
        Ok(test_response) => {
            // Now returns TestResponse with version field directly
            tracing::info!("✅ Connection test successful!");
            tracing::info!("   API Version: {}", test_response.version);
        }
        Err(e) => {
            tracing::error!("❌ Connection test failed: {}", e);
        }
    }

    // ==========================================================================
    // 2. Get server time with typed u64 response
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("⏰ Getting server time (typed u64)...");

    match client.get_time().await {
        Ok(timestamp) => {
            // Now returns u64 timestamp directly (milliseconds since epoch)
            tracing::info!("✅ Server time received!");
            tracing::info!("   Timestamp: {} ms", timestamp);

            // Convert to human-readable format
            let secs = timestamp / 1000;
            let datetime = chrono_lite_format(secs);
            tracing::info!("   Date/Time: {}", datetime);
        }
        Err(e) => {
            tracing::error!("❌ Failed to get server time: {}", e);
        }
    }

    // ==========================================================================
    // 3. Client identification with typed HelloResponse
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("👋 Sending client hello (typed HelloResponse)...");

    match client.hello("deribit-websocket-example", "0.2.0").await {
        Ok(hello_response) => {
            // Now returns HelloResponse with version field directly
            tracing::info!("✅ Hello successful!");
            tracing::info!("   Server API Version: {}", hello_response.version);
        }
        Err(e) => {
            tracing::error!("❌ Hello failed: {}", e);
        }
    }

    // ==========================================================================
    // 4. Enable heartbeat
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("💓 Enabling heartbeat with 30 second interval...");

    match client.set_heartbeat(30).await {
        Ok(result) => {
            tracing::info!("✅ Heartbeat enabled: {}", result);
            tracing::info!("   Server will send heartbeat every 30 seconds");
        }
        Err(e) => {
            tracing::error!("❌ Failed to enable heartbeat: {}", e);
        }
    }

    // Wait a bit to demonstrate heartbeat is working
    tracing::info!("⏳ Waiting 5 seconds to verify heartbeat...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // ==========================================================================
    // 5. Disable heartbeat
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("💔 Disabling heartbeat...");

    match client.disable_heartbeat().await {
        Ok(result) => {
            tracing::info!("✅ Heartbeat disabled: {}", result);
        }
        Err(e) => {
            tracing::error!("❌ Failed to disable heartbeat: {}", e);
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
    tracing::info!("✅ Session management example completed!");

    Ok(())
}

/// Simple timestamp to date string conversion (no external chrono dependency)
fn chrono_lite_format(secs: u64) -> String {
    // Simple formatting - just show epoch seconds for now
    format!("Unix timestamp: {} seconds since epoch", secs)
}
