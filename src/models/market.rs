//! Market models and response types.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// Market type (binary or scalar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketType {
    Binary,
    Scalar,
    /// Unknown market type returned by the API.
    #[serde(other)]
    Unknown,
}

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketStatus {
    Initialized,
    Inactive,
    Active,
    Closed,
    Determined,
    Disputed,
    Amended,
    Finalized,
    /// Unknown status returned by the API.
    #[serde(other)]
    Unknown,
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
            MarketStatus::Unknown => "unknown",
        }
    }
}

/// Market result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketResult {
    Yes,
    No,
    #[serde(rename = "")]
    None,
    /// Unknown result returned by the API.
    #[serde(other)]
    Unknown,
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
#[non_exhaustive]
pub enum StrikeType {
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Between,
    Functional,
    Custom,
    Structured,
    /// Unknown strike type returned by the API.
    #[serde(other)]
    Unknown,
}

/// Price range configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub start: String,
    pub end: String,
    pub step: String,
}

/// A selected leg in a multivariate event market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MveSelectedLeg {
    /// The event ticker for this leg.
    pub event_ticker: String,
    /// The market ticker for this leg.
    pub market_ticker: String,
    /// The side of the leg (e.g., "yes" or "no").
    pub side: String,
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
    /// Total contracts traded (fixed-point decimal string).
    #[serde(default)]
    pub volume_fp: Option<String>,
    /// 24-hour trading volume.
    #[serde(default)]
    pub volume_24h: Option<i64>,
    /// 24-hour trading volume (fixed-point decimal string).
    #[serde(default)]
    pub volume_24h_fp: Option<String>,
    /// Contracts outstanding.
    #[serde(default)]
    pub open_interest: Option<i64>,
    /// Contracts outstanding (fixed-point decimal string).
    #[serde(default)]
    pub open_interest_fp: Option<String>,

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
    /// Custom strike value - can be a string or an object depending on market type.
    #[serde(default)]
    pub custom_strike: Option<serde_json::Value>,

    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<MveSelectedLeg>>,
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
    /// The value is clamped to the valid range of 1..=1000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit.clamp(1, 1000));
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

/// A price level in the orderbook with dollar pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevelDollars {
    /// Price in dollars (e.g., "0.50").
    pub price: String,
    /// Quantity at this price level.
    pub quantity: i64,
}

/// Custom deserializer for price level arrays from the API.
mod price_level_serde {
    use super::PriceLevelDollars;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<PriceLevelDollars>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Vec<Vec<serde_json::Value>>> = Option::deserialize(deserializer)?;
        match opt {
            None => Ok(None),
            Some(levels) => {
                let mut result = Vec::with_capacity(levels.len());
                for level in levels {
                    if level.len() >= 2 {
                        let price = match &level[0] {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => continue,
                        };
                        let quantity = match &level[1] {
                            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
                            _ => continue,
                        };
                        result.push(PriceLevelDollars { price, quantity });
                    }
                }
                Ok(Some(result))
            }
        }
    }

    pub fn serialize<S>(
        value: &Option<Vec<PriceLevelDollars>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(levels) => {
                let arrays: Vec<Vec<serde_json::Value>> = levels
                    .iter()
                    .map(|pl| {
                        vec![
                            serde_json::Value::String(pl.price.clone()),
                            serde_json::Value::Number(pl.quantity.into()),
                        ]
                    })
                    .collect();
                arrays.serialize(serializer)
            }
        }
    }
}

/// Custom deserializer for orderbook that handles null as empty.
mod orderbook_serde {
    use super::Orderbook;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Orderbook, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<Orderbook> = Option::deserialize(deserializer)?;
        Ok(opt.unwrap_or_default())
    }
}

/// Custom deserializer for Vec that treats null as empty.
mod null_as_empty_vec {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let opt: Option<Vec<T>> = Option::deserialize(deserializer)?;
        Ok(opt.unwrap_or_default())
    }
}

/// An orderbook for a market.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Orderbook {
    /// YES price levels as [price_cents, quantity] pairs.
    #[serde(default, deserialize_with = "null_as_empty_vec::deserialize")]
    pub yes: Vec<Vec<i64>>,
    /// NO price levels as [price_cents, quantity] pairs.
    #[serde(default, deserialize_with = "null_as_empty_vec::deserialize")]
    pub no: Vec<Vec<i64>>,
    /// YES price levels with dollar pricing.
    #[serde(default, with = "price_level_serde")]
    pub yes_dollars: Option<Vec<PriceLevelDollars>>,
    /// NO price levels with dollar pricing.
    #[serde(default, with = "price_level_serde")]
    pub no_dollars: Option<Vec<PriceLevelDollars>>,
}

