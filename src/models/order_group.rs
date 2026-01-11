//! Order group models and response types.

use serde::{Deserialize, Serialize};

use super::{common::OrderType, order::Order};

/// Request body for POST /portfolio/order_groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderGroupRequest {
    /// List of orders to create in the group.
    pub orders: Vec<CreateOrderGroupOrder>,
}

/// Order details for creating an order group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderGroupOrder {
    /// Market ticker.
    pub ticker: String,
    /// Side of the order (yes or no).
    pub side: crate::models::Side,
    /// Action (buy or sell).
    pub action: crate::models::Action,
    /// Number of contracts.
    pub count: i64,
    /// Order type (limit or market).
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    /// Yes price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    /// No price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
    /// Client-assigned order identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

impl CreateOrderGroupOrder {
    /// Create a new order for an order group.
    #[must_use]
    pub fn new(
        ticker: impl Into<String>,
        side: crate::models::Side,
        action: crate::models::Action,
        count: i64,
    ) -> Self {
        Self {
            ticker: ticker.into(),
            side,
            action,
            count,
            order_type: None,
            yes_price: None,
            no_price: None,
            client_order_id: None,
        }
    }

    #[must_use]
    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    #[must_use]
    pub fn yes_price(mut self, price: i64) -> Self {
        self.yes_price = Some(price);
        self
    }

    #[must_use]
    pub fn no_price(mut self, price: i64) -> Self {
        self.no_price = Some(price);
        self
    }

    #[must_use]
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }
}

/// Request body for PUT /portfolio/order_groups/{order_group_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderGroupRequest {
    /// List of orders to update or create.
    pub orders: Vec<UpdateOrderGroupOrder>,
}

/// Order details for updating an order group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderGroupOrder {
    /// Existing order ID to update (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Client order ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Market ticker.
    pub ticker: String,
    /// Side of the order (yes or no).
    pub side: crate::models::Side,
    /// Action (buy or sell).
    pub action: crate::models::Action,
    /// Number of contracts.
    pub count: i64,
    /// Order type (limit or market).
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    /// Yes price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,
    /// No price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,
}

impl UpdateOrderGroupOrder {
    /// Create a new order update for an order group.
    #[must_use]
    pub fn new(
        ticker: impl Into<String>,
        side: crate::models::Side,
        action: crate::models::Action,
        count: i64,
    ) -> Self {
        Self {
            order_id: None,
            client_order_id: None,
            ticker: ticker.into(),
            side,
            action,
            count,
            order_type: None,
            yes_price: None,
            no_price: None,
        }
    }

    #[must_use]
    pub fn order_id(mut self, id: impl Into<String>) -> Self {
        self.order_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    #[must_use]
    pub fn yes_price(mut self, price: i64) -> Self {
        self.yes_price = Some(price);
        self
    }

    #[must_use]
    pub fn no_price(mut self, price: i64) -> Self {
        self.no_price = Some(price);
        self
    }
}

/// Response from POST /portfolio/order_groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroupResponse {
    pub order_group: OrderGroup,
}

/// An order group containing multiple orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGroup {
    pub order_group_id: String,
    pub orders: Vec<Order>,
    pub created_at: i64,
    pub updated_at: i64,
}
