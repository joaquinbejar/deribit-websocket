//! Basic WebSocket client example for Deribit
//!
//! This example demonstrates how to:
//! - Connect to Deribit WebSocket API
//! - Subscribe to market data channels
//! - Handle incoming messages
//! - Gracefully disconnect

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Starting Deribit WebSocket Client Example");

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

    // Test the connection
    tracing::info!("🧪 Testing connection...");
    match client.test_connection().await {
        Ok(response) => tracing::info!("✅ Connection test successful: {:?}", response),
        Err(e) => tracing::error!("❌ Connection test failed: {}", e),
    }

    // Get server time
    tracing::info!("⏰ Getting server time...");
    match client.get_time().await {
        Ok(response) => tracing::info!("✅ Server time: {:?}", response),
        Err(e) => tracing::error!("❌ Failed to get server time: {}", e),
    }

    // Subscribe to market data channels
    tracing::info!("📡 Subscribing to BTC-PERPETUAL ticker...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(response) => {
            tracing::info!("✅ Subscription successful!");
            tracing::info!("✅ Subscription successful: {:?}", response)
        }
        Err(e) => {
            tracing::error!("❌ Subscription failed: {}", e);
        }
    }

    // Display active subscriptions
    let subscriptions = client.get_subscriptions().await;
    tracing::info!("📋 Active subscriptions: {:?}", subscriptions);

    // Simulate receiving messages for a short time
    tracing::info!("⏳ Processing messages for 10 seconds...");
    let start_time = std::time::Instant::now();
    let mut message_count = 0;

    while start_time.elapsed() < std::time::Duration::from_secs(5) {
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            client.receive_message(),
        )
        .await
        {
            Ok(Ok(message)) => {
                message_count += 1;
                tracing::info!("📨 Message #{}: {}", message_count, message);
            }
            Ok(Err(e)) => {
                tracing::error!("❌ Error receiving message: {}", e);
                break;
            }
            Err(_) => {
                tracing::debug!("⏳ No message received (timeout)");
            }
        }
    }

    tracing::info!("📊 Total messages received: {}", message_count);

    // Unsubscribe from channels
    tracing::info!("📡 Unsubscribing from channels...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.unsubscribe(channels).await {
        Ok(response) => tracing::info!("✅ Unsubscription successful: {:?}", response),
        Err(e) => tracing::error!("❌ Unsubscription failed: {}", e),
    }

    // Disconnect from the server
    tracing::info!("👋 Disconnecting...");
    client.disconnect().await?;
    tracing::info!("✅ Disconnected successfully");

    tracing::info!("🎉 Example completed successfully!");
    Ok(())
}
