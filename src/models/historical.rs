//! Historical data models and query parameters.

use serde::{Deserialize, Serialize};

use super::market::{CandlestickPeriod, MveFilter};
use super::query::QueryBuilder;

/// Response from GET /historical/cutoff.
///
/// Contains timestamps indicating when historical data was last updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCutoffResponse {
    /// ISO 8601 datetime when market settlement data was last archived.
    pub market_settled_ts: String,
    /// ISO 8601 datetime when trade data was last archived.
    pub trades_created_ts: String,
    /// ISO 8601 datetime when order data was last archived.
    pub orders_updated_ts: String,
}

/// Query parameters for GET /historical/markets.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetHistoricalMarketsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by specific market tickers. Comma-separated list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickers: Option<String>,
    /// Filter by event ticker. Comma-separated, max 10.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter by multivariate event status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mve_filter: Option<MveFilter>,
}

impl GetHistoricalMarketsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is not in the range 1..=1000.
    /// Use [`try_limit`](Self::try_limit) for fallible construction.
    #[must_use]
    pub fn limit(self, limit: i64) -> Self {
        self.try_limit(limit).expect("invalid limit")
    }

    /// Set the maximum number of results to return with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `limit` is not in the range 1..=1000.
    pub fn try_limit(mut self, limit: i64) -> crate::error::Result<Self> {
        if limit <= 0 || limit > 1000 {
            return Err(crate::error::Error::InvalidLimit(limit, 1, 1000));
        }
        self.limit = Some(limit);
        Ok(self)
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filter by specific market tickers. Comma-separated list.
    #[must_use]
    pub fn tickers(mut self, tickers: impl Into<String>) -> Self {
        self.tickers = Some(tickers.into());
        self
    }

    /// Filter by event ticker. Comma-separated, max 10.
    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    /// Filter by multivariate event status.
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
        qb.push_opt("tickers", self.tickers.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt(
            "mve_filter",
            self.mve_filter.as_ref().map(MveFilter::as_str),
        );
        qb.build()
    }
}

/// Query parameters for GET /historical/markets/{ticker}/candlesticks.
///
/// Similar to [`GetCandlesticksParams`](super::market::GetCandlesticksParams)
/// but without the `include_latest_before_start` option.
#[derive(Debug, Clone, Serialize)]
pub struct GetHistoricalCandlesticksParams {
    /// Start timestamp (Unix seconds).
    pub start_ts: i64,
    /// End timestamp (Unix seconds).
    pub end_ts: i64,
    /// Candlestick period interval.
    pub period_interval: CandlestickPeriod,
}

impl GetHistoricalCandlesticksParams {
    /// Create new historical candlesticks query parameters.
    ///
    /// # Panics
    ///
    /// Panics if `start_ts >= end_ts`.
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(start_ts: i64, end_ts: i64, period_interval: CandlestickPeriod) -> Self {
        Self::try_new(start_ts, end_ts, period_interval).expect("invalid candlestick parameters")
    }

    /// Create new historical candlesticks query parameters with validation.
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
        })
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("start_ts", self.start_ts);
        qb.push("end_ts", self.end_ts);
        qb.push("period_interval", self.period_interval.as_minutes());
        qb.build()
    }
}

/// Query parameters for GET /historical/fills.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetHistoricalFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetHistoricalFillsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn max_ts(mut self, ts: i64) -> Self {
        self.max_ts = Some(ts);
        self
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is not in the range 1..=200.
    /// Use [`try_limit`](Self::try_limit) for fallible construction.
    #[must_use]
    pub fn limit(self, limit: i64) -> Self {
        self.try_limit(limit).expect("invalid limit")
    }

    /// Set the maximum number of results to return with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `limit` is not in the range 1..=200.
    pub fn try_limit(mut self, limit: i64) -> crate::error::Result<Self> {
        if limit <= 0 || limit > 200 {
            return Err(crate::error::Error::InvalidLimit(limit, 1, 200));
        }
        self.limit = Some(limit);
        Ok(self)
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

/// Query parameters for GET /historical/orders.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetHistoricalOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetHistoricalOrdersParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn max_ts(mut self, ts: i64) -> Self {
        self.max_ts = Some(ts);
        self
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is not in the range 1..=200.
    /// Use [`try_limit`](Self::try_limit) for fallible construction.
    #[must_use]
    pub fn limit(self, limit: i64) -> Self {
        self.try_limit(limit).expect("invalid limit")
    }

    /// Set the maximum number of results to return with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `limit` is not in the range 1..=200.
    pub fn try_limit(mut self, limit: i64) -> crate::error::Result<Self> {
        if limit <= 0 || limit > 200 {
            return Err(crate::error::Error::InvalidLimit(limit, 1, 200));
        }
        self.limit = Some(limit);
        Ok(self)
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

/// Response from GET /historical/markets/{ticker}/candlesticks.
///
/// Historical candlesticks use dollar-denominated string fields exclusively,
/// unlike live candlesticks which include both cent integers and dollar strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCandlesticksResponse {
    /// Market ticker.
    pub ticker: String,
    /// Array of historical candlestick data.
    pub candlesticks: Vec<HistoricalCandlestick>,
}

/// A single historical candlestick data point.
///
/// All monetary values are fixed-point dollar strings (e.g. `"0.5600"`).
/// Volume and open interest are fixed-point count strings (e.g. `"100.0000"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCandlestick {
    /// End of the period (Unix timestamp in seconds).
    pub end_period_ts: i64,
    /// YES bid OHLC data in dollars.
    pub yes_bid: HistoricalOhlc,
    /// YES ask OHLC data in dollars.
    pub yes_ask: HistoricalOhlc,
    /// Trade price OHLC data in dollars.
    pub price: HistoricalPriceOhlc,
    /// Trading volume during the period (fixed-point count string).
    pub volume: String,
    /// Open interest at end of period (fixed-point count string).
    pub open_interest: String,
}

