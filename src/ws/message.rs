//! WebSocket message types for Kalshi streaming API.

use serde::{Deserialize, Serialize};

use crate::error::DisconnectReason;
use crate::ws::Channel;
// Re-export common types from models to avoid duplication
pub use crate::models::{Action, OrderStatus, OrderType, Side};

/// Order group event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderGroupEventType {
    Created,
    Triggered,
    Reset,
    Deleted,
    LimitUpdated,
}

/// Order group update data for order group lifecycle events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroupUpdateData {
    /// The order group identifier.
    pub order_group_id: String,
    /// Type of order group event.
    pub event_type: OrderGroupEventType,
    /// Contracts limit (fixed-point decimal string), present on limit updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit_fp: Option<String>,
}

/// User order event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserOrderEventType {
    /// A new order was created.
    Created,
    /// An existing order was amended/updated.
    Updated,
    /// An order was canceled.
    Canceled,
    /// An order was fully executed/filled.
    Executed,
    /// Unknown event type (forward compatibility).
    #[serde(other)]
    Unknown,
}

/// User order data for real-time order update notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrderData {
    /// The order identifier.
    pub order_id: String,
    /// User identifier (spec-required).
    pub user_id: String,
    /// Market ticker identifier (spec-required).
    pub ticker: String,
    /// Order status (resting, canceled, executed; spec-required).
    pub status: OrderStatus,
    /// Side of the order (spec-required).
    pub side: Side,
    /// Whether the order is on the yes side (spec-required).
    pub is_yes: bool,
    /// Price in fixed-point dollars (spec-required).
    pub yes_price_dollars: String,
    /// Fill count (fixed-point decimal string, spec-required).
    pub fill_count_fp: String,
    /// Remaining count (fixed-point decimal string, spec-required).
    pub remaining_count_fp: String,
    /// Initial count (fixed-point decimal string, spec-required).
    pub initial_count_fp: String,
    /// Taker fill cost in fixed-point dollars (spec-required).
    pub taker_fill_cost_dollars: String,
    /// Maker fill cost in fixed-point dollars (spec-required).
    pub maker_fill_cost_dollars: String,
    /// Taker fees in fixed-point dollars (spec-required).
    pub taker_fees_dollars: String,
    /// Maker fees in fixed-point dollars (spec-required).
    pub maker_fees_dollars: String,
    /// Client-provided order identifier (spec-required).
    pub client_order_id: String,
    /// Order creation timestamp (ISO 8601, spec-required).
    pub created_time: String,
    /// The type of event (created, updated, canceled, executed; not in v2 spec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<UserOrderEventType>,
    /// Action type (buy or sell; not in v2 spec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// Order type (limit, market; not in v2 spec).
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    /// Price in fixed-point dollars (legacy, not in v2 spec).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,
    /// Last update timestamp (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_time: Option<String>,
    /// Order expiration timestamp (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<String>,
    /// Order group this order belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,
    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subaccount_number: Option<i32>,
    /// Self-trade prevention type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<String>,
}

/// Market lifecycle event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketLifecycleEventType {
    Created,
    Activated,
    Deactivated,
    CloseDateUpdated,
    Determined,
    Settled,
}

/// Wrapper for all WebSocket stream updates.
///
/// This is the top-level message structure received from the WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdate {
    /// The channel this message belongs to.
    #[serde(rename = "type")]
    pub channel: String,
    /// Server-generated subscription identifier.
    pub sid: i64,
    /// Sequence number for ordering messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
    /// The message payload.
    pub msg: StreamMessage,
}

