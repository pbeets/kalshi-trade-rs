//! WebSocket command types for the Kalshi streaming API.
//!
//! This module defines the commands that can be sent through the WebSocket
//! connection to subscribe/unsubscribe from market data channels.

use tokio::sync::oneshot;

/// Action for updating a subscription's market list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Add markets to the subscription.
    AddMarkets,
    /// Remove markets from the subscription.
    DeleteMarkets,
}

/// A successful channel subscription.
#[derive(Debug, Clone)]
pub struct ChannelSubscription {
    /// The channel that was subscribed to.
    pub channel: String,
    /// The subscription ID assigned by the server.
    pub sid: i64,
}

/// A failed channel subscription.
#[derive(Debug, Clone)]
pub struct ChannelError {
    /// The channel that failed to subscribe.
    pub channel: Option<String>,
    /// Error code from the server.
    pub code: String,
    /// Error message from the server.
    pub message: String,
}

/// Result of a subscription operation.
///
/// Kalshi processes each channel in a subscribe request independently,
/// so some channels may succeed while others fail. This struct captures
/// both successful and failed subscriptions.
#[derive(Debug, Clone)]
pub struct SubscribeResult {
    /// Successfully subscribed channels with their SIDs.
    pub successful: Vec<ChannelSubscription>,
    /// Channels that failed to subscribe.
    pub failed: Vec<ChannelError>,
}

impl SubscribeResult {
    /// Returns all subscription IDs from successful subscriptions.
    pub fn sids(&self) -> Vec<i64> {
        self.successful.iter().map(|s| s.sid).collect()
    }

    /// Returns true if all requested channels were subscribed successfully.
    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty()
    }

    /// Returns true if at least one channel was subscribed successfully.
    pub fn has_any_success(&self) -> bool {
        !self.successful.is_empty()
    }
}

/// Result of an unsubscription operation.
///
/// Contains the subscription IDs that were successfully unsubscribed.
#[derive(Debug, Clone)]
pub struct UnsubscribeResult {
    /// Subscription IDs that were unsubscribed.
    pub sids: Vec<i64>,
}

impl UnsubscribeResult {
    /// Returns the subscription IDs that were unsubscribed.
    pub fn sids(&self) -> &[i64] {
        &self.sids
    }

    /// Returns the number of subscriptions that were unsubscribed.
    pub fn count(&self) -> usize {
        self.sids.len()
    }
}

/// Sharding configuration for high-throughput communications channel.
///
/// When subscribing to the communications channel for RFQ/quote updates,
/// sharding can be used to distribute traffic across multiple connections.
#[derive(Debug, Clone, Default)]
pub struct CommunicationsSharding {
    /// Number of shards to distribute traffic across.
    /// Each shard receives approximately 1/shard_factor of the traffic.
    pub shard_factor: Option<i32>,
    /// This connection's shard key (0 to shard_factor-1).
    /// Determines which subset of traffic this connection receives.
    pub shard_key: Option<i32>,
}

impl CommunicationsSharding {
    /// Create a new sharding configuration.
    ///
    /// # Arguments
    ///
    /// * `shard_factor` - Number of shards (e.g., 4 for 4 connections).
    /// * `shard_key` - This connection's shard (0 to shard_factor-1).
    ///
    /// # Example
    ///
    /// ```
    /// use kalshi_trade_rs::ws::CommunicationsSharding;
    ///
    /// // Connection 1 of 4 shards
    /// let sharding = CommunicationsSharding::new(4, 0);
    /// // Connection 2 of 4 shards
    /// let sharding = CommunicationsSharding::new(4, 1);
    /// ```
    #[must_use]
    pub fn new(shard_factor: i32, shard_key: i32) -> Self {
        Self {
            shard_factor: Some(shard_factor),
            shard_key: Some(shard_key),
        }
    }
}

/// Commands that can be sent to the WebSocket stream handler.
#[derive(Debug)]
pub enum StreamCommand {
    /// Subscribe to one or more channels for specified market tickers.
    Subscribe {
        /// Channel names to subscribe to (e.g., "orderbook_delta", "ticker", "trade").
        channels: Vec<String>,
        /// Market ticker symbols to subscribe to.
        market_tickers: Vec<String>,
        /// Optional sharding config for communications channel.
        sharding: Option<CommunicationsSharding>,
        /// Oneshot channel to receive the subscription result.
        response: oneshot::Sender<Result<SubscribeResult, String>>,
    },

    /// Unsubscribe from one or more subscriptions by their subscription IDs.
    Unsubscribe {
        /// Subscription IDs to unsubscribe from.
        sids: Vec<i64>,
        /// Oneshot channel to receive the unsubscription result.
        response: oneshot::Sender<Result<UnsubscribeResult, String>>,
    },

    /// Update markets for an existing subscription.
    UpdateSubscription {
        /// Subscription ID to update.
        sid: i64,
        /// Markets to add or remove.
        markets: Vec<String>,
        /// Action to perform.
        action: UpdateAction,
        /// Oneshot channel to receive the updated markets list.
        response: oneshot::Sender<Result<Vec<String>, String>>,
    },

    /// Close the WebSocket connection gracefully.
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communications_sharding_new() {
        let sharding = CommunicationsSharding::new(4, 2);
        assert_eq!(sharding.shard_factor, Some(4));
        assert_eq!(sharding.shard_key, Some(2));
    }

    #[test]
    fn test_communications_sharding_default() {
        let sharding = CommunicationsSharding::default();
        assert_eq!(sharding.shard_factor, None);
        assert_eq!(sharding.shard_key, None);
    }
}
