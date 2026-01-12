//! Communications API endpoints (RFQs and Quotes).
//!
//! This module provides functions for interacting with the Kalshi Communications API.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        AcceptQuoteRequest, CommunicationsIdResponse, CreateQuoteRequest, CreateRfqRequest,
        GetQuoteResponse, GetRfqResponse, ListQuotesParams, ListQuotesResponse, ListRfqsParams,
        ListRfqsResponse, QuoteResponse, RfqResponse,
    },
};

/// Create a new RFQ (Request for Quote).
///
/// Submits an RFQ to the exchange.
pub async fn create_rfq(http: &HttpClient, request: CreateRfqRequest) -> Result<RfqResponse> {
    http.post("/communications/rfqs", &request).await
}

/// Create a new quote for an RFQ.
///
/// Submits a quote in response to an RFQ.
pub async fn create_quote(http: &HttpClient, request: CreateQuoteRequest) -> Result<QuoteResponse> {
    http.post("/communications/quotes", &request).await
}

/// Accept a quote.
///
/// Accepts a specific quote for an RFQ.
pub async fn accept_quote(
    http: &HttpClient,
    quote_id: &str,
    request: AcceptQuoteRequest,
) -> Result<QuoteResponse> {
    let path = format!("/communications/quotes/{}/accept", quote_id);
    http.post(&path, &request).await
}

/// Cancel an RFQ.
///
/// Cancels an existing RFQ.
pub async fn cancel_rfq(http: &HttpClient, rfq_id: &str) -> Result<RfqResponse> {
    let path = format!("/communications/rfqs/{}", rfq_id);
    http.delete_with_response(&path).await
}

/// Cancel a quote.
///
/// Cancels an existing quote.
pub async fn cancel_quote(http: &HttpClient, quote_id: &str) -> Result<QuoteResponse> {
    let path = format!("/communications/quotes/{}", quote_id);
    http.delete_with_response(&path).await
}

/// Get details of an RFQ.
///
/// Returns information about a specific RFQ.
pub async fn get_rfq(http: &HttpClient, rfq_id: &str) -> Result<GetRfqResponse> {
    let path = format!("/communications/rfqs/{}", rfq_id);
    http.get(&path).await
}

/// Get details of a quote.
///
/// Returns information about a specific quote.
pub async fn get_quote(http: &HttpClient, quote_id: &str) -> Result<GetQuoteResponse> {
    let path = format!("/communications/quotes/{}", quote_id);
    http.get(&path).await
}

/// List RFQs with filtering and pagination.
///
/// Returns a list of RFQs matching the provided parameters.
pub async fn list_rfqs(http: &HttpClient, params: ListRfqsParams) -> Result<ListRfqsResponse> {
    let path = format!("/communications/rfqs{}", params.to_query_string());
    http.get(&path).await
}

/// List quotes with filtering and pagination.
///
/// Returns a list of quotes matching the provided parameters.
pub async fn list_quotes(http: &HttpClient, params: ListQuotesParams) -> Result<ListQuotesResponse> {
    let path = format!("/communications/quotes{}", params.to_query_string());
    http.get(&path).await
}

/// Get the communications ID of the logged-in user.
///
/// Returns a public communications ID which is used to identify
/// the user in RFQ/quote interactions.
pub async fn get_communications_id(http: &HttpClient) -> Result<CommunicationsIdResponse> {
    http.get("/communications/id").await
}

/// Confirm a quote.
///
/// Confirms a quote and starts a timer for order execution.
/// Returns 204 No Content on success.
pub async fn confirm_quote(http: &HttpClient, quote_id: &str) -> Result<()> {
    let path = format!("/communications/quotes/{}/confirm", quote_id);
    http.put_no_content(&path).await
}