/// Enum representing all possible WebSocket message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamMessage {
    /// Orderbook snapshot with full state.
    OrderbookSnapshot(OrderbookSnapshotData),
    /// Incremental orderbook update.
    OrderbookDelta(OrderbookDeltaData),
    /// Market ticker update.
    Ticker(TickerData),
    /// Public trade notification.
    Trade(TradeData),
    /// User fill notification.
    Fill(FillData),
    /// User position update.
    MarketPosition(MarketPositionData),
    /// Market lifecycle event.
    MarketLifecycle(MarketLifecycleData),
    /// Event lifecycle event.
    EventLifecycle(EventLifecycleData),
    /// RFQ or quote communication.
    Communication(CommunicationData),
    /// Order group update notification.
    OrderGroupUpdate(OrderGroupUpdateData),
    /// User order update notification.
    UserOrder(Box<UserOrderData>),
    /// Multivariate collection lookup notification.
    MultivariateLookup(MultivariateLookupData),
    /// Connection was closed cleanly.
    ///
    /// This is a local event, not received from the server.
    /// Indicates an expected close (user-requested or server sent close frame).
    #[serde(skip)]
    Closed {
        /// The reason for the clean close.
        reason: DisconnectReason,
    },

    /// Connection was lost unexpectedly.
    ///
    /// This is a local event, not received from the server.
    /// Indicates an unexpected disconnection (error, timeout, network failure).
    /// You should reconnect with backoff when receiving this.
    ///
    /// The `subscriptions` field contains the channels and markets that were
    /// subscribed at the time of disconnection, useful for resubscribing after
    /// reconnection.
    #[serde(skip)]
    ConnectionLost {
        /// The reason for the connection loss.
        reason: DisconnectReason,
        /// Subscriptions that were active at disconnection: (channel, markets).
        subscriptions: Vec<(Channel, Vec<String>)>,
    },
    /// Channel was unsubscribed.
    ///
    /// Confirms that a specific subscription ID (sid) has been unsubscribed.
    Unsubscribed,
}

/// Orderbook snapshot data containing the full orderbook state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshotData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Market UUID identifier (spec-required).
    pub market_id: String,
    /// Yes side price levels in dollars: [[price_dollars, count_fp], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_dollars_fp: Option<Vec<(String, String)>>,
    /// No side price levels in dollars: [[price_dollars, count_fp], ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_dollars_fp: Option<Vec<(String, String)>>,
}

/// Orderbook delta data representing an incremental update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookDeltaData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Market UUID identifier.
    pub market_id: String,
    /// Price in dollar format (spec-required).
    pub price_dollars: String,
    /// Delta (fixed-point decimal string, spec-required).
    pub delta_fp: String,
    /// Side of the orderbook being updated.
    pub side: Side,
    /// Client order ID if the subscriber triggered this change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Subaccount number if the subscriber triggered this change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
    /// Timestamp for when the orderbook change was recorded (RFC3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
}

/// Ticker data containing market price and volume information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Market UUID identifier.
    pub market_id: String,
    /// Price formatted in dollars (spec-required).
    pub price_dollars: String,
    /// Yes bid in dollars (spec-required).
    pub yes_bid_dollars: String,
    /// Yes ask in dollars (spec-required).
    pub yes_ask_dollars: String,
    /// Volume (fixed-point decimal string, spec-required).
    pub volume_fp: String,
    /// Open interest (fixed-point decimal string, spec-required).
    pub open_interest_fp: String,
    /// Number of dollars traded in the market so far.
    #[serde(default)]
    pub dollar_volume: i64,
    /// Number of dollars positioned in the market currently.
    #[serde(default)]
    pub dollar_open_interest: i64,
    /// Unix timestamp for when the update happened (in seconds).
    #[serde(default)]
    pub ts: i64,
    /// High-precision timestamp (ISO 8601, spec-required).
    pub time: String,
    /// No bid in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_bid_dollars: Option<String>,
    /// Contracts at best yes bid (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_bid_size_fp: Option<String>,
    /// Contracts at best yes ask (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_ask_size_fp: Option<String>,
    /// Contracts at best bid (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_size_fp: Option<String>,
    /// Contracts at best ask (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_size_fp: Option<String>,
    /// Contracts in most recent trade (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_trade_size_fp: Option<String>,
}

/// Trade data for public trade notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    /// Unique identifier for the trade (spec-required).
    pub trade_id: String,
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Yes price formatted in dollars (spec-required).
    pub yes_price_dollars: String,
    /// No price formatted in dollars (spec-required).
    pub no_price_dollars: String,
    /// Count (fixed-point decimal string, spec-required).
    pub count_fp: String,
    /// Side that took liquidity.
    pub taker_side: Side,
    /// Unix timestamp in seconds.
    pub ts: i64,
}

