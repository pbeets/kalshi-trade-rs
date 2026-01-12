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

impl CreateRfqRequest {
    /// Create a new RFQ request.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `size` is not positive. Use [`try_new`](Self::try_new)
    /// for fallible construction.
    #[must_use]
    pub fn new(ticker: impl Into<String>, size: i64, side: Side) -> Self {
        debug_assert!(size > 0, "size must be positive, got {}", size);
        Self {
            ticker: ticker.into(),
            size,
            side,
        }
    }

    /// Create a new RFQ request with validation.
    pub fn try_new(ticker: impl Into<String>, size: i64, side: Side) -> crate::error::Result<Self> {
        if size <= 0 {
            return Err(crate::error::Error::InvalidQuantity(size));
        }
        Ok(Self {
            ticker: ticker.into(),
            size,
            side,
        })
    }
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

impl CreateQuoteRequest {
    /// Create a new quote request.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `yes_price` is not between 1 and 99, or if `count`
    /// is not positive. Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(
        rfq_id: impl Into<String>,
        ticker: impl Into<String>,
        yes_price: i64,
        count: i64,
    ) -> Self {
        debug_assert!(
            (1..=99).contains(&yes_price),
            "yes_price must be between 1 and 99, got {}",
            yes_price
        );
        debug_assert!(count > 0, "count must be positive, got {}", count);
        Self {
            rfq_id: rfq_id.into(),
            ticker: ticker.into(),
            yes_price,
            count,
        }
    }

    /// Create a new quote request with validation.
    pub fn try_new(
        rfq_id: impl Into<String>,
        ticker: impl Into<String>,
        yes_price: i64,
        count: i64,
    ) -> crate::error::Result<Self> {
        if !(1..=99).contains(&yes_price) {
            return Err(crate::error::Error::InvalidPrice(yes_price));
        }
        if count <= 0 {
            return Err(crate::error::Error::InvalidQuantity(count));
        }
        Ok(Self {
            rfq_id: rfq_id.into(),
            ticker: ticker.into(),
            yes_price,
            count,
        })
    }
}

/// Request body for POST /communications/quotes/{quote_id}/accept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptQuoteRequest {
    /// Accepted yes price in cents.
    pub yes_price: i64,
    /// Quantity to accept.
    pub count: i64,
}

impl AcceptQuoteRequest {
    /// Create a new accept quote request.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `yes_price` is not between 1 and 99, or if `count`
    /// is not positive. Use [`try_new`](Self::try_new) for fallible construction.
    #[must_use]
    pub fn new(yes_price: i64, count: i64) -> Self {
        debug_assert!(
            (1..=99).contains(&yes_price),
            "yes_price must be between 1 and 99, got {}",
            yes_price
        );
        debug_assert!(count > 0, "count must be positive, got {}", count);
        Self { yes_price, count }
    }

    /// Create a new accept quote request with validation.
    pub fn try_new(yes_price: i64, count: i64) -> crate::error::Result<Self> {
        if !(1..=99).contains(&yes_price) {
            return Err(crate::error::Error::InvalidPrice(yes_price));
        }
        if count <= 0 {
            return Err(crate::error::Error::InvalidQuantity(count));
        }
        Ok(Self { yes_price, count })
    }
}

/// Query parameters for GET /communications/rfqs.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ListRfqsParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by event ticker (comma-separated, max 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter by market ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    /// Number of results per page (1-100, default 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Filter by RFQ status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by creator user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_user_id: Option<String>,
}

