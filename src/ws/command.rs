//! WebSocket command types for the Kalshi streaming API.
//!
//! This module defines the commands that can be sent through the WebSocket
//! connection to subscribe/unsubscribe from market data channels.

use serde_json::Value as JsonValue;
use tokio::sync::oneshot;

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

/// Commands that can be sent to the WebSocket stream handler.
#[derive(Debug)]
pub enum StreamCommand {
    /// Subscribe to one or more channels for specified market tickers.
    Subscribe {
        /// Channel names to subscribe to (e.g., "orderbook_delta", "ticker", "trade").
        channels: Vec<String>,
        /// Market ticker symbols to subscribe to.
        market_tickers: Vec<String>,
        /// Oneshot channel to receive the subscription result.
        response: oneshot::Sender<Result<SubscribeResult, String>>,
    },

    /// Unsubscribe from one or more subscriptions by their subscription IDs.
    Unsubscribe {
        /// Subscription IDs to unsubscribe from.
        sids: Vec<i64>,
        /// Oneshot channel to receive the unsubscription result.
        response: oneshot::Sender<Result<JsonValue, String>>,
    },

    /// Close the WebSocket connection gracefully.
    Close,
}
