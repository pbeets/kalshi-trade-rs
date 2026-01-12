//! Order Groups API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Order Groups API.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateOrderGroupRequest, GetOrderGroupsParams, OrderGroupResponse, OrderGroupsResponse,
        UpdateOrderGroupRequest,
    },
};

/// URL-encode an ID for use in path segments.
fn encode_id(id: &str) -> String {
    form_urlencoded::byte_serialize(id.as_bytes()).collect()
}

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
    let path = format!("/portfolio/order_groups/{}", encode_id(order_group_id));
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
    let path = format!("/portfolio/order_groups/{}", encode_id(order_group_id));
    http.put(&path, &request).await
}

/// List all order groups.
///
/// Returns all order groups for the authenticated user.
pub async fn list_order_groups(
    http: &HttpClient,
    params: GetOrderGroupsParams,
) -> Result<OrderGroupsResponse> {
    let path = format!("/portfolio/order_groups{}", params.to_query_string());
    http.get(&path).await
}

/// Delete an order group.
///
/// Deletes an order group and cancels all orders within it.
/// This permanently removes the group.
pub async fn delete_order_group(
    http: &HttpClient,
    order_group_id: &str,
) -> Result<OrderGroupResponse> {
    let path = format!("/portfolio/order_groups/{}", encode_id(order_group_id));
    http.delete_with_response(&path).await
}

/// Reset an order group.
///
/// Resets the order group's matched contracts counter to zero,
/// allowing new orders to be placed again after the limit was hit.
pub async fn reset_order_group(
    http: &HttpClient,
    order_group_id: &str,
) -> Result<OrderGroupResponse> {
    let path = format!("/portfolio/order_groups/{}/reset", encode_id(order_group_id));
    http.put_empty(&path).await
}
