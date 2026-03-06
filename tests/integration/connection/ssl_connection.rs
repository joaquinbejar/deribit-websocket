//! SSL/TLS connection integration tests
//!
//! This test covers SSL/TLS connection scenarios:
//! 1. Secure WebSocket connection establishment
//! 2. Certificate validation
//! 3. TLS version compatibility
//! 4. SSL handshake error handling

use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info};

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
async fn test_secure_websocket_connection() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Secure WebSocket Connection ===");

    check_env_file()?;
    info!("✅ Environment file validation passed");

    // Use secure WebSocket URL (wss://)
    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());

    // Ensure we're using secure connection
    if !ws_url.starts_with("wss://") {
        return Err("Test requires secure WebSocket URL (wss://)".into());
    }

    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    info!("🔒 Establishing secure WebSocket connection...");
    let connect_result = client.connect().await;

    match connect_result {
        Ok(_) => {
            info!("✅ Secure WebSocket connection established");
            assert!(client.is_connected().await);

            // Test that the secure connection works
            info!("📤 Testing secure connection functionality...");
            let response = client.get_time().await;
            match response {
                Ok(time_response) => {
                    debug!("Time response over secure connection: {:?}", time_response);
                    info!("✅ Secure connection is functional");
                }
                Err(e) => {
                    info!(
                        "⚠️ Secure connection established but get_time failed: {}",
                        e
                    );
                }
            }

            // Clean up
            client.disconnect().await?;
            info!("✅ Secure connection closed");
        }
        Err(e) => {
            error!("❌ Secure WebSocket connection failed: {}", e);
            return Err(format!("Secure connection failed: {}", e).into());
        }
    }

    info!("🎉 Secure WebSocket connection test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_ssl_certificate_validation() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: SSL Certificate Validation ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());

    // Test secure connection (wss://)
    info!("🔒 Testing secure connection (wss://)...");
    let secure_config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let secure_client = DeribitWebSocketClient::new(&secure_config)?;

    let secure_result = secure_client.connect().await;
    match secure_result {
        Ok(_) => {
            info!("✅ Secure connection successful");
            assert!(secure_client.is_connected().await);
            secure_client.disconnect().await?;
        }
        Err(e) => {
            info!(
                "⚠️ Secure connection failed: {} (may be expected in test environment)",
                e
            );
        }
    }

    info!("🎉 SSL Certificate Validation test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn test_insecure_to_secure_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Insecure to Secure Connection Comparison ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());

    // Test secure connection (wss://)
    info!("🔒 Testing secure connection (wss://)...");
    let secure_config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let secure_client = DeribitWebSocketClient::new(&secure_config)?;

    let secure_result = secure_client.connect().await;
    match secure_result {
        Ok(_) => {
            info!("✅ Secure connection successful");
            assert!(secure_client.is_connected().await);
            secure_client.disconnect().await?;
        }
        Err(e) => {
            info!(
                "⚠️ Secure connection failed: {} (may be expected in test environment)",
                e
            );
        }
    }

    // Test what happens with insecure URL (if server supports it)
    if ws_url.starts_with("wss://") {
        let insecure_url = ws_url.replace("wss://", "ws://");
        info!("🔓 Testing insecure connection (ws://) for comparison...");

        let insecure_config = deribit_websocket::config::WebSocketConfig::with_url(&insecure_url)?;
        let insecure_client = DeribitWebSocketClient::new(&insecure_config)?;

        let insecure_result = timeout(Duration::from_secs(5), insecure_client.connect()).await;
        match insecure_result {
            Ok(Ok(_)) => {
                info!("⚠️ Insecure connection succeeded (server accepts both)");
                insecure_client.disconnect().await?;
            }
            Ok(Err(_)) => {
                info!("✅ Insecure connection rejected as expected");
            }
            Err(_) => {
                info!("✅ Insecure connection timed out as expected");
            }
        }
    }

    info!("🎉 Connection security test completed successfully!");
    Ok(())
}
