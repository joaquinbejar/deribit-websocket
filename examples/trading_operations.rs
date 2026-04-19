//! Trading Operations Example
//!
//! This example demonstrates the trading features added in v0.2.0:
//! - `buy()` - Place a buy order
//! - `sell()` - Place a sell order
//! - `edit()` - Modify an existing order
//! - `cancel()` - Cancel a specific order
//! - `cancel_all()` - Cancel all orders
//! - `cancel_all_by_currency()` - Cancel all orders for a currency
//! - `cancel_all_by_instrument()` - Cancel all orders for an instrument
//!
//! These features were added in issue #5.
//!
//! **NOTE**: This example requires authentication with valid API credentials.
//! Set the following environment variables:
//! - DERIBIT_CLIENT_ID
//! - DERIBIT_CLIENT_SECRET

use deribit_websocket::config::WebSocketConfig;
use deribit_websocket::model::trading::{OrderRequest, TimeInForce};
use deribit_websocket::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the rustls crypto provider that matches the active TLS feature.
    deribit_websocket::install_default_crypto_provider()
        .map_err(|e| format!("Failed to install crypto provider: {e}"))?;

    // Initialize logging
    unsafe {
        std::env::set_var("DERIBIT_LOG_LEVEL", "DEBUG");
    }
    setup_logger();

    tracing::info!("🚀 Trading Operations Example");
    tracing::info!("Demonstrating: buy, sell, edit, cancel orders");

    // Load credentials from environment
    dotenv::dotenv().ok();
    let client_id = env::var("DERIBIT_CLIENT_ID").expect("DERIBIT_CLIENT_ID must be set");
    let client_secret =
        env::var("DERIBIT_CLIENT_SECRET").expect("DERIBIT_CLIENT_SECRET must be set");

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

    // ==========================================================================
    // Authenticate (required for trading operations)
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔐 Authenticating...");

    match client.authenticate(&client_id, &client_secret).await {
        Ok(auth_response) => {
            tracing::info!("✅ Authentication successful!");
            tracing::info!("   Token type: {}", auth_response.token_type);
            tracing::info!("   Expires in: {} seconds", auth_response.expires_in);
            tracing::info!("   Scope: {}", auth_response.scope);
        }
        Err(e) => {
            tracing::error!("❌ Authentication failed: {}", e);
            client.disconnect().await?;
            return Err(e.into());
        }
    }

    // ==========================================================================
    // 1. Place a BUY order (limit order)
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📈 Placing a BUY limit order...");

    let buy_request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 10.0, 50000.0)
        .with_time_in_force(TimeInForce::GoodTilCancelled)
        .with_label("example_buy_order".to_string());

    tracing::info!("   Instrument: BTC-PERPETUAL");
    tracing::info!("   Amount: 10 USD");
    tracing::info!("   Price: $50,000 (limit)");
    tracing::info!("   Type: Limit, GTC");

    match client.buy(buy_request).await {
        Ok(order_response) => {
            tracing::info!("✅ Buy order placed successfully!");
            tracing::info!("   Order ID: {}", order_response.order.order_id);
            tracing::info!("   State: {}", order_response.order.order_state);
            tracing::info!("   Direction: {:?}", order_response.order.direction);

            // Store order ID for later operations
            let order_id = order_response.order.order_id.clone();

            // ==========================================================================
            // 2. Edit the order (change price)
            // ==========================================================================
            tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            tracing::info!("✏️ Editing the order (changing price)...");

            let edit_request =
                deribit_websocket::model::trading::EditOrderRequest::new(order_id.clone(), 10.0)
                    .with_price(49000.0); // New price

            match client.edit(edit_request).await {
                Ok(edit_response) => {
                    tracing::info!("✅ Order edited successfully!");
                    tracing::info!("   New Order ID: {}", edit_response.order.order_id);
                    tracing::info!("   New Price: {:?}", edit_response.order.price);
                }
                Err(e) => {
                    tracing::error!("❌ Edit failed: {}", e);
                }
            }

            // ==========================================================================
            // 3. Cancel the order
            // ==========================================================================
            tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            tracing::info!("❌ Cancelling the order...");

            match client.cancel(&order_id).await {
                Ok(cancel_response) => {
                    tracing::info!("✅ Order cancelled successfully!");
                    tracing::info!("   Order ID: {}", cancel_response.order_id);
                    tracing::info!("   State: {}", cancel_response.order_state);
                }
                Err(e) => {
                    tracing::error!("❌ Cancel failed: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Buy order failed: {}", e);
        }
    }

    // ==========================================================================
    // 4. Place a SELL order
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("📉 Placing a SELL limit order...");

    let sell_request = OrderRequest::limit("BTC-PERPETUAL".to_string(), 10.0, 100000.0)
        .with_time_in_force(TimeInForce::GoodTilCancelled)
        .with_label("example_sell_order".to_string());

    match client.sell(sell_request).await {
        Ok(order_response) => {
            tracing::info!("✅ Sell order placed successfully!");
            tracing::info!("   Order ID: {}", order_response.order.order_id);
            tracing::info!("   State: {}", order_response.order.order_state);
        }
        Err(e) => {
            tracing::error!("❌ Sell order failed: {}", e);
        }
    }

    // ==========================================================================
    // 5. Cancel all orders by instrument
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🗑️ Cancelling all orders for BTC-PERPETUAL...");

    match client.cancel_all_by_instrument("BTC-PERPETUAL").await {
        Ok(count) => {
            tracing::info!("✅ Cancelled {} orders for BTC-PERPETUAL", count);
        }
        Err(e) => {
            tracing::error!("❌ Cancel all by instrument failed: {}", e);
        }
    }

    // ==========================================================================
    // 6. Cancel all orders by currency
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🗑️ Cancelling all BTC orders...");

    match client.cancel_all_by_currency("BTC").await {
        Ok(count) => {
            tracing::info!("✅ Cancelled {} BTC orders", count);
        }
        Err(e) => {
            tracing::error!("❌ Cancel all by currency failed: {}", e);
        }
    }

    // ==========================================================================
    // 7. Cancel ALL orders
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🗑️ Cancelling ALL orders...");

    match client.cancel_all().await {
        Ok(count) => {
            tracing::info!("✅ Cancelled {} orders in total", count);
        }
        Err(e) => {
            tracing::error!("❌ Cancel all failed: {}", e);
        }
    }

    // ==========================================================================
    // Cleanup
    // ==========================================================================
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🔌 Disconnecting...");
    client.disconnect().await?;
    tracing::info!("✅ Disconnected successfully");

    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("✅ Trading operations example completed!");

    Ok(())
}
