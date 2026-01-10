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

/// Get the current exchange status.
///
/// Returns whether the exchange and trading are currently active.
/// This is a public endpoint that does not require authentication.
pub async fn get_exchange_status(http: &HttpClient) -> Result<ExchangeStatusResponse> {
    http.get("/exchange/status").await
}

/// Get the exchange schedule.
///
/// Returns the weekly trading schedule and any scheduled maintenance windows.
/// All times are in Eastern Time (ET).
/// This is a public endpoint that does not require authentication.
pub async fn get_exchange_schedule(http: &HttpClient) -> Result<ExchangeScheduleResponse> {
    http.get("/exchange/schedule").await
}

/// Get exchange announcements.
///
/// Returns all exchange-wide announcements including info, warning, and error messages.
/// This is a public endpoint that does not require authentication.
pub async fn get_exchange_announcements(
    http: &HttpClient,
) -> Result<ExchangeAnnouncementsResponse> {
    http.get("/exchange/announcements").await
}

/// Get the user data timestamp.
///
/// Returns an approximate indication of when user portfolio data
/// (balance, orders, fills, positions) was last validated.
/// Useful for determining data freshness.
pub async fn get_user_data_timestamp(http: &HttpClient) -> Result<UserDataTimestampResponse> {
    http.get("/exchange/user_data_timestamp").await
}
