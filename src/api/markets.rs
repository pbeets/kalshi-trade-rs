//! Markets API endpoints.
//!
//! This module provides functions for interacting with the Kalshi Markets API,
//! including listing markets, getting market details, orderbooks, trades, and candlesticks.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        BatchCandlesticksResponse, CandlesticksResponse, GetBatchCandlesticksParams,
        GetCandlesticksParams, GetMarketsParams, GetOrderbookParams, GetTradesParams,
        MarketResponse, MarketsResponse, OrderbookResponse, TradesResponse,
    },
};

/// URL-encode a ticker for use in path segments.
fn encode_ticker(ticker: &str) -> String {
    form_urlencoded::byte_serialize(ticker.as_bytes()).collect()
}

/// Get a list of markets with optional filtering.
///
/// Returns markets matching the provided query parameters.
pub async fn get_markets(http: &HttpClient, params: GetMarketsParams) -> Result<MarketsResponse> {
    let path = format!("/markets{}", params.to_query_string());
    http.get(&path).await
}

/// Get details for a specific market by ticker.
pub async fn get_market(http: &HttpClient, ticker: &str) -> Result<MarketResponse> {
    let path = format!("/markets/{}", encode_ticker(ticker));
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
    let path = format!(
        "/markets/{}/orderbook{}",
        encode_ticker(ticker),
        params.to_query_string()
    );
    http.get(&path).await
}

/// Get trades with optional filtering.
///
/// Returns a list of executed trades on the exchange.
pub async fn get_trades(http: &HttpClient, params: GetTradesParams) -> Result<TradesResponse> {
    let path = format!("/markets/trades{}", params.to_query_string());
    http.get(&path).await
}

/// Get candlestick (OHLCV) data for a specific market.
///
/// Returns historical price data in candlestick format.
pub async fn get_candlesticks(
    http: &HttpClient,
    series_ticker: &str,
    ticker: &str,
    params: GetCandlesticksParams,
) -> Result<CandlesticksResponse> {
    let path = format!(
        "/series/{}/markets/{}/candlesticks{}",
        encode_ticker(series_ticker),
        encode_ticker(ticker),
        params.to_query_string()
    );
    http.get(&path).await
}

/// Get candlestick data for multiple markets in a single request.
///
/// Supports up to 100 market tickers per request.
/// Returns up to 10,000 candlesticks total across all markets.
pub async fn get_batch_candlesticks(
    http: &HttpClient,
    params: GetBatchCandlesticksParams,
) -> Result<BatchCandlesticksResponse> {
    let path = format!("/markets/candlesticks{}", params.to_query_string());
    http.get(&path).await
}
