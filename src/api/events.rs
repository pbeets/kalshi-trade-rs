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

/// Returns events matching the provided query parameters (excludes multivariate events).
pub async fn get_events(http: &HttpClient, params: GetEventsParams) -> Result<EventsResponse> {
    let path = format!("/events{}", params.to_query_string());
    http.get(&path).await
}

pub async fn get_event(
    http: &HttpClient,
    event_ticker: &str,
    params: GetEventParams,
) -> Result<EventResponse> {
    let path = format!("/events/{}{}", event_ticker, params.to_query_string());
    http.get(&path).await
}

pub async fn get_event_metadata(
    http: &HttpClient,
    event_ticker: &str,
) -> Result<EventMetadataResponse> {
    let path = format!("/events/{}/metadata", event_ticker);
    http.get(&path).await
}

/// Returns dynamically created events from multivariate event collections.
pub async fn get_multivariate_events(
    http: &HttpClient,
    params: GetMultivariateEventsParams,
) -> Result<MultivariateEventsResponse> {
    let path = format!("/events/multivariate{}", params.to_query_string());
    http.get(&path).await
}

/// Returns candlestick data aggregated across all markets in an event.
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

/// Returns historical forecast data at specific percentiles for an event.
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
