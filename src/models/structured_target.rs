//! Structured target models and response types.
//!
//! Structured targets represent specific data targets for market resolution.

use serde::{Deserialize, Serialize};

use crate::models::query::QueryBuilder;

/// A structured target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredTarget {
    /// The unique identifier for the structured target.
    #[serde(default)]
    pub structured_target_id: Option<String>,
    /// The title or name.
    #[serde(default)]
    pub title: Option<String>,
    /// The description.
    #[serde(default)]
    pub description: Option<String>,
    /// The target type.
    #[serde(default)]
    pub target_type: Option<String>,
    /// The associated series ticker.
    #[serde(default)]
    pub series_ticker: Option<String>,
    /// The associated event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// The target value.
    #[serde(default)]
    pub target_value: Option<f64>,
    /// The target value as a string.
    #[serde(default)]
    pub target_value_string: Option<String>,
    /// Additional configuration.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_ts: Option<String>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_ts: Option<String>,
}

/// Query parameters for GET /structured_targets.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetStructuredTargetsParams {
    /// Maximum number of results (1-2000).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetStructuredTargetsParams {
    /// Create new empty parameters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of results.
    ///
    /// The value is clamped to the valid range of 1..=2000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit.clamp(1, 2000));
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
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

/// Response from GET /structured_targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredTargetsResponse {
    /// The list of structured targets.
    pub structured_targets: Vec<StructuredTarget>,
    /// Pagination cursor for next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /structured_targets/{structured_target_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredTargetResponse {
    /// The structured target data.
    pub structured_target: StructuredTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_empty() {
        let params = GetStructuredTargetsParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_query_string_with_limit() {
        let params = GetStructuredTargetsParams::new().limit(100);
        assert!(params.to_query_string().contains("limit=100"));
    }

    #[test]
    fn test_limit_clamping() {
        let params = GetStructuredTargetsParams::new().limit(5000);
        assert_eq!(params.limit, Some(2000)); // clamped to max
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{"structured_targets": [], "cursor": null}"#;
        let response: StructuredTargetsResponse = serde_json::from_str(json).unwrap();
        assert!(response.structured_targets.is_empty());
    }
}
