//! Incentive Programs API endpoints.
//!
//! This module provides functions for retrieving incentive programs,
//! which are rewards programs for trading activity on specific markets.

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetIncentiveProgramsParams, IncentiveProgramsResponse},
};

/// List incentive programs with optional filtering and pagination.
///
/// Returns incentive programs matching the provided query parameters.
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::GetIncentiveProgramsParams;
///
/// // Get only active programs
/// let params = GetIncentiveProgramsParams::new()
///     .status("active")
///     .limit(50);
/// let response = client.get_incentive_programs_with_params(params).await?;
/// for program in response.incentive_programs {
///     println!("{}: {:?}", program.name.unwrap_or_default(), program.status);
/// }
/// ```
pub async fn get_incentive_programs(
    http: &HttpClient,
    params: GetIncentiveProgramsParams,
) -> Result<IncentiveProgramsResponse> {
    let path = format!("/incentive_programs{}", params.to_query_string());
    http.get(&path).await
}
