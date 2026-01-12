//! Incentive Programs API endpoints.
//!
//! This module provides functions for retrieving incentive programs,
//! which are rewards programs for trading activity on specific markets.

use crate::{client::HttpClient, error::Result, models::IncentiveProgramsResponse};

/// List all available incentive programs.
///
/// Returns all incentive programs, including active and upcoming programs.
///
/// # Example
///
/// ```ignore
/// let response = client.get_incentive_programs().await?;
/// for program in response.incentive_programs {
///     println!("{}: {:?}", program.name.unwrap_or_default(), program.status);
/// }
/// ```
pub async fn get_incentive_programs(http: &HttpClient) -> Result<IncentiveProgramsResponse> {
    http.get("/incentive_programs").await
}
