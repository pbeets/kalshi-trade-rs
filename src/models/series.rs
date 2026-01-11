//! Series API models and response types.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// A series on the Kalshi exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// Series ticker.
    pub ticker: String,
    /// Frequency of the series (e.g., "daily", "weekly", "monthly", "custom").
    pub frequency: String,
    /// Title of the series.
    pub title: String,
}

/// Response from GET /series/{series_ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesResponse {
    pub series: Series,
}

/// Response from GET /series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesListResponse {
    pub series: Vec<Series>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for GET /series.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetSeriesParams {
    /// Limit the number of results returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter by series ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
}

impl GetSeriesParams {
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
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn series_ticker(mut self, series_ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(series_ticker.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.build()
    }
}
