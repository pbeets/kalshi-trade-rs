//! Event models and response types.
//!
//! Types for events and related data.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::market::Market;
use super::query::QueryBuilder;

/// Event status for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Open,
    Closed,
    Settled,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Open => "open",
            EventStatus::Closed => "closed",
            EventStatus::Settled => "settled",
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An event on the Kalshi exchange.
///
/// Events group related markets together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_ticker: String,
    pub series_ticker: String,
    pub title: String,

    #[serde(default)]
    pub sub_title: Option<String>,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(default)]
    pub collateral_return_type: Option<String>,

    #[serde(default)]
    pub mutually_exclusive: Option<bool>,

    #[serde(default)]
    pub available_on_brokers: Option<bool>,

    #[serde(default)]
    pub product_metadata: Option<serde_json::Value>,

    /// Specific date for date-strike events (RFC3339).
    #[serde(default)]
    pub strike_date: Option<String>,

    /// Time period for period-strike events.
    #[serde(default)]
    pub strike_period: Option<String>,

    /// Nested markets (only present if with_nested_markets=true).
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
}

/// A milestone related to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(rename = "type")]
    #[serde(default)]
    pub milestone_type: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub start_date: Option<String>,

    #[serde(default)]
    pub end_date: Option<String>,

    #[serde(default)]
    pub related_event_tickers: Option<Vec<String>>,
}

/// Response from GET /events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<Event>,

    #[serde(default)]
    pub cursor: Option<String>,

    /// Milestones (only present if with_milestones=true).
    #[serde(default)]
    pub milestones: Option<Vec<Milestone>>,
}

/// Response from GET /events/{event_ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub event: Event,

    /// Markets in this event.
    ///
    /// **Deprecated:** Use `get_event_with_params()` with `with_nested_markets(true)`
    /// instead, which returns markets in the `event.markets` field.
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
}

/// Query parameters for GET /events.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_milestones: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_close_ts: Option<i64>,
}

impl GetEventsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return (1-200).
    ///
    /// Values outside this range are clamped.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit.clamp(1, 200));
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn status(mut self, status: EventStatus) -> Self {
        self.status = Some(status);
        self
    }

    #[must_use]
    pub fn series_ticker(mut self, series_ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(series_ticker.into());
        self
    }

    /// Include nested markets in the response.
    #[must_use]
    pub fn with_nested_markets(mut self, include: bool) -> Self {
        self.with_nested_markets = Some(include);
        self
    }

    /// Include milestones in the response.
    #[must_use]
    pub fn with_milestones(mut self, include: bool) -> Self {
        self.with_milestones = Some(include);
        self
    }

    /// Filter events with at least one market closing after this Unix timestamp.
    #[must_use]
    pub fn min_close_ts(mut self, ts: i64) -> Self {
        self.min_close_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("status", self.status.map(|s| s.as_str()));
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.push_opt("with_nested_markets", self.with_nested_markets);
        qb.push_opt("with_milestones", self.with_milestones);
        qb.push_opt("min_close_ts", self.min_close_ts);
        qb.build()
    }
}

/// Query parameters for GET /events/{event_ticker}.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetEventParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>,
}

impl GetEventParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Include nested markets in the event object.
    #[must_use]
    pub fn with_nested_markets(mut self, include: bool) -> Self {
        self.with_nested_markets = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("with_nested_markets", self.with_nested_markets);
        qb.build()
    }
}

// =========================================================================
// Event Metadata Types
// =========================================================================

/// Metadata for a market within an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDetail {
    /// The ticker of the market.
    pub market_ticker: String,
    /// Path to an image representing this market.
    pub image_url: String,
    /// Color code for the market.
    pub color_code: String,
}

/// A settlement source for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSource {
    /// Name of the settlement source.
    #[serde(default)]
    pub name: Option<String>,
    /// URL of the settlement source.
    #[serde(default)]
    pub url: Option<String>,
}

/// Response from GET /events/{event_ticker}/metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadataResponse {
    /// Path to an image representing this event.
    pub image_url: String,
    /// Metadata for the markets in this event.
    pub market_details: Vec<MarketDetail>,
    /// Settlement sources for this event.
    pub settlement_sources: Vec<SettlementSource>,
    /// Path to an image representing the featured market.
    #[serde(default)]
    pub featured_image_url: Option<String>,
    /// Event competition.
    #[serde(default)]
    pub competition: Option<String>,
    /// Event scope based on the competition.
    #[serde(default)]
    pub competition_scope: Option<String>,
}

// =========================================================================
// Multivariate Events Types
// =========================================================================

