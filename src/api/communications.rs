//! Communications API endpoints (RFQs and Quotes).
//!
//! This module provides functions for interacting with the Kalshi Communications API.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        AcceptQuoteRequest, CreateQuoteRequest, CreateRfqRequest, GetQuoteResponse, GetRfqResponse,
        ListQuotesResponse, ListRfqsResponse, QuoteResponse, RfqResponse,
    },
};

/// Create a new RFQ (Request for Quote).
pub async fn create_rfq(http: &HttpClient, request: CreateRfqRequest) -> Result<RfqResponse> {
    http.post("/communications/rfqs", &request).await
}

/// Create a new quote for an RFQ.
pub async fn create_quote(http: &HttpClient, request: CreateQuoteRequest) -> Result<QuoteResponse> {
    http.post("/communications/quotes", &request).await
}

/// Accept a quote.
pub async fn accept_quote(
    http: &HttpClient,
    quote_id: &str,
    request: AcceptQuoteRequest,
) -> Result<QuoteResponse> {
    let path = format!("/communications/quotes/{}/accept", quote_id);
    http.post(&path, &request).await
}

/// Cancel an RFQ.
pub async fn cancel_rfq(http: &HttpClient, rfq_id: &str) -> Result<RfqResponse> {
    let path = format!("/communications/rfqs/{}", rfq_id);
    http.delete_with_response(&path).await
}

/// Cancel a quote.
pub async fn cancel_quote(http: &HttpClient, quote_id: &str) -> Result<QuoteResponse> {
    let path = format!("/communications/quotes/{}", quote_id);
    http.delete_with_response(&path).await
}

/// Get details of an RFQ.
pub async fn get_rfq(http: &HttpClient, rfq_id: &str) -> Result<GetRfqResponse> {
    let path = format!("/communications/rfqs/{}", rfq_id);
    http.get(&path).await
}

/// Get details of a quote.
pub async fn get_quote(http: &HttpClient, quote_id: &str) -> Result<GetQuoteResponse> {
    let path = format!("/communications/quotes/{}", quote_id);
    http.get(&path).await
}

/// List RFQs.
pub async fn list_rfqs(http: &HttpClient) -> Result<ListRfqsResponse> {
    http.get("/communications/rfqs").await
}

/// List quotes.
pub async fn list_quotes(http: &HttpClient) -> Result<ListQuotesResponse> {
    http.get("/communications/quotes").await
}
