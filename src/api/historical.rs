//! Historical data API endpoints.
//!
//! These endpoints provide access to historical market data that has been
//! archived past the cutoff timestamp. Cutoff, markets, and candlestick
//! endpoints do not require authentication. Fills and orders require auth.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        FillsResponse, GetHistoricalCandlesticksParams, GetHistoricalFillsParams,
        GetHistoricalMarketsParams, GetHistoricalOrdersParams, HistoricalCandlesticksResponse,
        HistoricalCutoffResponse, MarketResponse, MarketsResponse, OrdersResponse,
    },
};

fn encode_ticker(ticker: &str) -> String {
    form_urlencoded::byte_serialize(ticker.as_bytes()).collect()
}

/// Returns the cutoff timestamps for historical data.
pub async fn get_historical_cutoff(http: &HttpClient) -> Result<HistoricalCutoffResponse> {
    http.get("/historical/cutoff").await
}

/// Returns historical markets matching the provided query parameters.
pub async fn get_historical_markets(
    http: &HttpClient,
    params: GetHistoricalMarketsParams,
) -> Result<MarketsResponse> {
    let path = format!("/historical/markets{}", params.to_query_string());
    http.get(&path).await
}

/// Returns a specific historical market by ticker.
pub async fn get_historical_market(http: &HttpClient, ticker: &str) -> Result<MarketResponse> {
    let path = format!("/historical/markets/{}", encode_ticker(ticker));
    http.get(&path).await
}

/// Returns historical candlestick data for a market.
pub async fn get_historical_candlesticks(
    http: &HttpClient,
    ticker: &str,
    params: GetHistoricalCandlesticksParams,
) -> Result<HistoricalCandlesticksResponse> {
    let path = format!(
        "/historical/markets/{}/candlesticks{}",
        encode_ticker(ticker),
        params.to_query_string()
    );
    http.get(&path).await
}

/// Returns historical fills.
pub async fn get_historical_fills(
    http: &HttpClient,
    params: GetHistoricalFillsParams,
) -> Result<FillsResponse> {
    let path = format!("/historical/fills{}", params.to_query_string());
    http.get(&path).await
}

/// Returns historical orders.
pub async fn get_historical_orders(
    http: &HttpClient,
    params: GetHistoricalOrdersParams,
) -> Result<OrdersResponse> {
    let path = format!("/historical/orders{}", params.to_query_string());
    http.get(&path).await
}
