//! Markets API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Markets API,
//! including listing markets, getting market details, orderbooks, and trades.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        GetMarketsParams, GetOrderbookParams, GetTradesParams, MarketResponse, MarketsResponse,
        OrderbookResponse, TradesResponse,
    },
};

/// Get a list of markets with optional filtering.
///
/// Returns markets matching the provided query parameters.
pub async fn get_markets(http: &HttpClient, params: GetMarketsParams) -> Result<MarketsResponse> {
    let path = format!("/markets{}", params.to_query_string());
    http.get(&path).await
}

/// Get details for a specific market by ticker.
pub async fn get_market(http: &HttpClient, ticker: &str) -> Result<MarketResponse> {
    let path = format!("/markets/{}", ticker);
    http.get(&path).await
}

/// Get the orderbook for a specific market.
///
/// Returns the current order book with bid/ask prices and quantities.
pub async fn get_orderbook(
    http: &HttpClient,
    ticker: &str,
    params: GetOrderbookParams,
) -> Result<OrderbookResponse> {
    let path = format!("/markets/{}/orderbook{}", ticker, params.to_query_string());
    http.get(&path).await
}

/// Get trades with optional filtering.
///
/// Returns a list of executed trades on the exchange.
pub async fn get_trades(http: &HttpClient, params: GetTradesParams) -> Result<TradesResponse> {
    let path = format!("/markets/trades{}", params.to_query_string());
    http.get(&path).await
}
