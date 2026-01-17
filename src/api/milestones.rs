//! Milestones API endpoints for data points tracked for market resolution.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetMilestonesParams, MilestoneResponse, MilestonesResponse},
};

fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// Returns milestones matching the provided query parameters.
pub async fn get_milestones(
    http: &HttpClient,
    params: GetMilestonesParams,
) -> Result<MilestonesResponse> {
    let path = format!("/milestones{}", params.to_query_string());
    http.get(&path).await
}

pub async fn get_milestone(http: &HttpClient, milestone_id: &str) -> Result<MilestoneResponse> {
    let path = format!("/milestones/{}", encode_path_segment(milestone_id));
    http.get(&path).await
}
