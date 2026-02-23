//! Balance models.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// Response from the get_balance endpoint.
///
/// All monetary values are in cents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    /// Available balance in cents.
    pub balance: i64,
    /// Total portfolio value in cents.
    pub portfolio_value: i64,
    /// Last update timestamp.
    pub updated_ts: i64,
}

impl BalanceResponse {
    /// Returns the available balance in dollars.
    ///
    /// # Example
    ///
    /// ```
    /// use kalshi_trade_rs::models::BalanceResponse;
    ///
    /// let balance = BalanceResponse {
    ///     balance: 12345,
    ///     portfolio_value: 50000,
    ///     updated_ts: 1234567890,
    /// };
    /// assert_eq!(balance.balance_dollars(), 123.45);
    /// ```
    #[inline]
    #[must_use]
    pub fn balance_dollars(&self) -> f64 {
        self.balance as f64 / 100.0
    }

    /// Returns the portfolio value in dollars.
    #[inline]
    #[must_use]
    pub fn portfolio_value_dollars(&self) -> f64 {
        self.portfolio_value as f64 / 100.0
    }
}

/// Query parameters for the get_balance endpoint.
///
/// Omitting the `subaccount` field returns the combined balance across all subaccounts.
/// Use `subaccount(0)` to get the primary account balance only.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetBalanceParams {
    /// Filter by subaccount number (0 for primary, 1-32 for subaccounts).
    /// When omitted, returns combined balance across all subaccounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl GetBalanceParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by subaccount number.
    ///
    /// Use 0 for the primary account, or 1-32 for numbered subaccounts.
    /// When omitted, returns combined balance across all subaccounts.
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("subaccount", self.subaccount);
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_dollars() {
        let balance = BalanceResponse {
            balance: 12345,
            portfolio_value: 50000,
            updated_ts: 0,
        };
        assert!((balance.balance_dollars() - 123.45).abs() < f64::EPSILON);
        assert!((balance.portfolio_value_dollars() - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_negative_balance() {
        let balance = BalanceResponse {
            balance: -5000,
            portfolio_value: 0,
            updated_ts: 0,
        };
        assert!((balance.balance_dollars() - (-50.0)).abs() < f64::EPSILON);
    }
}