/// Fill data for user order fill notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillData {
    /// Unique identifier for this fill.
    pub trade_id: String,
    /// Order ID that was filled.
    pub order_id: String,
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Whether the user was a taker.
    pub is_taker: bool,
    /// Side of the fill.
    pub side: Side,
    /// Price formatted in dollars (spec-required).
    pub yes_price_dollars: String,
    /// Count (fixed-point decimal string, spec-required).
    pub count_fp: String,
    /// Exchange fee cost as a fixed-point dollar string (spec-required).
    pub fee_cost: String,
    /// Action type (buy or sell).
    pub action: Action,
    /// Unix timestamp in seconds.
    pub ts: i64,
    /// Position after this fill (fixed-point decimal string, spec-required).
    pub post_position_fp: String,
    /// Side that was purchased (spec-required).
    pub purchased_side: Side,
    /// Client-provided order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Subaccount number for the fill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Market position data for user position updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositionData {
    /// User identifier (spec-required).
    pub user_id: String,
    /// Market ticker identifier (spec-required).
    pub market_ticker: String,
    /// Position (fixed-point decimal string, spec-required).
    pub position_fp: String,
    /// Position cost in centi-cents (1/10,000th of a dollar).
    #[serde(default)]
    pub position_cost: i64,
    /// Position cost in fixed-point dollars (spec-required).
    pub position_cost_dollars: String,
    /// Realized profit/loss in centi-cents.
    #[serde(default)]
    pub realized_pnl: i64,
    /// Realized PnL in fixed-point dollars (spec-required).
    pub realized_pnl_dollars: String,
    /// Total fees paid in centi-cents.
    #[serde(default)]
    pub fees_paid: i64,
    /// Fees paid in fixed-point dollars (spec-required).
    pub fees_paid_dollars: String,
    /// Position fee cost in centi-cents.
    #[serde(default)]
    pub position_fee_cost: i64,
    /// Position fee cost in fixed-point dollars (spec-required).
    pub position_fee_cost_dollars: String,
    /// Volume (fixed-point decimal string, spec-required).
    pub volume_fp: String,
    /// Subaccount number for the position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

/// Additional metadata included with market creation events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAdditionalMetadata {
    /// Market name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Market title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Subtitle for the yes side.
    #[serde(rename = "yes_sub_title", skip_serializing_if = "Option::is_none")]
    pub yes_subtitle: Option<String>,
    /// Subtitle for the no side.
    #[serde(rename = "no_sub_title", skip_serializing_if = "Option::is_none")]
    pub no_subtitle: Option<String>,
    /// Market rules.
    #[serde(rename = "rules_primary", skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    /// Whether early close is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_close_early: Option<bool>,
    /// Expiration timestamp.
    #[serde(
        rename = "expected_expiration_ts",
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration_ts: Option<i64>,
    /// Strike type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_type: Option<String>,
    /// Strike value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_value: Option<String>,
    /// Secondary rules text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_secondary: Option<String>,
    /// Event ticker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Floor strike value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_strike: Option<f64>,
    /// Cap strike value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_strike: Option<f64>,
    /// Custom strike object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_strike: Option<serde_json::Value>,
}

/// Collateral return type for event lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollateralReturnType {
    /// Mechanical net collateral return.
    #[serde(rename = "MECNET")]
    Mecnet,
    /// Direct net collateral return.
    #[serde(rename = "DIRECNET")]
    Direcnet,
    /// No collateral return type specified.
    #[serde(rename = "")]
    None,
}

/// Event lifecycle data for event state change notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLifecycleData {
    /// Event ticker identifier.
    pub event_ticker: String,
    /// Event title.
    pub title: String,
    /// Event subtitle.
    pub subtitle: String,
    /// Collateral return type.
    pub collateral_return_type: CollateralReturnType,
    /// Series ticker this event belongs to.
    pub series_ticker: String,
    /// Strike date as a Unix timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_date: Option<i64>,
    /// Strike period descriptor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_period: Option<String>,
}

/// Market lifecycle data for market state change events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketLifecycleData {
    /// Type of lifecycle event.
    pub event_type: MarketLifecycleEventType,
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Unix timestamp when market opened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_ts: Option<i64>,
    /// Unix timestamp for scheduled close.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_ts: Option<i64>,
    /// Market determination result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Settlement value as fixed-point dollar string. Present on market_determined events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_value: Option<String>,
    /// Unix timestamp of determination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determination_ts: Option<i64>,
    /// Unix timestamp of settlement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_ts: Option<i64>,
    /// Whether trading is paused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_deactivated: Option<bool>,
    /// Additional metadata (only on market creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_metadata: Option<MarketAdditionalMetadata>,
}

