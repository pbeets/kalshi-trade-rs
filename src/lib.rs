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
pub mod batch;
pub mod client;
pub mod error;
pub mod models;
pub mod ws;

// Re-export commonly used types at the crate root
pub use auth::KalshiConfig;
pub use client::{Environment, HttpClient, KalshiClient};
pub use error::{Error, MAX_BATCH_SIZE, Result};
pub use models::{
    AcceptQuoteRequest, Action, AmendOrderRequest, AmendOrderResponse, Announcement,
    AnnouncementStatus, AnnouncementType, ApiKey, ApiKeysResponse, BalanceResponse,
    BatchCancelOrderResult, BatchCancelOrdersRequest, BatchCancelOrdersResponse,
    BatchCandlesticksResponse, BatchCreateOrdersRequest, BatchCreateOrdersResponse,
    BatchLiveDataResponse, BatchOrderError, BatchOrderResult, CancelOrderResponse, Candlestick,
    CandlestickPeriod, CandlesticksResponse, CommunicationsIdResponse, CompetitionFilter,
    CreateApiKeyRequest, CreateApiKeyResponse, CreateOrderGroupRequest, CreateOrderGroupResponse,
    CreateOrderRequest, CreateQuoteRequest, CreateRfqRequest, DecreaseOrderRequest,
    DeleteApiKeyResponse, Event, EventPosition, EventResponse, EventStatus, EventsResponse,
    ExchangeAnnouncementsResponse, ExchangeSchedule, ExchangeScheduleResponse,
    ExchangeStatusResponse, FeeChangesResponse, FeeType, Fill, FillsResponse,
    FiltersBySportResponse, GenerateApiKeyRequest, GenerateApiKeyResponse,
    GetBatchCandlesticksParams, GetBatchLiveDataParams, GetCandlesticksParams, GetEventParams,
    GetEventsParams, GetFcmOrdersParams, GetFcmPositionsParams, GetFeeChangesParams,
    GetFillsParams, GetIncentiveProgramsParams, GetMarketsParams, GetMilestonesParams,
    GetOrderGroupResponse, GetOrderGroupsParams, GetOrderbookParams, GetOrdersParams,
    GetPositionsParams, GetQueuePositionsParams, GetQuoteResponse, GetRfqResponse, GetSeriesParams,
    GetSettlementsParams, GetStructuredTargetsParams, GetTradesParams, IncentiveProgram,
    IncentiveProgramsResponse, ListQuotesParams, ListQuotesResponse, ListRfqsParams,
    ListRfqsResponse, LiveData, LiveDataResponse, MaintenanceWindow, Market, MarketCandlesticks,
    MarketFilterStatus, MarketPosition, MarketResponse, MarketResult, MarketStatus, MarketType,
    MarketsResponse, Milestone, MilestoneInfo, MilestoneResponse, MilestonesResponse, MveFilter,
    OhlcData, Order, OrderGroupSummary, OrderGroupsResponse, OrderQueuePositionResponse,
    OrderResponse, OrderStatus, OrderType, Orderbook, OrderbookResponse, OrdersResponse,
    PositionsResponse, PriceLevelDollars, PriceOhlcData, PriceRange, QueuePosition,
    QueuePositionsResponse, Quote, QuoteResponse, Rfq, RfqResponse, SelfTradePreventionType,
    Series, SeriesFeeChange, SeriesListResponse, SeriesResponse, Settlement, SettlementStatus,
    SettlementsResponse, Side, SportFilter, StandardHoursPeriod, StrikeType, StructuredTarget,
    StructuredTargetResponse, StructuredTargetsResponse, TagsByCategoriesResponse, TakerSide,
    TimeInForce, Trade, TradesResponse, TradingSession, UserDataTimestampResponse,
    cents_to_dollars,
};

// Re-export WebSocket types for convenience
pub use ws::{
    Channel, ConnectStrategy, KalshiStreamClient, KalshiStreamHandle, StreamMessage, StreamUpdate,
    SubscribeResult,
};

// Re-export batch management types
pub use batch::{
    AggregatedCancelResponse, AggregatedCreateResponse, BatchManager, BatchManagerBuilder,
    BatchOperationResult, RateLimitTier, RetryConfig,
};
