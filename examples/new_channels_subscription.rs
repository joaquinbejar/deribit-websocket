//! Example demonstrating subscription to new public channels (Issue #10)
//!
//! This example shows how to subscribe to:
//! - Grouped order book (book.{instrument}.{group}.{depth}.{interval})
//! - Incremental ticker (incremental_ticker.{instrument})
//! - Trades by kind (trades.{kind}.{currency}.{interval})
//! - Price ranking (deribit_price_ranking.{index_name})
//! - Price statistics (deribit_price_statistics.{index_name})
//! - Volatility index (deribit_volatility_index.{index_name})

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("Starting New Channels Subscription Example");

    // Create client configuration for testnet
    let config = WebSocketConfig::default()
        .with_heartbeat_interval(std::time::Duration::from_secs(30))
        .with_max_reconnect_attempts(3);

    // Create the WebSocket client
    let client = DeribitWebSocketClient::new(&config)?;
    tracing::info!("Client created successfully");

    // Connect to the server
    tracing::info!("Connecting to Deribit WebSocket...");
    client.connect().await?;
    tracing::info!("Connected to Deribit WebSocket");

    // Demonstrate creating subscription channels programmatically
    tracing::info!("Creating subscription channels...");

    // 1. Grouped Order Book - aggregated order book with custom depth and interval
    let grouped_book = SubscriptionChannel::GroupedOrderBook {
        instrument: "BTC-PERPETUAL".to_string(),
        group: "100".to_string(),
        depth: "10".to_string(),
        interval: "100ms".to_string(),
    };
    tracing::info!("Grouped order book channel: {}", grouped_book);

    // 2. Incremental Ticker - efficient ticker updates (deltas only)
    let incremental_ticker = SubscriptionChannel::IncrementalTicker("BTC-PERPETUAL".to_string());
    tracing::info!("Incremental ticker channel: {}", incremental_ticker);

    // 3. Trades by Kind - trades for all futures or options.
    //
    // NOTE: `raw` interval requires authentication (server code 13778:
    // `raw_subscriptions_not_available_for_unauthorized`) and, because
    // subscribe is all-or-nothing, including a `raw` channel in an
    // unauthenticated batch causes every channel in the batch to be
    // rejected. Public callers must use a bucketed interval such as
    // `100ms` or `agg2`.
    let trades_by_kind = SubscriptionChannel::TradesByKind {
        kind: "future".to_string(),
        currency: "BTC".to_string(),
        interval: "100ms".to_string(),
    };
    tracing::info!("Trades by kind channel: {}", trades_by_kind);

    // 4. Price Ranking - exchange price ranking data
    let price_ranking = SubscriptionChannel::PriceRanking("btc_usd".to_string());
    tracing::info!("Price ranking channel: {}", price_ranking);

    // 5. Price Statistics - price statistics for an index
    let price_statistics = SubscriptionChannel::PriceStatistics("btc_usd".to_string());
    tracing::info!("Price statistics channel: {}", price_statistics);

    // 6. Volatility Index - Deribit volatility index (DVOL)
    let volatility_index = SubscriptionChannel::VolatilityIndex("btc_usd".to_string());
    tracing::info!("Volatility index channel: {}", volatility_index);

    // Subscribe to all new channels
    let channels = vec![
        grouped_book.channel_name(),
        incremental_ticker.channel_name(),
        trades_by_kind.channel_name(),
        price_ranking.channel_name(),
        price_statistics.channel_name(),
        volatility_index.channel_name(),
    ];

    tracing::info!("Subscribing to {} channels...", channels.len());
    for channel in &channels {
        tracing::info!("  - {}", channel);
    }

    match client.subscribe(channels.clone()).await {
        Ok(response) => {
            tracing::info!("Subscription successful!");
            tracing::debug!("Response: {:?}", response);
        }
        Err(e) => {
            tracing::error!("Subscription failed: {}", e);
        }
    }

    // Display active subscriptions
    let subscriptions = client.get_subscriptions().await;
    tracing::info!("Active subscriptions: {:?}", subscriptions);

    // Process messages for a short time
    tracing::info!("Processing messages for 10 seconds...");
    let start_time = std::time::Instant::now();
    let mut message_count = 0;

    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            client.receive_message(),
        )
        .await
        {
            Ok(Ok(message)) => {
                message_count += 1;
                // Parse the channel from the message to identify the source
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&message) {
                    if let Some(params) = parsed.get("params")
                        && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
                    {
                        let channel_type = SubscriptionChannel::from_string(channel);
                        tracing::info!(
                            "Message #{} from {:?}: {}",
                            message_count,
                            channel_type,
                            &message[..message.len().min(200)]
                        );
                    }
                } else {
                    tracing::info!("Message #{}: {}", message_count, message);
                }
            }
            Ok(Err(e)) => {
                tracing::error!("Error receiving message: {}", e);
                break;
            }
            Err(_) => {
                tracing::debug!("No message received (timeout)");
            }
        }
    }

    tracing::info!("Total messages received: {}", message_count);

    // Unsubscribe from channels
    tracing::info!("Unsubscribing from channels...");
    match client.unsubscribe(channels).await {
        Ok(response) => tracing::info!("Unsubscription successful: {:?}", response),
        Err(e) => tracing::error!("Unsubscription failed: {}", e),
    }

    // Disconnect from the server
    tracing::info!("Disconnecting...");
    client.disconnect().await?;
    tracing::info!("Disconnected successfully");

    tracing::info!("Example completed successfully!");
    Ok(())
}
