//! Series API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Series API,
//! including getting series details, listing series, and fee changes.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{FeeChangesResponse, GetFeeChangesParams, GetSeriesParams, SeriesListResponse, SeriesResponse},
};

/// URL-encode a ticker for use in path segments.
fn encode_ticker(ticker: &str) -> String {
    form_urlencoded::byte_serialize(ticker.as_bytes()).collect()
}

/// Get details for a specific series by ticker.
pub async fn get_series(http: &HttpClient, ticker: &str) -> Result<SeriesResponse> {
    let path = format!("/series/{}", encode_ticker(ticker));
    http.get(&path).await
}

/// Get a list of series with optional filtering.
///
/// Returns series matching the provided query parameters.
pub async fn get_series_list(
    http: &HttpClient,
    params: GetSeriesParams,
) -> Result<SeriesListResponse> {
    let path = format!("/series{}", params.to_query_string());
    http.get(&path).await
}

/// Get series fee changes.
///
/// Returns upcoming and optionally historical fee changes for series.
/// By default, only upcoming fee changes are returned.
pub async fn get_fee_changes(
    http: &HttpClient,
    params: GetFeeChangesParams,
) -> Result<FeeChangesResponse> {
    let path = format!("/series/fee_changes{}", params.to_query_string());
    http.get(&path).await
}
