//! Orderbook aggregation and state management.
//!
//! This module provides tools for maintaining live orderbook state from
//! WebSocket delta updates. The main type is [`OrderbookAggregator`], which
//! processes orderbook snapshots and deltas to maintain current market state.
//!
//! # Quick Start
//!
//! ```no_run
//! use kalshi_trade_rs::auth::KalshiConfig;
//! use kalshi_trade_rs::orderbook::OrderbookAggregator;
//! use kalshi_trade_rs::ws::{Channel, KalshiStreamClient};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = KalshiConfig::from_env()?;
//! let client = KalshiStreamClient::connect(&config).await?;
//! let mut handle = client.handle();
//!
//! // Subscribe to orderbook updates
//! handle.subscribe(Channel::OrderbookDelta, &["TICKER-1"]).await?;
//!
//! // Create aggregator and spawn processor
//! let aggregator = OrderbookAggregator::new();
//! let agg_clone = aggregator.clone();
//! tokio::spawn(async move {
//!     agg_clone.process_updates(handle).await;
//! });
//!
//! // Query orderbook state
//! loop {
//!     if let Some(summary) = aggregator.summary("TICKER-1") {
//!         if summary.initialized {
//!             println!("Spread: {:?} cents", summary.spread);
//!             println!("Best bid: {:?}", summary.best_bid);
//!             println!("Best ask: {:?}", summary.best_ask);
//!         }
//!     }
//!     tokio::time::sleep(Duration::from_millis(100)).await;
//! }
//! # }
//! ```
//!
//! # How It Works
//!
//! 1. Subscribe to [`Channel::OrderbookDelta`](crate::ws::Channel::OrderbookDelta) for markets
//! 2. WebSocket sends an initial [`OrderbookSnapshotData`](crate::ws::OrderbookSnapshotData) with full state
//! 3. WebSocket sends [`OrderbookDeltaData`](crate::ws::OrderbookDeltaData) for incremental updates
//! 4. Aggregator maintains state and provides query methods
//!
//! # Pull vs Push Access
//!
//! The aggregator supports two access patterns:
//!
//! ## Pull-based (polling)
//!
//! Query state at your own cadence using methods like [`summary()`](OrderbookAggregator::summary),
//! [`best_bid()`](OrderbookAggregator::best_bid), [`spread()`](OrderbookAggregator::spread), etc.
//!
//! ## Push-based (streaming)
//!
//! Subscribe to updates via [`update_receiver()`](OrderbookAggregator::update_receiver) to
//! react to every orderbook change as it happens.
//!
//! # YES/NO Price Relationship
//!
//! In Kalshi prediction markets:
//! - YES bid at 45 = someone will buy YES at 45¢
//! - NO bid at 55 = someone will sell YES at 45¢ (since 100 - 55 = 45)
//! - Best YES ask = 100 - best NO bid price
//!
//! The aggregator handles this conversion automatically when reporting
//! best ask prices.

mod aggregator;
mod state;

pub use aggregator::{
    OrderbookAggregator, OrderbookDelta, OrderbookSummary, OrderbookUpdate, SequenceGap,
};