/// Response from GET /markets/{ticker}/orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookResponse {
    /// The orderbook. Empty if the market has no orders (API returns null).
    #[serde(deserialize_with = "orderbook_serde::deserialize")]
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
#[non_exhaustive]
pub enum TakerSide {
    Yes,
    No,
    /// Unknown taker side returned by the API.
    #[serde(other)]
    Unknown,
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
    /// Contract quantity (fixed-point decimal string).
    #[serde(default)]
    pub count_fp: Option<String>,
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
    /// The value is clamped to the valid range of 1..=1000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit.clamp(1, 1000));
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

/// Candlestick period interval in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum CandlestickPeriod {
    /// 1 minute candles.
    OneMinute = 1,
    /// 1 hour candles.
    OneHour = 60,
    /// 1 day candles.
    OneDay = 1440,
}

impl CandlestickPeriod {
    /// Get the period as minutes.
    pub fn as_minutes(&self) -> i32 {
        *self as i32
    }
}

/// OHLC (Open/High/Low/Close) candlestick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcData {
    /// Open price in cents.
    #[serde(default)]
    pub open: Option<i64>,
    /// Open price in dollars.
    #[serde(default)]
    pub open_dollars: Option<String>,
    /// Low price in cents.
    #[serde(default)]
    pub low: Option<i64>,
    /// Low price in dollars.
    #[serde(default)]
    pub low_dollars: Option<String>,
    /// High price in cents.
    #[serde(default)]
    pub high: Option<i64>,
    /// High price in dollars.
    #[serde(default)]
    pub high_dollars: Option<String>,
    /// Close price in cents.
    #[serde(default)]
    pub close: Option<i64>,
    /// Close price in dollars.
    #[serde(default)]
    pub close_dollars: Option<String>,
}

/// Extended OHLC data with additional price fields for trade prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceOhlcData {
    /// Open price in cents.
    #[serde(default)]
    pub open: Option<i64>,
    /// Open price in dollars.
    #[serde(default)]
    pub open_dollars: Option<String>,
    /// Low price in cents.
    #[serde(default)]
    pub low: Option<i64>,
    /// Low price in dollars.
    #[serde(default)]
    pub low_dollars: Option<String>,
    /// High price in cents.
    #[serde(default)]
    pub high: Option<i64>,
    /// High price in dollars.
    #[serde(default)]
    pub high_dollars: Option<String>,
    /// Close price in cents.
    #[serde(default)]
    pub close: Option<i64>,
    /// Close price in dollars.
    #[serde(default)]
    pub close_dollars: Option<String>,
    /// Mean price in cents.
    #[serde(default)]
    pub mean: Option<i64>,
    /// Mean price in dollars.
    #[serde(default)]
    pub mean_dollars: Option<String>,
    /// Previous close price in cents.
    #[serde(default)]
    pub previous: Option<i64>,
    /// Previous close price in dollars.
    #[serde(default)]
    pub previous_dollars: Option<String>,
    /// Min price in cents (alias for low).
    #[serde(default)]
    pub min: Option<i64>,
    /// Min price in dollars.
    #[serde(default)]
    pub min_dollars: Option<String>,
    /// Max price in cents (alias for high).
    #[serde(default)]
    pub max: Option<i64>,
    /// Max price in dollars.
    #[serde(default)]
    pub max_dollars: Option<String>,
}

/// A single candlestick data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    /// End of the period (Unix timestamp in seconds).
    pub end_period_ts: i64,
    /// YES bid OHLC data.
    #[serde(default)]
    pub yes_bid: Option<OhlcData>,
    /// YES ask OHLC data.
    #[serde(default)]
    pub yes_ask: Option<OhlcData>,
    /// Trade price OHLC data.
    #[serde(default)]
    pub price: Option<PriceOhlcData>,
    /// Trading volume during the period.
    #[serde(default)]
    pub volume: Option<i64>,
    /// Trading volume during the period (fixed-point decimal string).
    #[serde(default)]
    pub volume_fp: Option<String>,
    /// Open interest at end of period.
    #[serde(default)]
    pub open_interest: Option<i64>,
    /// Open interest at end of period (fixed-point decimal string).
    #[serde(default)]
    pub open_interest_fp: Option<String>,
}

/// Response from GET /series/{series_ticker}/markets/{ticker}/candlesticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlesticksResponse {
    /// Market ticker.
    pub ticker: String,
    /// Array of candlestick data.
    pub candlesticks: Vec<Candlestick>,
}

