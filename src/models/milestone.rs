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
    pub id: Option<String>,
    /// The milestone type (e.g., "basketball_game", "one_off_milestone").
    #[serde(default, rename = "type")]
    pub milestone_type: Option<String>,
    /// The title or name of the milestone.
    #[serde(default)]
    pub title: Option<String>,
    /// The category (e.g., "Sports", "sports").
    #[serde(default)]
    pub category: Option<String>,
    /// Notification message for the milestone.
    #[serde(default)]
    pub notification_message: Option<String>,
    /// The start date (RFC3339 timestamp).
    #[serde(default)]
    pub start_date: Option<String>,
    /// The end date (RFC3339 timestamp).
    #[serde(default)]
    pub end_date: Option<String>,
    /// Primary event tickers associated with this milestone.
    #[serde(default)]
    pub primary_event_tickers: Option<Vec<String>>,
    /// Related event tickers.
    #[serde(default)]
    pub related_event_tickers: Option<Vec<String>>,
    /// The source identifier.
    #[serde(default)]
    pub source_id: Option<String>,
    /// Last update timestamp (RFC3339).
    #[serde(default)]
    pub last_updated_ts: Option<String>,
    /// Additional milestone details.
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

/// Query parameters for GET /milestones.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetMilestonesParams {
    /// Filter milestones starting after this timestamp (RFC3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_start_date: Option<String>,
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Filter by competition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competition: Option<String>,
    /// Filter by source ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    /// Filter by milestone type.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub milestone_type: Option<String>,
    /// Filter by related event ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_event_ticker: Option<String>,
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
    ///     .minimum_start_date("2025-01-01T00:00:00Z");
    /// ```
    #[must_use]
    pub fn minimum_start_date(mut self, date: impl Into<String>) -> Self {
        self.minimum_start_date = Some(date.into());
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

    /// Filter by category.
    #[must_use]
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Filter by competition.
    #[must_use]
    pub fn competition(mut self, competition: impl Into<String>) -> Self {
        self.competition = Some(competition.into());
        self
    }

    /// Filter by source ID.
    #[must_use]
    pub fn source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = Some(source_id.into());
        self
    }

    /// Filter by milestone type.
    #[must_use]
    pub fn milestone_type(mut self, milestone_type: impl Into<String>) -> Self {
        self.milestone_type = Some(milestone_type.into());
        self
    }

    /// Filter by related event ticker.
    #[must_use]
    pub fn related_event_ticker(mut self, related_event_ticker: impl Into<String>) -> Self {
        self.related_event_ticker = Some(related_event_ticker.into());
        self
    }

    /// Build the query string.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("minimum_start_date", self.minimum_start_date.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("category", self.category.as_ref());
        qb.push_opt("competition", self.competition.as_ref());
        qb.push_opt("source_id", self.source_id.as_ref());
        qb.push_opt("type", self.milestone_type.as_ref());
        qb.push_opt("related_event_ticker", self.related_event_ticker.as_ref());
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
            .minimum_start_date("2025-01-01T00:00:00Z")
            .limit(50);
        let qs = params.to_query_string();
        assert!(qs.contains("minimum_start_date="));
        assert!(qs.contains("limit=50"));
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{"milestones": [], "cursor": null}"#;
        let response: MilestonesResponse = serde_json::from_str(json).unwrap();
        assert!(response.milestones.is_empty());
    }
}
