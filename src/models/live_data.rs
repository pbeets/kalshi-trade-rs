//! Live data models and response types.
//!
//! Live data endpoints provide real-time data for milestones without requiring WebSocket.

use serde::{Deserialize, Serialize};

use crate::models::query::QueryBuilder;

/// Live data entry for a single milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveData {
    /// The milestone ID.
    pub milestone_id: String,
    /// The milestone type (e.g., "price", "score").
    #[serde(default)]
    pub milestone_type: Option<String>,
    /// The current value.
    #[serde(default)]
    pub value: Option<f64>,
    /// The value as a string.
    #[serde(default)]
    pub value_string: Option<String>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_ts: Option<i64>,
    /// Associated event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// Associated series ticker.
    #[serde(default)]
    pub series_ticker: Option<String>,
}

/// Response from GET /live_data/{type}/milestone/{milestone_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataResponse {
    /// The live data entry.
    pub live_data: LiveData,
}

/// Query parameters for GET /live_data/batch.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetBatchLiveDataParams {
    /// Milestone IDs to retrieve (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_ids: Option<String>,
    /// Milestone type filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_type: Option<String>,
}

impl GetBatchLiveDataParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set milestone IDs (comma-separated or as a slice).
    #[must_use]
    pub fn milestone_ids(mut self, ids: impl Into<String>) -> Self {
        self.milestone_ids = Some(ids.into());
        self
    }

    /// Set milestone IDs from a slice.
    #[must_use]
    pub fn from_ids(ids: &[&str]) -> Self {
        Self {
            milestone_ids: Some(ids.join(",")),
            milestone_type: None,
        }
    }

    /// Set the milestone type filter.
    #[must_use]
    pub fn milestone_type(mut self, milestone_type: impl Into<String>) -> Self {
        self.milestone_type = Some(milestone_type.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("milestone_ids", self.milestone_ids.as_ref());
        qb.push_opt("milestone_type", self.milestone_type.as_ref());
        qb.build()
    }
}

/// Response from GET /live_data/batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLiveDataResponse {
    /// List of live data entries.
    pub live_data: Vec<LiveData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        let params = GetBatchLiveDataParams::from_ids(&["ms1", "ms2", "ms3"]);
        assert_eq!(params.milestone_ids, Some("ms1,ms2,ms3".to_string()));
    }

    #[test]
    fn test_query_string() {
        let params = GetBatchLiveDataParams::new()
            .milestone_ids("ms1,ms2")
            .milestone_type("price");
        let qs = params.to_query_string();
        assert!(qs.contains("milestone_ids="));
        assert!(qs.contains("milestone_type=price"));
    }
}
