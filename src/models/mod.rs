//! Data models for the Kalshi API.
//!
//! All monetary values are in cents unless noted otherwise.
//! Fields ending in `_dollars` are fixed-point dollar strings.

mod balance;
mod common;
mod event;
mod exchange;
mod fill;
mod market;
mod order;
mod position;
pub(crate) mod query;
mod search;

// Re-export all public types
pub use balance::BalanceResponse;
pub use common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side, cents_to_dollars};
pub use event::{
    Event, EventResponse, EventStatus, EventsResponse, GetEventParams, GetEventsParams, Milestone,
};
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
pub use order::{
    AmendOrderRequest, AmendOrderResponse, BatchCancelOrderResult, BatchCancelOrdersRequest,
    BatchCancelOrdersResponse, BatchCreateOrdersRequest, BatchCreateOrdersResponse,
    BatchOrderError, BatchOrderResult, CancelOrderResponse, CreateOrderRequest,
    DecreaseOrderRequest, GetOrdersParams, Order, OrderResponse, OrdersResponse, TimeInForce,
};
pub use position::{EventPosition, GetPositionsParams, MarketPosition, PositionsResponse};
pub use search::{
    CompetitionFilter, FiltersBySportResponse, SportFilter, TagsByCategoriesResponse,
};
