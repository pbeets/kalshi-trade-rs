//! Incentive Programs API for rewards programs based on trading activity.

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetIncentiveProgramsParams, IncentiveProgramsResponse},
};

/// Returns incentive programs matching the provided query parameters.
pub async fn get_incentive_programs(
    http: &HttpClient,
    params: GetIncentiveProgramsParams,
) -> Result<IncentiveProgramsResponse> {
    let path = format!("/incentive_programs{}", params.to_query_string());
    http.get(&path).await
}
