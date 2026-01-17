//! Series API models and response types.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

// ============================================================================
// Fee Changes
// ============================================================================

/// A scheduled fee change for a series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesFeeChange {
    /// Unique identifier for the fee change.
    pub id: String,
    /// Series ticker affected by the change.
    pub series_ticker: String,
    /// The type of fee structure.
    pub fee_type: FeeType,
    /// The fee multiplier value.
    pub fee_multiplier: f64,
    /// ISO 8601 timestamp when the change takes effect.
    pub scheduled_ts: String,
}

/// Type of fee structure for a series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FeeType {
    /// Quadratic fee structure.
    Quadratic,
    /// Quadratic fee structure with maker fees.
    QuadraticWithMakerFees,
    /// Flat fee structure.
    Flat,
    /// Unknown fee type returned by the API.
    #[serde(other)]
    Unknown,
}

/// Response from GET /series/fee_changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeChangesResponse {
    /// Array of fee change records.
    #[serde(default)]
    pub series_fee_change_arr: Vec<SeriesFeeChange>,
}

/// Query parameters for GET /series/fee_changes.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetFeeChangesParams {
    /// Filter by series ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    /// If true, include historical fee changes. Default is false (upcoming only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_historical: Option<bool>,
}

impl GetFeeChangesParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by series ticker.
    #[must_use]
    pub fn series_ticker(mut self, ticker: impl Into<String>) -> Self {
        self.series_ticker = Some(ticker.into());
        self
    }

    /// Include historical fee changes (default is false, upcoming only).
    #[must_use]
    pub fn show_historical(mut self, show: bool) -> Self {
        self.show_historical = Some(show);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("series_ticker", self.series_ticker.as_ref());
        qb.push_opt("show_historical", self.show_historical);
        qb.build()
    }
}

// ============================================================================
// Series
// ============================================================================

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_type_deserialize() {
        assert_eq!(
            serde_json::from_str::<FeeType>(r#""quadratic""#).unwrap(),
            FeeType::Quadratic
        );
        assert_eq!(
            serde_json::from_str::<FeeType>(r#""quadratic_with_maker_fees""#).unwrap(),
            FeeType::QuadraticWithMakerFees
        );
        assert_eq!(
            serde_json::from_str::<FeeType>(r#""flat""#).unwrap(),
            FeeType::Flat
        );
        assert_eq!(
            serde_json::from_str::<FeeType>(r#""unknown_future_type""#).unwrap(),
            FeeType::Unknown
        );
    }

    #[test]
    fn test_fee_change_response_deserialize() {
        let json = r#"{
            "series_fee_change_arr": [
                {
                    "id": "fc-123",
                    "series_ticker": "KXBTC",
                    "fee_type": "quadratic",
                    "fee_multiplier": 1.5,
                    "scheduled_ts": "2025-02-01T00:00:00Z"
                }
            ]
        }"#;
        let response: FeeChangesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.series_fee_change_arr.len(), 1);
        let change = &response.series_fee_change_arr[0];
        assert_eq!(change.id, "fc-123");
        assert_eq!(change.series_ticker, "KXBTC");
        assert_eq!(change.fee_type, FeeType::Quadratic);
        assert!((change.fee_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_fee_changes_params_query_string() {
        let params = GetFeeChangesParams::new()
            .series_ticker("KXBTC")
            .show_historical(true);
        let qs = params.to_query_string();
        assert!(qs.contains("series_ticker=KXBTC"));
        assert!(qs.contains("show_historical=true"));
    }

    #[test]
    fn test_get_fee_changes_params_empty() {
        let params = GetFeeChangesParams::new();
        assert_eq!(params.to_query_string(), "");
    }
}
