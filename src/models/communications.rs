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
}
