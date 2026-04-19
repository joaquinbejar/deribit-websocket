//! Advanced Mass Quote Example
//!
//! This example demonstrates advanced mass quote features including:
//! - Multiple MMP groups for different strategies
//! - MMP trigger monitoring
//! - Dynamic quote management
//! - Error handling and recovery

use deribit_websocket::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    setup_logger();
    let mut client = DeribitWebSocketClient::default();

    tracing::info!("🚀 Starting Advanced Mass Quote Example");

    // Shared state for tracking quotes and MMP triggers
    let quote_count = Arc::new(Mutex::new(0u32));
    let mmp_triggers = Arc::new(Mutex::new(Vec::<String>::new()));

    let quote_count_clone = Arc::clone(&quote_count);
    let mmp_triggers_clone = Arc::clone(&mmp_triggers);

    // Set up message handler for MMP triggers and quote updates
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
            {
                // Handle MMP triggers
                if channel.starts_with("mmp.triggers") {
                    let mut count = quote_count_clone.lock().unwrap();
                    *count += 1;

                    tracing::warn!("🚨 MMP Trigger #{}: Channel: {}", *count, channel);

                    if let Some(data) = params.get("data") {
                        if let Some(currency) = data.get("currency") {
                            tracing::warn!("   💱 Currency: {}", currency);
                        }
                        if let Some(mmp_group) = data.get("mmp_group") {
                            tracing::warn!("   🏷️ MMP Group: {}", mmp_group);
                        }
                        if let Some(reason) = data.get("reason") {
                            tracing::warn!("   ⚡ Reason: {}", reason);
                        }
                        if let Some(frozen_time) = data.get("frozen_time") {
                            tracing::warn!("   ❄️ Frozen Time: {}ms", frozen_time);
                        }

                        // Track trigger for later analysis
                        let mut triggers = mmp_triggers_clone.lock().unwrap();
                        triggers.push(format!(
                            "{}:{}",
                            channel,
                            data.get("reason")
                                .unwrap_or(&Value::String("unknown".to_string()))
                        ));
                    }
                }
                // Handle quote executions
                else if channel == "user.trades"
                    && let Some(data) = params.get("data")
                    && let Some(trades) = data.as_array()
                {
                    for trade in trades {
                        if let Some(quote_id) = trade.get("quote_id") {
                            tracing::info!("💰 Quote Executed: ID {}", quote_id);

                            if let Some(instrument) = trade.get("instrument_name") {
                                tracing::info!("   🎯 Instrument: {}", instrument);
                            }
                            if let Some(side) = trade.get("direction") {
                                tracing::info!("   📊 Side: {}", side);
                            }
                            if let Some(amount) = trade.get("amount") {
                                tracing::info!("   📏 Amount: {}", amount);
                            }
                            if let Some(price) = trade.get("price") {
                                tracing::info!("   💵 Price: {}", price);
                            }
                        }
                    }
                }
            }
            Ok(())
        },
        |message, error| {
            tracing::error!("❌ Error processing message '{}': {}", message, error);
        },
    );

    // Connect and authenticate
    let client = client;
    client.connect().await?;
    tracing::info!("✅ Connected to Deribit WebSocket");

    let (client_id, client_secret) = client.config.get_credentials().unwrap();
    client.authenticate(client_id, client_secret).await?;
    tracing::info!("🔐 Authenticated successfully");

    // Subscribe to MMP triggers and user trades
    client
        .subscribe(vec![
            "mmp.triggers.any".to_string(),
            "user.trades.any.any".to_string(),
        ])
        .await?;
    tracing::info!("📡 Subscribed to MMP triggers and user trades");

    // Step 1: Create multiple MMP groups for different strategies
    tracing::info!("📋 Setting up multiple MMP groups...");

    let mmp_groups = vec![
        ("btc_tight_spread", 5.0, 2.5, 500, 2000), // Tight spread, low limits
        ("btc_wide_spread", 20.0, 10.0, 1000, 5000), // Wide spread, higher limits
        ("btc_scalping", 2.0, 1.0, 200, 1000),     // Scalping, very tight
    ];

    // MMP configuration requires the feature to be activated on the
    // account by Deribit staff. A 11050 `bad_request` with payload
    // `"MMP disabled"` is the expected response on accounts without
    // activation — log it and carry on so the rest of the demo still
    // exercises its API surface without crashing.
    for (group_name, qty_limit, delta_limit, interval, frozen_time) in &mmp_groups {
        let config = MmpGroupConfig::new(
            group_name.to_string(),
            *qty_limit,
            *delta_limit,
            *interval,
            *frozen_time,
        )?;

        match client.set_mmp_config(config).await {
            Ok(()) => tracing::info!("✅ MMP group '{}' configured", group_name),
            Err(e) => tracing::warn!(
                "⚠️  MMP group '{}' config failed: {} — expected if MMP is not activated",
                group_name,
                e
            ),
        }
    }

    // Step 2: Create layered quotes across multiple groups
    tracing::info!("💰 Creating layered mass quotes...");

    let mut all_quotes = HashMap::new();

    // Tight spread quotes
    let tight_quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.05, 49500.0)
            .with_quote_set_id("tight_layer_1".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.05, 50500.0)
            .with_quote_set_id("tight_layer_1".to_string()),
    ];

    // Wide spread quotes
    let wide_quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.2, 48000.0)
            .with_quote_set_id("wide_layer_1".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.2, 52000.0)
            .with_quote_set_id("wide_layer_1".to_string()),
    ];

    // Scalping quotes
    let scalp_quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.01, 49900.0)
            .with_quote_set_id("scalp_layer_1".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.01, 50100.0)
            .with_quote_set_id("scalp_layer_1".to_string()),
    ];

    all_quotes.insert("btc_tight_spread", tight_quotes);
    all_quotes.insert("btc_wide_spread", wide_quotes);
    all_quotes.insert("btc_scalping", scalp_quotes);

    // Place quotes for each group
    for (group_name, quotes) in all_quotes {
        let request = MassQuoteRequest::new(group_name.to_string(), quotes)
            .with_quote_id(format!("{}_batch_1", group_name))
            .with_detailed_errors();

        match client.mass_quote(request).await {
            Ok(response) => {
                tracing::info!(
                    "✅ {} quotes: {} placed, {} errors",
                    group_name,
                    response.success_count,
                    response.error_count
                );
            }
            Err(e) => {
                tracing::error!("❌ Failed to place {} quotes: {}", group_name, e);
            }
        }
    }

    // Step 3: Monitor and manage quotes
    tracing::info!("👀 Monitoring quotes for 30 seconds...");

    let start_time = std::time::Instant::now();
    let monitor_duration = std::time::Duration::from_secs(30);

    while start_time.elapsed() < monitor_duration {
        // Check MMP status every 5 seconds
        if start_time.elapsed().as_secs().is_multiple_of(5) {
            match client.get_mmp_config(None).await {
                Ok(configs) => {
                    tracing::info!("📊 MMP Status Check:");
                    for config in configs {
                        tracing::info!(
                            "   🏷️ Group: {} - Enabled: {}, Qty Limit: {}",
                            config.mmp_group,
                            config.enabled,
                            config.quantity_limit
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to get MMP config: {}", e);
                }
            }
        }

        // Process any incoming messages
        tokio::select! {
            result = client.receive_and_process_message() => {
                if let Err(e) = result {
                    tracing::warn!("⚠️ Message processing error: {}", e);
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Continue monitoring
            }
        }
    }

    // Step 4: Demonstrate quote management operations
    tracing::info!("🔧 Demonstrating quote management...");

    // Cancel scalping quotes first (most aggressive)
    let cancel_scalp = CancelQuotesRequest::by_quote_set_id("scalp_layer_1".to_string());
    match client.cancel_quotes(cancel_scalp).await {
        Ok(response) => {
            tracing::info!("✅ Cancelled {} scalping quotes", response.cancelled_count);
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel scalping quotes: {}", e);
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Update wide spread quotes with new prices
    tracing::info!("🔄 Updating wide spread quotes...");

    let updated_wide_quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.15, 47500.0)
            .with_quote_set_id("wide_layer_2".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.15, 52500.0)
            .with_quote_set_id("wide_layer_2".to_string()),
    ];

    let update_request = MassQuoteRequest::new("btc_wide_spread".to_string(), updated_wide_quotes)
        .with_quote_id("wide_update_1".to_string());

    match client.mass_quote(update_request).await {
        Ok(response) => {
            tracing::info!("✅ Updated wide spread: {} placed", response.success_count);
        }
        Err(e) => {
            tracing::error!("❌ Failed to update wide spread: {}", e);
        }
    }

    // Step 5: Final cleanup
    tracing::info!("🧹 Final cleanup...");

    // Cancel all remaining quotes
    let cancel_all = CancelQuotesRequest::by_currency("BTC".to_string());
    match client.cancel_quotes(cancel_all).await {
        Ok(response) => {
            tracing::info!("✅ Cancelled {} remaining quotes", response.cancelled_count);
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel all quotes: {}", e);
        }
    }

    // Disable all MMP groups
    for (group_name, _, _, _, _) in &mmp_groups {
        let disable_config = MmpGroupConfig::new(
            group_name.to_string(),
            1.0,
            0.5,
            0, // Disable
            1000,
        )?
        .disable();

        match client.set_mmp_config(disable_config).await {
            Ok(()) => {
                tracing::info!("✅ Disabled MMP group '{}'", group_name);
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to disable MMP group '{}': {}", group_name, e);
            }
        }
    }

    // Step 6: Summary
    let trigger_count = mmp_triggers.lock().unwrap().len();
    tracing::info!("📈 Advanced Mass Quote Example Summary:");
    tracing::info!("   🏷️ MMP Groups Created: {}", mmp_groups.len());
    tracing::info!("   🚨 MMP Triggers Received: {}", trigger_count);
    tracing::info!("   ✅ Example completed successfully!");

    Ok(())
}
