//! Subscription management for WebSocket client

use pretty_simple_display::DisplaySimple;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-export subscription channel from deribit-base
use crate::model::SubscriptionChannel;

/// Subscription information
#[derive(Clone, Serialize, Deserialize, DisplaySimple)]
pub struct Subscription {
    /// Channel name
    pub channel: String,
    /// Channel type
    pub channel_type: SubscriptionChannel,
    /// Instrument name (if applicable)
    pub instrument: Option<String>,
    /// Whether subscription is active
    pub active: bool,
}

/// Subscription manager
#[derive(Debug, Default)]
pub struct SubscriptionManager {
    subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    /// Add a subscription
    pub fn add_subscription(
        &mut self,
        channel: String,
        channel_type: SubscriptionChannel,
        instrument: Option<String>,
    ) {
        let subscription = Subscription {
            channel: channel.clone(),
            channel_type,
            instrument,
            active: true,
        };
        self.subscriptions.insert(channel, subscription);
    }

    /// Add a subscription from channel type
    pub fn add_subscription_from_channel(&mut self, channel_type: SubscriptionChannel) {
        let channel = channel_type.to_string();
        let instrument = match &channel_type {
            SubscriptionChannel::Ticker(inst)
            | SubscriptionChannel::OrderBook(inst)
            | SubscriptionChannel::Trades(inst)
            | SubscriptionChannel::IncrementalTicker(inst) => Some(inst.clone()),
            SubscriptionChannel::ChartTrades { instrument, .. }
            | SubscriptionChannel::GroupedOrderBook { instrument, .. } => Some(instrument.clone()),
            SubscriptionChannel::UserChanges { instrument, .. } => Some(instrument.clone()),
            SubscriptionChannel::TradesByKind { currency, .. } => Some(currency.clone()),
            SubscriptionChannel::PriceIndex(index_name)
            | SubscriptionChannel::PriceRanking(index_name)
            | SubscriptionChannel::PriceStatistics(index_name)
            | SubscriptionChannel::VolatilityIndex(index_name) => Some(index_name.clone()),
            SubscriptionChannel::EstimatedExpirationPrice(inst)
            | SubscriptionChannel::MarkPrice(inst)
            | SubscriptionChannel::Funding(inst)
            | SubscriptionChannel::Quote(inst) => Some(inst.clone()),
            SubscriptionChannel::Perpetual { instrument, .. } => Some(instrument.clone()),
            SubscriptionChannel::InstrumentState { currency, .. } => Some(currency.clone()),
            SubscriptionChannel::BlockRfqTrades(currency)
            | SubscriptionChannel::BlockTradeConfirmationsByCurrency(currency) => {
                Some(currency.clone())
            }
            SubscriptionChannel::UserMmpTrigger(index_name) => Some(index_name.clone()),
            SubscriptionChannel::UserOrders
            | SubscriptionChannel::UserTrades
            | SubscriptionChannel::UserPortfolio
            | SubscriptionChannel::PlatformState
            | SubscriptionChannel::PlatformStatePublicMethods
            | SubscriptionChannel::BlockTradeConfirmations
            | SubscriptionChannel::UserAccessLog
            | SubscriptionChannel::UserLock
            | SubscriptionChannel::Unknown(_) => None,
        };
        self.add_subscription(channel, channel_type, instrument);
    }

    /// Remove a subscription
    pub fn remove_subscription(&mut self, channel: &str) -> Option<Subscription> {
        self.subscriptions.remove(channel)
    }

    /// Get all active subscriptions
    pub fn active_subscriptions(&self) -> Vec<&Subscription> {
        self.subscriptions.values().filter(|s| s.active).collect()
    }

    /// Get subscription by channel
    pub fn get_subscription(&self, channel: &str) -> Option<&Subscription> {
        self.subscriptions.get(channel)
    }

    /// Mark subscription as inactive
    pub fn deactivate_subscription(&mut self, channel: &str) {
        if let Some(subscription) = self.subscriptions.get_mut(channel) {
            subscription.active = false;
        }
    }

    /// Reactivate all subscriptions
    pub fn reactivate_all(&mut self) {
        for subscription in self.subscriptions.values_mut() {
            subscription.active = true;
        }
    }

    /// Deactivate all subscriptions in place. Entries are preserved so a
    /// later call to `reactivate_all` can restore them on reconnect.
    pub fn deactivate_all(&mut self) {
        for subscription in self.subscriptions.values_mut() {
            subscription.active = false;
        }
    }

    /// Clear all subscriptions
    pub fn clear(&mut self) {
        self.subscriptions.clear();
    }

    /// Get all channel names
    pub fn get_all_channels(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }

    /// Get active channel names
    pub fn get_active_channels(&self) -> Vec<String> {
        self.subscriptions
            .iter()
            .filter(|(_, sub)| sub.active)
            .map(|(channel, _)| channel.clone())
            .collect()
    }
}

// Custom Debug implementation for Subscription
impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("channel", &self.channel)
            .field("channel_type", &self.channel_type)
            .field("instrument", &self.instrument)
            .field("active", &self.active)
            .finish()
    }
}
