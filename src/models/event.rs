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
}
