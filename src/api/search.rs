//! Search API endpoints.
//!
//! This module provides functions for discovering markets through
//! tags and sport-based filters.

use crate::{
    client::HttpClient,
    error::Result,
    models::{FiltersBySportResponse, TagsByCategoriesResponse},
};

/// Get tags organized by series categories.
///
/// Returns a mapping of series categories to their associated tags,
/// which can be used for filtering and search functionality.
pub async fn get_tags_by_categories(http: &HttpClient) -> Result<TagsByCategoriesResponse> {
    http.get("/search/tags_by_categories").await
}

/// Get filtering options organized by sport.
///
/// Returns available scopes and competitions for each sport,
/// along with a recommended display order.
pub async fn get_filters_by_sport(http: &HttpClient) -> Result<FiltersBySportResponse> {
    http.get("/search/filters_by_sport").await
}
