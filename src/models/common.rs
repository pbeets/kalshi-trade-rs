//! Common types used across the Kalshi API.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Side of a position or order (Yes or No).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Yes,
    No,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Yes => write!(f, "YES"),
            Side::No => write!(f, "NO"),
        }
    }
}

/// Action type for an order (Buy or Sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Buy,
    Sell,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Buy => write!(f, "BUY"),
            Action::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type (Limit or Market).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Resting,
    Canceled,
    Executed,
}

impl OrderStatus {
    /// Returns the lowercase API representation.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Resting => "resting",
            OrderStatus::Canceled => "canceled",
            OrderStatus::Executed => "executed",
        }
    }
}

/// Self-trade prevention type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelfTradePreventionType {
    TakerAtCross,
    Maker,
}

/// Convert cents to dollars.
///
/// All Kalshi API monetary values are in cents. This helper converts to dollars
/// for display purposes.
///
/// # Example
///
/// ```
/// use kalshi_trade_rs::cents_to_dollars;
///
/// let balance_cents = 12345_i64;
/// println!("Balance: ${:.2}", cents_to_dollars(balance_cents));
/// // Output: Balance: $123.45
/// ```
#[inline]
#[must_use]
pub fn cents_to_dollars(cents: i64) -> f64 {
    cents as f64 / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_side() {
        assert_eq!(format!("{}", Side::Yes), "YES");
        assert_eq!(format!("{}", Side::No), "NO");
    }

    #[test]
    fn test_display_action() {
        assert_eq!(format!("{}", Action::Buy), "BUY");
        assert_eq!(format!("{}", Action::Sell), "SELL");
    }
}
