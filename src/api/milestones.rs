//! Milestones API endpoints.
//!
//! This module provides functions for retrieving milestones,
//! which represent data points that can be tracked and used for market resolution.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetMilestonesParams, MilestoneResponse, MilestonesResponse},
};

/// URL-encode a string for use in path segments.
fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// List milestones with optional filtering.
///
/// Returns milestones matching the provided query parameters.
///
/// # Arguments
///
/// * `params` - Query parameters for filtering (min_start_date, limit, cursor)
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::GetMilestonesParams;
///
/// let params = GetMilestonesParams::new()
///     .min_start_date("2025-01-01T00:00:00Z")
///     .limit(100);
/// let response = client.get_milestones_with_params(params).await?;
/// for milestone in response.milestones {
///     println!("{}: {:?}", milestone.milestone_id.unwrap_or_default(), milestone.value);
/// }
/// ```
pub async fn get_milestones(
    http: &HttpClient,
    params: GetMilestonesParams,
) -> Result<MilestonesResponse> {
    let path = format!("/milestones{}", params.to_query_string());
    http.get(&path).await
}

/// Get a specific milestone by ID.
///
/// Returns detailed information about a single milestone.
///
/// # Arguments
///
/// * `milestone_id` - The unique identifier of the milestone
///
/// # Example
///
/// ```ignore
/// let response = client.get_milestone("ms_123").await?;
/// println!("Milestone: {:?}", response.milestone.title);
/// ```
pub async fn get_milestone(http: &HttpClient, milestone_id: &str) -> Result<MilestoneResponse> {
    let path = format!("/milestones/{}", encode_path_segment(milestone_id));
    http.get(&path).await
}
