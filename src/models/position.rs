//! Position models and query parameters.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// A position in a specific market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPosition {
    pub ticker: String,
    /// In cents.
    pub total_traded: i64,
    #[serde(default)]
    pub total_traded_dollars: Option<String>,
    /// Negative = NO position, positive = YES position.
    pub position: i64,
    /// Position (fixed-point decimal string).
    #[serde(default)]
    pub position_fp: Option<String>,
    /// Position cost in cents.
    pub market_exposure: i64,
    #[serde(default)]
    pub market_exposure_dollars: Option<String>,
    /// In cents.
    pub realized_pnl: i64,
    #[serde(default)]
    pub realized_pnl_dollars: Option<String>,
    pub resting_orders_count: i64,
    /// Resting orders count (fixed-point decimal string).
    #[serde(default)]
    pub resting_orders_count_fp: Option<String>,
    /// In cents.
    pub fees_paid: i64,
    #[serde(default)]
    pub fees_paid_dollars: Option<String>,
    #[serde(default)]
    pub last_updated_ts: Option<String>,
}

/// A position aggregated at the event level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPosition {
    pub event_ticker: String,
    /// In cents.
    pub total_cost: i64,
    #[serde(default)]
    pub total_cost_dollars: Option<String>,
    pub total_cost_shares: i64,
    /// Total cost shares (fixed-point decimal string).
    #[serde(default)]
    pub total_cost_shares_fp: Option<String>,
    /// In cents.
    pub event_exposure: i64,
    #[serde(default)]
    pub event_exposure_dollars: Option<String>,
    /// In cents.
    pub realized_pnl: i64,
    #[serde(default)]
    pub realized_pnl_dollars: Option<String>,
    /// In cents.
    pub fees_paid: i64,
    #[serde(default)]
    pub fees_paid_dollars: Option<String>,
}

/// Response from the get_positions endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsResponse {
    pub market_positions: Vec<MarketPosition>,
    pub event_positions: Vec<EventPosition>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for the get_positions endpoint.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetPositionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter by subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl GetPositionsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
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
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn count_filter(mut self, filter: impl Into<String>) -> Self {
        self.count_filter = Some(filter.into());
        self
    }

    /// Filter by subaccount number.
    ///
    /// Use 0 for the primary account, or 1-32 for numbered subaccounts.
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("count_filter", self.count_filter.as_ref());
        qb.push_opt("subaccount", self.subaccount);
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_empty() {
        let params = GetPositionsParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_query_string_single_param() {
        let params = GetPositionsParams::new().limit(10);
        assert_eq!(params.to_query_string(), "?limit=10");
    }

    #[test]
    fn test_query_string_url_encoding() {
        let params = GetPositionsParams::new().ticker("TEST&SPECIAL=chars");
        assert_eq!(params.to_query_string(), "?ticker=TEST%26SPECIAL%3Dchars");
    }
}
