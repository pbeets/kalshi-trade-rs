//! FCM (Futures Commission Merchant) API endpoints.
//!
//! Endpoints for FCM members to access orders and positions filtered by subtrader ID.
//! Requires FCM member access level.

use crate::{
    client::HttpClient,
    error::Result,
    models::{GetFcmOrdersParams, GetFcmPositionsParams, OrdersResponse, PositionsResponse},
};

/// Returns orders for a specific subtrader (FCM members only).
pub async fn get_fcm_orders(
    http: &HttpClient,
    params: GetFcmOrdersParams,
) -> Result<OrdersResponse> {
    let path = format!("/fcm/orders{}", params.to_query_string());
    http.get(&path).await
}

/// Returns market positions for a specific subtrader (FCM members only).
pub async fn get_fcm_positions(
    http: &HttpClient,
    params: GetFcmPositionsParams,
) -> Result<PositionsResponse> {
    let path = format!("/fcm/positions{}", params.to_query_string());
    http.get(&path).await
}
