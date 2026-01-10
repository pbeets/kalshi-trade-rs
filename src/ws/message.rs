//! WebSocket message types for Kalshi streaming API.
//!
//! This module contains all the message and data types used for WebSocket communication
//! with the Kalshi exchange.

use serde::{Deserialize, Serialize};

// Re-export common types from models to avoid duplication
pub use crate::models::{Action, Side};

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
    /// RFQ or quote communication.
    Communication(CommunicationData),
    /// Connection was lost or closed.
    ///
    /// This is a local event, not received from the server.
    /// When you receive this, you should reconnect.
    #[serde(skip)]
    Disconnected {
        /// Human-readable reason for disconnection.
        reason: String,
        /// Whether this was a clean close (server sent close frame).
        was_clean: bool,
    },
}

/// A price level in the orderbook: [price_cents, contracts].
pub type PriceLevel = [i64; 2];

/// A price level with dollar representation: [price_dollars, contracts].
pub type PriceLevelDollars = (String, i64);

/// Orderbook snapshot data containing the full orderbook state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshotData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Yes side price levels: [price_cents, contracts].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes: Option<Vec<PriceLevel>>,
    /// Yes side price levels in dollars: [price_dollars, contracts].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_dollars: Option<Vec<PriceLevelDollars>>,
    /// No side price levels: [price_cents, contracts].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no: Option<Vec<PriceLevel>>,
    /// No side price levels in dollars: [price_dollars, contracts].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_dollars: Option<Vec<PriceLevelDollars>>,
}

/// Orderbook delta data representing an incremental update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookDeltaData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Price level being updated (1-99 cents).
    pub price: i64,
    /// Change in quantity (positive = increase, negative = decrease).
    pub delta: i64,
    /// Side of the orderbook being updated.
    pub side: Side,
    /// Price in dollar format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_dollars: Option<String>,
    /// Client order ID if the subscriber triggered this change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

/// Ticker data containing market price and volume information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Last traded price in cents (1-99).
    pub price: i64,
    /// Best bid price for yes side in cents (1-99).
    pub yes_bid: i64,
    /// Best ask price for yes side in cents (1-99).
    pub yes_ask: i64,
    /// Number of individual contracts traded.
    pub volume: i64,
    /// Number of active contracts (open interest).
    pub open_interest: i64,
    /// Dollar volume traded.
    pub dollar_volume: i64,
    /// Dollar value of open interest.
    pub dollar_open_interest: i64,
    /// Unix timestamp in seconds.
    pub ts: i64,
    /// Price formatted in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_dollars: Option<String>,
    /// Yes bid in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_bid_dollars: Option<String>,
    /// No bid in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_bid_dollars: Option<String>,
}

/// Trade data for public trade notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    /// Market ticker identifier.
    pub market_ticker: String,
    /// Yes side price in cents (1-99).
    pub yes_price: i64,
    /// No side price in cents (1-99).
    pub no_price: i64,
    /// Number of contracts traded.
    pub count: i64,
    /// Side that took liquidity.
    pub taker_side: Side,
    /// Unix timestamp in seconds.
    pub ts: i64,
    /// Yes price formatted in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,
    /// No price formatted in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,
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
    /// Price in cents (1-99).
    pub yes_price: i64,
    /// Price formatted in dollars.
    pub yes_price_dollars: String,
    /// Number of contracts filled.
    pub count: i64,
    /// Action type (buy or sell).
    pub action: Action,
    /// Unix timestamp in seconds.
    pub ts: i64,
    /// Client-provided order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Position after this fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_position: Option<i64>,
    /// Side that was purchased.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchased_side: Option<Side>,
}

/// Market position data for user position updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositionData {
    /// User identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Market ticker identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    /// Net position (positive = long, negative = short).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Cost basis in centi-cents (1/10,000th of a dollar).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_cost: Option<i64>,
    /// Realized profit/loss in centi-cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<i64>,
    /// Fees paid in centi-cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fees_paid: Option<i64>,
    /// Total volume traded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<i64>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_subtitle: Option<String>,
    /// Subtitle for the no side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_subtitle: Option<String>,
    /// Market rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    /// Whether early close is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_close_early: Option<bool>,
    /// Expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    /// Strike type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_type: Option<String>,
    /// Strike value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike_value: Option<String>,
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
}

