//! Balance models.

use serde::{Deserialize, Serialize};

/// Response from the get_balance endpoint.
///
/// All monetary values are in cents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: i64,
    pub portfolio_value: i64,
    pub updated_ts: i64,
}
