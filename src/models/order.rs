//! Order models and query parameters.

use serde::{Deserialize, Serialize};

use super::common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side};
use super::query::QueryBuilder;

/// An order in the Kalshi exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub ticker: String,
    pub side: Side,
    pub action: Action,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    /// Price in cents.
    pub yes_price: i64,
    /// Price in cents.
    pub no_price: i64,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub yes_price_dollars: Option<String>,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub no_price_dollars: Option<String>,
    pub fill_count: i64,
    pub remaining_count: i64,
    pub initial_count: i64,
    /// Fees in cents.
    #[serde(default)]
    pub taker_fees: Option<i64>,
    /// Fees in cents.
    #[serde(default)]
    pub maker_fees: Option<i64>,
    /// Cost in cents.
    #[serde(default)]
    pub taker_fill_cost: Option<i64>,
    /// Cost in cents.
    #[serde(default)]
    pub maker_fill_cost: Option<i64>,
    #[serde(default)]
    pub taker_fill_cost_dollars: Option<String>,
    #[serde(default)]
    pub maker_fill_cost_dollars: Option<String>,
    #[serde(default)]
    pub taker_fees_dollars: Option<String>,
    #[serde(default)]
    pub maker_fees_dollars: Option<String>,
    /// Deprecated.
    #[serde(default)]
    pub queue_position: Option<i64>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub cancel_order_on_pause: Option<bool>,
}

/// Response from the get_orders endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for the get_orders endpoint.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetOrdersParams {
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
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
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
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("status", self.status.map(|s| s.as_str()));
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_with_status() {
        let params = GetOrdersParams::new().status(OrderStatus::Resting);
        assert_eq!(params.to_query_string(), "?status=resting");
    }
}