impl ListRfqsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn market_ticker(mut self, market_ticker: impl Into<String>) -> Self {
        self.market_ticker = Some(market_ticker.into());
        self
    }

    /// Set the number of results per page (clamped to 1-100).
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit.clamp(1, 100));
        self
    }

    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    #[must_use]
    pub fn creator_user_id(mut self, creator_user_id: impl Into<String>) -> Self {
        self.creator_user_id = Some(creator_user_id.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        use crate::models::query::QueryBuilder;
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("market_ticker", self.market_ticker.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("status", self.status.as_ref());
        qb.push_opt("creator_user_id", self.creator_user_id.as_ref());
        qb.build()
    }
}

/// Query parameters for GET /communications/quotes.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ListQuotesParams {
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by event ticker (comma-separated, max 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Filter by market ticker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    /// Number of results per page (1-500, default 500).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Filter by quote status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by quote creator user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_creator_user_id: Option<String>,
    /// Filter by RFQ creator user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_user_id: Option<String>,
    /// Filter by RFQ creator subtrader ID (FCM members only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_creator_subtrader_id: Option<String>,
    /// Filter by RFQ ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfq_id: Option<String>,
}

impl ListQuotesParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn market_ticker(mut self, market_ticker: impl Into<String>) -> Self {
        self.market_ticker = Some(market_ticker.into());
        self
    }

    /// Set the number of results per page (clamped to 1-500).
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit.clamp(1, 500));
        self
    }

    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    #[must_use]
    pub fn quote_creator_user_id(mut self, id: impl Into<String>) -> Self {
        self.quote_creator_user_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn rfq_creator_user_id(mut self, id: impl Into<String>) -> Self {
        self.rfq_creator_user_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn rfq_creator_subtrader_id(mut self, id: impl Into<String>) -> Self {
        self.rfq_creator_subtrader_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn rfq_id(mut self, rfq_id: impl Into<String>) -> Self {
        self.rfq_id = Some(rfq_id.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        use crate::models::query::QueryBuilder;
        let mut qb = QueryBuilder::new();
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("market_ticker", self.market_ticker.as_ref());
        qb.push_opt("limit", self.limit);
        qb.push_opt("status", self.status.as_ref());
        qb.push_opt("quote_creator_user_id", self.quote_creator_user_id.as_ref());
        qb.push_opt("rfq_creator_user_id", self.rfq_creator_user_id.as_ref());
        qb.push_opt("rfq_creator_subtrader_id", self.rfq_creator_subtrader_id.as_ref());
        qb.push_opt("rfq_id", self.rfq_id.as_ref());
        qb.build()
    }
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

/// Response from GET /communications/id.
///
/// Returns the user's public communications ID for RFQ/quote functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationsIdResponse {
    /// The user's public communications ID used to identify them in RFQ/quote interactions.
    pub communications_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rfq_validation() {
        assert!(CreateRfqRequest::try_new("TICKER", 10, Side::Yes).is_ok());
        assert!(matches!(
            CreateRfqRequest::try_new("TICKER", 0, Side::Yes),
            Err(crate::error::Error::InvalidQuantity(0))
        ));
        assert!(matches!(
            CreateRfqRequest::try_new("TICKER", -5, Side::Yes),
            Err(crate::error::Error::InvalidQuantity(-5))
        ));
    }

    #[test]
    fn test_create_quote_validation() {
        assert!(CreateQuoteRequest::try_new("RFQ123", "TICKER", 50, 10).is_ok());

        // Invalid price
        assert!(matches!(
            CreateQuoteRequest::try_new("RFQ123", "TICKER", 0, 10),
            Err(crate::error::Error::InvalidPrice(0))
        ));
        assert!(matches!(
            CreateQuoteRequest::try_new("RFQ123", "TICKER", 100, 10),
            Err(crate::error::Error::InvalidPrice(100))
        ));

        // Invalid quantity
        assert!(matches!(
            CreateQuoteRequest::try_new("RFQ123", "TICKER", 50, 0),
            Err(crate::error::Error::InvalidQuantity(0))
        ));
    }

    #[test]
    fn test_accept_quote_validation() {
        assert!(AcceptQuoteRequest::try_new(50, 10).is_ok());

        // Invalid price
        assert!(matches!(
            AcceptQuoteRequest::try_new(0, 10),
            Err(crate::error::Error::InvalidPrice(0))
        ));
        assert!(matches!(
            AcceptQuoteRequest::try_new(100, 10),
            Err(crate::error::Error::InvalidPrice(100))
        ));

        // Invalid quantity
        assert!(matches!(
            AcceptQuoteRequest::try_new(50, 0),
            Err(crate::error::Error::InvalidQuantity(0))
        ));
    }

    #[test]
    fn test_communications_id_response_deserialize() {
        let json = r#"{"communications_id": "comms-abc123"}"#;
        let response: CommunicationsIdResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.communications_id, "comms-abc123");
    }
}
