//! WebSocket streaming client for Kalshi API.
//!
//! This module provides a streaming client with a background session for
//! real-time market data and account updates.
//!
//! # Quick Start
//!
//! ```no_run
//! use kalshi_trade_rs::auth::KalshiConfig;
//! use kalshi_trade_rs::ws::{Channel, ConnectStrategy, KalshiStreamClient, StreamMessage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = KalshiConfig::from_env()?;
//!
//! // Connect with retry strategy for production use
//! let client = KalshiStreamClient::connect_with_strategy(
//!     &config,
//!     ConnectStrategy::Retry,
//! ).await?;
//!
//! let mut handle = client.handle();
//!
//! // Subscribe to ticker updates for specific markets
//! handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
//!
//! // Process updates
//! while let Ok(update) = handle.update_receiver.recv().await {
//!     match &update.msg {
//!         StreamMessage::Ticker(data) => {
//!             println!("{}: {}¢", data.market_ticker, data.price);
//!         }
//!         StreamMessage::ConnectionLost { reason } => {
//!             eprintln!("Connection lost: {}", reason);
//!             break; // Handle reconnection (see below)
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Connection Strategies
//!
//! Two strategies control initial connection behavior:
//!
//! - [`ConnectStrategy::Simple`] - Single attempt, fast-fail on error. Good for testing.
//! - [`ConnectStrategy::Retry`] - Exponential backoff until connected. Recommended for production.
//!
//! **Note**: These strategies only apply to the *initial* connection. Once connected,
//! handling disconnections is your responsibility (see Reconnection Pattern below).
//!
//! # Reconnection Pattern
//!
//! The library does not automatically reconnect after a connection is lost. This is
//! intentional - reconnection policies vary by application (e.g., max retries, backoff
//! strategy, whether to resubscribe to the same markets).
//!
//! When the connection is lost, you'll receive [`StreamMessage::ConnectionLost`] on
//! your update receiver. Implement reconnection like this:
//!
//! ```no_run
//! use kalshi_trade_rs::auth::KalshiConfig;
//! use kalshi_trade_rs::ws::{Channel, ConnectStrategy, KalshiStreamClient, StreamMessage};
//! use std::time::Duration;
//!
//! async fn run_with_reconnect(config: &KalshiConfig) -> Result<(), Box<dyn std::error::Error>> {
//!     let markets = vec!["INXD-25JAN17-B5955", "KXBTC-25DEC31-100000"];
//!     let mut attempt = 0;
//!
//!     loop {
//!         // Connect (Retry strategy handles initial connection retries)
//!         let client = match KalshiStreamClient::connect_with_strategy(
//!             config,
//!             ConnectStrategy::Retry,
//!         ).await {
//!             Ok(c) => {
//!                 attempt = 0; // Reset on successful connect
//!                 c
//!             }
//!             Err(e) => {
//!                 eprintln!("Connection failed: {}", e);
//!                 continue;
//!             }
//!         };
//!
//!         let mut handle = client.handle();
//!
//!         // Resubscribe to markets
//!         for market in &markets {
//!             if let Err(e) = handle.subscribe(Channel::Ticker, &[market]).await {
//!                 eprintln!("Subscribe failed: {}", e);
//!             }
//!         }
//!
//!         // Process updates until disconnection
//!         while let Ok(update) = handle.update_receiver.recv().await {
//!             match &update.msg {
//!                 StreamMessage::Ticker(data) => {
//!                     println!("{}: {}¢", data.market_ticker, data.price);
//!                 }
//!                 StreamMessage::ConnectionLost { reason } => {
//!                     eprintln!("Connection lost: {}", reason);
//!                     break; // Exit inner loop to reconnect
//!                 }
//!                 StreamMessage::Closed { .. } => {
//!                     return Ok(()); // Graceful close, don't reconnect
//!                 }
//!                 _ => {}
//!             }
//!         }
//!
//!         // Backoff before reconnecting
//!         attempt += 1;
//!         let backoff = Duration::from_secs(std::cmp::min(attempt * 2, 60));
//!         eprintln!("Reconnecting in {:?}...", backoff);
//!         tokio::time::sleep(backoff).await;
//!     }
//! }
//! ```
//!
//! See the `examples/stream_reconnect.rs` for a complete working example.
//!
//! # Available Channels
//!
//! Market data channels (require market tickers):
//! - [`Channel::Ticker`] - Price and volume updates
//! - [`Channel::Trade`] - Executed trades
//! - [`Channel::OrderbookDelta`] - Orderbook changes
//! - [`Channel::MarketLifecycle`] - Market status changes
//!
//! User channels (no market tickers needed):
//! - [`Channel::Fill`] - Your executed fills
//! - [`Channel::MarketPositions`] - Position changes
//! - [`Channel::Communications`] - RFQ/quote updates
//! - [`Channel::Multivariate`] - Multivariate event updates
//!
//! # Subscription Management
//!
//! ```no_run
//! # use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle};
//! # async fn example(handle: &mut KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
//! // Subscribe to markets
//! handle.subscribe(Channel::Ticker, &["MARKET-A", "MARKET-B"]).await?;
//!
//! // Add more markets to existing subscription
//! handle.subscribe(Channel::Ticker, &["MARKET-C"]).await?;
//!
//! // Remove specific markets
//! handle.unsubscribe(Channel::Ticker, &["MARKET-A"]).await?;
//!
//! // Unsubscribe from entire channel
//! handle.unsubscribe_all(Channel::Ticker).await?;
//!
//! // Check current subscriptions
//! println!("Markets: {:?}", handle.markets(Channel::Ticker));
//! println!("Subscribed: {}", handle.is_subscribed(Channel::Ticker));
//! # Ok(())
//! # }
//! ```

