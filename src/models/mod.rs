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
mod live_data;
mod market;
mod multivariate;
mod order;
mod order_group;
mod position;
pub(crate) mod query;
mod search;
mod series;
mod settlement;
mod subaccount;

// Re-export all public types
pub use balance::BalanceResponse;
pub use common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side, cents_to_dollars};
pub use communications::{
    AcceptQuoteRequest, CommunicationsIdResponse, CreateQuoteRequest, CreateRfqRequest,
    GetQuoteResponse, GetRfqResponse, ListQuotesParams, ListQuotesResponse, ListRfqsParams,
    ListRfqsResponse, Quote, QuoteResponse, Rfq, RfqResponse,
};
pub use event::{
    Event, EventCandlesticksResponse, EventForecastPercentileHistoryResponse, EventMetadataResponse,
    EventResponse, EventStatus, EventsResponse, ForecastHistoryPoint, ForecastPeriod,
    GetEventCandlesticksParams, GetEventForecastPercentileHistoryParams, GetEventParams,
    GetEventsParams, GetMultivariateEventsParams, MarketDetail, Milestone,
    MultivariateEventsResponse, PercentilePoint, SettlementSource, MAX_FORECAST_PERCENTILES,
};
pub use exchange::{
    Announcement, AnnouncementStatus, AnnouncementType, ExchangeAnnouncementsResponse,
    ExchangeSchedule, ExchangeScheduleResponse, ExchangeStatusResponse, MaintenanceWindow,
    StandardHoursPeriod, TradingSession, UserDataTimestampResponse,
};
pub use fill::{Fill, FillsResponse, GetFillsParams};
pub use live_data::{
    BatchLiveDataResponse, GetBatchLiveDataParams, LiveData, LiveDataResponse,
};
pub use market::{
    BatchCandlesticksResponse, Candlestick, CandlestickPeriod, CandlesticksResponse,
    GetBatchCandlesticksParams, GetCandlesticksParams, GetMarketsParams, GetOrderbookParams,
    GetTradesParams, Market, MarketCandlesticks, MarketFilterStatus, MarketResponse, MarketResult,
    MarketStatus, MarketType, MarketsResponse, MveFilter, OhlcData, Orderbook, OrderbookResponse,
    PriceLevelDollars, PriceOhlcData, PriceRange, StrikeType, TakerSide, Trade, TradesResponse,
};
pub use multivariate::{
    CollectionVariable, CreateMarketInCollectionRequest, CreateMarketInCollectionResponse,
    GetLookupHistoryParams, GetMultivariateCollectionsParams, LookupHistoryEntry,
    LookupHistoryResponse, LookupTickersRequest, LookupTickersResponse,
    MultivariateCollectionResponse, MultivariateCollectionsResponse, MultivariateEventCollection,
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
    CreateOrderGroupOrder, CreateOrderGroupRequest, GetOrderGroupsParams, OrderGroup,
    OrderGroupResponse, OrderGroupsResponse, UpdateOrderGroupOrder, UpdateOrderGroupRequest,
};
pub use position::{EventPosition, GetPositionsParams, MarketPosition, PositionsResponse};
pub use search::{
    CompetitionFilter, FiltersBySportResponse, SportFilter, TagsByCategoriesResponse,
};
pub use series::{
    FeeChangesResponse, FeeType, GetFeeChangesParams, GetSeriesParams, Series, SeriesFeeChange,
    SeriesListResponse, SeriesResponse,
};
pub use settlement::{GetSettlementsParams, Settlement, SettlementsResponse};
pub use subaccount::{
    CreateSubaccountRequest, CreateSubaccountResponse, GetSubaccountTransfersParams,
    RestingOrderValueResponse, Subaccount, SubaccountBalance, SubaccountBalancesResponse,
    SubaccountTransfer, SubaccountTransfersResponse, TransferBetweenSubaccountsRequest,
    TransferResponse,
};
