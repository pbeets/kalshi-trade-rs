//! FCM (Futures Commission Merchant) models and query parameters.
//!
//! These endpoints are specialized for FCM members only and allow
//! filtering orders and positions by subtrader ID.

use serde::Serialize;

use super::common::OrderStatus;
use super::query::QueryBuilder;

/// Settlement status filter for FCM positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementStatus {
    /// Return all positions regardless of settlement status.
    All,
    /// Return only unsettled positions (default).
    Unsettled,
    /// Return only settled positions.
    Settled,
}

impl SettlementStatus {
    fn as_str(&self) -> &'static str {
        match self {
            SettlementStatus::All => "all",
            SettlementStatus::Unsettled => "unsettled",
            SettlementStatus::Settled => "settled",
        }
    }
}

/// Query parameters for GET /fcm/orders endpoint.
///
/// This endpoint is for FCM members to get orders filtered by subtrader ID.
///
/// # Example
///
/// ```
/// use kalshi_trade_rs::{GetFcmOrdersParams, OrderStatus};
///
/// let params = GetFcmOrdersParams::new("subtrader-123")
///     .status(OrderStatus::Resting)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct GetFcmOrdersParams {
    /// Restricts the response to orders for a specific subtrader (required).
    pub subtrader_id: String,
    /// Pagination cursor for retrieving next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Event ticker filter. Multiple event tickers can be provided as
    /// a comma-separated list (maximum 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Market ticker filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    /// Restricts the response to orders after this timestamp (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    /// Restricts the response to orders before this timestamp (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    /// Restricts the response to orders with a specific status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    /// Number of results per page (1-1000, default: 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl GetFcmOrdersParams {
    /// Create a new query with the required subtrader ID.
    #[must_use]
    pub fn new(subtrader_id: impl Into<String>) -> Self {
        Self {
            subtrader_id: subtrader_id.into(),
            cursor: None,
            event_ticker: None,
            ticker: None,
            min_ts: None,
            max_ts: None,
            status: None,
            limit: None,
        }
    }

    /// Set the pagination cursor.
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

    /// Filter by market ticker.
    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter orders created after this timestamp.
    #[must_use]
    pub fn min_ts(mut self, ts: i64) -> Self {
        self.min_ts = Some(ts);
        self
    }

    /// Filter orders created before this timestamp.
    #[must_use]
    pub fn max_ts(mut self, ts: i64) -> Self {
        self.max_ts = Some(ts);
        self
    }

    /// Filter by order status.
    #[must_use]
    pub fn status(mut self, status: OrderStatus) -> Self {
        self.status = Some(status);
        self
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
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("subtrader_id", &self.subtrader_id);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("status", self.status.map(|s| s.as_str()));
        qb.push_opt("limit", self.limit);
        qb.build()
    }
}

/// Query parameters for GET /fcm/positions endpoint.
///
/// This endpoint is for FCM members to get positions filtered by subtrader ID.
///
/// # Example
///
/// ```
/// use kalshi_trade_rs::{GetFcmPositionsParams, SettlementStatus};
///
/// let params = GetFcmPositionsParams::new("subtrader-123")
///     .settlement_status(SettlementStatus::Unsettled)
///     .limit(100);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct GetFcmPositionsParams {
    /// Restricts the response to positions for a specific subtrader (required).
    pub subtrader_id: String,
    /// Market ticker filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    /// Event ticker filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Restricts positions to those with non-zero values in specified fields
    /// (comma-separated list).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_filter: Option<String>,
    /// Settlement status filter (default: unsettled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_status: Option<SettlementStatus>,
    /// Number of results per page (1-1000, default: 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Pagination cursor for retrieving next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetFcmPositionsParams {
    /// Create a new query with the required subtrader ID.
    #[must_use]
    pub fn new(subtrader_id: impl Into<String>) -> Self {
        Self {
            subtrader_id: subtrader_id.into(),
            ticker: None,
            event_ticker: None,
            count_filter: None,
            settlement_status: None,
            limit: None,
            cursor: None,
        }
    }

    /// Filter by market ticker.
    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Filter by event ticker.
    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    /// Filter positions to those with non-zero values in specified fields.
    ///
    /// Fields can be provided as a comma-separated list.
    #[must_use]
    pub fn count_filter(mut self, filter: impl Into<String>) -> Self {
        self.count_filter = Some(filter.into());
        self
    }

    /// Set the settlement status filter.
    #[must_use]
    pub fn settlement_status(mut self, status: SettlementStatus) -> Self {
        self.settlement_status = Some(status);
        self
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

    /// Set the pagination cursor.
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push("subtrader_id", &self.subtrader_id);
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("count_filter", self.count_filter.as_ref());
        qb.push_opt("settlement_status", self.settlement_status.map(|s| s.as_str()));
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcm_orders_params_required_only() {
        let params = GetFcmOrdersParams::new("subtrader-123");
        assert_eq!(
            params.to_query_string(),
            "?subtrader_id=subtrader-123"
        );
    }

    #[test]
    fn test_fcm_orders_params_with_options() {
        let params = GetFcmOrdersParams::new("subtrader-123")
            .status(OrderStatus::Resting)
            .limit(50)
            .ticker("KXBTC-25JAN");
        let qs = params.to_query_string();
        assert!(qs.contains("subtrader_id=subtrader-123"));
        assert!(qs.contains("status=resting"));
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("ticker=KXBTC-25JAN"));
    }

    #[test]
    fn test_fcm_positions_params_required_only() {
        let params = GetFcmPositionsParams::new("subtrader-456");
        assert_eq!(
            params.to_query_string(),
            "?subtrader_id=subtrader-456"
        );
    }

    #[test]
    fn test_fcm_positions_params_with_settlement_status() {
        let params = GetFcmPositionsParams::new("subtrader-456")
            .settlement_status(SettlementStatus::All)
            .limit(100);
        let qs = params.to_query_string();
        assert!(qs.contains("subtrader_id=subtrader-456"));
        assert!(qs.contains("settlement_status=all"));
        assert!(qs.contains("limit=100"));
    }
}
