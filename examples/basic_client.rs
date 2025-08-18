//! Basic WebSocket client example for Deribit
//!
//! This example demonstrates how to:
//! - Connect to Deribit WebSocket API
//! - Subscribe to market data channels
//! - Handle incoming messages
//! - Gracefully disconnect

use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider for rustls
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    // Initialize logging
    env_logger::init();

    println!("🚀 Starting Deribit WebSocket Client Example");

    // Create client configuration for testnet
    let config = WebSocketConfig::testnet()
        .with_heartbeat_interval(std::time::Duration::from_secs(30))
        .with_max_reconnect_attempts(3);

    // Create the WebSocket client
    let client = DeribitWebSocketClient::new(config)?;
    println!("✅ Client created successfully");

    // Connect to the server
    println!("🔌 Connecting to Deribit WebSocket...");
    client.connect().await?;
    println!("✅ Connected to Deribit WebSocket");

    // Test the connection
    println!("🧪 Testing connection...");
    match client.test_connection().await {
        Ok(response) => println!("✅ Connection test successful: {:?}", response),
        Err(e) => println!("❌ Connection test failed: {}", e),
    }

    // Get server time
    println!("⏰ Getting server time...");
    match client.get_time().await {
        Ok(response) => println!("✅ Server time: {:?}", response),
        Err(e) => println!("❌ Failed to get server time: {}", e),
    }

    // Subscribe to market data channels
    println!("📊 Subscribing to market data...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.subscribe(channels).await {
        Ok(response) => println!("✅ Subscription successful: {:?}", response),
        Err(e) => println!("❌ Subscription failed: {}", e),
    }

    // Display active subscriptions
    let subscriptions = client.get_subscriptions().await;
    println!("📋 Active subscriptions: {:?}", subscriptions);

    // Simulate receiving messages for a short time
    println!("👂 Listening for messages (5 seconds)...");
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
                println!(
                    "📨 Message {}: {}",
                    message_count,
                    if message.len() > 100 {
                        format!("{}...", &message[..100])
                    } else {
                        message
                    }
                );
            }
            Ok(Err(e)) => {
                println!("❌ Error receiving message: {}", e);
                break;
            }
            Err(_) => {
                // Timeout - no message received, continue
                print!(".");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
    }

    println!("\n📊 Received {} messages total", message_count);

    // Unsubscribe from channels
    println!("🔕 Unsubscribing from channels...");
    let channels = vec![
        "ticker.BTC-PERPETUAL".to_string(),
        "book.BTC-PERPETUAL.100ms".to_string(),
    ];

    match client.unsubscribe(channels).await {
        Ok(response) => println!("✅ Unsubscription successful: {:?}", response),
        Err(e) => println!("❌ Unsubscription failed: {}", e),
    }

    // Disconnect from the server
    println!("🔌 Disconnecting from server...");
    client.disconnect().await?;
    println!("✅ Disconnected successfully");

    println!("🎉 Example completed successfully!");
    Ok(())
}
