//! Account models and response types.

use serde::{Deserialize, Serialize};

/// Response from GET /account/limits.
///
/// Contains information about the user's API tier and rate limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTierLimitsResponse {
    /// The user's API usage tier (e.g., "standard", "premier").
    pub usage_tier: String,
    /// Maximum read requests per second.
    pub read_limit: i64,
    /// Maximum write requests per second.
    pub write_limit: i64,
}
