//! Series API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Series API,
//! including getting series details and listing series.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetSeriesParams, SeriesListResponse, SeriesResponse},
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
pub async fn get_series_list(
    http: &HttpClient,
    params: GetSeriesParams,
) -> Result<SeriesListResponse> {
    let path = format!("/series{}", params.to_query_string());
    http.get(&path).await
}