/// Leg definition for multivariate RFQs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MveLeg {
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Market ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    /// Side of the leg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
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
    /// Number of contracts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<i64>,
    /// Target cost in centi-cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost: Option<i64>,
    /// Target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_dollars: Option<String>,
    /// Multivariate collection ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_collection_ticker: Option<String>,
    /// Multivariate selected legs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_selected_legs: Option<Vec<MveLeg>>,
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
}

/// Quote data for quote created events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Associated RFQ identifier.
    pub rfq_id: String,
    /// Anonymized quote creator ID.
    pub quote_creator_id: String,
    /// Anonymized RFQ creator ID.
    pub rfq_creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Yes bid price in cents.
    pub yes_bid: i64,
    /// No bid price in cents.
    pub no_bid: i64,
    /// Yes bid in dollars.
    pub yes_bid_dollars: String,
    /// No bid in dollars.
    pub no_bid_dollars: String,
    /// Creation timestamp (ISO 8601).
    pub created_ts: String,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Yes contracts offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_contracts_offered: Option<i64>,
    /// No contracts offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_contracts_offered: Option<i64>,
    /// RFQ target cost in centi-cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost: Option<i64>,
    /// RFQ target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost_dollars: Option<String>,
}

/// Quote accepted event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteAcceptedData {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Associated RFQ identifier.
    pub rfq_id: String,
    /// Anonymized quote creator ID.
    pub quote_creator_id: String,
    /// Anonymized RFQ creator ID.
    pub rfq_creator_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Yes bid price in cents.
    pub yes_bid: i64,
    /// No bid price in cents.
    pub no_bid: i64,
    /// Yes bid in dollars.
    pub yes_bid_dollars: String,
    /// No bid in dollars.
    pub no_bid_dollars: String,
    /// Event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Yes contracts offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_contracts_offered: Option<i64>,
    /// No contracts offered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_contracts_offered: Option<i64>,
    /// RFQ target cost in centi-cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost: Option<i64>,
    /// RFQ target cost in dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_target_cost_dollars: Option<String>,
    /// Side that was accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_side: Option<Side>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_serialization() {
        assert_eq!(serde_json::to_string(&Side::Yes).unwrap(), "\"yes\"");
        assert_eq!(serde_json::to_string(&Side::No).unwrap(), "\"no\"");
    }

    #[test]
    fn test_side_deserialization() {
        assert_eq!(serde_json::from_str::<Side>("\"yes\"").unwrap(), Side::Yes);
        assert_eq!(serde_json::from_str::<Side>("\"no\"").unwrap(), Side::No);
    }

    #[test]
    fn test_orderbook_delta_deserialization() {
        let json = r#"{
            "market_ticker": "KXBTC-24DEC31-100000",
            "price": 45,
            "delta": 10,
            "side": "yes"
        }"#;
        let delta: OrderbookDeltaData = serde_json::from_str(json).unwrap();
        assert_eq!(delta.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(delta.price, 45);
        assert_eq!(delta.delta, 10);
        assert_eq!(delta.side, Side::Yes);
    }

    #[test]
    fn test_ticker_data_deserialization() {
        let json = r#"{
            "market_ticker": "KXBTC-24DEC31-100000",
            "price": 45,
            "yes_bid": 44,
            "yes_ask": 46,
            "volume": 1000,
            "open_interest": 500,
            "dollar_volume": 45000,
            "dollar_open_interest": 22500,
            "ts": 1704067200
        }"#;
        let ticker: TickerData = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(ticker.price, 45);
        assert_eq!(ticker.yes_bid, 44);
        assert_eq!(ticker.yes_ask, 46);
    }

    #[test]
    fn test_trade_data_deserialization() {
        let json = r#"{
            "market_ticker": "KXBTC-24DEC31-100000",
            "yes_price": 45,
            "no_price": 55,
            "count": 10,
            "taker_side": "yes",
            "ts": 1704067200
        }"#;
        let trade: TradeData = serde_json::from_str(json).unwrap();
        assert_eq!(trade.market_ticker, "KXBTC-24DEC31-100000");
        assert_eq!(trade.yes_price, 45);
        assert_eq!(trade.no_price, 55);
        assert_eq!(trade.count, 10);
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
}
