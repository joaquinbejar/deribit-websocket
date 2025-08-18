//! Basic WebSocket connection integration tests
//!
//! This test covers the fundamental WebSocket connection flow:
//! 1. Establish a WebSocket connection to Deribit
//! 2. Verify connection state
//! 3. Send basic test messages
//! 4. Receive and validate responses
//! 5. Close connection gracefully

use std::path::Path;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

use deribit_base::prelude::*;
use deribit_websocket::prelude::*;

/// Check if .env file exists and contains required variables
fn check_env_file() -> Result<(), Box<dyn std::error::Error>> {
    // Check if .env file exists
    if !Path::new(".env").exists() {
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
async fn test_basic_websocket_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging for test visibility
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Basic WebSocket Connection ===");

    // Step 0: Check .env file exists and has required variables
    check_env_file()?;
    info!("✅ Environment file validation passed");

    // Step 1: Create configuration from environment
    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    info!("✅ Configuration loaded: {}", ws_url);

    // Step 2: Create client
    let client = DeribitWebSocketClient::new(config)?;
    info!("✅ WebSocket client created successfully");

    // Step 3: Establish WebSocket connection
    info!("🔌 Attempting to connect to Deribit WebSocket server...");
    client.connect().await?;
    info!("✅ WebSocket connection established");

    // Step 4: Verify connection state
    assert!(
        client.is_connected().await,
        "Client should report as connected"
    );
    info!("✅ Connection state verified");

    // Step 5: Send a basic test message (get_time)
    info!("📤 Sending test message (get_time)...");
    let response = client.get_time().await?;
    debug!("📨 Received response: {:?}", response);
    info!("✅ Test message sent and response received");

    // Step 6: Close connection gracefully
    info!("👋 Closing connection...");
    client.disconnect().await?;
    info!("✅ Connection closed gracefully");

    // Step 7: Verify disconnected state
    assert!(
        !client.is_connected().await,
        "Client should not be connected after disconnect"
    );
    info!("✅ Disconnection verified");

    info!("🎉 Basic WebSocket connection test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_connection_to_invalid_host() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Connection with Invalid URL ===");

    // Check .env file exists
    check_env_file()?;

    // Create config with invalid URL
    let config =
        deribit_websocket::config::WebSocketConfig::with_url("wss://invalid.example.com/ws")?;
    let client = DeribitWebSocketClient::new(config)?;

    // Attempt to connect - this should fail
    let connect_result = timeout(Duration::from_secs(5), client.connect()).await;

    match connect_result {
        Ok(Err(_)) => {
            info!("✅ Connection failed as expected with invalid URL");
        }
        Err(_) => {
            info!("✅ Connection timed out as expected with invalid URL");
        }
        Ok(Ok(_)) => {
            return Err("Connection should have failed with invalid URL".into());
        }
    }

    info!("🎉 Invalid URL test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_connection_timeout() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Connection Timeout ===");

    check_env_file()?;

    // Use a non-responsive URL (blackhole)
    let config = deribit_websocket::config::WebSocketConfig::with_url("wss://10.255.255.1/ws")?;
    let client = DeribitWebSocketClient::new(config)?;

    // Attempt to connect with short timeout
    let connect_result = timeout(Duration::from_secs(2), client.connect()).await;

    match connect_result {
        Err(_) => {
            info!("✅ Connection timed out as expected");
        }
        Ok(Err(_)) => {
            info!("✅ Connection failed as expected");
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
async fn test_multiple_concurrent_connections() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Multiple Connections ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;

    // Create multiple clients
    let client1 = DeribitWebSocketClient::new(config.clone())?;
    let client2 = DeribitWebSocketClient::new(config)?;

    info!("🔌 Connecting first client...");
    client1.connect().await?;
    assert!(client1.is_connected().await);
    info!("✅ First client connected");

    // Wait a bit to avoid overwhelming the server
    sleep(Duration::from_millis(500)).await;

    info!("🔌 Connecting second client...");
    client2.connect().await?;
    assert!(client2.is_connected().await);
    info!("✅ Second client connected");

    // Both clients should be able to send messages
    info!("📤 Testing both clients...");
    let response1 = client1.get_time().await?;
    let response2 = client2.get_time().await?;
    debug!("Client 1 response: {:?}", response1);
    debug!("Client 2 response: {:?}", response2);
    info!("✅ Both clients working correctly");

    // Disconnect both clients
    info!("👋 Disconnecting clients...");
    client1.disconnect().await?;
    client2.disconnect().await?;

    assert!(!client1.is_connected().await);
    assert!(!client2.is_connected().await);
    info!("✅ Both clients disconnected");

    info!("🎉 Multiple connections test completed successfully!");
    Ok(())
}
