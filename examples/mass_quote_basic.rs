//! Basic Mass Quote Example
//!
//! This example demonstrates how to use Deribit's mass quote functionality
//! to place multiple quotes efficiently with MMP (Market Maker Protection) groups.

use deribit_websocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()?;

    setup_logger();
    let client = DeribitWebSocketClient::default();

    tracing::info!("🚀 Starting Mass Quote Basic Example");

    let mut client = client;
    client.connect().await?;
    tracing::info!("✅ Connected to Deribit WebSocket");

    let (client_id, client_secret) = client.config.get_credentials().unwrap();
    client.authenticate(client_id, client_secret).await?;
    tracing::info!("🔐 Authenticated successfully");

    // Step 1: Try to create MMP group configuration (requires manual activation by Deribit staff)
    tracing::info!("📋 Attempting to set up MMP group configuration...");
    tracing::info!("ℹ️  Note: MMP requires manual activation by Deribit staff for each account");

    let mmp_config = MmpGroupConfig::new(
        "btc_market_making".to_string(),
        10.0, // quantity_limit: 10 BTC max per quote
        5.0,  // delta_limit: 5 BTC (must be < quantity_limit)
        1000, // interval: 1 second
        5000, // frozen_time: 5 seconds after trigger
    )?;

    match client.set_mmp_config(mmp_config).await {
        Ok(()) => {
            tracing::info!("✅ MMP group 'btc_market_making' configured successfully");
        }
        Err(e) => {
            tracing::warn!("⚠️  MMP configuration failed: {}", e);
            tracing::info!("📝 This is expected if MMP is not activated for this account");
            tracing::info!("📞 Contact Deribit support to request MMP activation");
        }
    }

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

    // Create mass quote request without MMP group (since MMP may not be available)
    let mass_quote_request = MassQuoteRequest::new("default".to_string(), quotes.clone())
        .with_quote_id("quote_batch_1".to_string())
        .with_detailed_errors();

    // Step 3: Attempt to place mass quotes
    tracing::info!("ℹ️  Note: Mass Quote feature requires separate activation by Deribit staff");
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
            tracing::warn!("⚠️  Mass quote failed: {}", e);
            tracing::info!(
                "📝 This is expected if Mass Quote feature is not activated for this account"
            );
            tracing::info!("📞 Contact Deribit support to request Mass Quote activation");
            tracing::info!("🔄 Continuing with individual quote placement demonstration...");

            // Demonstrate individual quote placement as fallback
            demonstrate_individual_quotes(&mut client, quotes).await?;
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

    // Step 7: Clean up MMP group (only if MMP was successfully configured)
    tracing::info!("🧽 Attempting to clean up MMP group...");

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
            tracing::warn!(
                "⚠️ Failed to disable MMP group: {} (expected if MMP not activated)",
                e
            );
        }
    }

    tracing::info!("🎯 Mass Quote Basic Example completed successfully!");

    Ok(())
}

/// Demonstrate individual quote placement when mass quote is not available
async fn demonstrate_individual_quotes(
    _client: &mut DeribitWebSocketClient,
    quotes: Vec<Quote>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📋 Demonstrating individual quote placement...");

    for (i, quote) in quotes.iter().enumerate() {
        tracing::info!(
            "   📤 Placing quote {}: {} {} @ {} (Set: {})",
            i + 1,
            quote.side.to_uppercase(),
            quote.amount,
            quote.price,
            quote.quote_set_id.as_deref().unwrap_or("none")
        );

        // In a real implementation, you would place individual orders here
        // This is just a demonstration of the quote structure
    }

    tracing::info!("✅ Individual quote demonstration completed");
    Ok(())
}
