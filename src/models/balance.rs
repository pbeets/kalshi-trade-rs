//! Balance models.

use serde::{Deserialize, Serialize};

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
