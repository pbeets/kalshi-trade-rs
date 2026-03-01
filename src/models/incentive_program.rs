//! Incentive program models and response types.
//!
//! Incentive programs are rewards programs for trading activity on specific markets.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// An incentive program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveProgram {
    /// The unique identifier for the incentive program.
    pub id: String,
    /// The unique identifier of the market associated with this incentive program.
    pub market_id: String,
    /// The associated market ticker.
    pub market_ticker: String,
    /// The incentive type (e.g., "volume", "liquidity").
    pub incentive_type: String,
    /// The start date of the incentive program (RFC3339 timestamp).
    pub start_date: String,
    /// The end date of the incentive program (RFC3339 timestamp).
    pub end_date: String,
    /// The reward amount for the period (in cents).
    pub period_reward: i64,
    /// Whether the program has been paid out.
    pub paid_out: bool,
    /// Discount factor in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_factor_bps: Option<i64>,
    /// Target size for the program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_size: Option<i64>,
    /// Target size (fixed-point decimal string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_size_fp: Option<String>,
}

/// Response from GET /incentive_programs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveProgramsResponse {
    /// The list of incentive programs.
    pub incentive_programs: Vec<IncentiveProgram>,
    /// Cursor for pagination.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Query parameters for GET /incentive_programs.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetIncentiveProgramsParams {
    /// Filter by status: "all", "active", "upcoming", "closed", or "paid_out".
    /// Defaults to "all".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by type: "all", "liquidity", or "volume".
    /// Defaults to "all".
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_type: Option<String>,
    /// Maximum number of results to return. Defaults to 100, max 10000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetIncentiveProgramsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by status.
    ///
    /// Valid values: "all", "active", "upcoming", "closed", "paid_out".
    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Filter by program type.
    ///
    /// Valid values: "all", "liquidity", "volume".
    #[must_use]
    pub fn program_type(mut self, program_type: impl Into<String>) -> Self {
        self.program_type = Some(program_type.into());
        self
    }

    /// Set the maximum number of results to return.
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("status", self.status.as_ref());
        qb.push_opt("type", self.program_type.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}