/// Candlestick data for a single market in batch response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCandlesticks {
    /// Market ticker.
    pub market_ticker: String,
    /// Array of candlestick data.
    pub candlesticks: Vec<Candlestick>,
}

/// Response from GET /markets/candlesticks (batch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCandlesticksResponse {
    /// Array of market candlestick data.
    pub markets: Vec<MarketCandlesticks>,
}

/// Query parameters for GET /series/{series_ticker}/markets/{ticker}/candlesticks.
#[derive(Debug, Clone, Serialize)]
pub struct GetCandlesticksParams {
    /// Start timestamp (Unix seconds).
    pub start_ts: i64,
    /// End timestamp (Unix seconds).
    pub end_ts: i64,
    /// Candlestick period interval.
    pub period_interval: CandlestickPeriod,
    /// Include synthetic candlestick before start_ts for price continuity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_latest_before_start: Option<bool>,
}

impl GetCandlesticksParams {
    /// Create new candlesticks query parameters.
    ///
    /// # Panics
    ///
    /// Panics if `start_ts >= end_ts`.
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(start_ts: i64, end_ts: i64, period_interval: CandlestickPeriod) -> Self {
        Self::try_new(start_ts, end_ts, period_interval).expect("invalid candlestick parameters")
    }

    /// Create new candlesticks query parameters with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `start_ts >= end_ts`.
    pub fn try_new(
        start_ts: i64,
        end_ts: i64,
        period_interval: CandlestickPeriod,
    ) -> crate::error::Result<Self> {
        if start_ts >= end_ts {
            return Err(crate::error::Error::InvalidTimestampRange(start_ts, end_ts));
        }
        Ok(Self {
            start_ts,
            end_ts,
            period_interval,
            include_latest_before_start: None,
        })
    }

    /// Include synthetic candlestick before start_ts for price continuity.
    #[must_use]
    pub fn include_latest_before_start(mut self, include: bool) -> Self {
        self.include_latest_before_start = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("start_ts", self.start_ts);
        qb.push("end_ts", self.end_ts);
        qb.push("period_interval", self.period_interval.as_minutes());
        qb.push_opt(
            "include_latest_before_start",
            self.include_latest_before_start,
        );
        qb.build()
    }
}

/// Query parameters for GET /markets/candlesticks (batch).
#[derive(Debug, Clone, Serialize)]
pub struct GetBatchCandlesticksParams {
    /// Comma-separated list of market tickers (max 100).
    pub market_tickers: String,
    /// Start timestamp (Unix seconds).
    pub start_ts: i64,
    /// End timestamp (Unix seconds).
    pub end_ts: i64,
    /// Candlestick period interval in minutes.
    pub period_interval: i32,
    /// Include synthetic candlestick before start_ts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_latest_before_start: Option<bool>,
}

impl GetBatchCandlesticksParams {
    /// Create new batch candlesticks query parameters.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - More than 100 market tickers are provided (count by splitting on commas)
    /// - `start_ts >= end_ts`
    ///
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(
        market_tickers: impl Into<String>,
        start_ts: i64,
        end_ts: i64,
        period_interval: CandlestickPeriod,
    ) -> Self {
        Self::try_new(market_tickers, start_ts, end_ts, period_interval)
            .expect("invalid batch candlestick parameters")
    }

    /// Create new batch candlesticks query parameters with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - More than 100 market tickers are provided
    /// - `start_ts >= end_ts`
    pub fn try_new(
        market_tickers: impl Into<String>,
        start_ts: i64,
        end_ts: i64,
        period_interval: CandlestickPeriod,
    ) -> crate::error::Result<Self> {
        let tickers = market_tickers.into();
        let ticker_count = tickers.split(',').filter(|s| !s.is_empty()).count();
        if ticker_count > crate::error::MAX_BATCH_CANDLESTICKS_TICKERS {
            return Err(crate::error::Error::TooManyMarketTickers(ticker_count));
        }
        if start_ts >= end_ts {
            return Err(crate::error::Error::InvalidTimestampRange(start_ts, end_ts));
        }
        Ok(Self {
            market_tickers: tickers,
            start_ts,
            end_ts,
            period_interval: period_interval.as_minutes(),
            include_latest_before_start: None,
        })
    }

    /// Create from a list of tickers.
    ///
    /// # Panics
    ///
    /// Panics if more than 100 tickers are provided or if
    /// `start_ts >= end_ts`. Use [`try_from_tickers`](Self::try_from_tickers) for
    /// fallible construction.
    #[must_use]
    pub fn from_tickers(
        tickers: &[&str],
        start_ts: i64,
        end_ts: i64,
        period_interval: CandlestickPeriod,
    ) -> Self {
        Self::new(tickers.join(","), start_ts, end_ts, period_interval)
    }

