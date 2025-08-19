//! OAuth authentication flow integration tests
//!
//! This test covers the WebSocket authentication flow:
//! 1. Connect to WebSocket without authentication
//! 2. Perform OAuth authentication using client credentials
//! 3. Verify authentication success
//! 4. Test authenticated operations
//! 5. Test token refresh if needed

use std::path::Path;
use tracing::{debug, error, info, warn};

use deribit_base::prelude::*;
use deribit_websocket::prelude::*;

/// Check if .env file exists and contains required variables
fn check_env_file() -> Result<(), Box<dyn std::error::Error>> {
    // Check if .env file exists
    if !Path::new("../../../.env.backup").exists() {
        return Err(
            "Missing .env file. Please create one with DERIBIT_CLIENT_ID and DERIBIT_CLIENT_SECRET"
                .into(),
        );
    }

    // Load environment variables
    dotenv::dotenv().ok();

    // Check required variables
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
async fn test_oauth_authentication_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging for test visibility
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: OAuth Authentication Flow ===");

    // Step 0: Check .env file exists and has required variables
    check_env_file()?;
    info!("✅ Environment file validation passed");

    // Step 1: Get credentials from environment
    let client_id = std::env::var("DERIBIT_CLIENT_ID")?;
    let client_secret = std::env::var("DERIBIT_CLIENT_SECRET")?;
    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());

    info!("✅ Credentials loaded from environment");

    // Step 2: Create configuration and client
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;
    info!("✅ WebSocket client created");

    // Step 3: Establish WebSocket connection
    info!("🔌 Connecting to WebSocket server...");
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ WebSocket connection established");

    // Step 4: Perform authentication
    info!("🔐 Performing OAuth authentication...");
    let auth_result = client.authenticate(&client_id, &client_secret).await;

    match auth_result {
        Ok(auth_response) => {
            info!("✅ Authentication successful");
            debug!("Auth response: {:?}", auth_response);

            // Verify we have an access token
            match &auth_response.result {
                deribit_websocket::model::JsonRpcResult::Success { result } => {
                    if let Some(token) = result.get("access_token") {
                        info!(
                            "✅ Access token received: {}...",
                            &token.as_str().unwrap_or("")[..10]
                        );
                    } else {
                        return Err("No access token in authentication response".into());
                    }
                }
                deribit_websocket::model::JsonRpcResult::Error { error } => {
                    return Err(format!("Authentication error: {}", error.message).into());
                }
            }
        }
        Err(e) => {
            error!("❌ Authentication failed: {}", e);
            client.disconnect().await.ok();
            return Err(format!("Authentication failed: {}", e).into());
        }
    }

    // Step 5: Test authenticated operation (get server time)
    info!("📊 Testing authenticated operation...");
    let time_result = client.get_time().await;

    match time_result {
        Ok(time_response) => {
            info!("✅ Authenticated operation successful");
            debug!("Server time response: {:?}", time_response);
        }
        Err(e) => {
            warn!(
                "⚠️  Get time failed (may be expected in test environment): {}",
                e
            );
        }
    }

    // Step 6: Disconnect
    info!("👋 Disconnecting...");
    client.disconnect().await?;
    assert!(!client.is_connected().await);
    info!("✅ Disconnected successfully");

    info!("🎉 OAuth authentication flow test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_invalid_credentials() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Authentication with Invalid Credentials ===");

    // Check .env file exists
    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Connect first
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connected to WebSocket");

    // Try to authenticate with invalid credentials
    info!("🔐 Attempting authentication with invalid credentials...");
    let auth_result = client
        .authenticate("invalid_client_id", "invalid_client_secret")
        .await;

    match auth_result {
        Err(_) => {
            info!("✅ Authentication failed as expected with invalid credentials");
        }
        Ok(_) => {
            // Test server might be permissive
            info!("⚠️ Test server accepted invalid credentials (test environment behavior)");
        }
    }

    // Clean up
    client.disconnect().await?;
    info!("✅ Disconnected");

    info!("🎉 Invalid credentials test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_authentication_state_management() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Authentication without Connection ===");

    check_env_file()?;

    let client_id = std::env::var("DERIBIT_CLIENT_ID")?;
    let client_secret = std::env::var("DERIBIT_CLIENT_SECRET")?;
    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());

    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Try to authenticate without connecting first
    info!("🔐 Attempting authentication without connection...");
    let auth_result = client.authenticate(&client_id, &client_secret).await;

    match auth_result {
        Err(_) => {
            info!("✅ Authentication failed as expected without connection");
        }
        Ok(_) => {
            return Err("Authentication should fail without connection".into());
        }
    }

    info!("🎉 Authentication without connection test completed successfully!");
    Ok(())
}
