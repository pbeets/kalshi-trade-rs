//! Structured Targets API endpoints for data targets used in market resolution.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetStructuredTargetsParams, StructuredTargetResponse, StructuredTargetsResponse},
};

fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// Returns structured targets matching the provided query parameters.
pub async fn get_structured_targets(
    http: &HttpClient,
    params: GetStructuredTargetsParams,
) -> Result<StructuredTargetsResponse> {
    let path = format!("/structured_targets{}", params.to_query_string());
    http.get(&path).await
}

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