/// Query parameters for GET /events/multivariate.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetMultivariateEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_ticker: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_nested_markets: Option<bool>,
}

impl GetMultivariateEventsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return (1-200).
    ///
    /// Values outside this range are clamped.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit.clamp(1, 200));
        self
    }

    /// Set the pagination cursor for fetching subsequent pages.
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filter by series ticker.
    ///
    /// Cannot be used together with `collection_ticker`.
    #[must_use]
    pub fn series_ticker(mut self, series_ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(series_ticker.into());
        self
    }

    /// Filter by collection ticker.
    ///
    /// Cannot be used together with `series_ticker`.
    #[must_use]
    pub fn collection_ticker(mut self, collection_ticker: impl Into<String>) -> Self {
        self.collection_ticker = Some(collection_ticker.into());
        self
    }

    /// Include nested markets in the response.
    #[must_use]
    pub fn with_nested_markets(mut self, include: bool) -> Self {
        self.with_nested_markets = Some(include);
        self
    }

    /// Validate the parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if both `series_ticker` and `collection_ticker` are set.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.series_ticker.is_some() && self.collection_ticker.is_some() {
            return Err(crate::error::Error::MutuallyExclusiveParams);
        }
        Ok(())
    }

    /// Build the query string, validating parameters first.
    ///
    /// # Errors
    ///
    /// Returns an error if both `series_ticker` and `collection_ticker` are set.
    pub fn try_to_query_string(&self) -> crate::error::Result<String> {
        self.validate()?;
        Ok(self.to_query_string())
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.push_opt("collection_ticker", self.collection_ticker.as_ref());
        qb.push_opt("with_nested_markets", self.with_nested_markets);
        qb.build()
    }
}

/// Response from GET /events/multivariate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateEventsResponse {
    pub events: Vec<Event>,

    #[serde(default)]
    pub cursor: Option<String>,
}

// =========================================================================
// Event Candlesticks Types
// =========================================================================

use super::market::Candlestick;

/// Query parameters for GET /series/{series_ticker}/events/{ticker}/candlesticks.
#[derive(Debug, Clone, Serialize)]
pub struct GetEventCandlesticksParams {
    /// Start timestamp (Unix seconds).
    pub start_ts: i64,
    /// End timestamp (Unix seconds).
    pub end_ts: i64,
    /// Candlestick period interval in minutes (1, 60, or 1440).
    pub period_interval: i32,
}

impl GetEventCandlesticksParams {
    /// Create new event candlesticks query parameters.
    ///
    /// # Arguments
    ///
    /// * `start_ts` - Start timestamp (Unix seconds)
    /// * `end_ts` - End timestamp (Unix seconds)
    /// * `period_interval` - Period in minutes (1, 60, or 1440)
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `start_ts >= end_ts`.
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(start_ts: i64, end_ts: i64, period_interval: super::market::CandlestickPeriod) -> Self {
        debug_assert!(
            start_ts < end_ts,
            "start_ts ({}) must be less than end_ts ({})",
            start_ts,
            end_ts
        );
        Self {
            start_ts,
            end_ts,
            period_interval: period_interval.as_minutes(),
        }
    }

    /// Create new event candlesticks query parameters with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `start_ts >= end_ts`.
    pub fn try_new(
        start_ts: i64,
        end_ts: i64,
        period_interval: super::market::CandlestickPeriod,
    ) -> crate::error::Result<Self> {
        if start_ts >= end_ts {
            return Err(crate::error::Error::InvalidTimestampRange(start_ts, end_ts));
        }
        Ok(Self {
            start_ts,
            end_ts,
            period_interval: period_interval.as_minutes(),
        })
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("start_ts", self.start_ts);
        qb.push("end_ts", self.end_ts);
        qb.push("period_interval", self.period_interval);
        qb.build()
    }
}

/// Response from GET /series/{series_ticker}/events/{ticker}/candlesticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCandlesticksResponse {
    /// Array of market tickers in the event.
    pub market_tickers: Vec<String>,
    /// Array of candlestick arrays, one per market.
    pub market_candlesticks: Vec<Vec<Candlestick>>,
    /// Adjusted end timestamp if requested range exceeded maximum.
    #[serde(default)]
    pub adjusted_end_ts: Option<i64>,
}

// =========================================================================
// Event Forecast Percentile History Types
// =========================================================================

/// Period interval for forecast percentile history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ForecastPeriod {
    /// 5-second intervals.
    FiveSeconds = 0,
    /// 1 minute intervals.
    OneMinute = 1,
    /// 1 hour intervals.
    OneHour = 60,
    /// 1 day intervals.
    OneDay = 1440,
}

