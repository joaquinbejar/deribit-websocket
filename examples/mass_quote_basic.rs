//! Basic Mass Quote Example
//!
//! This example demonstrates how to use Deribit's mass quote functionality
//! to place multiple quotes efficiently with MMP (Market Maker Protection) groups.

use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize crypto provider and logging
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install crypto provider")?;

    setup_logger();
    let client = DeribitWebSocketClient::default();

    tracing::info!("🚀 Starting Mass Quote Basic Example");
    if !client.config.has_credentials() {
        tracing::info!("🎯 Running in demo mode - showing Mass Quote API usage");
        demonstrate_mass_quote_api();
        return Ok(());
    }

    client.connect().await?;
    tracing::info!("✅ Connected to Deribit WebSocket");

    let (client_id, client_secret) = client.config.get_credentials().unwrap();
    client.authenticate(client_id, client_secret).await?;
    tracing::info!("🔐 Authenticated successfully");

    // Step 1: Create MMP group configuration
    tracing::info!("📋 Setting up MMP group configuration...");

    let mmp_config = MmpGroupConfig::new(
        "btc_market_making".to_string(),
        10.0, // quantity_limit: 10 BTC max per quote
        5.0,  // delta_limit: 5 BTC (must be < quantity_limit)
        1000, // interval: 1 second
        5000, // frozen_time: 5 seconds after trigger
    )?;

    client.set_mmp_config(mmp_config).await?;
    tracing::info!("✅ MMP group 'btc_market_making' configured");

    // Step 2: Create quotes for BTC perpetual
    tracing::info!("💰 Creating mass quotes for BTC-PERPETUAL...");

    let quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.1, 45000.0)
            .with_quote_set_id("spread_1".to_string())
            .with_post_only(true),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.1, 55000.0)
            .with_quote_set_id("spread_1".to_string())
            .with_post_only(true),
        Quote::buy("BTC-PERPETUAL".to_string(), 0.2, 44000.0)
            .with_quote_set_id("spread_2".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.2, 56000.0)
            .with_quote_set_id("spread_2".to_string()),
    ];

    let mass_quote_request = MassQuoteRequest::new("btc_market_making".to_string(), quotes)
        .with_quote_id("quote_batch_1".to_string())
        .with_detailed_errors();

    // Step 3: Place mass quotes
    match client.mass_quote(mass_quote_request).await {
        Ok(response) => {
            tracing::info!(
                "✅ Mass quote successful: {} placed, {} errors",
                response.success_count,
                response.error_count
            );

            if let Some(errors) = response.errors {
                for error in errors {
                    tracing::warn!(
                        "❌ Quote error for {} {}: {} ({})",
                        error.instrument_name,
                        error.side,
                        error.error_message,
                        error.error_code
                    );
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Mass quote failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 4: Check open orders
    tracing::info!("📊 Checking open orders...");

    match client
        .get_open_orders(Some("BTC".to_string()), None, Some("quote".to_string()))
        .await
    {
        Ok(orders) => {
            tracing::info!("📈 Found {} open quotes:", orders.len());
            for order in &orders {
                tracing::info!(
                    "   💱 {} {} @ {} (Set: {}, Group: {})",
                    order.side.to_uppercase(),
                    order.amount,
                    order.price,
                    order.quote_set_id.as_deref().unwrap_or("none"),
                    order.mmp_group
                );
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to get open orders: {}", e);
        }
    }

    // Step 5: Wait a bit, then cancel quotes by set ID
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    tracing::info!("🗑️ Cancelling quotes from set 'spread_1'...");

    let cancel_request = CancelQuotesRequest::by_quote_set_id("spread_1".to_string());
    match client.cancel_quotes(cancel_request).await {
        Ok(response) => {
            tracing::info!(
                "✅ Cancelled {} quotes from set 'spread_1'",
                response.cancelled_count
            );
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel quotes: {}", e);
        }
    }

    // Step 6: Cancel remaining quotes
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    tracing::info!("🧹 Cancelling all remaining BTC quotes...");

    let cancel_all_request = CancelQuotesRequest::by_currency("BTC".to_string());
    match client.cancel_quotes(cancel_all_request).await {
        Ok(response) => {
            tracing::info!("✅ Cancelled {} remaining quotes", response.cancelled_count);
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to cancel remaining quotes: {}", e);
        }
    }

    // Step 7: Clean up MMP group (optional)
    tracing::info!("🧽 Cleaning up MMP group...");

    let cleanup_config = MmpGroupConfig::new(
        "btc_market_making".to_string(),
        10.0,
        5.0,
        0, // interval = 0 disables the group
        5000,
    )?
    .disable();

    match client.set_mmp_config(cleanup_config).await {
        Ok(()) => {
            tracing::info!("✅ MMP group disabled and cleaned up");
        }
        Err(e) => {
            tracing::warn!("⚠️ Failed to disable MMP group: {}", e);
        }
    }

    tracing::info!("🎯 Mass Quote Basic Example completed successfully!");

    Ok(())
}

/// Demonstrate Mass Quote API usage without requiring real connection
fn demonstrate_mass_quote_api() {
    tracing::info!("📊 === Mass Quote API Demonstration ===");

    // Step 1: MMP Group Configuration
    tracing::info!("🏷️ Step 1: Creating MMP Group Configuration");

    let mmp_config = match MmpGroupConfig::new(
        "btc_market_making".to_string(),
        10.0, // quantity_limit: 10 BTC max per quote
        5.0,  // delta_limit: 5 BTC (must be < quantity_limit)
        1000, // interval: 1 second
        5000, // frozen_time: 5 seconds after trigger
    ) {
        Ok(config) => {
            tracing::info!(
                "✅ MMP Config created: Group '{}', Qty Limit: {}, Delta Limit: {}",
                config.mmp_group,
                config.quantity_limit,
                config.delta_limit
            );
            config
        }
        Err(e) => {
            tracing::error!("❌ Failed to create MMP config: {}", e);
            return;
        }
    };

    tracing::info!("{mmp_config}");

    // Step 2: Create Quotes
    tracing::info!("💰 Step 2: Creating Mass Quotes");

    let quotes = vec![
        Quote::buy("BTC-PERPETUAL".to_string(), 0.1, 45000.0)
            .with_quote_set_id("spread_1".to_string())
            .with_post_only(true),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.1, 55000.0)
            .with_quote_set_id("spread_1".to_string())
            .with_post_only(true),
        Quote::buy("BTC-PERPETUAL".to_string(), 0.2, 44000.0)
            .with_quote_set_id("spread_2".to_string()),
        Quote::sell("BTC-PERPETUAL".to_string(), 0.2, 56000.0)
            .with_quote_set_id("spread_2".to_string()),
    ];

    for (i, quote) in quotes.iter().enumerate() {
        tracing::info!(
            "   📋 Quote {}: {} {} @ {} (Set: {})",
            i + 1,
            quote.side.to_uppercase(),
            quote.amount,
            quote.price,
            quote.quote_set_id.as_deref().unwrap_or("none")
        );
    }

    // Step 3: Create Mass Quote Request
    tracing::info!("📤 Step 3: Creating Mass Quote Request");

    let mass_quote_request = MassQuoteRequest::new("btc_market_making".to_string(), quotes)
        .with_quote_id("quote_batch_1".to_string())
        .with_detailed_errors();

    match mass_quote_request.validate() {
        Ok(()) => {
            tracing::info!("✅ Mass quote request validation passed");
            tracing::info!("   🏷️ MMP Group: {}", mass_quote_request.mmp_group);
            tracing::info!("   📊 Quote Count: {}", mass_quote_request.quotes.len());
            tracing::info!(
                "   🆔 Quote ID: {}",
                mass_quote_request.quote_id.as_deref().unwrap_or("none")
            );
        }
        Err(e) => {
            tracing::error!("❌ Mass quote request validation failed: {}", e);
            return;
        }
    }

    // Step 4: Demonstrate Quote Cancellation
    tracing::info!("🗑️ Step 4: Quote Cancellation Options");

    let cancel_by_set = CancelQuotesRequest::by_quote_set_id("spread_1".to_string());
    tracing::info!(
        "   📋 Cancel by Quote Set ID: {:?}",
        cancel_by_set.quote_set_id
    );

    let cancel_by_currency = CancelQuotesRequest::by_currency("BTC".to_string());
    tracing::info!(
        "   💱 Cancel by Currency: {:?}",
        cancel_by_currency.currency
    );

    let cancel_by_instrument = CancelQuotesRequest::by_instrument("BTC-PERPETUAL".to_string());
    tracing::info!(
        "   🎯 Cancel by Instrument: {:?}",
        cancel_by_instrument.instrument_name
    );

    // Step 5: Summary
    tracing::info!("📈 === Demo Summary ===");
    tracing::info!("✅ MMP Group Configuration: Ready");
    tracing::info!("✅ Mass Quote Request: Validated");
    tracing::info!("✅ Quote Cancellation: Options demonstrated");
    tracing::info!("🎯 To run with real connection, set environment variables:");
    tracing::info!("   export DERIBIT_CLIENT_ID=your_client_id");
    tracing::info!("   export DERIBIT_CLIENT_SECRET=your_client_secret");
}
