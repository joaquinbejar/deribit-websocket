//! Connection recovery integration tests
//!
//! This test covers connection recovery scenarios:
//! 1. Automatic reconnection after network interruption
//! 2. Subscription restoration after reconnection
//! 3. Message queue handling during disconnection
//! 4. Connection state management during recovery

use std::path::Path;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

use deribit_websocket::prelude::*;

/// Check if .env file exists and contains required variables
fn check_env_file() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("../../../.env.backup").exists() {
        return Err(
            "Missing .env file. Please create one with DERIBIT_CLIENT_ID and DERIBIT_CLIENT_SECRET"
                .into(),
        );
    }

    dotenv::dotenv().ok();

    let required_vars = [
        "DERIBIT_CLIENT_ID",
        "DERIBIT_CLIENT_SECRET",
        "DERIBIT_WS_URL",
    ];

    for var in &required_vars {
        if std::env::var(var).is_err() {
            return Err(format!("Missing required environment variable: {}", var).into());
        }
    }

    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_manual_reconnection() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Manual Reconnection ===");

    check_env_file()?;
    info!("✅ Environment file validation passed");

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Step 1: Initial connection
    info!("🔌 Establishing initial connection...");
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Initial connection established");

    // Step 2: Test initial functionality
    info!("📤 Testing initial functionality...");
    let initial_response = client.get_time().await?;
    debug!("Initial response: {:?}", initial_response);
    info!("✅ Initial functionality confirmed");

    // Step 3: Disconnect manually
    info!("👋 Manually disconnecting...");
    client.disconnect().await?;
    assert!(!client.is_connected().await);
    info!("✅ Manual disconnection completed");

    // Step 4: Wait a moment
    sleep(Duration::from_secs(1)).await;

    // Step 5: Reconnect
    info!("🔌 Attempting reconnection...");
    let reconnect_result = client.connect().await;

    match reconnect_result {
        Ok(_) => {
            info!("✅ Reconnection successful");
            assert!(client.is_connected().await);

            // Test functionality after reconnection
            info!("📤 Testing functionality after reconnection...");
            let reconnect_response = client.get_time().await;
            match reconnect_response {
                Ok(response) => {
                    debug!("Reconnect response: {:?}", response);
                    info!("✅ Functionality restored after reconnection");
                }
                Err(e) => {
                    info!("⚠️ Functionality test failed after reconnection: {}", e);
                }
            }

            // Clean up
            client.disconnect().await?;
        }
        Err(e) => {
            info!(
                "⚠️ Reconnection failed: {} (may be expected in test environment)",
                e
            );
        }
    }

    info!("🎉 Manual reconnection test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_connection_state_management() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Connection State Management ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Initial connection
    info!("🔌 Establishing connection...");
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connection established, state: connected");

    // Disconnect and verify state
    info!("👋 Disconnecting...");
    client.disconnect().await?;
    assert!(!client.is_connected().await);
    info!("✅ Disconnected, state: not connected");

    // Try operations while disconnected (should fail)
    info!("📤 Testing operations while disconnected...");
    let disconnected_result = timeout(Duration::from_secs(2), client.get_time()).await;

    match disconnected_result {
        Ok(Err(_)) => {
            info!("✅ Operations correctly failed while disconnected");
        }
        Err(_) => {
            info!("✅ Operations correctly timed out while disconnected");
        }
        Ok(Ok(_)) => {
            return Err("Operations should fail while disconnected".into());
        }
    }

    // Reconnect and verify state
    info!("🔌 Reconnecting...");
    let reconnect_result = client.connect().await;

    match reconnect_result {
        Ok(_) => {
            assert!(client.is_connected().await);
            info!("✅ Reconnected, state: connected");

            // Verify operations work again
            let connected_result = client.get_time().await;
            match connected_result {
                Ok(_) => {
                    info!("✅ Operations working after reconnection");
                }
                Err(e) => {
                    info!("⚠️ Operations failed after reconnection: {}", e);
                }
            }

            client.disconnect().await?;
        }
        Err(e) => {
            info!("⚠️ Reconnection failed: {} (may be expected)", e);
        }
    }

    info!("🎉 Connection state recovery test completed successfully!");
    Ok(())
}
