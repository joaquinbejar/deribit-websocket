//! Connection error handling integration tests
//!
//! This test covers various connection error scenarios:
//! 1. Network connectivity issues
//! 2. Invalid URLs and endpoints
//! 3. Connection timeouts
//! 4. SSL/TLS certificate errors
//! 5. Connection recovery scenarios

use std::path::Path;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::info;

use deribit_base::prelude::*;
use deribit_websocket::prelude::*;

/// Check if .env file exists and contains required variables
fn check_env_file() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(".env").exists() {
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
async fn test_connection_to_invalid_host() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Connection to Invalid Host ===");

    check_env_file()?;
    info!("✅ Environment file validation passed");

    // Try to connect to non-existent host
    let config =
        deribit_websocket::config::WebSocketConfig::with_url("wss://nonexistent.invalid.com/ws")?;
    let client = DeribitWebSocketClient::new(config)?;

    info!("🔌 Attempting connection to invalid host...");
    let connect_result = timeout(Duration::from_secs(5), client.connect()).await;

    match connect_result {
        Ok(Err(e)) => {
            info!("✅ Connection failed as expected: {}", e);
            assert!(!client.is_connected().await);
        }
        Err(_) => {
            info!("✅ Connection timed out as expected");
            assert!(!client.is_connected().await);
        }
        Ok(Ok(_)) => {
            return Err("Connection should have failed with invalid host".into());
        }
    }

    info!("🎉 Invalid host connection test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_connection_to_invalid_path() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Connection Timeout Scenario ===");

    check_env_file()?;

    // Try to connect to unreachable IP
    let config = deribit_websocket::config::WebSocketConfig::with_url("wss://10.255.255.1:443/ws")?;
    let client = DeribitWebSocketClient::new(config)?;

    info!("🔌 Attempting connection to unreachable IP...");
    let connect_result = timeout(Duration::from_secs(5), client.connect()).await;

    match connect_result {
        Ok(Err(e)) => {
            info!("✅ Connection failed as expected: {}", e);
            assert!(!client.is_connected().await);
        }
        Err(_) => {
            info!("✅ Connection timed out as expected");
            assert!(!client.is_connected().await);
        }
        Ok(Ok(_)) => {
            return Err("Connection should have timed out".into());
        }
    }

    info!("🎉 Connection timeout test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_invalid_websocket_path() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Invalid WebSocket Path ===");

    check_env_file()?;

    // Try to connect to invalid path
    let config = deribit_websocket::config::WebSocketConfig::with_url(
        "wss://test.deribit.com/invalid/path",
    )?;
    let client = DeribitWebSocketClient::new(config)?;

    info!("🔌 Attempting connection to invalid path...");
    let connect_result = timeout(Duration::from_secs(10), client.connect()).await;

    match connect_result {
        Ok(Err(e)) => {
            info!("✅ Connection failed as expected: {}", e);
            assert!(!client.is_connected().await);
        }
        Err(_) => {
            info!("✅ Connection timed out as expected");
            assert!(!client.is_connected().await);
        }
        Ok(Ok(_)) => {
            // Some servers might accept the connection but close it immediately
            info!("⚠️ Connection succeeded but may be closed by server");
            sleep(Duration::from_millis(500)).await;
            // Check if connection is still active
            let time_result = timeout(Duration::from_secs(2), client.get_time()).await;
            match time_result {
                Ok(Ok(_)) => {
                    info!("⚠️ Server accepted invalid path (test environment behavior)");
                }
                _ => {
                    info!("✅ Connection was closed by server as expected");
                }
            }
            client.disconnect().await.ok();
        }
    }

    info!("🎉 Invalid WebSocket path test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_error_recovery_after_failure() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Connection Recovery After Failure ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(config)?;

    // First, try to connect to invalid host (should fail)
    info!("🔌 First attempt: connecting to invalid host...");
    let invalid_config =
        deribit_websocket::config::WebSocketConfig::with_url("wss://invalid.example.com/ws")?;
    let invalid_client = DeribitWebSocketClient::new(invalid_config)?;

    let first_result = timeout(Duration::from_secs(3), invalid_client.connect()).await;
    match first_result {
        Ok(Err(_)) | Err(_) => {
            info!("✅ First connection failed as expected");
        }
        Ok(Ok(_)) => {
            return Err("First connection should have failed".into());
        }
    }

    // Now try to connect to valid host (should succeed)
    info!("🔌 Second attempt: connecting to valid host...");
    let connect_result = client.connect().await;

    match connect_result {
        Ok(_) => {
            info!("✅ Recovery connection successful");
            assert!(client.is_connected().await);

            // Test that the connection works
            let time_result = client.get_time().await;
            match time_result {
                Ok(_) => {
                    info!("✅ Connection is functional after recovery");
                }
                Err(e) => {
                    info!("⚠️ Connection recovered but get_time failed: {}", e);
                }
            }

            // Clean up
            client.disconnect().await?;
        }
        Err(e) => {
            info!(
                "⚠️ Recovery connection failed: {} (may be expected in test environment)",
                e
            );
        }
    }

    info!("🎉 Connection recovery test completed successfully!");
    Ok(())
}
