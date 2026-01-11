//! Settlement models and query parameters.

use serde::{Deserialize, Serialize};

use super::market::MarketResult;
use super::query::QueryBuilder;

/// A settlement record for a market position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    /// Market ticker.
    pub ticker: String,
    /// Event ticker.
    pub event_ticker: String,
    /// Market result (yes/no).
    pub market_result: MarketResult,
    /// Number of YES contracts held at settlement.
    pub yes_count: i64,
    /// Total cost of YES contracts in cents.
    pub yes_total_cost: i64,
    /// Number of NO contracts held at settlement.
    pub no_count: i64,
    /// Total cost of NO contracts in cents.
    pub no_total_cost: i64,
    /// Revenue from settlement in cents.
    pub revenue: i64,
    /// Settlement timestamp.
    pub settled_time: String,
    /// Fee cost in dollars.
    #[serde(default)]
    pub fee_cost: Option<String>,
    /// Settlement value in cents.
    #[serde(default)]
    pub value: Option<i64>,
}

/// Response from GET /portfolio/settlements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementsResponse {
    pub settlements: Vec<Settlement>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for GET /portfolio/settlements.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetSettlementsParams {
    /// Maximum number of results (1-200, default 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by market ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    /// Filter by event ticker (comma-separated, max 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter items after this Unix timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    /// Filter items before this Unix timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
}

impl GetSettlementsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results to return.
    ///
    /// The value is clamped to the valid range of 1..=200.
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

    /// Filter by market ticker.
    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by event ticker (comma-separated for multiple, max 10).
    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    /// Filter items after this Unix timestamp.
    #[must_use]
    pub fn min_ts(mut self, ts: i64) -> Self {
        self.min_ts = Some(ts);
        self
    }

    /// Filter items before this Unix timestamp.
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
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settlements_query_string() {
        let params = GetSettlementsParams::new().ticker("AAPL-25JAN").limit(50);
        let qs = params.to_query_string();
        assert!(qs.contains("ticker=AAPL-25JAN"));
        assert!(qs.contains("limit=50"));
    }

    #[test]
    fn test_settlements_limit_clamping() {
        let params = GetSettlementsParams::new().limit(500);
        assert_eq!(params.limit, Some(200));

        let params = GetSettlementsParams::new().limit(0);
        assert_eq!(params.limit, Some(1));
    }
}
