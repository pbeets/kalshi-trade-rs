//! Data models for the Kalshi API.
//!
//! All monetary values are in cents unless noted otherwise.
//! Fields ending in `_dollars` are fixed-point dollar strings.

mod balance;
mod common;
mod communications;
mod event;
mod exchange;
mod fill;
mod market;
mod order;
mod order_group;
mod position;
pub(crate) mod query;
mod search;
mod series;
mod settlement;

// Re-export all public types
pub use balance::BalanceResponse;
pub use common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side, cents_to_dollars};
pub use communications::{
    AcceptQuoteRequest, CreateQuoteRequest, CreateRfqRequest, GetQuoteResponse, GetRfqResponse,
    ListQuotesParams, ListQuotesResponse, ListRfqsParams, ListRfqsResponse, Quote, QuoteResponse,
    Rfq, RfqResponse,
};
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
    BatchCandlesticksResponse, Candlestick, CandlestickPeriod, CandlesticksResponse,
    GetBatchCandlesticksParams, GetCandlesticksParams, GetMarketsParams, GetOrderbookParams,
    GetTradesParams, Market, MarketCandlesticks, MarketFilterStatus, MarketResponse, MarketResult,
    MarketStatus, MarketType, MarketsResponse, MveFilter, OhlcData, Orderbook, OrderbookResponse,
    PriceLevelDollars, PriceOhlcData, PriceRange, StrikeType, TakerSide, Trade, TradesResponse,
};
pub use order::{
    AmendOrderRequest, AmendOrderResponse, BatchCancelOrderResult, BatchCancelOrdersRequest,
    BatchCancelOrdersResponse, BatchCreateOrdersRequest, BatchCreateOrdersResponse,
    BatchOrderError, BatchOrderResult, CancelOrderResponse, CreateOrderRequest,
    DecreaseOrderRequest, GetOrdersParams, GetQueuePositionsParams, Order,
    OrderQueuePositionResponse, OrderResponse, OrdersResponse, QueuePosition,
    QueuePositionsResponse, TimeInForce,
};
pub use order_group::{
    CreateOrderGroupOrder, CreateOrderGroupRequest, OrderGroup, OrderGroupResponse,
    UpdateOrderGroupOrder, UpdateOrderGroupRequest,
};
pub use position::{EventPosition, GetPositionsParams, MarketPosition, PositionsResponse};
pub use search::{
    CompetitionFilter, FiltersBySportResponse, SportFilter, TagsByCategoriesResponse,
};
pub use series::{GetSeriesParams, Series, SeriesListResponse, SeriesResponse};
pub use settlement::{GetSettlementsParams, Settlement, SettlementsResponse};
