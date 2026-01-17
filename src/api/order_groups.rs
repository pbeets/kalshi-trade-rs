//! Order Groups API endpoints.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateOrderGroupRequest, CreateOrderGroupResponse, GetOrderGroupResponse,
        GetOrderGroupsParams, OrderGroupsResponse,
    },
};

fn encode_id(id: &str) -> String {
    form_urlencoded::byte_serialize(id.as_bytes()).collect()
}

/// Creates an order group with a contracts limit (auto-cancels all when limit hit).
pub async fn create_order_group(
    http: &HttpClient,
    request: CreateOrderGroupRequest,
) -> Result<CreateOrderGroupResponse> {
    http.post("/portfolio/order_groups/create", &request).await
}

/// Returns order group details including order IDs and auto-cancel status.
pub async fn get_order_group(
    http: &HttpClient,
    order_group_id: &str,
) -> Result<GetOrderGroupResponse> {
    let path = format!("/portfolio/order_groups/{}", encode_id(order_group_id));
    http.get(&path).await
}

/// Returns all order groups for the authenticated user.
pub async fn list_order_groups(
    http: &HttpClient,
    params: GetOrderGroupsParams,
) -> Result<OrderGroupsResponse> {
    let path = format!("/portfolio/order_groups{}", params.to_query_string());
    http.get(&path).await
}

/// Deletes an order group and cancels all orders within it.
pub async fn delete_order_group(http: &HttpClient, order_group_id: &str) -> Result<()> {
    let path = format!("/portfolio/order_groups/{}", encode_id(order_group_id));
    http.delete(&path).await
}

/// Resets the matched contracts counter to zero, re-enabling order placement.
pub async fn reset_order_group(http: &HttpClient, order_group_id: &str) -> Result<()> {
    let path = format!(
        "/portfolio/order_groups/{}/reset",
        encode_id(order_group_id)
    );
    http.put_empty_json(&path).await
}
