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
    /// Filter by series category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Filter by associated tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// If true, includes internal product metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_product_metadata: Option<bool>,
    /// If true, includes total volume traded across all events in each series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_volume: Option<bool>,
}

impl GetSeriesParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter results by series category.
    #[must_use]
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Filter results by associated tags.
    #[must_use]
    pub fn tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    /// Include internal product metadata in results.
    #[must_use]
    pub fn include_product_metadata(mut self, include: bool) -> Self {
        self.include_product_metadata = Some(include);
        self
    }

    /// Include total volume traded across all events in each series.
    #[must_use]
    pub fn include_volume(mut self, include: bool) -> Self {
        self.include_volume = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("category", self.category.as_ref());
        qb.push_opt("tags", self.tags.as_ref());
        qb.push_opt("include_product_metadata", self.include_product_metadata);
        qb.push_opt("include_volume", self.include_volume);
        qb.build()
    }
}
