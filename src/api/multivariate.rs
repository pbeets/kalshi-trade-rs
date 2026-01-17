//! Multivariate Event Collections API for dynamic market creation based on variable combinations.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateMarketInCollectionRequest, CreateMarketInCollectionResponse, GetLookupHistoryParams,
        GetMultivariateCollectionsParams, LookupHistoryResponse, LookupTickersRequest,
        LookupTickersResponse, MultivariateCollectionResponse, MultivariateCollectionsResponse,
    },
};

fn encode_ticker(ticker: &str) -> String {
    form_urlencoded::byte_serialize(ticker.as_bytes()).collect()
}

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

/// Creates a market with specified variables (returns existing ticker if already exists).
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

/// Looks up the market ticker for a variable combination (404 if not found).
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
