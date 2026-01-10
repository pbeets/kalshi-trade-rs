//! Market models and response types.
//!
//! Types for markets, orderbooks, and trades.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// Market type (binary or scalar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    Binary,
    Scalar,
}

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Initialized,
    Inactive,
    Active,
    Closed,
    Determined,
    Disputed,
    Amended,
    Finalized,
}

impl MarketStatus {
    /// Returns the lowercase API representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketStatus::Initialized => "initialized",
            MarketStatus::Inactive => "inactive",
            MarketStatus::Active => "active",
            MarketStatus::Closed => "closed",
            MarketStatus::Determined => "determined",
            MarketStatus::Disputed => "disputed",
            MarketStatus::Amended => "amended",
            MarketStatus::Finalized => "finalized",
        }
    }
}

/// Market result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketResult {
    Yes,
    No,
    #[serde(rename = "")]
    None,
}

/// Filter status for list markets query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketFilterStatus {
    Unopened,
    Open,
    Paused,
    Closed,
    Settled,
}

impl MarketFilterStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketFilterStatus::Unopened => "unopened",
            MarketFilterStatus::Open => "open",
            MarketFilterStatus::Paused => "paused",
            MarketFilterStatus::Closed => "closed",
            MarketFilterStatus::Settled => "settled",
        }
    }
}

/// Multivariate event filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MveFilter {
    /// Only multivariate events.
    Only,
    /// Exclude multivariate events.
    Exclude,
}

impl MveFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            MveFilter::Only => "only",
            MveFilter::Exclude => "exclude",
        }
    }
}

/// Strike type for a market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrikeType {
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Between,
    Functional,
    Custom,
    Structured,
}

/// Price range configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub start: String,
    pub end: String,
    pub step: String,
}

/// A market on the Kalshi exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub ticker: String,
    pub event_ticker: String,
    pub market_type: MarketType,

    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub yes_sub_title: Option<String>,
    #[serde(default)]
    pub no_sub_title: Option<String>,

    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub open_time: Option<String>,
    #[serde(default)]
    pub close_time: Option<String>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub latest_expiration_time: Option<String>,
    #[serde(default)]
    pub expected_expiration_time: Option<String>,
    #[serde(default)]
    pub settlement_timer_seconds: Option<i64>,

    pub status: MarketStatus,

    /// Best YES bid price in dollars.
    #[serde(default)]
    pub yes_bid_dollars: Option<String>,
    /// Best YES ask price in dollars.
    #[serde(default)]
    pub yes_ask_dollars: Option<String>,
    /// Best NO bid price in dollars.
    #[serde(default)]
    pub no_bid_dollars: Option<String>,
    /// Best NO ask price in dollars.
    #[serde(default)]
    pub no_ask_dollars: Option<String>,
    /// Last trade price in dollars.
    #[serde(default)]
    pub last_price_dollars: Option<String>,

    /// Previous YES bid (24h ago) in dollars.
    #[serde(default)]
    pub previous_yes_bid_dollars: Option<String>,
    /// Previous YES ask (24h ago) in dollars.
    #[serde(default)]
    pub previous_yes_ask_dollars: Option<String>,
    /// Previous price (24h ago) in dollars.
    #[serde(default)]
    pub previous_price_dollars: Option<String>,

    /// Total contracts traded.
    #[serde(default)]
    pub volume: Option<i64>,
    /// 24-hour trading volume.
    #[serde(default)]
    pub volume_24h: Option<i64>,
    /// Contracts outstanding.
    #[serde(default)]
    pub open_interest: Option<i64>,

    /// Notional value per contract in dollars.
    #[serde(default)]
    pub notional_value_dollars: Option<String>,
    /// Available order liquidity in dollars.
    #[serde(default)]
    pub liquidity_dollars: Option<String>,

    #[serde(default)]
    pub result: Option<MarketResult>,
    #[serde(default)]
    pub can_close_early: Option<bool>,
    #[serde(default)]
    pub early_close_condition: Option<String>,

    #[serde(default)]
    pub settlement_value_dollars: Option<String>,
    #[serde(default)]
    pub settlement_ts: Option<String>,
    #[serde(default)]
    pub fee_waiver_expiration_time: Option<String>,

    #[serde(default)]
    pub rules_primary: Option<String>,
    #[serde(default)]
    pub rules_secondary: Option<String>,
    #[serde(default)]
    pub price_level_structure: Option<String>,
    #[serde(default)]
    pub price_ranges: Option<Vec<PriceRange>>,

    #[serde(default)]
    pub strike_type: Option<StrikeType>,
    #[serde(default)]
    pub floor_strike: Option<f64>,
    #[serde(default)]
    pub cap_strike: Option<f64>,
    #[serde(default)]
    pub functional_strike: Option<String>,
    #[serde(default)]
    pub custom_strike: Option<String>,

    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<String>>,
    #[serde(default)]
    pub primary_participant_key: Option<String>,
    #[serde(default)]
    pub is_provisional: Option<bool>,
}

