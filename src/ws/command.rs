//! WebSocket command types for the Kalshi streaming API.
//!
//! This module defines the commands that can be sent through the WebSocket
//! connection to subscribe/unsubscribe from market data channels.

use serde_json::Value as JsonValue;
use tokio::sync::oneshot;

/// Result of a subscription operation.
#[derive(Debug, Clone)]
pub struct SubscribeResult {
    /// Subscription IDs returned by the server for each successful subscription.
    pub sids: Vec<i64>,
    /// The raw JSON response from the server.
    pub response: JsonValue,
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
