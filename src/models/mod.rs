//! Data models for the Kalshi API.
//!
//! All monetary values are in cents unless noted otherwise.
//! Fields ending in `_dollars` are fixed-point dollar strings.

mod balance;
mod common;
mod exchange;
mod fill;
mod market;
mod order;
mod position;
pub(crate) mod query;

// Re-export all public types
pub use balance::BalanceResponse;
pub use common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side, cents_to_dollars};
pub use exchange::{
    Announcement, AnnouncementStatus, AnnouncementType, ExchangeAnnouncementsResponse,
    ExchangeSchedule, ExchangeScheduleResponse, ExchangeStatusResponse, MaintenanceWindow,
    StandardHoursPeriod, TradingSession, UserDataTimestampResponse,
};
pub use fill::{Fill, FillsResponse, GetFillsParams};
pub use market::{
    GetMarketsParams, GetOrderbookParams, GetTradesParams, Market, MarketFilterStatus,
    MarketResponse, MarketResult, MarketStatus, MarketType, MarketsResponse, MveFilter, Orderbook,
    OrderbookResponse, PriceLevelDollars, PriceRange, StrikeType, TakerSide, Trade, TradesResponse,
};
pub use order::{GetOrdersParams, Order, OrdersResponse};
pub use position::{EventPosition, GetPositionsParams, MarketPosition, PositionsResponse};
