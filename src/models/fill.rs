//! Fill models and query parameters.

use serde::{Deserialize, Serialize};

use super::common::{Action, Side};
use super::query::QueryBuilder;

/// A fill represents a matched trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: String,
    /// Legacy field, same as fill_id.
    #[serde(default)]
    pub trade_id: Option<String>,
    pub order_id: String,
    pub ticker: String,
    /// Legacy field, same as ticker.
    #[serde(default)]
    pub market_ticker: Option<String>,
    pub side: Side,
    pub action: Action,
    pub count: i64,
    /// Count (fixed-point decimal string).
    #[serde(default)]
    pub count_fp: Option<String>,
    /// Price in cents.
    pub yes_price: i64,
    /// Price in cents.
    pub no_price: i64,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub yes_price_fixed: Option<String>,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub no_price_fixed: Option<String>,
    /// Whether this fill removed liquidity.
    pub is_taker: bool,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub created_time: String,
    /// Legacy Unix timestamp field.
    #[serde(default)]
    pub ts: Option<i64>,
}

/// Response from the get_fills endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillsResponse {
    pub fills: Vec<Fill>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for the get_fills endpoint.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetFillsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl GetFillsParams {
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
    pub fn order_id(mut self, order_id: impl Into<String>) -> Self {
        self.order_id = Some(order_id.into());
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
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("order_id", self.order_id.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("subaccount", self.subaccount);
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_multiple_params() {
        let params = GetFillsParams::new().ticker("AAPL").limit(50);
        assert_eq!(params.to_query_string(), "?ticker=AAPL&limit=50");
    }
}
