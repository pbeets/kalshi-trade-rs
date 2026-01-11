//! Events API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Events API,
//! including listing events and getting event details.

use crate::{
    client::HttpClient,
    error::Result,
    models::{EventResponse, EventsResponse, GetEventParams, GetEventsParams},
};

/// Get a list of events with optional filtering.
///
/// Returns events matching the provided query parameters.
/// Note: This endpoint excludes multivariate events.
pub async fn get_events(http: &HttpClient, params: GetEventsParams) -> Result<EventsResponse> {
    let path = format!("/events{}", params.to_query_string());
    http.get(&path).await
}

/// Get details for a specific event by ticker.
pub async fn get_event(
    http: &HttpClient,
    event_ticker: &str,
    params: GetEventParams,
) -> Result<EventResponse> {
    let path = format!("/events/{}{}", event_ticker, params.to_query_string());
    http.get(&path).await
}
