//! Fill models and query parameters.

use serde::{Deserialize, Serialize};

use super::common::{Action, Side};
use super::query::QueryBuilder;

/// A fill represents a matched trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub fill_id: String,
    /// Deprecated: legacy field, same as `fill_id`.
    pub trade_id: String,
    pub order_id: String,
    pub ticker: String,
    /// Deprecated: legacy field, same as `ticker`.
    pub market_ticker: String,
    pub side: Side,
    pub action: Action,
    /// Count (fixed-point decimal string, e.g. `"10.00"`).
    pub count_fp: String,
    /// Fill price for the yes side in fixed-point dollars.
    pub yes_price_dollars: String,
    /// Fill price for the no side in fixed-point dollars.
    pub no_price_dollars: String,
    /// Whether this fill removed liquidity.
    pub is_taker: bool,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub created_time: Option<String>,
    /// Deprecated: legacy Unix timestamp field.
    #[serde(default)]
    pub ts: Option<i64>,
    /// Exchange fee cost as a fixed-point dollar string.
    pub fee_cost: String,
    /// Subaccount number this fill belongs to (0 for primary, 1-32 for subaccounts).
    #[serde(default)]
    pub subaccount_number: Option<i32>,
}

/// Response from the get_fills endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillsResponse {
    pub fills: Vec<Fill>,
    pub cursor: String,
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

    #[test]
    fn test_fill_deserialize_current_shape() {
        // Current Kalshi /portfolio/fills shape: only yes_price_dollars /
        // no_price_dollars. Ignores any legacy yes_price_fixed / no_price_fixed
        // keys if the server still sends them.
        let json = r#"{
            "action": "buy",
            "count_fp": "1.00",
            "created_time": "2026-03-21T15:34:08.771917Z",
            "fee_cost": "0.000000",
            "fill_id": "b855cb66-b3fa-757e-dc84-6abdb31c80ec",
            "is_taker": false,
            "market_ticker": "KXEPLGOAL-26MAR21BRILFC-LFCRNGUMO73-1",
            "no_price_dollars": "0.9500",
            "order_id": "fced73b6-9f6f-4024-83a1-af8904190140",
            "side": "yes",
            "subaccount_number": 0,
            "ticker": "KXEPLGOAL-26MAR21BRILFC-LFCRNGUMO73-1",
            "trade_id": "b855cb66-b3fa-757e-dc84-6abdb31c80ec",
            "ts": 1774107248,
            "yes_price_dollars": "0.0500"
        }"#;
        let fill: Fill = serde_json::from_str(json).expect("Fill must deserialize");
        assert_eq!(fill.yes_price_dollars, "0.0500");
        assert_eq!(fill.no_price_dollars, "0.9500");
    }
}