/// OHLC data for historical bid/ask levels.
///
/// All values are fixed-point dollar strings (e.g. `"0.5600"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalOhlc {
    /// Open price in dollars.
    pub open: String,
    /// Low price in dollars.
    pub low: String,
    /// High price in dollars.
    pub high: String,
    /// Close price in dollars.
    pub close: String,
}

/// Extended OHLC data for historical trade prices.
///
/// All values are nullable fixed-point dollar strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPriceOhlc {
    /// Open price in dollars.
    #[serde(default)]
    pub open: Option<String>,
    /// Low price in dollars.
    #[serde(default)]
    pub low: Option<String>,
    /// High price in dollars.
    #[serde(default)]
    pub high: Option<String>,
    /// Close price in dollars.
    #[serde(default)]
    pub close: Option<String>,
    /// Volume-weighted average price in dollars.
    #[serde(default)]
    pub mean: Option<String>,
    /// Previous period's close price in dollars.
    #[serde(default)]
    pub previous: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_markets_query_string() {
        let params = GetHistoricalMarketsParams::new()
            .tickers("AAPL,GOOG")
            .limit(50);
        assert_eq!(params.to_query_string(), "?limit=50&tickers=AAPL%2CGOOG");
    }

    #[test]
    fn test_historical_candlesticks_query_string() {
        let params = GetHistoricalCandlesticksParams::new(1000, 2000, CandlestickPeriod::OneHour);
        assert_eq!(
            params.to_query_string(),
            "?start_ts=1000&end_ts=2000&period_interval=60"
        );
    }

    #[test]
    fn test_historical_fills_query_string() {
        let params = GetHistoricalFillsParams::new().ticker("AAPL").limit(100);
        assert_eq!(params.to_query_string(), "?ticker=AAPL&limit=100");
    }

    #[test]
    fn test_historical_orders_query_string() {
        let params = GetHistoricalOrdersParams::new().ticker("AAPL").max_ts(1000);
        assert_eq!(params.to_query_string(), "?ticker=AAPL&max_ts=1000");
    }

    #[test]
    fn test_historical_markets_empty_query() {
        let params = GetHistoricalMarketsParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_historical_candlestick_deserialize() {
        let json = r#"{
            "ticker": "KXBTC-25JAN10-B50000",
            "candlesticks": [{
                "end_period_ts": 1704067200,
                "yes_bid": {
                    "open": "0.5600",
                    "low": "0.5400",
                    "high": "0.5800",
                    "close": "0.5700"
                },
                "yes_ask": {
                    "open": "0.5700",
                    "low": "0.5500",
                    "high": "0.5900",
                    "close": "0.5800"
                },
                "price": {
                    "open": "0.5600",
                    "low": "0.5400",
                    "high": "0.5800",
                    "close": "0.5700",
                    "mean": "0.5600",
                    "previous": "0.5500"
                },
                "volume": "100.0000",
                "open_interest": "50.0000"
            }]
        }"#;

        let response: HistoricalCandlesticksResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.ticker, "KXBTC-25JAN10-B50000");
        assert_eq!(response.candlesticks.len(), 1);

        let candle = &response.candlesticks[0];
        assert_eq!(candle.end_period_ts, 1704067200);
        assert_eq!(candle.volume, "100.0000");
        assert_eq!(candle.open_interest, "50.0000");

        assert_eq!(candle.yes_bid.open, "0.5600");
        assert_eq!(candle.yes_bid.close, "0.5700");

        assert_eq!(candle.price.mean, Some("0.5600".to_string()));
        assert_eq!(candle.price.previous, Some("0.5500".to_string()));
    }

    #[test]
    fn test_historical_candlestick_nullable_price() {
        let json = r#"{
            "ticker": "TEST",
            "candlesticks": [{
                "end_period_ts": 1000,
                "yes_bid": {
                    "open": "0.0000",
                    "low": "0.0000",
                    "high": "0.0000",
                    "close": "0.0000"
                },
                "yes_ask": {
                    "open": "0.0000",
                    "low": "0.0000",
                    "high": "0.0000",
                    "close": "0.0000"
                },
                "price": {
                    "open": null,
                    "low": null,
                    "high": null,
                    "close": null,
                    "mean": null,
                    "previous": null
                },
                "volume": "0.0000",
                "open_interest": "0.0000"
            }]
        }"#;

        let response: HistoricalCandlesticksResponse = serde_json::from_str(json).unwrap();
        let candle = &response.candlesticks[0];
        assert!(candle.price.open.is_none());
        assert!(candle.price.mean.is_none());
        assert_eq!(candle.yes_bid.open, "0.0000");
    }
}
