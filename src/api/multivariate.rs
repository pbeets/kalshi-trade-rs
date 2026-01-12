//! Multivariate Event Collections API endpoints.
//!
//! This module provides functions for working with multivariate event collections,
//! which support dynamic market creation based on variable combinations.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateMarketInCollectionRequest, CreateMarketInCollectionResponse,
        GetLookupHistoryParams, GetMultivariateCollectionsParams, LookupHistoryResponse,
        LookupTickersRequest, LookupTickersResponse, MultivariateCollectionResponse,
        MultivariateCollectionsResponse,
    },
};

/// URL-encode a ticker for use in path segments.
fn encode_ticker(ticker: &str) -> String {
    form_urlencoded::byte_serialize(ticker.as_bytes()).collect()
}

/// List multivariate event collections.
///
/// Returns a paginated list of multivariate event collections.
pub async fn get_multivariate_collections(
    http: &HttpClient,
    params: GetMultivariateCollectionsParams,
) -> Result<MultivariateCollectionsResponse> {
    let path = format!(
        "/multivariate_event_collections{}",
        params.to_query_string()
    );
    http.get(&path).await
}

/// Get a specific multivariate event collection.
///
/// Returns details about a single multivariate event collection.
pub async fn get_multivariate_collection(
    http: &HttpClient,
    collection_ticker: &str,
) -> Result<MultivariateCollectionResponse> {
    let path = format!(
        "/multivariate_event_collections/{}",
        encode_ticker(collection_ticker)
    );
    http.get(&path).await
}

/// Create a market in a multivariate event collection.
///
/// Creates a new market with the specified variable combination.
/// If the market already exists, returns the existing ticker.
pub async fn create_market_in_collection(
    http: &HttpClient,
    collection_ticker: &str,
    request: CreateMarketInCollectionRequest,
) -> Result<CreateMarketInCollectionResponse> {
    let path = format!(
        "/multivariate_event_collections/{}",
        encode_ticker(collection_ticker)
    );
    http.post(&path, &request).await
}

/// Get lookup history for a multivariate event collection.
///
/// Returns a paginated list of previous lookups in this collection.
pub async fn get_lookup_history(
    http: &HttpClient,
    collection_ticker: &str,
    params: GetLookupHistoryParams,
) -> Result<LookupHistoryResponse> {
    let path = format!(
        "/multivariate_event_collections/{}/lookup{}",
        encode_ticker(collection_ticker),
        params.to_query_string()
    );
    http.get(&path).await
}

/// Lookup tickers for a variable combination.
///
/// Looks up the market ticker for a given variable combination.
/// Returns 404 if the market doesn't exist (use create_market_in_collection to create it).
pub async fn lookup_tickers(
    http: &HttpClient,
    collection_ticker: &str,
    request: LookupTickersRequest,
) -> Result<LookupTickersResponse> {
    let path = format!(
        "/multivariate_event_collections/{}/lookup",
        encode_ticker(collection_ticker)
    );
    http.put(&path, &request).await
}
