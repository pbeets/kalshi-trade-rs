//! Orders API endpoints.
//!
//! This module provides functions for creating, canceling, amending, and
//! managing orders on the Kalshi exchange.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        AmendOrderRequest, AmendOrderResponse, BatchCancelOrdersRequest, BatchCancelOrdersResponse,
        BatchCreateOrdersRequest, BatchCreateOrdersResponse, CancelOrderResponse,
        CreateOrderRequest, DecreaseOrderRequest, OrderResponse,
    },
};

/// Create a new order.
///
/// Submits an order to the exchange.
pub async fn create_order(
    http: &HttpClient,
    request: CreateOrderRequest,
) -> Result<OrderResponse> {
    http.post("/portfolio/orders", &request).await
}

/// Get a specific order by ID.
pub async fn get_order(http: &HttpClient, order_id: &str) -> Result<OrderResponse> {
    let path = format!("/portfolio/orders/{}", order_id);
    http.get(&path).await
}

/// Cancel an order by ID.
pub async fn cancel_order(http: &HttpClient, order_id: &str) -> Result<CancelOrderResponse> {
    let path = format!("/portfolio/orders/{}", order_id);
    http.delete_with_response(&path).await
}

/// Amend an existing order.
///
/// Modifies the price and/or quantity of an existing order.
pub async fn amend_order(
    http: &HttpClient,
    order_id: &str,
    request: AmendOrderRequest,
) -> Result<AmendOrderResponse> {
    let path = format!("/portfolio/orders/{}/amend", order_id);
    http.post(&path, &request).await
}

/// Decrease an order's quantity.
///
/// Reduces the remaining quantity of an order.
pub async fn decrease_order(
    http: &HttpClient,
    order_id: &str,
    request: DecreaseOrderRequest,
) -> Result<OrderResponse> {
    let path = format!("/portfolio/orders/{}/decrease", order_id);
    http.post(&path, &request).await
}

/// Create multiple orders in a single request.
///
/// Supports up to 20 orders per batch.
pub async fn batch_create_orders(
    http: &HttpClient,
    request: BatchCreateOrdersRequest,
) -> Result<BatchCreateOrdersResponse> {
    http.post("/portfolio/orders/batched", &request).await
}

/// Cancel multiple orders in a single request.
///
/// Supports up to 20 orders per batch.
pub async fn batch_cancel_orders(
    http: &HttpClient,
    request: BatchCancelOrdersRequest,
) -> Result<BatchCancelOrdersResponse> {
    http.delete_with_body("/portfolio/orders/batched", &request)
        .await
}