impl ForecastPeriod {
    /// Get the period as minutes (0 for 5-second intervals).
    pub fn as_minutes(&self) -> i32 {
        *self as i32
    }
}

/// Maximum number of percentiles allowed in a forecast history request.
pub use crate::error::MAX_FORECAST_PERCENTILES;

/// Query parameters for GET /series/{series_ticker}/events/{ticker}/forecast_percentile_history.
#[derive(Debug, Clone, Serialize)]
pub struct GetEventForecastPercentileHistoryParams {
    /// Percentile values to retrieve (0-10000, max 10 values).
    pub percentiles: Vec<i32>,
    /// Start timestamp (Unix seconds).
    pub start_ts: i64,
    /// End timestamp (Unix seconds).
    pub end_ts: i64,
    /// Forecast period interval.
    pub period_interval: i32,
}

impl GetEventForecastPercentileHistoryParams {
    /// Create new forecast percentile history query parameters.
    ///
    /// # Arguments
    ///
    /// * `percentiles` - Percentile values to retrieve (0-10000, max 10)
    /// * `start_ts` - Start timestamp (Unix seconds)
    /// * `end_ts` - End timestamp (Unix seconds)
    /// * `period_interval` - Period interval for the forecast
    ///
    /// # Panics
    ///
    /// Panics in debug builds if:
    /// - More than 10 percentiles are provided
    /// - Any percentile value is outside 0-10000
    /// - `start_ts >= end_ts`
    ///
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(
        percentiles: Vec<i32>,
        start_ts: i64,
        end_ts: i64,
        period_interval: ForecastPeriod,
    ) -> Self {
        debug_assert!(
            percentiles.len() <= MAX_FORECAST_PERCENTILES,
            "forecast percentile history supports max {} percentiles, got {}",
            MAX_FORECAST_PERCENTILES,
            percentiles.len()
        );
        debug_assert!(
            percentiles.iter().all(|&p| (0..=10000).contains(&p)),
            "all percentile values must be between 0 and 10000"
        );
        debug_assert!(
            start_ts < end_ts,
            "start_ts ({}) must be less than end_ts ({})",
            start_ts,
            end_ts
        );
        Self {
            percentiles,
            start_ts,
            end_ts,
            period_interval: period_interval.as_minutes(),
        }
    }

    /// Create new forecast percentile history query parameters with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - More than 10 percentiles are provided
    /// - Any percentile value is outside 0-10000
    /// - `start_ts >= end_ts`
    pub fn try_new(
        percentiles: Vec<i32>,
        start_ts: i64,
        end_ts: i64,
        period_interval: ForecastPeriod,
    ) -> crate::error::Result<Self> {
        if percentiles.len() > MAX_FORECAST_PERCENTILES {
            return Err(crate::error::Error::TooManyPercentiles(percentiles.len()));
        }
        if let Some(&p) = percentiles.iter().find(|&&p| !(0..=10000).contains(&p)) {
            return Err(crate::error::Error::PercentileOutOfRange(p));
        }
        if start_ts >= end_ts {
            return Err(crate::error::Error::InvalidTimestampRange(start_ts, end_ts));
        }
        Ok(Self {
            percentiles,
            start_ts,
            end_ts,
            period_interval: period_interval.as_minutes(),
        })
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        // Percentiles needs to be passed as repeated query params
        for p in &self.percentiles {
            qb.push("percentiles", p);
        }
        qb.push("start_ts", self.start_ts);
        qb.push("end_ts", self.end_ts);
        qb.push("period_interval", self.period_interval);
        qb.build()
    }
}

/// A forecast value at a specific percentile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentilePoint {
    /// The percentile value (0-10000).
    pub percentile: i32,
    /// The raw numerical forecast value.
    #[serde(default)]
    pub raw_numerical_forecast: Option<f64>,
    /// The processed numerical forecast value.
    #[serde(default)]
    pub numerical_forecast: Option<f64>,
    /// The human-readable formatted forecast value.
    #[serde(default)]
    pub formatted_forecast: Option<String>,
}

/// A single forecast history data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastHistoryPoint {
    /// The event ticker this forecast is for.
    pub event_ticker: String,
    /// Unix timestamp for the inclusive end of the forecast period.
    pub end_period_ts: i64,
    /// Length of the forecast period in minutes.
    pub period_interval: i32,
    /// Array of forecast values at different percentiles.
    pub percentile_points: Vec<PercentilePoint>,
}

