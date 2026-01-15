//! WebSocket streaming client for Kalshi API.
//!
//! This module provides a streaming client using the actor pattern for
//! real-time market data and account updates.
//!
//! # Example
//!
//! ```no_run
//! use kalshi_trade_rs::auth::KalshiConfig;
//! use kalshi_trade_rs::ws::{Channel, ConnectStrategy, KalshiStreamClient};
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
//! // Add more markets (automatically uses add_markets under the hood)
//! handle.subscribe(Channel::Ticker, &["KXBTC-25DEC31-100000"]).await?;
//!
//! // Check what markets we're subscribed to
//! println!("Ticker markets: {:?}", handle.markets(Channel::Ticker));
//!
//! // Process updates - handle disconnection events
//! while let Ok(update) = handle.update_receiver.recv().await {
//!     match &update.msg {
//!         kalshi_trade_rs::ws::StreamMessage::Closed { reason } => {
//!             println!("Connection closed: {}", reason);
//!             break;
//!         }
//!         kalshi_trade_rs::ws::StreamMessage::ConnectionLost { reason } => {
//!             eprintln!("Connection lost: {}", reason);
//!             break; // Reconnect with backoff here
//!         }
//!         _ => println!("Update: {:?}", update),
//!     }
//! }
//!
//! // Unsubscribe from specific markets
//! handle.unsubscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
//!
//! // Or unsubscribe from entire channel
//! handle.unsubscribe_all(Channel::Ticker).await?;
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

mod channel;
mod client;
mod command;
mod message;
mod protocol;
mod session;

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
/// Kalshi sends Ping frames every 10 seconds with body "heartbeat".
/// The client responds with Pong frames. The client also sends its own
/// Ping frames to verify the connection is alive in both directions.
///
/// # Two-Way Health Monitoring
///
/// 1. **Client → Server**: Client sends pings at `ping_interval`, expects pong
///    within `ping_timeout`.
/// 2. **Server → Client**: Client expects Kalshi pings at ~10s intervals. If no
///    ping received within `server_ping_timeout`, connection is considered dead.
///
/// # Startup Grace Period
///
/// The server ping timeout monitoring only begins after the first ping is
/// received from Kalshi. This prevents false timeouts at connection startup
/// if there's a delay before Kalshi sends its first ping.
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

    /// Timeout for receiving pings from Kalshi server.
    ///
    /// Kalshi sends pings every 10 seconds. If no ping is received within
    /// this duration (after the first ping has been received), the connection
    /// is considered dead.
    ///
    /// Default: 30 seconds (allows for up to 2 missed pings before timeout).
    pub server_ping_timeout: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            // Kalshi pings every 10s, so 30s allows for up to 2 missed pings
            // before considering the connection dead
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
