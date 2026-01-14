//! Subaccount models and response types.

use serde::{Deserialize, Serialize};

use crate::models::query::QueryBuilder;

/// Request body for POST /portfolio/subaccounts (create subaccount).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubaccountRequest {
    /// Optional name for the subaccount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl CreateSubaccountRequest {
    /// Create a new subaccount request without a name.
    #[must_use]
    pub fn new() -> Self {
        Self { name: None }
    }

    /// Create a new subaccount request with a name.
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }

    /// Set the subaccount name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Default for CreateSubaccountRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from POST /portfolio/subaccounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubaccountResponse {
    /// The created subaccount.
    pub subaccount: Subaccount,
}

/// A subaccount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subaccount {
    /// Subaccount number (1-32, or 0 for primary).
    pub subaccount_id: i32,
    /// Optional subaccount name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Request body for POST /portfolio/subaccounts/transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferBetweenSubaccountsRequest {
    /// Source subaccount (0 for primary, 1-32 for subaccounts).
    pub from_subaccount: i32,
    /// Destination subaccount (0 for primary, 1-32 for subaccounts).
    pub to_subaccount: i32,
    /// Amount to transfer in cents.
    pub amount: i64,
}

impl TransferBetweenSubaccountsRequest {
    /// Create a new transfer request.
    ///
    /// # Arguments
    ///
    /// * `from` - Source subaccount (0 for primary, 1-32 for numbered subaccounts)
    /// * `to` - Destination subaccount (0 for primary, 1-32 for numbered subaccounts)
    /// * `amount` - Amount to transfer in cents (must be positive)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `from` or `to` is not in range 0-32
    /// - `from` equals `to`
    /// - `amount` is not positive
    ///
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(from: i32, to: i32, amount: i64) -> Self {
        Self::try_new(from, to, amount).expect("invalid transfer request parameters")
    }

    /// Create a new transfer request with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `from` or `to` is not in range 0-32
    /// - `from` equals `to`
    /// - `amount` is not positive
    pub fn try_new(from: i32, to: i32, amount: i64) -> crate::error::Result<Self> {
        if !(0..=32).contains(&from) {
            return Err(crate::error::Error::InvalidSubaccountId(from));
        }
        if !(0..=32).contains(&to) {
            return Err(crate::error::Error::InvalidSubaccountId(to));
        }
        if from == to {
            return Err(crate::error::Error::SameSubaccountTransfer);
        }
        if amount <= 0 {
            return Err(crate::error::Error::InvalidTransferAmount(amount));
        }
        Ok(Self {
            from_subaccount: from,
            to_subaccount: to,
            amount,
        })
    }

    /// Returns the transfer amount in dollars.
    #[inline]
    #[must_use]
    pub fn amount_dollars(&self) -> f64 {
        self.amount as f64 / 100.0
    }
}

/// Response from POST /portfolio/subaccounts/transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    /// The transfer details.
    pub transfer: SubaccountTransfer,
}

/// A transfer between subaccounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountTransfer {
    /// Unique transfer ID.
    pub transfer_id: String,
    /// Source subaccount (0 for primary, 1-32 for subaccounts).
    pub from_subaccount: i32,
    /// Destination subaccount (0 for primary, 1-32 for subaccounts).
    pub to_subaccount: i32,
    /// Amount transferred in cents.
    pub amount: i64,
    /// Transfer timestamp (ISO 8601 or Unix epoch).
    #[serde(default)]
    pub created_time: Option<String>,
}

impl SubaccountTransfer {
    /// Returns the transfer amount in dollars.
    #[inline]
    #[must_use]
    pub fn amount_dollars(&self) -> f64 {
        self.amount as f64 / 100.0
    }
}

/// Response from GET /portfolio/subaccounts/balances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountBalancesResponse {
    /// List of balances for all subaccounts.
    pub subaccount_balances: Vec<SubaccountBalance>,
}

/// Balance for a single subaccount.
///
/// **Note:** This endpoint returns balance in **centicents** (1/100th of a cent),
/// which differs from `GET /portfolio/balance` which returns balance in cents.
/// Use [`balance_dollars()`](Self::balance_dollars) for convenient conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountBalance {
    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    pub subaccount_number: i32,
    /// Available balance in centicents (1/100th of a cent).
    /// Divide by 10,000 to get dollars.
    pub balance: i64,
    /// Last update timestamp (Unix seconds).
    #[serde(default)]
    pub updated_ts: Option<i64>,
}