/// Response from GET /series/{series_ticker}/events/{ticker}/forecast_percentile_history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventForecastPercentileHistoryResponse {
    /// Array of forecast percentile data points over time.
    pub forecast_history: Vec<ForecastHistoryPoint>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_events_query_string() {
        let params = GetEventsParams::new()
            .status(EventStatus::Open)
            .limit(50)
            .with_nested_markets(true);
        let qs = params.to_query_string();
        assert!(qs.contains("status=open"));
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("with_nested_markets=true"));
    }

    #[test]
    fn test_get_event_query_string() {
        let params = GetEventParams::new().with_nested_markets(true);
        assert_eq!(params.to_query_string(), "?with_nested_markets=true");
    }

    #[test]
    fn test_get_multivariate_events_query_string() {
        let params = GetMultivariateEventsParams::new()
            .collection_ticker("COLL-123")
            .limit(50)
            .with_nested_markets(true);
        let qs = params.to_query_string();
        assert!(qs.contains("collection_ticker=COLL-123"));
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("with_nested_markets=true"));
    }

    #[test]
    fn test_get_multivariate_events_limit_clamping() {
        let params = GetMultivariateEventsParams::new().limit(0);
        assert_eq!(params.limit, Some(1));

        let params = GetMultivariateEventsParams::new().limit(500);
        assert_eq!(params.limit, Some(200));

        let params = GetMultivariateEventsParams::new().limit(100);
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_get_event_candlesticks_query_string() {
        use super::super::market::CandlestickPeriod;

        let params = GetEventCandlesticksParams::new(1000, 2000, CandlestickPeriod::OneHour);
        let qs = params.to_query_string();
        assert!(qs.contains("start_ts=1000"));
        assert!(qs.contains("end_ts=2000"));
        assert!(qs.contains("period_interval=60"));
    }

    #[test]
    fn test_get_event_candlesticks_validation() {
        use super::super::market::CandlestickPeriod;

        let result = GetEventCandlesticksParams::try_new(2000, 1000, CandlestickPeriod::OneHour);
        assert!(result.is_err());

        let result = GetEventCandlesticksParams::try_new(1000, 2000, CandlestickPeriod::OneHour);
        assert!(result.is_ok());
    }

    #[test]
    fn test_forecast_period_values() {
        assert_eq!(ForecastPeriod::FiveSeconds.as_minutes(), 0);
        assert_eq!(ForecastPeriod::OneMinute.as_minutes(), 1);
        assert_eq!(ForecastPeriod::OneHour.as_minutes(), 60);
        assert_eq!(ForecastPeriod::OneDay.as_minutes(), 1440);
    }

    #[test]
    fn test_get_forecast_percentile_history_query_string() {
        let params = GetEventForecastPercentileHistoryParams::new(
            vec![2500, 5000, 7500],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        let qs = params.to_query_string();
        assert!(qs.contains("percentiles=2500"));
        assert!(qs.contains("percentiles=5000"));
        assert!(qs.contains("percentiles=7500"));
        assert!(qs.contains("start_ts=1000"));
        assert!(qs.contains("end_ts=2000"));
        assert!(qs.contains("period_interval=60"));
    }

    #[test]
    fn test_get_forecast_percentile_history_validation() {
        // Too many percentiles
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_err());

        // Invalid timestamp range
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![5000],
            2000,
            1000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_err());

        // Valid params
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![2500, 5000, 7500],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_forecast_percentile_history_range_validation() {
        // Percentile below 0
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![-1],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_err());

        // Percentile above 10000
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![10001],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_err());

        // Boundary values (0 and 10000 should be valid)
        let result = GetEventForecastPercentileHistoryParams::try_new(
            vec![0, 10000],
            1000,
            2000,
            ForecastPeriod::OneHour,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_multivariate_events_mutual_exclusivity() {
        // Both series_ticker and collection_ticker set - should fail
        let params = GetMultivariateEventsParams::new()
            .series_ticker("SERIES-1")
            .collection_ticker("COLL-1");
        assert!(params.validate().is_err());

        // Only series_ticker - should pass
        let params = GetMultivariateEventsParams::new().series_ticker("SERIES-1");
        assert!(params.validate().is_ok());

        // Only collection_ticker - should pass
        let params = GetMultivariateEventsParams::new().collection_ticker("COLL-1");
        assert!(params.validate().is_ok());

        // Neither set - should pass
        let params = GetMultivariateEventsParams::new();
        assert!(params.validate().is_ok());

        // try_to_query_string should also validate
        let params = GetMultivariateEventsParams::new()
            .series_ticker("SERIES-1")
            .collection_ticker("COLL-1");
        assert!(params.try_to_query_string().is_err());
    }
}
