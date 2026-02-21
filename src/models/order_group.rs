//! Order group models and response types.

use serde::{Deserialize, Serialize};

use super::query::QueryBuilder;

/// Request body for POST /portfolio/order_groups/create.
///
/// Creates an empty order group with a contracts limit. Orders are then
/// associated with the group by including the `order_group_id` when creating them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderGroupRequest {
    /// The maximum number of contracts that can be matched within this group.
    /// When this limit is hit, all orders in the group are cancelled.
    /// Provide `contracts_limit` or `contracts_limit_fp` (or both — if both
    /// are provided they must agree).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit: Option<i64>,
    /// The maximum number of contracts (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit_fp: Option<String>,
}

impl CreateOrderGroupRequest {
    /// Create a new order group request from an integer count.
    ///
    /// # Arguments
    ///
    /// * `contracts_limit` - Maximum contracts before auto-cancel (must be >= 1)
    ///
    /// # Panics
    ///
    /// Panics if `contracts_limit` is less than 1.
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(contracts_limit: i64) -> Self {
        Self::try_new(contracts_limit).expect("invalid contracts limit")
    }

    /// Create a new order group request with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `contracts_limit` is less than 1.
    pub fn try_new(contracts_limit: i64) -> crate::error::Result<Self> {
        if contracts_limit < 1 {
            return Err(crate::error::Error::InvalidContractsLimit(contracts_limit));
        }
        Ok(Self {
            contracts_limit: Some(contracts_limit),
            contracts_limit_fp: None,
        })
    }

    /// Create a new order group request from a fixed-point string.
    #[must_use]
    pub fn from_fp(contracts_limit_fp: impl Into<String>) -> Self {
        Self {
            contracts_limit: None,
            contracts_limit_fp: Some(contracts_limit_fp.into()),
        }
    }
}

/// Response from POST /portfolio/order_groups/create.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderGroupResponse {
    /// The unique identifier for the created order group.
    pub order_group_id: String,
}

/// Request body for PUT /portfolio/order_groups/{order_group_id}/limit.
///
/// Provide `contracts_limit` or `contracts_limit_fp` (or both — if both are
/// provided they must agree). The limit is a rolling 15-second window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderGroupLimitRequest {
    /// The new maximum number of contracts for this group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit: Option<i64>,
    /// The new maximum number of contracts (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_limit_fp: Option<String>,
}

impl UpdateOrderGroupLimitRequest {
    /// Create a new update order group limit request from an integer count.
    ///
    /// # Arguments
    ///
    /// * `contracts_limit` - New maximum contracts before auto-cancel (must be >= 1)
    ///
    /// # Panics
    ///
    /// Panics if `contracts_limit` is less than 1.
    /// Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(contracts_limit: i64) -> Self {
        Self::try_new(contracts_limit).expect("invalid contracts limit")
    }

    /// Create a new update order group limit request with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `contracts_limit` is less than 1.
    pub fn try_new(contracts_limit: i64) -> crate::error::Result<Self> {
        if contracts_limit < 1 {
            return Err(crate::error::Error::InvalidContractsLimit(contracts_limit));
        }
        Ok(Self {
            contracts_limit: Some(contracts_limit),
            contracts_limit_fp: None,
        })
    }

    /// Create a new update order group limit request from a fixed-point string.
    #[must_use]
    pub fn from_fp(contracts_limit_fp: impl Into<String>) -> Self {
        Self {
            contracts_limit: None,
            contracts_limit_fp: Some(contracts_limit_fp.into()),
        }
    }
}

/// Response from GET /portfolio/order_groups/{order_group_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderGroupResponse {
    /// Whether auto-cancel is enabled for this order group.
    pub is_auto_cancel_enabled: bool,
    /// List of order IDs that belong to this order group.
    pub orders: Vec<String>,
    /// Contracts limit for this order group.
    #[serde(default)]
    pub contracts_limit: Option<i64>,
    /// Contracts limit (fixed-point decimal string).
    #[serde(default)]
    pub contracts_limit_fp: Option<String>,
}

/// An order group summary (used in list response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroupSummary {
    /// Unique identifier for the order group.
    pub id: String,
    /// Whether auto-cancel is enabled for this order group.
    pub is_auto_cancel_enabled: bool,
    /// Contracts limit for this order group.
    #[serde(default)]
    pub contracts_limit: Option<i64>,
    /// Contracts limit (fixed-point decimal string).
    #[serde(default)]
    pub contracts_limit_fp: Option<String>,
}

/// Query parameters for GET /portfolio/order_groups.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetOrderGroupsParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Number of results per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl GetOrderGroupsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set the number of results per page.
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("limit", self.limit);
        qb.build()
    }
}

/// Response from GET /portfolio/order_groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroupsResponse {
    /// List of order groups.
    pub order_groups: Vec<OrderGroupSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_order_group_request() {
        let req = CreateOrderGroupRequest::new(100);
        assert_eq!(req.contracts_limit, Some(100));

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"contracts_limit":100}"#);
    }

    #[test]
    fn test_create_order_group_request_from_fp() {
        let req = CreateOrderGroupRequest::from_fp("100.00");
        assert_eq!(req.contracts_limit, None);
        assert_eq!(req.contracts_limit_fp.as_deref(), Some("100.00"));

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"contracts_limit_fp":"100.00"}"#);
    }

    #[test]
    fn test_get_order_groups_params_query_string() {
        let params = GetOrderGroupsParams::new();
        assert_eq!(params.to_query_string(), "");

        let params = GetOrderGroupsParams::new().limit(50);
        assert!(params.to_query_string().contains("limit=50"));

        let params = GetOrderGroupsParams::new().cursor("abc123").limit(25);
        let qs = params.to_query_string();
        assert!(qs.contains("cursor=abc123"));
        assert!(qs.contains("limit=25"));
    }

    #[test]
    fn test_create_order_group_response_deserialize() {
        let json = r#"{"order_group_id": "og_123abc"}"#;
        let response: CreateOrderGroupResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.order_group_id, "og_123abc");
    }

    #[test]
    fn test_get_order_group_response_deserialize() {
        let json = r#"{"is_auto_cancel_enabled": true, "orders": ["order1", "order2"]}"#;
        let response: GetOrderGroupResponse = serde_json::from_str(json).unwrap();
        assert!(response.is_auto_cancel_enabled);
        assert_eq!(response.orders.len(), 2);
    }

    #[test]
    fn test_order_groups_response_deserialize() {
        let json = r#"{"order_groups": [{"id": "og_1", "is_auto_cancel_enabled": false}]}"#;
        let response: OrderGroupsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.order_groups.len(), 1);
        assert_eq!(response.order_groups[0].id, "og_1");
    }
}
