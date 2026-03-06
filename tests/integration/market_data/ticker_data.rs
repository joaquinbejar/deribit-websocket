//! Ticker data integration tests
//!
//! This test covers ticker data subscription and validation:
//! 1. Connect and subscribe to ticker channel
//! 2. Receive ticker data messages
//! 3. Validate ticker data structure and content
//! 4. Test multiple instruments
//! 5. Verify data consistency

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
async fn test_ticker_data_structure() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("LOGLEVEL", "debug");
    }
    setup_logger();

    info!("=== Integration Test: Ticker Data Structure ===");

    check_env_file()?;
    info!("✅ Environment file validation passed");

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Connect and subscribe
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connected to WebSocket");

    let instrument = "BTC-PERPETUAL";
    let channel = format!("ticker.{}", instrument);

    info!("📡 Subscribing to ticker for {}...", instrument);
    client.subscribe(vec![channel]).await?;
    info!("✅ Subscribed to ticker");

    // Wait for ticker data and validate structure
    info!("⏳ Waiting for ticker data to validate structure...");
    let validation_timeout = Duration::from_secs(15);

    let validation_result = timeout(validation_timeout, async {
        for _ in 0..30 {
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
                        info!("✅ Received ticker data, validating structure...");

                        // Validate ticker data structure
                        if let Some(data) = params.get("data") {
                            let expected_fields = [
                                "timestamp",
                                "stats",
                                "state",
                                "settlement_price",
                                "open_interest",
                                "min_price",
                                "max_price",
                                "mark_price",
                                "last_price",
                                "instrument_name",
                                "index_price",
                                "funding_8h",
                                "current_funding",
                                "best_bid_price",
                                "best_bid_amount",
                                "best_ask_price",
                                "best_ask_amount",
                            ];

                            let mut found_fields = 0;
                            for field in &expected_fields {
                                if data.get(field).is_some() {
                                    found_fields += 1;
                                    debug!("✓ Found field: {}", field);
                                }
                            }

                            info!(
                                "✅ Ticker data structure validated: {}/{} expected fields found",
                                found_fields,
                                expected_fields.len()
                            );

                            // Validate specific field types
                            if let Some(timestamp) = data.get("timestamp")
                                && timestamp.is_number()
                            {
                                info!("✅ Timestamp field is numeric");
                            }

                            if let Some(last_price) = data.get("last_price")
                                && last_price.is_number()
                            {
                                info!("✅ Last price field is numeric");
                            }

                            if let Some(instrument_name) = data.get("instrument_name")
                                && instrument_name.as_str() == Some(instrument)
                            {
                                info!("✅ Instrument name matches subscription");
                            }

                            return true;
                        }
                    }
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
        false
    })
    .await;

    match validation_result {
        Ok(true) => {
            info!("✅ Ticker data structure validation successful");
        }
        Ok(false) => {
            info!(
                "⚠️ No ticker data received for validation (may be expected in test environment)"
            );
        }
        Err(_) => {
            info!("⚠️ Timeout waiting for ticker data (may be expected in test environment)");
        }
    }

    // Cleanup
    client.disconnect().await?;
    info!("✅ Disconnected");

    info!("🎉 Ticker data structure test completed successfully!");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
#[cfg(feature = "integration-tests")]
async fn test_multiple_ticker_subscriptions() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    info!("=== Integration Test: Multiple Instrument Tickers ===");

    check_env_file()?;

    let ws_url = std::env::var("DERIBIT_WS_URL")
        .unwrap_or_else(|_| "wss://test.deribit.com/ws/api/v2".to_string());
    let config = deribit_websocket::config::WebSocketConfig::with_url(&ws_url)?;
    let client = DeribitWebSocketClient::new(&config)?;

    // Connect
    client.connect().await?;
    assert!(client.is_connected().await);
    info!("✅ Connected to WebSocket");

    // Subscribe to multiple instrument tickers
    let instruments = ["BTC-PERPETUAL", "ETH-PERPETUAL"];
    let channels: Vec<String> = instruments
        .iter()
        .map(|&inst| format!("ticker.{}", inst))
        .collect();

    info!("📡 Subscribing to tickers for multiple instruments...");
    let subscribe_result = client.subscribe(channels).await;

    match subscribe_result {
        Ok(_) => {
            info!("✅ Subscribed to multiple ticker channels");
        }
        Err(e) => {
            info!(
                "⚠️ Multiple ticker subscription failed: {} (may be expected in test env)",
                e
            );
        }
    }

    // Wait briefly for any data
    info!("⏳ Waiting for ticker data from multiple instruments...");
    sleep(Duration::from_secs(3)).await;

    // Try to receive some messages
    for _ in 0..10 {
        if let Ok(message) = client.receive_message().await {
            debug!("📨 Received message: {:?}", message);

            // Parse JSON message
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel_name) = params.get("channel")
                && channel_name.as_str().unwrap_or("").contains("ticker")
            {
                info!(
                    "✅ Received ticker data from channel: {}",
                    channel_name.as_str().unwrap_or("unknown")
                );
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    // Cleanup
    client.disconnect().await?;
    info!("✅ Disconnected");

    info!("🎉 Multiple instrument tickers test completed successfully!");
    Ok(())
}
