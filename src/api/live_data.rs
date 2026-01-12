//! Live Data API endpoints.
//!
//! This module provides functions for retrieving live data for milestones
//! without requiring WebSocket connections.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{BatchLiveDataResponse, GetBatchLiveDataParams, LiveDataResponse},
};

/// URL-encode a string for use in path segments.
fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// Get live data for a specific milestone.
///
/// Returns current live data for a single milestone identified by type and ID.
///
/// # Arguments
///
/// * `milestone_type` - The type of milestone (e.g., "price", "score")
/// * `milestone_id` - The unique milestone identifier
pub async fn get_live_data(
    http: &HttpClient,
    milestone_type: &str,
    milestone_id: &str,
) -> Result<LiveDataResponse> {
    let path = format!(
        "/live_data/{}/milestone/{}",
        encode_path_segment(milestone_type),
        encode_path_segment(milestone_id)
    );
    http.get(&path).await
}

/// Get live data for multiple milestones in batch.
///
/// Returns live data for multiple milestones in a single request.
pub async fn get_batch_live_data(
    http: &HttpClient,
    params: GetBatchLiveDataParams,
) -> Result<BatchLiveDataResponse> {
    let path = format!("/live_data/batch{}", params.to_query_string());
    http.get(&path).await
}
