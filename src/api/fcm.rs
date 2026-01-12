//! FCM (Futures Commission Merchant) API endpoints.
//!
//! This module provides endpoints for FCM members to access orders and positions
//! filtered by subtrader ID. These endpoints require FCM member access level.

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetFcmOrdersParams, GetFcmPositionsParams, OrdersResponse, PositionsResponse},
};

/// Get FCM orders filtered by subtrader ID.
///
/// This endpoint is for FCM members to retrieve orders for a specific subtrader.
/// Requires FCM member access level.
///
/// # Arguments
///
/// * `http` - The HTTP client
/// * `params` - Query parameters including the required subtrader_id
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::{GetFcmOrdersParams, OrderStatus};
///
/// let params = GetFcmOrdersParams::new("subtrader-123")
///     .status(OrderStatus::Resting)
///     .limit(100);
/// let response = client.get_fcm_orders(params).await?;
/// for order in response.orders {
///     println!("Order {}: {:?}", order.order_id, order.status);
/// }
/// ```
pub async fn get_fcm_orders(
    http: &HttpClient,
    params: GetFcmOrdersParams,
) -> Result<OrdersResponse> {
    let path = format!("/fcm/orders{}", params.to_query_string());
    http.get(&path).await
}

/// Get FCM positions filtered by subtrader ID.
///
/// This endpoint is for FCM members to retrieve market positions for a specific subtrader.
/// Requires FCM member access level.
///
/// # Arguments
///
/// * `http` - The HTTP client
/// * `params` - Query parameters including the required subtrader_id
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::{GetFcmPositionsParams, SettlementStatus};
///
/// let params = GetFcmPositionsParams::new("subtrader-123")
///     .settlement_status(SettlementStatus::Unsettled)
///     .limit(100);
/// let response = client.get_fcm_positions(params).await?;
/// for pos in response.market_positions {
///     println!("{}: {} contracts", pos.ticker, pos.position);
/// }
/// ```
pub async fn get_fcm_positions(
    http: &HttpClient,
    params: GetFcmPositionsParams,
) -> Result<PositionsResponse> {
    let path = format!("/fcm/positions{}", params.to_query_string());
    http.get(&path).await
}
