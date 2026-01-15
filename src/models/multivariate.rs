//! Multivariate event collection models and response types.
//!
//! Multivariate event collections support dynamic market creation based on
//! variable combinations (e.g., different strike prices, dates).

use serde::{Deserialize, Serialize};

use crate::models::query::QueryBuilder;

/// A multivariate event collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateEventCollection {
    /// The collection ticker.
    pub collection_ticker: String,
    /// Collection title.
    #[serde(default)]
    pub title: Option<String>,
    /// Collection description.
    #[serde(default)]
    pub description: Option<String>,
    /// Variables that can be combined to create markets.
    #[serde(default)]
    pub variables: Option<Vec<CollectionVariable>>,
    /// Associated series ticker.
    #[serde(default)]
    pub series_ticker: Option<String>,
    /// Collection status.
    #[serde(default)]
    pub status: Option<String>,
}

/// A variable in a multivariate event collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionVariable {
    /// Variable name.
    pub name: String,
    /// Variable type (e.g., "date", "number", "string").
    #[serde(rename = "type")]
    #[serde(default)]
    pub variable_type: Option<String>,
    /// Possible values for this variable.
    #[serde(default)]
    pub values: Option<Vec<serde_json::Value>>,
    /// Minimum value (for numeric variables).
    #[serde(default)]
    pub min: Option<serde_json::Value>,
    /// Maximum value (for numeric variables).
    #[serde(default)]
    pub max: Option<serde_json::Value>,
}

/// Query parameters for GET /multivariate_event_collections.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetMultivariateCollectionsParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Number of results per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Filter by series ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    /// Filter by status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl GetMultivariateCollectionsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn series_ticker(mut self, ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.push_opt("status", self.status.as_ref());
        qb.build()
    }
}

/// Response from GET /multivariate_event_collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateCollectionsResponse {
    /// List of collections.
    pub collections: Vec<MultivariateEventCollection>,
    /// Pagination cursor for next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /multivariate_event_collections/{collection_ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultivariateCollectionResponse {
    /// The collection details.
    pub collection: MultivariateEventCollection,
}

/// Request body for POST /multivariate_event_collections/{collection_ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketInCollectionRequest {
    /// Variable values for creating the market.
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

impl CreateMarketInCollectionRequest {
    /// Create a new request with the given variable values.
    #[must_use]
    pub fn new(variables: std::collections::HashMap<String, serde_json::Value>) -> Self {
        Self { variables }
    }

    /// Create an empty request (variables can be added later).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
        }
    }

    /// Add a variable value.
    #[must_use]
    pub fn variable(
        mut self,
        name: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }
}

/// Response from POST /multivariate_event_collections/{collection_ticker}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketInCollectionResponse {
    /// The created market ticker.
    pub ticker: String,
    /// The created event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// Whether the market already existed.
    #[serde(default)]
    pub already_existed: Option<bool>,
}

/// Request body for PUT /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTickersRequest {
    /// Variable values to look up.
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

impl LookupTickersRequest {
    /// Create a new lookup request.
    #[must_use]
    pub fn new(variables: std::collections::HashMap<String, serde_json::Value>) -> Self {
        Self { variables }
    }

    /// Create an empty lookup request.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
        }
    }

    /// Add a variable value for lookup.
    #[must_use]
    pub fn variable(
        mut self,
        name: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }
}

/// Response from PUT /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTickersResponse {
    /// The market ticker if found.
    #[serde(default)]
    pub ticker: Option<String>,
    /// The event ticker if found.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// Whether the market exists.
    #[serde(default)]
    pub exists: Option<bool>,
}

/// A lookup history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupHistoryEntry {
    /// Variable values used in the lookup.
    pub variables: std::collections::HashMap<String, serde_json::Value>,
    /// The resulting market ticker.
    #[serde(default)]
    pub ticker: Option<String>,
    /// The resulting event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// Lookup timestamp.
    #[serde(default)]
    pub created_time: Option<String>,
}

/// Query parameters for GET /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetLookupHistoryParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Number of results per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl GetLookupHistoryParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.build()
    }
}

/// Response from GET /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupHistoryResponse {
    /// List of lookup history entries.
    pub lookups: Vec<LookupHistoryEntry>,
    /// Pagination cursor for next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_market_request_builder() {
        let request = CreateMarketInCollectionRequest::empty()
            .variable("date", "2025-01-15")
            .variable("strike", 50000);

        assert_eq!(request.variables.len(), 2);
        assert_eq!(request.variables.get("date").unwrap(), "2025-01-15");
    }

    #[test]
    fn test_query_params() {
        let params = GetMultivariateCollectionsParams::new()
            .series_ticker("KXBTC")
            .limit(50);
        let qs = params.to_query_string();
        assert!(qs.contains("series_ticker=KXBTC"));
        assert!(qs.contains("limit=50"));
    }
}