    /// Create from a list of tickers with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if more than 100 tickers are provided or if
    /// `start_ts >= end_ts`.
    pub fn try_from_tickers(
        tickers: &[&str],
        start_ts: i64,
        end_ts: i64,
        period_interval: CandlestickPeriod,
    ) -> crate::error::Result<Self> {
        Self::try_new(tickers.join(","), start_ts, end_ts, period_interval)
    }

    /// Include synthetic candlestick before start_ts for price continuity.
    #[must_use]
    pub fn include_latest_before_start(mut self, include: bool) -> Self {
        self.include_latest_before_start = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("market_tickers", &self.market_tickers);
        qb.push("start_ts", self.start_ts);
        qb.push("end_ts", self.end_ts);
        qb.push("period_interval", self.period_interval);
        qb.push_opt(
            "include_latest_before_start",
            self.include_latest_before_start,
        );
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

    #[test]
    fn test_limit_clamping() {
        // Test clamping to minimum
        let params = GetMarketsParams::new().limit(0);
        assert_eq!(params.limit, Some(1));

        let params = GetMarketsParams::new().limit(-10);
        assert_eq!(params.limit, Some(1));

        // Test clamping to maximum
        let params = GetMarketsParams::new().limit(2000);
        assert_eq!(params.limit, Some(1000));

        // Test value in range is unchanged
        let params = GetMarketsParams::new().limit(500);
        assert_eq!(params.limit, Some(500));

        // Same for GetTradesParams
        let params = GetTradesParams::new().limit(0);
        assert_eq!(params.limit, Some(1));

        let params = GetTradesParams::new().limit(9999);
        assert_eq!(params.limit, Some(1000));
    }

    #[test]
    fn test_market_type_deserialize_unknown() {
        let json = r#""some_future_type""#;
        let market_type: MarketType = serde_json::from_str(json).unwrap();
        assert_eq!(market_type, MarketType::Unknown);
    }

    #[test]
    fn test_market_status_deserialize_unknown() {
        let json = r#""pending_review""#;
        let status: MarketStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, MarketStatus::Unknown);
    }

    #[test]
    fn test_market_result_deserialize_unknown() {
        let json = r#""void""#;
        let result: MarketResult = serde_json::from_str(json).unwrap();
        assert_eq!(result, MarketResult::Unknown);
    }

    #[test]
    fn test_taker_side_deserialize_unknown() {
        let json = r#""both""#;
        let side: TakerSide = serde_json::from_str(json).unwrap();
        assert_eq!(side, TakerSide::Unknown);
    }

    #[test]
    fn test_strike_type_deserialize_unknown() {
        let json = r#""exotic""#;
        let strike: StrikeType = serde_json::from_str(json).unwrap();
        assert_eq!(strike, StrikeType::Unknown);
    }

