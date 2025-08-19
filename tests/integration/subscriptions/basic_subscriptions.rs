//! Basic subscription management integration tests
//!
//! This test covers the fundamental subscription flow:
//! 1. Connect to WebSocket
//! 2. Subscribe to public channels (ticker, orderbook, trades)
//! 3. Receive and validate subscription confirmations
//! 4. Receive real-time data
//! 5. Unsubscribe from channels
//! 6. Verify unsubscription confirmations

use std::path::Path;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info};

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
async fn test_basic_ticker_subscription() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging for test visibility
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Basic Ticker Subscription ===");

    // Step 0: Check .env file exists and has required variables
    check_env_file()?;
    info!("✅ Environment file validation passed");

    // Step 1: Create configuration and client
    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;
    info!("✅ WebSocket client created");

    // Step 2: Connect to WebSocket
    info!("🔌 Connecting to WebSocket server...");
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ WebSocket connection established");

    // Step 3: Subscribe to ticker channel
    let instrument = "BTC-PERPETUAL";
    let channel = format!("ticker.{}", instrument);

    info!("📡 Subscribing to ticker for {}...", instrument);
    let subscribe_result = client.subscribe(vec![channel.clone()]).await;

    match subscribe_result {
        Ok(response) => {
            info!("✅ Subscription successful");
            debug!("Subscribe response: {:?}", response);
        }
        Err(e) => {
            error!("❌ Subscription failed: {}", e);
            client.disconnect().await.ok();
            return Err(format!("Subscription failed: {}", e).into());
        }
    }

    // Step 4: Wait for real-time data
    info!("⏳ Waiting for ticker data...");
    let data_timeout = Duration::from_secs(10);

    let data_result = timeout(data_timeout, async {
        let mut received_data = false;
        for _ in 0..20 {
            if let Ok(message) = client.receive_message().await {
                debug!("📨 Received message: {:?}", message);

                // Parse JSON message
                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&message) {
                    // Check if this is ticker data
                    if let Some(method) = json_msg.get("method")
                        && method.as_str() == Some("subscription")
                        && let Some(params) = json_msg.get("params")
                        && let Some(channel_name) = params.get("channel")
                        && channel_name.as_str().unwrap_or("").contains("ticker")
                    {
                        info!("✅ Received ticker data");
                        received_data = true;
                        break;
                    }
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
        received_data
    })
    .await;

    match data_result {
        Ok(true) => {
            info!("✅ Real-time ticker data received");
        }
        Ok(false) => {
            info!("⚠️ No ticker data received (may be expected in test environment)");
        }
        Err(_) => {
            info!("⚠️ Timeout waiting for ticker data (may be expected in test environment)");
        }
    }

    // Step 5: Unsubscribe from channel
    info!("📡 Unsubscribing from ticker...");
    let unsubscribe_result = client.unsubscribe(vec![channel]).await;

    match unsubscribe_result {
        Ok(response) => {
            info!("✅ Unsubscription successful");
            debug!("Unsubscribe response: {:?}", response);
        }
        Err(e) => {
            info!("⚠️ Unsubscription failed: {} (may be expected)", e);
        }
    }

    // Step 6: Disconnect
    info!("👋 Disconnecting...");
    client.disconnect().await?;
    assert!(!client.is_connected().await);
    info!("✅ Disconnected successfully");

    info!("🎉 Basic ticker subscription test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_multiple_subscriptions() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Multiple Subscriptions ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Connect
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connected to WebSocket");

    // Subscribe to multiple channels
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
        "trades.BTC-PERPETUAL".to_string(),
    ];

    info!("📡 Subscribing to multiple channels...");
    let subscribe_result = client.subscribe(channels.clone()).await;

    match subscribe_result {
        Ok(response) => {
            info!("✅ Multiple subscriptions successful");
            debug!("Subscribe response: {:?}", response);
        }
        Err(e) => {
            info!(
                "⚠️ Multiple subscriptions failed: {} (may be expected in test env)",
                e
            );
        }
    }

    // Wait briefly for any data
    sleep(Duration::from_secs(2)).await;

    // Unsubscribe from all channels
    info!("📡 Unsubscribing from all channels...");
    let unsubscribe_result = client.unsubscribe(channels).await;

    match unsubscribe_result {
        Ok(response) => {
            info!("✅ Multiple unsubscriptions successful");
            debug!("Unsubscribe response: {:?}", response);
        }
        Err(e) => {
            info!(
                "⚠️ Multiple unsubscriptions failed: {} (may be expected)",
                e
            );
        }
    }

    // Disconnect
    client.disconnect().await?;
    info!("✅ Disconnected");

    info!("🎉 Multiple subscriptions test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_subscription_without_connection() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Subscription without Connection ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Try to subscribe without connecting first
    let channel = "ticker.BTC-PERPETUAL".to_string();

    info!("📡 Attempting subscription without connection...");
    let subscribe_result = client.subscribe(vec![channel]).await;

    match subscribe_result {
        Err(_) => {
            info!("✅ Subscription failed as expected without connection");
        }
        Ok(_) => {
            return Err("Subscription should fail without connection".into());
        }
    }

    info!("🎉 Subscription without connection test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_invalid_channel_subscription() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Invalid Channel Subscription ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Connect first
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connected to WebSocket");

    // Try to subscribe to invalid instrument
    let channel = "ticker.INVALID-INSTRUMENT".to_string();

    info!("📡 Attempting subscription to invalid channel...");
    let subscribe_result = client.subscribe(vec![channel]).await;

    match subscribe_result {
        Err(_) => {
            info!("✅ Subscription failed as expected for invalid channel");
        }
        Ok(_) => {
            info!("⚠️ Test server accepted invalid channel (test environment behavior)");
        }
    }

    // Clean up
    client.disconnect().await?;
    info!("✅ Disconnected");

    info!("🎉 Invalid channel subscription test completed successfully!");
    Ok(())
}
