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
    /// Events in this collection.
    #[serde(default)]
    pub events: Option<Vec<CollectionEvent>>,
    /// Open date for the collection.
    #[serde(default)]
    pub open_date: Option<String>,
    /// Close date for the collection.
    #[serde(default)]
    pub close_date: Option<String>,
    /// Associated events with quoter information.
    #[serde(default)]
    pub associated_events: Option<Vec<AssociatedEvent>>,
    /// Associated event tickers (deprecated, use `associated_events`).
    #[serde(default)]
    pub associated_event_tickers: Option<Vec<String>>,
    /// Whether the collection events are ordered.
    #[serde(default)]
    pub is_ordered: Option<bool>,
    /// Whether each event has a single market (deprecated).
    #[serde(default)]
    pub is_single_market_per_event: Option<bool>,
    /// Whether all events are YES-only (deprecated).
    #[serde(default)]
    pub is_all_yes: Option<bool>,
    /// Minimum order size.
    #[serde(default)]
    pub size_min: Option<i64>,
    /// Maximum order size.
    #[serde(default)]
    pub size_max: Option<i64>,
    /// Functional description of the collection.
    #[serde(default)]
    pub functional_description: Option<String>,
}

/// An event entry within a multivariate event collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionEvent {
    /// Event ticker.
    #[serde(default)]
    pub ticker: Option<String>,
    /// Whether this event only supports YES contracts.
    #[serde(default)]
    pub is_yes_only: Option<bool>,
    /// Minimum order size.
    #[serde(default)]
    pub size_min: Option<i64>,
    /// Maximum order size.
    #[serde(default)]
    pub size_max: Option<i64>,
}

/// An associated event in a multivariate collection with quoter information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedEvent {
    /// Event ticker.
    #[serde(default)]
    pub ticker: Option<String>,
    /// Whether this event only supports YES contracts.
    #[serde(default)]
    pub is_yes_only: Option<bool>,
    /// Minimum order size.
    #[serde(default)]
    pub size_min: Option<i64>,
    /// Maximum order size.
    #[serde(default)]
    pub size_max: Option<i64>,
    /// Active quoters for this event.
    #[serde(default)]
    pub active_quoters: Option<Vec<String>>,
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
///
/// Supports two formats:
/// - `variables`: key-value map for variable-based collections
/// - `selected_markets`: list of ticker pairs for selection-based collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketInCollectionRequest {
    /// Variable values for creating the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Selected market legs (alternative to variables).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_markets: Option<Vec<super::market::MveSelectedLeg>>,
}

impl CreateMarketInCollectionRequest {
    /// Create a new request with the given variable values.
    #[must_use]
    pub fn new(variables: std::collections::HashMap<String, serde_json::Value>) -> Self {
        Self {
            variables: Some(variables),
            selected_markets: None,
        }
    }

    /// Create an empty request (variables can be added later).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            variables: Some(std::collections::HashMap::new()),
            selected_markets: None,
        }
    }

    /// Add a variable value.
    #[must_use]
    pub fn variable(
        mut self,
        name: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.variables
            .get_or_insert_with(std::collections::HashMap::new)
            .insert(name.into(), value.into());
        self
    }

    /// Create a request with selected market legs.
    #[must_use]
    pub fn with_selected_markets(markets: Vec<super::market::MveSelectedLeg>) -> Self {
        Self {
            variables: None,
            selected_markets: Some(markets),
        }
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
///
/// Supports two formats:
/// - `variables`: key-value map for variable-based collections
/// - `selected_markets`: list of ticker pairs for selection-based collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTickersRequest {
    /// Variable values to look up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Selected market legs (alternative to variables).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_markets: Option<Vec<super::market::MveSelectedLeg>>,
}

impl LookupTickersRequest {
    /// Create a new lookup request.
    #[must_use]
    pub fn new(variables: std::collections::HashMap<String, serde_json::Value>) -> Self {
        Self {
            variables: Some(variables),
            selected_markets: None,
        }
    }

    /// Create an empty lookup request.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            variables: Some(std::collections::HashMap::new()),
            selected_markets: None,
        }
    }

    /// Add a variable value for lookup.
    #[must_use]
    pub fn variable(
        mut self,
        name: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.variables
            .get_or_insert_with(std::collections::HashMap::new)
            .insert(name.into(), value.into());
        self
    }

    /// Create a lookup request with selected market legs.
    #[must_use]
    pub fn with_selected_markets(markets: Vec<super::market::MveSelectedLeg>) -> Self {
        Self {
            variables: None,
            selected_markets: Some(markets),
        }
    }
}

/// Response from PUT /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTickersResponse {
    /// The market ticker if found.
    #[serde(default, alias = "market_ticker")]
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
    #[serde(default)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
    /// The resulting market ticker.
    #[serde(default, alias = "market_ticker")]
    pub ticker: Option<String>,
    /// The resulting event ticker.
    #[serde(default)]
    pub event_ticker: Option<String>,
    /// Selected markets (alternative to variables in some API versions).
    #[serde(default)]
    pub selected_markets: Option<Vec<super::market::MveSelectedLeg>>,
    /// Lookup timestamp.
    #[serde(default, alias = "last_queried_ts")]
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
    /// Only return lookups within this many seconds ago.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookback_seconds: Option<i64>,
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

    /// Only return lookups within this many seconds ago.
    #[must_use]
    pub fn lookback_seconds(mut self, seconds: i64) -> Self {
        self.lookback_seconds = Some(seconds);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("lookback_seconds", self.lookback_seconds);
        qb.build()
    }
}

/// Response from GET /multivariate_event_collections/{collection_ticker}/lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupHistoryResponse {
    /// List of lookup history entries.
    ///
    /// The API may return this as either `lookups` or `lookup_points`.
    #[serde(default, alias = "lookup_points")]
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

        let vars = request.variables.as_ref().unwrap();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars.get("date").unwrap(), "2025-01-15");
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
