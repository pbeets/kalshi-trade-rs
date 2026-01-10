//! Kalshi Trading API Client for Rust.
//!
//! This crate provides an async client for the Kalshi trading API,
//! including both REST endpoints and WebSocket streaming.
//!
//! # Quick Start
//!
//! ```ignore
//! use kalshi_trade_rs::{cents_to_dollars, KalshiClient, KalshiConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let config = KalshiConfig::from_env()?;
//!
//!     // Create the client
//!     let client = KalshiClient::new(config)?;
//!
//!     // Get account balance
//!     let balance = client.get_balance().await?;
//!     println!("Balance: ${:.2}", cents_to_dollars(balance.balance));
//!
//!     // Get positions
//!     let positions = client.get_positions().await?;
//!     for pos in positions.market_positions {
//!         println!("{}: {} contracts", pos.ticker, pos.position);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables
//!
//! The following environment variables are used for configuration:
//!
//! - `KALSHI_ENV`: Either "demo" or "prod" (default: "demo")
//! - `KALSHI_API_KEY_ID`: Your API key ID
//! - `KALSHI_PRIVATE_KEY_PATH`: Path to your RSA private key PEM file

mod api;
pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod ws;

// Re-export commonly used types at the crate root
pub use auth::KalshiConfig;
pub use client::{Environment, HttpClient, KalshiClient};
pub use error::{Error, Result};
pub use models::{
    Action, Announcement, AnnouncementStatus, AnnouncementType, BalanceResponse, EventPosition,
    ExchangeAnnouncementsResponse, ExchangeSchedule, ExchangeScheduleResponse,
    ExchangeStatusResponse, Fill, FillsResponse, GetFillsParams, GetMarketsParams,
    GetOrderbookParams, GetOrdersParams, GetPositionsParams, GetTradesParams, MaintenanceWindow,
    Market, MarketFilterStatus, MarketPosition, MarketResponse, MarketResult, MarketStatus,
    MarketType, MarketsResponse, MveFilter, Order, OrderStatus, OrderType, Orderbook,
    OrderbookResponse, OrdersResponse, PositionsResponse, PriceRange, SelfTradePreventionType,
    Side, StandardHoursPeriod, StrikeType, TakerSide, Trade, TradesResponse, TradingSession,
    UserDataTimestampResponse, cents_to_dollars,
};

// Re-export WebSocket types for convenience
pub use ws::{
    Channel, ConnectStrategy, KalshiStreamClient, KalshiStreamHandle, StreamMessage, StreamUpdate,
    SubscribeResult,
};
