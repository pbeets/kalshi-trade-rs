//! Order Groups API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Order Groups API.

use crate::{
    client::HttpClient,
    error::Result,
    models::{CreateOrderGroupRequest, OrderGroupResponse, UpdateOrderGroupRequest},
};

/// Create a new order group.
///
/// Creates multiple orders atomically as a group.
pub async fn create_order_group(
    http: &HttpClient,
    request: CreateOrderGroupRequest,
) -> Result<OrderGroupResponse> {
    http.post("/portfolio/order_groups", &request).await
}

/// Get an order group by ID.
///
/// Returns details about a specific order group and its contained orders.
pub async fn get_order_group(
    http: &HttpClient,
    order_group_id: &str,
) -> Result<OrderGroupResponse> {
    let path = format!("/portfolio/order_groups/{}", order_group_id);
    http.get(&path).await
}

/// Update an existing order group.
///
/// Modifies orders within an existing group.
pub async fn update_order_group(
    http: &HttpClient,
    order_group_id: &str,
    request: UpdateOrderGroupRequest,
) -> Result<OrderGroupResponse> {
    let path = format!("/portfolio/order_groups/{}", order_group_id);
    http.put(&path, &request).await
}
