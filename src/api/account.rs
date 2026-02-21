//! Account API endpoints.

use crate::{client::HttpClient, error::Result, models::ApiTierLimitsResponse};

/// Returns the user's API tier and rate limits.
pub async fn get_api_limits(http: &HttpClient) -> Result<ApiTierLimitsResponse> {
    http.get("/account/limits").await
}