impl SubaccountBalance {
    /// Returns the balance in dollars.
    ///
    /// Converts from centicents (API unit) to dollars by dividing by 10,000.
    #[inline]
    #[must_use]
    pub fn balance_dollars(&self) -> f64 {
        self.balance as f64 / 10000.0
    }
}

/// Query parameters for GET /portfolio/subaccounts/transfers.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetSubaccountTransfersParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Number of results per page (1-1000, default 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Filter by source subaccount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_subaccount: Option<i32>,
    /// Filter by destination subaccount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_subaccount: Option<i32>,
}

impl GetSubaccountTransfersParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set the number of results per page (clamped to 1-1000).
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit.clamp(1, 1000));
        self
    }

    #[must_use]
    pub fn from_subaccount(mut self, id: i32) -> Self {
        self.from_subaccount = Some(id);
        self
    }

    #[must_use]
    pub fn to_subaccount(mut self, id: i32) -> Self {
        self.to_subaccount = Some(id);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("from_subaccount", self.from_subaccount);
        qb.push_opt("to_subaccount", self.to_subaccount);
        qb.build()
    }
}

/// Response from GET /portfolio/subaccounts/transfers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubaccountTransfersResponse {
    /// List of transfers.
    pub transfers: Vec<SubaccountTransfer>,
    /// Pagination cursor for next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /portfolio/summary/total_resting_order_value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingOrderValueResponse {
    /// Total value of all resting orders in cents.
    pub total_resting_order_value: i64,
}

impl RestingOrderValueResponse {
    /// Returns the total resting order value in dollars.
    #[inline]
    #[must_use]
    pub fn total_resting_order_value_dollars(&self) -> f64 {
        self.total_resting_order_value as f64 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_validation() {
        assert!(TransferBetweenSubaccountsRequest::try_new(0, 1, 1000).is_ok());
        assert!(TransferBetweenSubaccountsRequest::try_new(1, 0, 1000).is_ok());
        assert!(TransferBetweenSubaccountsRequest::try_new(1, 2, 1000).is_ok());

        // Invalid from subaccount
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(-1, 1, 1000),
            Err(crate::error::Error::InvalidSubaccountId(-1))
        ));
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(33, 1, 1000),
            Err(crate::error::Error::InvalidSubaccountId(33))
        ));

        // Invalid to subaccount
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(0, 33, 1000),
            Err(crate::error::Error::InvalidSubaccountId(33))
        ));

        // Same subaccount
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(1, 1, 1000),
            Err(crate::error::Error::SameSubaccountTransfer)
        ));

        // Invalid amount
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(0, 1, 0),
            Err(crate::error::Error::InvalidTransferAmount(0))
        ));
        assert!(matches!(
            TransferBetweenSubaccountsRequest::try_new(0, 1, -100),
            Err(crate::error::Error::InvalidTransferAmount(-100))
        ));
    }

    #[test]
    fn test_amount_dollars() {
        let request = TransferBetweenSubaccountsRequest::new(0, 1, 12345);
        assert!((request.amount_dollars() - 123.45).abs() < f64::EPSILON);
    }

    #[test]
    fn test_balance_dollars() {
        // Balance is in centicents (1/100th of a cent), so 5000000 centicents = $500
        let balance = SubaccountBalance {
            subaccount_number: 0,
            balance: 5000000,
            updated_ts: None,
        };
        assert!((balance.balance_dollars() - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_query_string() {
        let params = GetSubaccountTransfersParams::new()
            .limit(50)
            .from_subaccount(0);
        let qs = params.to_query_string();
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("from_subaccount=0"));
    }

    #[test]
    fn test_create_subaccount_request() {
        let request = CreateSubaccountRequest::new();
        assert!(request.name.is_none());

        let request = CreateSubaccountRequest::with_name("Trading Bot");
        assert_eq!(request.name, Some("Trading Bot".to_string()));

        let request = CreateSubaccountRequest::new().name("Another Bot");
        assert_eq!(request.name, Some("Another Bot".to_string()));

        let request = CreateSubaccountRequest::default();
        assert!(request.name.is_none());
    }
}