/// Communication data for RFQ and quote messages.
///
/// This is a unified type that can represent various communication events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommunicationData {
    /// RFQ created event.
    RfqCreated(RfqData),
    /// RFQ deleted event.
    RfqDeleted(RfqDeletedData),
    /// Quote created event.
    QuoteCreated(QuoteData),
    /// Quote accepted event.
    QuoteAccepted(QuoteAcceptedData),
    /// Quote executed event.
    QuoteExecuted(QuoteExecutedData),
}

/// Leg definition for multivariate RFQs and lookups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MveLeg {
    /// Event ticker (spec-required).
    pub event_ticker: String,
    /// Market ticker (spec-required).
    pub market_ticker: String,
    /// Side of the leg (spec-required).
    pub side: Side,
    /// Yes settlement value in dollars for the selected leg.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_settlement_value_dollars: Option<String>,
}

/// Multivariate lookup notification data.
///
/// Sent when a multivariate collection is referenced or looked up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateLookupData {
    /// Collection ticker identifier.
    pub collection_ticker: String,
    /// Event ticker identifier.
    pub event_ticker: String,
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Selected markets in this multivariate lookup.
    pub selected_markets: Vec<MveLeg>,
}

/// RFQ (Request for Quote) data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfqData {
    /// Unique RFQ identifier.
    pub id: String,
    /// Anonymized creator ID.
    pub creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Creation timestamp (ISO 8601).
    pub created_ts: String,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_dollars: Option<String>,
    /// Multivariate collection ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_collection_ticker: Option<String>,
    /// Multivariate selected legs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_selected_legs: Option<Vec<MveLeg>>,
    /// Number of contracts (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts_fp: Option<String>,
}

/// RFQ deleted event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfqDeletedData {
    /// Unique RFQ identifier.
    pub id: String,
    /// Anonymized creator ID.
    pub creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Deletion timestamp (ISO 8601).
    pub deleted_ts: String,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_dollars: Option<String>,
    /// Number of contracts (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts_fp: Option<String>,
}

/// Quote data for quote created events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Associated RFQ identifier.
    pub rfq_id: String,
    /// Anonymized quote creator ID (spec-required).
    pub quote_creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Yes bid in dollars (spec-required).
    pub yes_bid_dollars: String,
    /// No bid in dollars (spec-required).
    pub no_bid_dollars: String,
    /// Creation timestamp (ISO 8601).
    pub created_ts: String,
    /// Anonymized RFQ creator ID (not in spec for quote_created).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rfq_creator_id: Option<String>,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// RFQ target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost_dollars: Option<String>,
    /// Yes contracts offered (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_contracts_offered_fp: Option<String>,
    /// No contracts offered (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_contracts_offered_fp: Option<String>,
}

/// Quote accepted event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteAcceptedData {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Associated RFQ identifier.
    pub rfq_id: String,
    /// Anonymized quote creator ID (spec-required).
    pub quote_creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Yes bid in dollars (spec-required).
    pub yes_bid_dollars: String,
    /// No bid in dollars (spec-required).
    pub no_bid_dollars: String,
    /// Acceptance timestamp (ISO 8601), if provided by the API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_ts: Option<String>,
    /// Anonymized RFQ creator ID (not in spec for quote_accepted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rfq_creator_id: Option<String>,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// RFQ target cost in dollars (spec-optional for quote_accepted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost_dollars: Option<String>,
    /// Side that was accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_side: Option<Side>,
    /// Contracts accepted (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts_accepted_fp: Option<String>,
    /// Yes contracts offered (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_contracts_offered_fp: Option<String>,
    /// No contracts offered (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_contracts_offered_fp: Option<String>,
}

/// Quote executed event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteExecutedData {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Associated RFQ identifier.
    pub rfq_id: String,
    /// Anonymized quote creator ID (spec-required).
    pub quote_creator_id: String,
    /// Anonymized RFQ creator ID (spec-required).
    pub rfq_creator_id: String,
    /// Order ID resulting from the execution (spec-required).
    pub order_id: String,
    /// Client-specified order ID (spec-required).
    pub client_order_id: String,
    /// Market ticker (spec-required).
    pub market_ticker: String,
    /// Execution timestamp (ISO 8601, spec-required).
    pub executed_ts: String,
}