    #[test]
    fn test_market_deserialize() {
        let json = r#"{
            "ticker": "KXBTC-25JAN10-B50000",
            "event_ticker": "KXBTC-25JAN10",
            "market_type": "binary",
            "status": "active",
            "title": "Will Bitcoin reach $50,000?",
            "volume": 1000,
            "volume_fp": "1000.5",
            "volume_24h": 500,
            "volume_24h_fp": "500.25",
            "open_interest": 250,
            "open_interest_fp": "250.125"
        }"#;
        let market: Market = serde_json::from_str(json).unwrap();
        assert_eq!(market.ticker, "KXBTC-25JAN10-B50000");
        assert_eq!(market.market_type, MarketType::Binary);
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.volume, Some(1000));
        assert_eq!(market.volume_fp, Some("1000.5".to_string()));
        assert_eq!(market.volume_24h_fp, Some("500.25".to_string()));
        assert_eq!(market.open_interest_fp, Some("250.125".to_string()));
    }

    #[test]
    fn test_orderbook_deserialize() {
        let json = r#"{
            "yes": [[50, 100], [55, 200]],
            "no": [[45, 150]],
            "yes_dollars": [["0.50", 100], ["0.55", 200]],
            "no_dollars": [["0.45", 150]]
        }"#;
        let orderbook: Orderbook = serde_json::from_str(json).unwrap();
        assert_eq!(orderbook.yes.len(), 2);
        assert_eq!(orderbook.yes[0], vec![50, 100]);
        assert_eq!(orderbook.no.len(), 1);

        let yes_dollars = orderbook.yes_dollars.unwrap();
        assert_eq!(yes_dollars.len(), 2);
        assert_eq!(yes_dollars[0].price, "0.50");
        assert_eq!(yes_dollars[0].quantity, 100);
    }

    #[test]
    fn test_orderbook_deserialize_empty() {
        let json = r#"{"yes": [], "no": []}"#;
        let orderbook: Orderbook = serde_json::from_str(json).unwrap();
        assert!(orderbook.yes.is_empty());
        assert!(orderbook.no.is_empty());
        assert!(orderbook.yes_dollars.is_none());
    }

    #[test]
    fn test_orderbook_response_null() {
        // API returns null for markets with no orders
        let json = r#"{"orderbook": null}"#;
        let response: OrderbookResponse = serde_json::from_str(json).unwrap();
        assert!(response.orderbook.yes.is_empty());
        assert!(response.orderbook.no.is_empty());
    }

    #[test]
    fn test_orderbook_null_fields() {
        // API can return null for yes/no fields within orderbook
        let json = r#"{"orderbook": {"yes": null, "no": null}}"#;
        let response: OrderbookResponse = serde_json::from_str(json).unwrap();
        assert!(response.orderbook.yes.is_empty());
        assert!(response.orderbook.no.is_empty());
    }

    #[test]
    fn test_trade_deserialize() {
        let json = r#"{
            "trade_id": "abc123",
            "ticker": "KXBTC-25JAN10-B50000",
            "count": 10,
            "count_fp": "10.5",
            "yes_price": 50,
            "no_price": 50,
            "taker_side": "yes",
            "created_time": "2025-01-10T12:00:00Z"
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.trade_id, "abc123");
        assert_eq!(trade.count, 10);
        assert_eq!(trade.count_fp, Some("10.5".to_string()));
        assert_eq!(trade.taker_side, TakerSide::Yes);
    }

    #[test]
    fn test_markets_response_deserialize() {
        let json = r#"{
            "markets": [{
                "ticker": "TEST-001",
                "event_ticker": "TEST",
                "market_type": "binary",
                "status": "active"
            }],
            "cursor": "next_page_token"
        }"#;
        let response: MarketsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.markets.len(), 1);
        assert_eq!(response.cursor, Some("next_page_token".to_string()));
    }

    #[test]
    fn test_candlesticks_params_validation() {
        // Valid params should work
        let params = GetCandlesticksParams::new(1000, 2000, CandlestickPeriod::OneHour);
        assert_eq!(params.start_ts, 1000);
        assert_eq!(params.end_ts, 2000);

        // try_new with valid params
        let params = GetCandlesticksParams::try_new(1000, 2000, CandlestickPeriod::OneMinute);
        assert!(params.is_ok());

        // try_new with invalid range
        let params = GetCandlesticksParams::try_new(2000, 1000, CandlestickPeriod::OneDay);
        assert!(params.is_err());

        // try_new with equal timestamps
        let params = GetCandlesticksParams::try_new(1000, 1000, CandlestickPeriod::OneDay);
        assert!(params.is_err());
    }

    #[test]
    fn test_batch_candlesticks_params_validation() {
        // Valid params should work
        let params =
            GetBatchCandlesticksParams::new("TICK1,TICK2", 1000, 2000, CandlestickPeriod::OneHour);
        assert_eq!(params.market_tickers, "TICK1,TICK2");

        // try_new with valid params
        let params =
            GetBatchCandlesticksParams::try_new("TICK1", 1000, 2000, CandlestickPeriod::OneMinute);
        assert!(params.is_ok());

        // try_new with too many tickers (101)
        let tickers: Vec<&str> = (0..101).map(|_| "TICK").collect();
        let params = GetBatchCandlesticksParams::try_from_tickers(
            &tickers,
            1000,
            2000,
            CandlestickPeriod::OneDay,
        );
        assert!(params.is_err());

        // try_new with exactly 100 tickers (should succeed)
        let tickers: Vec<&str> = (0..100).map(|_| "T").collect();
        let params = GetBatchCandlesticksParams::try_from_tickers(
            &tickers,
            1000,
            2000,
            CandlestickPeriod::OneDay,
        );
        assert!(params.is_ok());

        // try_new with invalid timestamp range
        let params =
            GetBatchCandlesticksParams::try_new("TICK1", 2000, 1000, CandlestickPeriod::OneHour);
        assert!(params.is_err());
    }

    #[test]
    fn test_candlestick_period_values() {
        assert_eq!(CandlestickPeriod::OneMinute.as_minutes(), 1);
        assert_eq!(CandlestickPeriod::OneHour.as_minutes(), 60);
        assert_eq!(CandlestickPeriod::OneDay.as_minutes(), 1440);
    }
}
