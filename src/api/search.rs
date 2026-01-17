//! Search API endpoints for discovering markets through tags and sport-based filters.

use crate::{
    client::HttpClient,
    error::Result,
    models::{FiltersBySportResponse, TagsByCategoriesResponse},
};

/// Returns series categories mapped to their associated tags for filtering/search.
pub async fn get_tags_by_categories(http: &HttpClient) -> Result<TagsByCategoriesResponse> {
    http.get("/search/tags_by_categories").await
}

/// Returns available scopes, competitions, and display order for each sport.
pub async fn get_filters_by_sport(http: &HttpClient) -> Result<FiltersBySportResponse> {
    http.get("/search/filters_by_sport").await
}
