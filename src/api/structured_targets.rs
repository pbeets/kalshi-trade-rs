//! Structured Targets API endpoints.
//!
//! This module provides functions for retrieving structured targets,
//! which represent specific data targets for market resolution.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetStructuredTargetsParams, StructuredTargetResponse, StructuredTargetsResponse},
};

/// URL-encode a string for use in path segments.
fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// List structured targets with optional pagination.
///
/// Returns structured targets matching the provided query parameters.
///
/// # Arguments
///
/// * `params` - Query parameters for pagination (limit, cursor)
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::GetStructuredTargetsParams;
///
/// let params = GetStructuredTargetsParams::new().limit(100);
/// let response = client.get_structured_targets_with_params(params).await?;
/// for target in response.structured_targets {
///     println!("{}: {:?}", target.structured_target_id.unwrap_or_default(), target.title);
/// }
/// ```
pub async fn get_structured_targets(
    http: &HttpClient,
    params: GetStructuredTargetsParams,
) -> Result<StructuredTargetsResponse> {
    let path = format!("/structured_targets{}", params.to_query_string());
    http.get(&path).await
}

/// Get a specific structured target by ID.
///
/// Returns detailed information about a single structured target.
///
/// # Arguments
///
/// * `structured_target_id` - The unique identifier of the structured target
///
/// # Example
///
/// ```ignore
/// let response = client.get_structured_target("st_123").await?;
/// println!("Target: {:?}", response.structured_target.title);
/// ```
pub async fn get_structured_target(
    http: &HttpClient,
    structured_target_id: &str,
) -> Result<StructuredTargetResponse> {
    let path = format!(
        "/structured_targets/{}",
        encode_path_segment(structured_target_id)
    );
    http.get(&path).await
}
