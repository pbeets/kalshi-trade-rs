//! WebSocket streaming client for Kalshi API.
//!
//! This module provides a streaming client using the actor pattern for
//! real-time market data and account updates.
//!
//! # Example
//!
//! ```no_run
//! use kalshi_trade_rs::auth::KalshiConfig;
//! use kalshi_trade_rs::ws::{Channel, KalshiStreamClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = KalshiConfig::from_env()?;
//! let client = KalshiStreamClient::connect(&config).await?;
//! let mut handle = client.handle();
//!
//! // Subscribe to ticker updates
//! let result = handle.subscribe(
//!     &[Channel::Ticker],
//!     Some(&["INXD-25JAN17-B5955"]),
//! ).await?;
//!
//! // Process updates
//! while let Ok(update) = handle.update_receiver.recv().await {
//!     println!("Update: {:?}", update);
//! }
//! # Ok(())
//! # }
//! ```

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
