//! Milestone models and response types.
//!
//! Milestones represent data points that can be tracked and used for market resolution.

use serde::{Deserialize, Serialize};

use crate::models::query::QueryBuilder;

/// A milestone data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneInfo {
    /// The unique milestone identifier.
    #[serde(default)]
    pub milestone_id: Option<String>,
    /// The milestone type (e.g., "price", "score").
    #[serde(default)]
    pub milestone_type: Option<String>,
    /// The title or name of the milestone.
    #[serde(default)]
    pub title: Option<String>,
    /// The description of the milestone.
    #[serde(default)]
    pub description: Option<String>,
    /// The start timestamp (RFC3339 or Unix).
    #[serde(default)]
    pub start_ts: Option<String>,
    /// The end timestamp (RFC3339 or Unix).
    #[serde(default)]
    pub end_ts: Option<String>,
    /// The associated series ticker.
    #[serde(default)]
    pub series_ticker: Option<String>,
    /// The associated event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// The current value.
    #[serde(default)]
    pub value: Option<f64>,
    /// The value as a string.
    #[serde(default)]
    pub value_string: Option<String>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_ts: Option<i64>,
    /// Additional milestone metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Query parameters for GET /milestones.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetMilestonesParams {
    /// Filter milestones starting after this timestamp (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_start_date: Option<String>,
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetMilestonesParams {
    /// Create new empty parameters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum start date filter (RFC3339 format).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetMilestonesParams::new()
    ///     .min_start_date("2025-01-01T00:00:00Z");
    /// ```
    #[must_use]
    pub fn min_start_date(mut self, date: impl Into<String>) -> Self {
        self.min_start_date = Some(date.into());
        self
    }

    /// Set the maximum number of results.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the pagination cursor.
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Build the query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("min_start_date", self.min_start_date.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

/// Response from GET /milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonesResponse {
    /// The list of milestones.
    pub milestones: Vec<MilestoneInfo>,
    /// Pagination cursor for next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /milestones/{milestone_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneResponse {
    /// The milestone data.
    pub milestone: MilestoneInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_empty() {
        let params = GetMilestonesParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_query_string_with_params() {
        let params = GetMilestonesParams::new()
            .min_start_date("2025-01-01T00:00:00Z")
            .limit(50);
        let qs = params.to_query_string();
        assert!(qs.contains("min_start_date="));
        assert!(qs.contains("limit=50"));
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{"milestones": [], "cursor": null}"#;
        let response: MilestonesResponse = serde_json::from_str(json).unwrap();
        assert!(response.milestones.is_empty());
    }
}