mod channel;
mod client;
mod command;
mod message;
mod protocol;
mod session;

use std::time::Duration;

pub use channel::Channel;
pub use client::{KalshiStreamClient, KalshiStreamHandle};
pub use command::{SubscribeResult, UnsubscribeResult};
pub use message::{
    Action, FillData, MarketLifecycleData, MarketLifecycleEventType, MarketPositionData,
    MultivariateLookupData, MveLeg, OrderbookDeltaData, OrderbookSnapshotData, Side, StreamMessage,
    StreamUpdate, TickerData, TradeData,
};

/// Connection strategy for the WebSocket client.
///
/// Controls how the client handles initial connection and reconnection attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectStrategy {
    /// Single connection attempt. Fast-fail on error.
    ///
    /// Recommended for testing or when you want explicit control over retries.
    #[default]
    Simple,

    /// Retry with exponential backoff, capped at 60 seconds.
    ///
    /// Recommended for production use. Will retry indefinitely until connected.
    Retry,
}

/// Configuration for connection health monitoring.
///
/// # Kalshi WebSocket Behavior
///
/// Kalshi sends Ping frames with body "heartbeat", but only on **idle connections**.
/// When there's activity (messages being sent or received), Kalshi may stop sending
/// heartbeat pings. The client also sends its own Ping frames to verify the
/// connection is alive.
///
/// # Two-Way Health Monitoring
///
/// 1. **Client → Server**: Client sends pings at `ping_interval`, expects pong
///    within `ping_timeout`.
/// 2. **Activity Monitoring**: If no messages are sent or received within
///    `server_ping_timeout`, the connection is considered dead. This handles
///    the case where Kalshi stops sending heartbeats during active sessions.
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Interval between client-initiated WebSocket ping frames.
    ///
    /// Default: 30 seconds.
    pub ping_interval: Duration,

    /// Timeout for pong response to client-initiated pings.
    ///
    /// Default: 10 seconds.
    pub ping_timeout: Duration,

    /// Timeout for any activity (messages sent or received).
    ///
    /// Kalshi only sends heartbeat pings on idle connections. When there's
    /// activity (subscriptions, data flow, etc.), Kalshi may stop sending pings.
    /// This timeout detects stale connections by tracking all activity.
    ///
    /// Default: 30 seconds.
    pub server_ping_timeout: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            // Activity timeout - if no messages sent/received in this duration,
            // the connection is considered dead
            server_ping_timeout: Duration::from_secs(30),
        }
    }
}

/// Connection timeout for initial connection attempts.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Base backoff duration, multiplied by attempt number.
pub(crate) const BACKOFF_BASE: Duration = Duration::from_millis(500);

/// Maximum backoff duration.
pub(crate) const MAX_BACKOFF: Duration = Duration::from_secs(60);