impl StreamMessage {
    /// Parse a message payload based on the channel type.
    ///
    /// This is used instead of untagged deserialization to correctly route
    /// messages based on the `type` field from the wire protocol.
    pub fn from_type_and_value(
        msg_type: &str,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        match msg_type {
            "orderbook_snapshot" => serde_json::from_value::<OrderbookSnapshotData>(value)
                .map(StreamMessage::OrderbookSnapshot),
            "orderbook_delta" => serde_json::from_value::<OrderbookDeltaData>(value)
                .map(StreamMessage::OrderbookDelta),
            "ticker" => serde_json::from_value::<TickerData>(value).map(StreamMessage::Ticker),
            "trade" => serde_json::from_value::<TradeData>(value).map(StreamMessage::Trade),
            "fill" => serde_json::from_value::<FillData>(value).map(StreamMessage::Fill),
            "market_position" | "market_positions" => {
                serde_json::from_value::<MarketPositionData>(value)
                    .map(StreamMessage::MarketPosition)
            }
            "market_lifecycle" | "market_lifecycle_v2" => {
                serde_json::from_value::<MarketLifecycleData>(value)
                    .map(StreamMessage::MarketLifecycle)
            }
            "event_lifecycle" => serde_json::from_value::<EventLifecycleData>(value)
                .map(StreamMessage::EventLifecycle),
            "communication" | "communications" => {
                serde_json::from_value::<CommunicationData>(value).map(StreamMessage::Communication)
            }
            // RFQ/Quote messages come with specific type names, not "communication"
            "rfq_created" => serde_json::from_value::<RfqData>(value)
                .map(|data| StreamMessage::Communication(CommunicationData::RfqCreated(data))),
            "rfq_deleted" => serde_json::from_value::<RfqDeletedData>(value)
                .map(|data| StreamMessage::Communication(CommunicationData::RfqDeleted(data))),
            "quote_created" => serde_json::from_value::<QuoteData>(value)
                .map(|data| StreamMessage::Communication(CommunicationData::QuoteCreated(data))),
            "quote_accepted" => serde_json::from_value::<QuoteAcceptedData>(value)
                .map(|data| StreamMessage::Communication(CommunicationData::QuoteAccepted(data))),
            "quote_executed" => serde_json::from_value::<QuoteExecutedData>(value)
                .map(|data| StreamMessage::Communication(CommunicationData::QuoteExecuted(data))),
            // Order group update notifications
            "order_group_updates" => serde_json::from_value::<OrderGroupUpdateData>(value)
                .map(StreamMessage::OrderGroupUpdate),
            // User order update notifications
            "user_order" | "user_orders" => serde_json::from_value::<UserOrderData>(value)
                .map(|d| StreamMessage::UserOrder(Box::new(d))),
            // Multivariate lookup notifications
            "multivariate_lookup" => serde_json::from_value::<MultivariateLookupData>(value)
                .map(StreamMessage::MultivariateLookup),
            _ => {
                // Fallback to untagged deserialization for unknown types
                serde_json::from_value::<StreamMessage>(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_delta_deserialization() {
        let json = r#"{
            "market_ticker": "KXBTC-24DEC31-100000",
            "market_id": "test-uuid",
            "price_dollars": "0.45",
            "delta_fp": "10.00",
            "side": "yes"
        }"#;
        let delta: OrderbookDeltaData = serde_json::from_str(json).unwrap();
        assert_eq!(delta.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(delta.market_id, "test-uuid");
        assert_eq!(delta.price_dollars, "0.45");
        assert_eq!(delta.delta_fp, "10.00");
        assert_eq!(delta.side, Side::Yes);
    }

    #[test]
    fn test_ticker_data_deserialization() {
        let json = r#"{
            "market_ticker": "KXBTC-24DEC31-100000",
            "market_id": "test-uuid",
            "price_dollars": "0.45",
            "yes_bid_dollars": "0.44",
            "yes_ask_dollars": "0.46",
            "volume_fp": "1000.00",
            "open_interest_fp": "500.00",
            "dollar_volume": 25000,
            "dollar_open_interest": 12000,
            "ts": 1704067200,
            "time": "2024-01-01T00:00:00Z"
        }"#;
        let ticker: TickerData = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(ticker.price_dollars, "0.45");
        assert_eq!(ticker.yes_bid_dollars, "0.44");
        assert_eq!(ticker.yes_ask_dollars, "0.46");
        assert_eq!(ticker.dollar_volume, 25000);
        assert_eq!(ticker.dollar_open_interest, 12000);
        assert_eq!(ticker.ts, 1704067200);
    }

    #[test]
    fn test_trade_data_deserialization() {
        let json = r#"{
            "trade_id": "trade-001",
            "market_ticker": "KXBTC-24DEC31-100000",
            "yes_price_dollars": "0.45",
            "no_price_dollars": "0.55",
            "count_fp": "10.00",
            "taker_side": "yes",
            "ts": 1704067200
        }"#;
        let trade: TradeData = serde_json::from_str(json).unwrap();
        assert_eq!(trade.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(trade.yes_price_dollars, "0.45");
        assert_eq!(trade.no_price_dollars, "0.55");
        assert_eq!(trade.count_fp, "10.00");
        assert_eq!(trade.taker_side, Side::Yes);
    }

    #[test]
    fn test_market_lifecycle_event_type() {
        assert_eq!(
            serde_json::to_string(&MarketLifecycleEventType::Created).unwrap(),
            "\"created\""
        );
        assert_eq!(
            serde_json::to_string(&MarketLifecycleEventType::CloseDateUpdated).unwrap(),
            "\"close_date_updated\""
        );
    }

    #[test]
    fn test_from_type_and_value_rfq_created() {
        let json = serde_json::json!({
            "type": "rfq_created",
            "id": "rfq-001",
            "creator_id": "anon-abc",
            "market_ticker": "KXBTC-25MAR01-100000",
            "created_ts": "2026-02-28T12:00:00Z"
        });
        let msg = StreamMessage::from_type_and_value("rfq_created", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::RfqCreated(data)) => {
                assert_eq!(data.id, "rfq-001");
                assert_eq!(data.market_ticker, "KXBTC-25MAR01-100000");
            }
            other => panic!("Expected RfqCreated, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_rfq_deleted() {
        let json = serde_json::json!({
            "type": "rfq_deleted",
            "id": "rfq-002",
            "creator_id": "anon-xyz",
            "market_ticker": "KXBTC-25MAR01-100000",
            "deleted_ts": "2026-02-28T12:00:30Z"
        });
        let msg = StreamMessage::from_type_and_value("rfq_deleted", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::RfqDeleted(data)) => {
                assert_eq!(data.id, "rfq-002");
                assert_eq!(data.creator_id, "anon-xyz");
            }
            other => panic!("Expected RfqDeleted, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_quote_created() {
        let json = serde_json::json!({
            "type": "quote_created",
            "quote_id": "q-001",
            "rfq_id": "rfq-001",
            "quote_creator_id": "anon-def",
            "rfq_creator_id": "anon-ghi",
            "market_ticker": "KXBTC-25MAR01-100000",
            "yes_bid_dollars": "0.45",
            "no_bid_dollars": "0.55",
            "created_ts": "2026-02-28T12:01:00Z"
        });
        let msg = StreamMessage::from_type_and_value("quote_created", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::QuoteCreated(data)) => {
                assert_eq!(data.quote_id, "q-001");
                assert_eq!(data.rfq_id, "rfq-001");
                assert_eq!(data.yes_bid_dollars, "0.45");
            }
            other => panic!("Expected QuoteCreated, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_quote_accepted() {
        let json = serde_json::json!({
            "type": "quote_accepted",
            "quote_id": "q-001",
            "rfq_id": "rfq-001",
            "quote_creator_id": "anon-def",
            "rfq_creator_id": "anon-ghi",
            "market_ticker": "KXBTC-25MAR01-100000",
            "yes_bid_dollars": "0.45",
            "no_bid_dollars": "0.55",
            "accepted_side": "yes"
        });
        let msg = StreamMessage::from_type_and_value("quote_accepted", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::QuoteAccepted(data)) => {
                assert_eq!(data.quote_id, "q-001");
                assert_eq!(data.accepted_side, Some(Side::Yes));
            }
            other => panic!("Expected QuoteAccepted, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_quote_executed() {
        let json = serde_json::json!({
            "type": "quote_executed",
            "quote_id": "q-001",
            "rfq_id": "rfq-001",
            "quote_creator_id": "anon-def",
            "rfq_creator_id": "anon-ghi",
            "order_id": "ord-999",
            "client_order_id": "my-order-1",
            "market_ticker": "KXBTC-25MAR01-100000",
            "executed_ts": "2026-02-28T12:02:00Z"
        });
        let msg = StreamMessage::from_type_and_value("quote_executed", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::QuoteExecuted(data)) => {
                assert_eq!(data.quote_id, "q-001");
                assert_eq!(data.rfq_id, "rfq-001");
                assert_eq!(data.order_id, "ord-999");
                assert_eq!(data.client_order_id, "my-order-1");
                assert_eq!(data.executed_ts, "2026-02-28T12:02:00Z");
            }
            other => panic!("Expected QuoteExecuted, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_quote_executed_minimal() {
        let json = serde_json::json!({
            "type": "quote_executed",
            "quote_id": "q-002",
            "rfq_id": "rfq-002",
            "quote_creator_id": "anon-abc",
            "rfq_creator_id": "anon-xyz",
            "order_id": "ord-100",
            "client_order_id": "client-100",
            "market_ticker": "KXBTC-25MAR01-100000",
            "executed_ts": "2026-02-28T12:05:00Z"
        });
        let msg = StreamMessage::from_type_and_value("quote_executed", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::QuoteExecuted(data)) => {
                assert_eq!(data.quote_id, "q-002");
                assert_eq!(data.rfq_id, "rfq-002");
                assert_eq!(data.order_id, "ord-100");
                assert_eq!(data.client_order_id, "client-100");
                assert_eq!(data.executed_ts, "2026-02-28T12:05:00Z");
            }
            other => panic!("Expected QuoteExecuted, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_event_lifecycle() {
        let json = serde_json::json!({
            "event_ticker": "KXBTC-25MAR01",
            "title": "Bitcoin above 100k?",
            "subtitle": "March 2025",
            "collateral_return_type": "MECNET",
            "series_ticker": "KXBTC",
            "strike_date": 1740787200,
            "strike_period": "2025-03-01"
        });
        let msg = StreamMessage::from_type_and_value("event_lifecycle", json).unwrap();
        match msg {
            StreamMessage::EventLifecycle(data) => {
                assert_eq!(data.event_ticker, "KXBTC-25MAR01");
                assert_eq!(data.title, "Bitcoin above 100k?");
                assert_eq!(data.subtitle, "March 2025");
                assert_eq!(data.collateral_return_type, CollateralReturnType::Mecnet);
                assert_eq!(data.series_ticker, "KXBTC");
                assert_eq!(data.strike_date, Some(1740787200));
                assert_eq!(data.strike_period, Some("2025-03-01".to_string()));
            }
            other => panic!("Expected EventLifecycle, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_event_lifecycle_minimal() {
        let json = serde_json::json!({
            "event_ticker": "KXETH-25MAR01",
            "title": "Ethereum above 5k?",
            "subtitle": "March 2025",
            "collateral_return_type": "",
            "series_ticker": "KXETH"
        });
        let msg = StreamMessage::from_type_and_value("event_lifecycle", json).unwrap();
        match msg {
            StreamMessage::EventLifecycle(data) => {
                assert_eq!(data.event_ticker, "KXETH-25MAR01");
                assert_eq!(data.collateral_return_type, CollateralReturnType::None);
                assert_eq!(data.strike_date, None);
                assert_eq!(data.strike_period, None);
            }
            other => panic!("Expected EventLifecycle, got {other:?}"),
        }
    }

    #[test]
    fn test_from_type_and_value_communication_tagged() {
        // Messages can also arrive with type="communication" and inner tag routing
        let json = serde_json::json!({
            "type": "quote_executed",
            "quote_id": "q-003",
            "rfq_id": "rfq-003",
            "quote_creator_id": "anon-abc",
            "rfq_creator_id": "anon-xyz",
            "order_id": "ord-200",
            "client_order_id": "client-200",
            "market_ticker": "KXBTC-25MAR01-100000",
            "executed_ts": "2026-02-28T12:10:00Z"
        });
        let msg = StreamMessage::from_type_and_value("communication", json).unwrap();
        match msg {
            StreamMessage::Communication(CommunicationData::QuoteExecuted(data)) => {
                assert_eq!(data.quote_id, "q-003");
            }
            other => panic!("Expected QuoteExecuted via tagged dispatch, got {other:?}"),
        }
    }
}
