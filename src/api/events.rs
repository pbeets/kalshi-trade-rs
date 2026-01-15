//! Events API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Events API,
//! including listing events and getting event details.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        EventCandlesticksResponse, EventForecastPercentileHistoryResponse, EventMetadataResponse,
        EventResponse, EventsResponse, GetEventCandlesticksParams,
        GetEventForecastPercentileHistoryParams, GetEventParams, GetEventsParams,
        GetMultivariateEventsParams, MultivariateEventsResponse,
    },
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

/// Get metadata for a specific event.
pub async fn get_event_metadata(
    http: &HttpClient,
    event_ticker: &str,
) -> Result<EventMetadataResponse> {
    let path = format!("/events/{}/metadata", event_ticker);
    http.get(&path).await
}

/// Get multivariate (combo) events.
///
/// Retrieves dynamically created events from multivariate event collections.
pub async fn get_multivariate_events(
    http: &HttpClient,
    params: GetMultivariateEventsParams,
) -> Result<MultivariateEventsResponse> {
    let path = format!("/events/multivariate{}", params.to_query_string());
    http.get(&path).await
}

/// Get aggregated candlestick data for an event.
///
/// Returns candlestick data aggregated across all markets in the event.
pub async fn get_event_candlesticks(
    http: &HttpClient,
    series_ticker: &str,
    event_ticker: &str,
    params: GetEventCandlesticksParams,
) -> Result<EventCandlesticksResponse> {
    let path = format!(
        "/series/{}/events/{}/candlesticks{}",
        series_ticker,
        event_ticker,
        params.to_query_string()
    );
    http.get(&path).await
}

/// Get forecast percentile history for an event.
///
/// Returns historical forecast data at specific percentiles.
pub async fn get_event_forecast_percentile_history(
    http: &HttpClient,
    series_ticker: &str,
    event_ticker: &str,
    params: GetEventForecastPercentileHistoryParams,
) -> Result<EventForecastPercentileHistoryResponse> {
    let path = format!(
        "/series/{}/events/{}/forecast_percentile_history{}",
        series_ticker,
        event_ticker,
        params.to_query_string()
    );
    http.get(&path).await
}
