//! Mass Quote Options Example
//!
//! This example demonstrates mass quoting for options instruments with:
//! - Delta-based quote management
//! - Options-specific MMP configuration
//! - Greeks monitoring and risk management

use deribit_websocket::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    setup_logger();
    let mut client = DeribitWebSocketClient::default();

    tracing::info!("🚀 Starting Mass Quote Options Example");

    // Shared state for tracking option data
    let quote_count = Arc::new(Mutex::new(0u32));
    let quote_count_clone = Arc::clone(&quote_count);

    // Set up message handler for options data
    client.set_message_handler(
        move |message: &str| -> Result<(), WebSocketError> {
            if let Ok(json_msg) = serde_json::from_str::<Value>(message)
                && let Some(method) = json_msg.get("method")
                && method.as_str() == Some("subscription")
                && let Some(params) = json_msg.get("params")
                && let Some(channel) = params.get("channel").and_then(|c| c.as_str())
            {
                // Handle mark price updates for options
                if channel.starts_with("markprice.options") {
                    let mut count = quote_count_clone.lock().unwrap();
                    *count += 1;

                    tracing::info!("📊 Mark Price Update #{}: {}", *count, channel);

                    if let Some(data) = params.get("data") {
                        if let Some(mark_price) = data.get("mark_price") {
                            tracing::info!("   💰 Mark Price: {}", mark_price);
                        }
                        if let Some(delta) = data.get("delta") {
                            tracing::info!("   📈 Delta: {}", delta);
                        }
                        if let Some(gamma) = data.get("gamma") {
                            tracing::info!("   📊 Gamma: {}", gamma);
                        }
                        if let Some(theta) = data.get("theta") {
                            tracing::info!("   ⏰ Theta: {}", theta);
                        }
                        if let Some(vega) = data.get("vega") {
                            tracing::info!("   📉 Vega: {}", vega);
                        }
                    }
                }
                // Handle user trades for options
                else if channel == "user.trades"
                    && let Some(data) = params.get("data")
                    && let Some(trades) = data.as_array()
                {
                    for trade in trades {
                        if let Some(instrument) = trade.get("instrument_name")
                            && instrument
                                .as_str()
                                .is_some_and(|s| s.contains("-C") || s.contains("-P"))
                        {
                            tracing::info!("💰 Options Trade Executed:");
                            tracing::info!("   🎯 Instrument: {}", instrument);

                            if let Some(side) = trade.get("direction") {
                                tracing::info!("   📊 Side: {}", side);
                            }
                            if let Some(amount) = trade.get("amount") {
                                tracing::info!("   📏 Amount: {}", amount);
                            }
                            if let Some(price) = trade.get("price") {
                                tracing::info!("   💵 Price: {}", price);
                            }
                            if let Some(mark_price) = trade.get("mark_price") {
                                tracing::info!("   🎯 Mark Price: {}", mark_price);
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

    // Subscribe to options mark prices and user trades
    client
        .subscribe(vec![
            "markprice.options.BTC-29MAR24-50000-C".to_string(),
            "markprice.options.BTC-29MAR24-50000-P".to_string(),
            "user.trades.any.any".to_string(),
        ])
        .await?;
    tracing::info!("📡 Subscribed to options mark prices and trades");

    // Step 1: Create MMP group for options trading.
    //
    // MMP configuration requires the feature to be activated on the
    // account by Deribit staff. A 11050 `bad_request` with payload
    // `"MMP disabled"` is the expected response on accounts without
    // activation — log it and carry on so the rest of the demo still
    // exercises its API surface without crashing.
    tracing::info!("📋 Setting up options MMP group...");

    let options_mmp_config = MmpGroupConfig::new(
        "btc_options_mm".to_string(),
        50.0,  // Higher quantity limit for options (in BTC equivalent)
        25.0,  // Delta limit
        2000,  // 2 second interval
        10000, // 10 second freeze after trigger
    )?;

    match client.set_mmp_config(options_mmp_config).await {
        Ok(()) => tracing::info!("✅ Options MMP group 'btc_options_mm' configured"),
        Err(e) => tracing::warn!(
            "⚠️  Options MMP group config failed: {} — expected if MMP is not activated",
            e
        ),
    }

    // Step 2: Create options quotes for calls and puts
    tracing::info!("📊 Creating options mass quotes...");

    // Example options instruments (these would be real instruments on testnet)
    let call_option = "BTC-29MAR24-50000-C";
    let put_option = "BTC-29MAR24-50000-P";

    let options_quotes = vec![
        // Call option quotes
        Quote::buy(call_option.to_string(), 0.1, 0.05)
            .with_quote_set_id("atm_calls".to_string())
            .with_post_only(true),
        Quote::sell(call_option.to_string(), 0.1, 0.08)
            .with_quote_set_id("atm_calls".to_string())
            .with_post_only(true),
        // Put option quotes
        Quote::buy(put_option.to_string(), 0.1, 0.04)
            .with_quote_set_id("atm_puts".to_string())
            .with_post_only(true),
        Quote::sell(put_option.to_string(), 0.1, 0.07)
            .with_quote_set_id("atm_puts".to_string())
            .with_post_only(true),
    ];

    let options_request = MassQuoteRequest::new("btc_options_mm".to_string(), options_quotes)
        .with_quote_id("options_batch_1".to_string())
        .with_detailed_errors();

    // Step 3: Place options quotes
    match client.mass_quote(options_request).await {
        Ok(response) => {
            tracing::info!(
                "✅ Options quotes: {} placed, {} errors",
                response.success_count,
                response.error_count
            );

            if let Some(errors) = response.errors {
                for error in errors {
                    tracing::warn!(
                        "❌ Options quote error for {} {}: {} ({})",
                        error.instrument_name,
                        error.side,
                        error.error_message,
                        error.error_code
                    );
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Options mass quote failed: {}", e);
        }
    }

    // Step 4: Monitor options for delta changes
    tracing::info!("👀 Monitoring options for 20 seconds...");

    let start_time = std::time::Instant::now();
    let monitor_duration = std::time::Duration::from_secs(20);

    while start_time.elapsed() < monitor_duration {
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

    // Step 5: Demonstrate delta-based cancellation
    tracing::info!("📊 Demonstrating delta-based quote management...");

    // Cancel quotes in a specific delta range (example: 0.3 to 0.7 delta)
    let delta_cancel_request = CancelQuotesRequest::by_delta_range(0.3, 0.7);
    match client.cancel_quotes(delta_cancel_request).await {
        Ok(response) => {
            tracing::info!(
                "✅ Cancelled {} quotes in delta range 0.3-0.7",
                response.cancelled_count
            );
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel quotes by delta: {}", e);
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Step 6: Update quotes with new strategy
    tracing::info!("🔄 Updating options quotes with tighter spreads...");

    let updated_options_quotes = vec![
        // Tighter call spreads
        Quote::buy(call_option.to_string(), 0.05, 0.055)
            .with_quote_set_id("tight_calls".to_string()),
        Quote::sell(call_option.to_string(), 0.05, 0.065)
            .with_quote_set_id("tight_calls".to_string()),
        // Tighter put spreads
        Quote::buy(put_option.to_string(), 0.05, 0.045).with_quote_set_id("tight_puts".to_string()),
        Quote::sell(put_option.to_string(), 0.05, 0.055)
            .with_quote_set_id("tight_puts".to_string()),
    ];

    let update_request =
        MassQuoteRequest::new("btc_options_mm".to_string(), updated_options_quotes)
            .with_quote_id("options_update_1".to_string());

    match client.mass_quote(update_request).await {
        Ok(response) => {
            tracing::info!(
                "✅ Updated options quotes: {} placed",
                response.success_count
            );
        }
        Err(e) => {
            tracing::error!("❌ Failed to update options quotes: {}", e);
        }
    }

    // Step 7: Check final positions
    tracing::info!("📊 Checking final options positions...");

    match client
        .get_open_orders(
            Some("BTC".to_string()),
            Some("option".to_string()),
            Some("quote".to_string()),
        )
        .await
    {
        Ok(orders) => {
            tracing::info!("📈 Found {} open options quotes:", orders.len());
            for order in &orders {
                tracing::info!(
                    "   📊 {} {} {} @ {} (Set: {})",
                    order.instrument_name,
                    order.side.to_uppercase(),
                    order.amount,
                    order.price,
                    order.quote_set_id.as_deref().unwrap_or("none")
                );
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to get open options orders: {}", e);
        }
    }

    // Step 8: Final cleanup
    tracing::info!("🧹 Cleaning up options quotes...");

    // Cancel all options quotes
    let cancel_all_options = CancelQuotesRequest::by_currency("BTC".to_string());
    match client.cancel_quotes(cancel_all_options).await {
        Ok(response) => {
            tracing::info!("✅ Cancelled {} options quotes", response.cancelled_count);
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel options quotes: {}", e);
        }
    }

    // Disable options MMP group
    let cleanup_config = MmpGroupConfig::new(
        "btc_options_mm".to_string(),
        50.0,
        25.0,
        0, // Disable
        10000,
    )?
    .disable();

    match client.set_mmp_config(cleanup_config).await {
        Ok(()) => {
            tracing::info!("✅ Options MMP group disabled");
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to disable options MMP group: {}", e);
        }
    }

    tracing::info!("🎯 Mass Quote Options Example completed successfully!");

    Ok(())
}
