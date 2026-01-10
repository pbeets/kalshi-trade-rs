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
//! // Subscribe to ticker updates
//! let result = handle.subscribe(
//!     &[Channel::Ticker],
//!     Some(&["INXD-25JAN17-B5955"]),
//! ).await?;
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
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

mod actor;
mod channel;
mod client;
mod command;
mod message;
mod protocol;
mod request_handler;

pub use channel::Channel;
pub use client::{KalshiStreamClient, KalshiStreamHandle};
pub use command::SubscribeResult;
pub use message::{
    Action, FillData, MarketLifecycleData, MarketLifecycleEventType, MarketPositionData,
    OrderbookDeltaData, OrderbookSnapshotData, Side, StreamMessage, StreamUpdate, TickerData,
    TradeData,
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
/// The client automatically responds with Pong frames (handled by tokio-tungstenite).
///
/// This configuration controls *client-initiated* pings, which serve as a backup
/// health check. The defaults are conservative since Kalshi already pings frequently.
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Interval between client-initiated WebSocket ping frames.
    ///
    /// Default: 30 seconds (Kalshi already pings every 10s, so this is a backup).
    pub ping_interval: Duration,

    /// Timeout for pong response before considering connection dead.
    ///
    /// Default: 10 seconds.
    pub ping_timeout: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            // Conservative interval since Kalshi pings every 10s
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
        }
    }
}

/// Connection timeout for initial connection attempts.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Base backoff duration, multiplied by attempt number.
pub(crate) const BACKOFF_BASE: Duration = Duration::from_millis(500);

/// Maximum backoff duration.
pub(crate) const MAX_BACKOFF: Duration = Duration::from_secs(60);
