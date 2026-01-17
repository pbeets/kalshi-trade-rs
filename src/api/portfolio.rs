//! Portfolio API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Portfolio API,
//! including balance, positions, fills, orders, and settlements.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        BalanceResponse, FillsResponse, GetFillsParams, GetOrdersParams, GetPositionsParams,
        GetSettlementsParams, OrdersResponse, PositionsResponse, SettlementsResponse,
    },
};

/// Returns the available balance and portfolio value in cents.
pub async fn get_balance(http: &HttpClient) -> Result<BalanceResponse> {
    http.get("/portfolio/balance").await
}

pub async fn get_positions(
    http: &HttpClient,
    params: GetPositionsParams,
) -> Result<PositionsResponse> {
    let path = format!("/portfolio/positions{}", params.to_query_string());
    http.get(&path).await
}

pub async fn get_fills(http: &HttpClient, params: GetFillsParams) -> Result<FillsResponse> {
    let path = format!("/portfolio/fills{}", params.to_query_string());
    http.get(&path).await
}

pub async fn get_orders(http: &HttpClient, params: GetOrdersParams) -> Result<OrdersResponse> {
    let path = format!("/portfolio/orders{}", params.to_query_string());
    http.get(&path).await
}

/// Returns historical settlement records showing P&L from settled markets.
pub async fn get_settlements(
    http: &HttpClient,
    params: GetSettlementsParams,
) -> Result<SettlementsResponse> {
    let path = format!("/portfolio/settlements{}", params.to_query_string());
    http.get(&path).await
}
