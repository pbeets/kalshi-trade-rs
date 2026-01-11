//! RFQ and Communications models.

use serde::{Deserialize, Serialize};

use super::Side;

/// Request body for POST /communications/rfqs (create RFQ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRfqRequest {
    /// Market ticker.
    pub ticker: String,
    /// Number of contracts requested.
    pub size: i64,
    /// Side of the order (yes or no).
    pub side: Side,
}

/// Request body for POST /communications/quotes (create quote).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuoteRequest {
    /// Associated RFQ ID.
    pub rfq_id: String,
    /// Market ticker.
    pub ticker: String,
    /// Yes price in cents.
    pub yes_price: i64,
    /// Quantity offered.
    pub count: i64,
}

/// Request body for POST /communications/quotes/{quote_id}/accept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptQuoteRequest {
    /// Accepted yes price in cents.
    pub yes_price: i64,
    /// Quantity to accept.
    pub count: i64,
}

/// Response from create RFQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfqResponse {
    pub rfq_id: String,
}

/// Response from create quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub quote_id: String,
}

/// RFQ details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rfq {
    pub id: String,
    pub creator_id: String,
    pub market_ticker: String,
    pub created_ts: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub contracts: Option<i64>,
    #[serde(default)]
    pub target_cost: Option<i64>,
    #[serde(default)]
    pub target_cost_dollars: Option<String>,
}

/// Quote details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub quote_id: String,
    pub rfq_id: String,
    pub quote_creator_id: String,
    pub rfq_creator_id: String,
    pub market_ticker: String,
    pub yes_bid: i64,
    pub no_bid: i64,
    pub yes_bid_dollars: String,
    pub no_bid_dollars: String,
    pub created_ts: String,
    #[serde(default)]
    pub event_ticker: Option<String>,
    #[serde(default)]
    pub yes_contracts_offered: Option<i64>,
    #[serde(default)]
    pub no_contracts_offered: Option<i64>,
    #[serde(default)]
    pub rfq_target_cost: Option<i64>,
    #[serde(default)]
    pub rfq_target_cost_dollars: Option<String>,
}

/// Response for getting a single RFQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRfqResponse {
    pub rfq: Rfq,
}

/// Response for getting a single quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetQuoteResponse {
    pub quote: Quote,
}

/// Response for listing RFQs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRfqsResponse {
    pub rfqs: Vec<Rfq>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response for listing quotes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuotesResponse {
    pub quotes: Vec<Quote>,
    #[serde(default)]
    pub cursor: Option<String>,
}
