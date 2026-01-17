//! Exchange API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Exchange API,
//! including exchange status, schedule, announcements, and user data timestamps.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        ExchangeAnnouncementsResponse, ExchangeScheduleResponse, ExchangeStatusResponse,
        UserDataTimestampResponse,
    },
};

/// Returns whether the exchange and trading are currently active.
///
/// Public endpoint - no authentication required.
pub async fn get_exchange_status(http: &HttpClient) -> Result<ExchangeStatusResponse> {
    http.get("/exchange/status").await
}

/// Returns the weekly trading schedule and scheduled maintenance windows.
///
/// All times are in Eastern Time (ET). Public endpoint - no authentication required.
pub async fn get_exchange_schedule(http: &HttpClient) -> Result<ExchangeScheduleResponse> {
    http.get("/exchange/schedule").await
}

/// Returns exchange-wide announcements including info, warning, and error messages.
///
/// Public endpoint - no authentication required.
pub async fn get_exchange_announcements(
    http: &HttpClient,
) -> Result<ExchangeAnnouncementsResponse> {
    http.get("/exchange/announcements").await
}

/// Returns when user portfolio data (balance, orders, fills, positions) was last validated.
///
/// Useful for determining data freshness when there may be delays in reflecting recent trades.
pub async fn get_user_data_timestamp(http: &HttpClient) -> Result<UserDataTimestampResponse> {
    http.get("/exchange/user_data_timestamp").await
}