/// Response from GET /markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<Market>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /markets/{ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub market: Market,
}

/// Query parameters for GET /markets.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MarketFilterStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_created_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_created_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_close_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_settled_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_settled_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

impl GetMarketsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `limit` is not in the range 1..=1000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        debug_assert!(
            limit > 0 && limit <= 1000,
            "limit must be between 1 and 1000, got {}",
            limit
        );
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filter by event ticker. Multiple tickers can be comma-separated (max 10).
    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn series_ticker(mut self, series_ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(series_ticker.into());
        self
    }

    /// Filter by specific market tickers. Comma-separated list.
    #[must_use]
    pub fn tickers(mut self, tickers: impl Into<String>) -> Self {
        self.tickers = Some(tickers.into());
        self
    }

    #[must_use]
    pub fn status(mut self, status: MarketFilterStatus) -> Self {
        self.status = Some(status);
        self
    }

    #[must_use]
    pub fn min_created_ts(mut self, ts: i64) -> Self {
        self.min_created_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn max_created_ts(mut self, ts: i64) -> Self {
        self.max_created_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn min_close_ts(mut self, ts: i64) -> Self {
        self.min_close_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn max_close_ts(mut self, ts: i64) -> Self {
        self.max_close_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn min_settled_ts(mut self, ts: i64) -> Self {
        self.min_settled_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn max_settled_ts(mut self, ts: i64) -> Self {
        self.max_settled_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn mve_filter(mut self, filter: MveFilter) -> Self {
        self.mve_filter = Some(filter);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.push_opt("tickers", self.tickers.as_ref());
        qb.push_opt("status", self.status.map(|s| s.as_str()));
        qb.push_opt("min_created_ts", self.min_created_ts);
        qb.push_opt("max_created_ts", self.max_created_ts);
        qb.push_opt("min_close_ts", self.min_close_ts);
        qb.push_opt("max_close_ts", self.max_close_ts);
        qb.push_opt("min_settled_ts", self.min_settled_ts);
        qb.push_opt("max_settled_ts", self.max_settled_ts);
        qb.push_opt("mve_filter", self.mve_filter.map(|f| f.as_str()));
        qb.build()
    }
}

/// An orderbook for a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// YES price levels as [price_cents, quantity] pairs.
    #[serde(default)]
    pub yes: Vec<Vec<i64>>,
    /// NO price levels as [price_cents, quantity] pairs.
    #[serde(default)]
    pub no: Vec<Vec<i64>>,
    /// YES price levels as [price_dollars, quantity] pairs.
    #[serde(default)]
    pub yes_dollars: Option<Vec<Vec<serde_json::Value>>>,
    /// NO price levels as [price_dollars, quantity] pairs.
    #[serde(default)]
    pub no_dollars: Option<Vec<Vec<serde_json::Value>>>,
}

/// Response from GET /markets/{ticker}/orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookResponse {
    pub orderbook: Orderbook,
}

/// Query parameters for GET /markets/{ticker}/orderbook.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetOrderbookParams {
    /// Depth of orderbook. 0 or negative means all levels. 1-100 for specific depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<i64>,
}

impl GetOrderbookParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the depth of the orderbook.
    ///
    /// 0 or negative means all levels. 1-100 for specific depth.
    #[must_use]
    pub fn depth(mut self, depth: i64) -> Self {
        self.depth = Some(depth);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("depth", self.depth);
        qb.build()
    }
}

/// Taker side of a trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TakerSide {
    Yes,
    No,
}

/// A trade on the exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub ticker: String,
    /// Trade price (deprecated).
    #[serde(default)]
    pub price: Option<f64>,
    /// Contract quantity.
    pub count: i64,
    /// Yes side price in cents.
    pub yes_price: i64,
    /// No side price in cents.
    pub no_price: i64,
    /// Yes price in dollars.
    #[serde(default)]
    pub yes_price_dollars: Option<String>,
    /// No price in dollars.
    #[serde(default)]
    pub no_price_dollars: Option<String>,
    pub taker_side: TakerSide,
    pub created_time: String,
}

/// Response from GET /markets/trades.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<Trade>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for GET /markets/trades.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetTradesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
}

impl GetTradesParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `limit` is not in the range 1..=1000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        debug_assert!(
            limit > 0 && limit <= 1000,
            "limit must be between 1 and 1000, got {}",
            limit
        );
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn min_ts(mut self, ts: i64) -> Self {
        self.min_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn max_ts(mut self, ts: i64) -> Self {
        self.max_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_markets_query_string() {
        let params = GetMarketsParams::new()
            .status(MarketFilterStatus::Open)
            .limit(50);
        assert!(params.to_query_string().contains("status=open"));
        assert!(params.to_query_string().contains("limit=50"));
    }

    #[test]
    fn test_get_trades_query_string() {
        let params = GetTradesParams::new().ticker("BTC-USD").limit(100);
        assert!(params.to_query_string().contains("ticker=BTC-USD"));
        assert!(params.to_query_string().contains("limit=100"));
    }
}
