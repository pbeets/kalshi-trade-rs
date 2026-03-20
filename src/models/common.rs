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
    #[deprecated(
        note = "Market orders are no longer supported by the Kalshi API. Use limit orders instead."
    )]
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

/// Exchange instance type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExchangeInstance {
    /// Event contract exchange.
    EventContract,
    /// Margined exchange.
    Margined,
    /// Unknown exchange instance.
    #[serde(other)]
    Unknown,
}

/// Custom deserializer that treats `null` as an empty `Vec`.
///
/// The Kalshi API sometimes returns `null` instead of `[]` for empty arrays.
/// Use with `#[serde(default, deserialize_with = "null_as_empty_vec::deserialize")]`.
pub(crate) mod null_as_empty_vec {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
    }
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
