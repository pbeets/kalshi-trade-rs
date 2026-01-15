//! RFQ and Communications models.

use serde::{Deserialize, Serialize};

use super::Side;
use super::market::MveSelectedLeg;

/// Request body for POST /communications/rfqs (create RFQ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRfqRequest {
    /// The ticker of the market for which to create an RFQ.
    pub market_ticker: String,
    /// The number of contracts for the RFQ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<i64>,
    /// The target cost for the RFQ in centi-cents (1/100th of a cent).
    /// Divide by 10,000 to get dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_cost_centi_cents: Option<i64>,
    /// Whether to rest the remainder of the RFQ after execution.
    pub rest_remainder: bool,
    /// Whether to delete existing RFQs as part of this RFQ's creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_existing: Option<bool>,
    /// The subtrader to create the RFQ for (FCM members only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtrader_id: Option<String>,
}

impl CreateRfqRequest {
    /// Create a new RFQ request with contracts, with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `contracts` is not positive.
    pub fn try_with_contracts(
        market_ticker: impl Into<String>,
        contracts: i64,
        rest_remainder: bool,
    ) -> crate::error::Result<Self> {
        if contracts <= 0 {
            return Err(crate::error::Error::InvalidContracts(contracts));
        }
        Ok(Self {
            market_ticker: market_ticker.into(),
            contracts: Some(contracts),
            target_cost_centi_cents: None,
            rest_remainder,
            replace_existing: None,
            subtrader_id: None,
        })
    }

    /// Create a new RFQ request with contracts.
    ///
    /// # Arguments
    ///
    /// * `market_ticker` - The market ticker
    /// * `contracts` - Number of contracts requested
    /// * `rest_remainder` - Whether to rest the remainder after execution
    ///
    /// # Panics
    ///
    /// Panics if `contracts` is not positive.
    #[must_use]
    pub fn with_contracts(
        market_ticker: impl Into<String>,
        contracts: i64,
        rest_remainder: bool,
    ) -> Self {
        Self::try_with_contracts(market_ticker, contracts, rest_remainder)
            .expect("invalid RFQ contracts")
    }

    /// Create a new RFQ request with target cost, with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `target_cost_centi_cents` is not positive.
    pub fn try_with_target_cost(
        market_ticker: impl Into<String>,
        target_cost_centi_cents: i64,
        rest_remainder: bool,
    ) -> crate::error::Result<Self> {
        if target_cost_centi_cents <= 0 {
            return Err(crate::error::Error::InvalidTargetCost(
                target_cost_centi_cents,
            ));
        }
        Ok(Self {
            market_ticker: market_ticker.into(),
            contracts: None,
            target_cost_centi_cents: Some(target_cost_centi_cents),
            rest_remainder,
            replace_existing: None,
            subtrader_id: None,
        })
    }

    /// Create a new RFQ request with target cost.
    ///
    /// # Arguments
    ///
    /// * `market_ticker` - The market ticker
    /// * `target_cost_centi_cents` - Target cost in centi-cents (1/100th of a cent)
    /// * `rest_remainder` - Whether to rest the remainder after execution
    ///
    /// # Panics
    ///
    /// Panics if `target_cost_centi_cents` is not positive.
    #[must_use]
    pub fn with_target_cost(
        market_ticker: impl Into<String>,
        target_cost_centi_cents: i64,
        rest_remainder: bool,
    ) -> Self {
        Self::try_with_target_cost(market_ticker, target_cost_centi_cents, rest_remainder)
            .expect("invalid RFQ target cost")
    }

    /// Create a new RFQ request with target cost in dollars, with validation.
    ///
    /// Converts the dollar amount to centi-cents for the API.
    ///
    /// # Errors
    ///
    /// Returns an error if `target_cost_dollars` is not positive.
    pub fn try_with_target_cost_dollars(
        market_ticker: impl Into<String>,
        target_cost_dollars: f64,
        rest_remainder: bool,
    ) -> crate::error::Result<Self> {
        if target_cost_dollars <= 0.0 {
            return Err(crate::error::Error::InvalidTargetCostDollars(
                target_cost_dollars,
            ));
        }
        let centi_cents = (target_cost_dollars * 10000.0).round() as i64;
        Ok(Self {
            market_ticker: market_ticker.into(),
            contracts: None,
            target_cost_centi_cents: Some(centi_cents),
            rest_remainder,
            replace_existing: None,
            subtrader_id: None,
        })
    }

    /// Create a new RFQ request with target cost in dollars.
    ///
    /// Converts the dollar amount to centi-cents for the API.
    ///
    /// # Arguments
    ///
    /// * `market_ticker` - The market ticker
    /// * `target_cost_dollars` - Target cost in dollars (must be positive)
    /// * `rest_remainder` - Whether to rest the remainder after execution
    ///
    /// # Panics
    ///
    /// Panics if `target_cost_dollars` is not positive.
    #[must_use]
    pub fn with_target_cost_dollars(
        market_ticker: impl Into<String>,
        target_cost_dollars: f64,
        rest_remainder: bool,
    ) -> Self {
        Self::try_with_target_cost_dollars(market_ticker, target_cost_dollars, rest_remainder)
            .expect("invalid RFQ target cost dollars")
    }

    /// Set whether to replace existing RFQs.
    #[must_use]
    pub fn replace_existing(mut self, replace: bool) -> Self {
        self.replace_existing = Some(replace);
        self
    }

    /// Set the subtrader ID (FCM members only).
    #[must_use]
    pub fn subtrader_id(mut self, id: impl Into<String>) -> Self {
        self.subtrader_id = Some(id.into());
        self
    }
}

/// Request body for POST /communications/quotes (create quote).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuoteRequest {
    /// The ID of the RFQ to quote on.
    pub rfq_id: String,
    /// The bid price for YES contracts, in dollars (fixed-point string, e.g. "0.56").
    pub yes_bid: String,
    /// The bid price for NO contracts, in dollars (fixed-point string, e.g. "0.44").
    pub no_bid: String,
    /// Whether to rest the remainder of the quote after execution.
    pub rest_remainder: bool,
}

impl CreateQuoteRequest {
    /// Create a new quote request.
    ///
    /// # Arguments
    ///
    /// * `rfq_id` - The ID of the RFQ to quote on
    /// * `yes_bid` - YES bid price in dollars (e.g. "0.56")
    /// * `no_bid` - NO bid price in dollars (e.g. "0.44")
    /// * `rest_remainder` - Whether to rest the remainder after execution
    #[must_use]
    pub fn new(
        rfq_id: impl Into<String>,
        yes_bid: impl Into<String>,
        no_bid: impl Into<String>,
        rest_remainder: bool,
    ) -> Self {
        Self {
            rfq_id: rfq_id.into(),
            yes_bid: yes_bid.into(),
            no_bid: no_bid.into(),
            rest_remainder,
        }
    }

    /// Create a quote request from cent values with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `yes_price_cents` is not between 1 and 99.
    pub fn try_from_cents(
        rfq_id: impl Into<String>,
        yes_price_cents: i64,
        rest_remainder: bool,
    ) -> crate::error::Result<Self> {
        if !(1..=99).contains(&yes_price_cents) {
            return Err(crate::error::Error::InvalidPrice(yes_price_cents));
        }
        let no_price_cents = 100 - yes_price_cents;
        Ok(Self {
            rfq_id: rfq_id.into(),
            yes_bid: format!("{:.4}", yes_price_cents as f64 / 100.0),
            no_bid: format!("{:.4}", no_price_cents as f64 / 100.0),
            rest_remainder,
        })
    }

    /// Create a quote request from cent values.
    ///
    /// Converts cent prices (1-99) to dollar strings for the API.
    ///
    /// # Arguments
    ///
    /// * `rfq_id` - The ID of the RFQ to quote on
    /// * `yes_price_cents` - YES price in cents (1-99)
    /// * `rest_remainder` - Whether to rest the remainder after execution
    ///
    /// # Panics
    ///
    /// Panics if `yes_price_cents` is not between 1 and 99.
    #[must_use]
    pub fn from_cents(
        rfq_id: impl Into<String>,
        yes_price_cents: i64,
        rest_remainder: bool,
    ) -> Self {
        Self::try_from_cents(rfq_id, yes_price_cents, rest_remainder).expect("invalid quote price")
    }
}

/// Request body for PUT /communications/quotes/{quote_id}/accept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptQuoteRequest {
    /// The side of the quote to accept ("yes" or "no").
    pub accepted_side: String,
}

impl AcceptQuoteRequest {
    /// Create a new accept quote request.
    ///
    /// # Arguments
    ///
    /// * `side` - The side to accept (use [`Side::Yes`] or [`Side::No`])
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            accepted_side: match side {
                Side::Yes => "yes".to_string(),
                Side::No => "no".to_string(),
            },
        }
    }

    /// Accept the YES side of the quote.
    #[must_use]
    pub fn yes() -> Self {
        Self {
            accepted_side: "yes".to_string(),
        }
    }

    /// Accept the NO side of the quote.
    #[must_use]
    pub fn no() -> Self {
        Self {
            accepted_side: "no".to_string(),
        }
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
        qb.push_opt(
            "rfq_creator_subtrader_id",
            self.rfq_creator_subtrader_id.as_ref(),
        );
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
    /// Unique identifier for the RFQ.
    pub id: String,
    /// Public communications ID of the RFQ creator.
    pub creator_id: String,
    /// The ticker of the market this RFQ is for.
    pub market_ticker: String,
    /// Number of contracts requested in the RFQ.
    #[serde(default)]
    pub contracts: Option<i64>,
    /// Total value of the RFQ in centi-cents (1/100th of a cent).
    /// Divide by 10,000 to get dollars.
    #[serde(default)]
    pub target_cost_centi_cents: Option<i64>,
    /// Current status of the RFQ (open, closed).
    #[serde(default)]
    pub status: Option<String>,
    /// Timestamp when the RFQ was created.
    pub created_ts: String,
    /// Timestamp when the RFQ was last updated.
    #[serde(default)]
    pub updated_ts: Option<String>,
    /// Timestamp when the RFQ was cancelled.
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    /// Ticker of the MVE collection this market belongs to.
    #[serde(default)]
    pub mve_collection_ticker: Option<String>,
    /// Selected legs for the MVE collection.
    #[serde(default)]
    pub mve_selected_legs: Option<Vec<MveSelectedLeg>>,
    /// Whether to rest the remainder of the RFQ after execution.
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    /// Reason for RFQ cancellation if cancelled.
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    /// User ID of the RFQ creator (private field).
    #[serde(default)]
    pub creator_user_id: Option<String>,
}

impl Rfq {
    /// Returns the target cost in dollars, if available.
    #[must_use]
    pub fn target_cost_dollars(&self) -> Option<f64> {
        self.target_cost_centi_cents.map(|cc| cc as f64 / 10000.0)
    }
}

/// Quote details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Unique identifier for the quote.
    pub id: String,
    /// ID of the RFQ this quote is responding to.
    pub rfq_id: String,
    /// Public communications ID of the quote creator.
    pub creator_id: String,
    /// Public communications ID of the RFQ creator.
    pub rfq_creator_id: String,
    /// The ticker of the market this quote is for.
    pub market_ticker: String,
    /// Number of contracts in the quote.
    #[serde(default)]
    pub contracts: Option<i64>,
    /// Bid price for YES contracts, in cents.
    #[serde(default)]
    pub yes_bid: Option<i64>,
    /// Bid price for NO contracts, in cents.
    #[serde(default)]
    pub no_bid: Option<i64>,
    /// YES bid price in dollars (fixed-point string with 4 decimal places).
    #[serde(default)]
    pub yes_bid_dollars: Option<String>,
    /// NO bid price in dollars (fixed-point string with 4 decimal places).
    #[serde(default)]
    pub no_bid_dollars: Option<String>,
    /// Timestamp when the quote was created.
    pub created_ts: String,
    /// Timestamp when the quote was last updated.
    #[serde(default)]
    pub updated_ts: Option<String>,
    /// Current status of the quote.
    #[serde(default)]
    pub status: Option<String>,
    /// The side that was accepted (yes or no).
    #[serde(default)]
    pub accepted_side: Option<String>,
    /// Timestamp when the quote was accepted.
    #[serde(default)]
    pub accepted_ts: Option<String>,
    /// Timestamp when the quote was confirmed.
    #[serde(default)]
    pub confirmed_ts: Option<String>,
    /// Timestamp when the quote was executed.
    #[serde(default)]
    pub executed_ts: Option<String>,
    /// Timestamp when the quote was cancelled.
    #[serde(default)]
    pub cancelled_ts: Option<String>,
    /// Whether to rest remainder after execution.
    #[serde(default)]
    pub rest_remainder: Option<bool>,
    /// Reason for cancellation if applicable.
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    /// User ID of quote creator (private field).
    #[serde(default)]
    pub creator_user_id: Option<String>,
    /// User ID of RFQ creator (private field).
    #[serde(default)]
    pub rfq_creator_user_id: Option<String>,
    /// Total value requested in RFQ in centi-cents (1/100th of a cent).
    /// Divide by 10,000 to get dollars.
    #[serde(default)]
    pub rfq_target_cost_centi_cents: Option<i64>,
    /// Order ID for RFQ creator (private field).
    #[serde(default)]
    pub rfq_creator_order_id: Option<String>,
    /// Order ID for quote creator (private field).
    #[serde(default)]
    pub creator_order_id: Option<String>,
}

impl Quote {
    /// Returns the RFQ target cost in dollars, if available.
    #[must_use]
    pub fn rfq_target_cost_dollars(&self) -> Option<f64> {
        self.rfq_target_cost_centi_cents
            .map(|cc| cc as f64 / 10000.0)
    }
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
    fn test_create_rfq_with_contracts() {
        let rfq = CreateRfqRequest::with_contracts("TICKER", 10, true);
        assert_eq!(rfq.market_ticker, "TICKER");
        assert_eq!(rfq.contracts, Some(10));
        assert!(rfq.target_cost_centi_cents.is_none());
        assert!(rfq.rest_remainder);
    }

    #[test]
    fn test_create_rfq_with_target_cost() {
        let rfq = CreateRfqRequest::with_target_cost("TICKER", 500000, false);
        assert_eq!(rfq.market_ticker, "TICKER");
        assert!(rfq.contracts.is_none());
        assert_eq!(rfq.target_cost_centi_cents, Some(500000));
        assert!(!rfq.rest_remainder);
    }

    #[test]
    fn test_create_rfq_with_target_cost_dollars() {
        let rfq = CreateRfqRequest::with_target_cost_dollars("TICKER", 50.0, true);
        assert_eq!(rfq.market_ticker, "TICKER");
        assert_eq!(rfq.target_cost_centi_cents, Some(500000)); // $50 = 500000 centi-cents
    }

    #[test]
    fn test_create_quote_new() {
        let quote = CreateQuoteRequest::new("RFQ123", "0.5600", "0.4400", true);
        assert_eq!(quote.rfq_id, "RFQ123");
        assert_eq!(quote.yes_bid, "0.5600");
        assert_eq!(quote.no_bid, "0.4400");
        assert!(quote.rest_remainder);
    }

    #[test]
    fn test_create_quote_from_cents() {
        let quote = CreateQuoteRequest::from_cents("RFQ123", 56, false);
        assert_eq!(quote.rfq_id, "RFQ123");
        assert_eq!(quote.yes_bid, "0.5600");
        assert_eq!(quote.no_bid, "0.4400");
        assert!(!quote.rest_remainder);
    }

    #[test]
    fn test_create_quote_from_cents_boundary_values() {
        // Test minimum valid value (1 cent)
        let quote = CreateQuoteRequest::from_cents("RFQ123", 1, false);
        assert_eq!(quote.yes_bid, "0.0100");
        assert_eq!(quote.no_bid, "0.9900");

        // Test maximum valid value (99 cents)
        let quote = CreateQuoteRequest::from_cents("RFQ123", 99, false);
        assert_eq!(quote.yes_bid, "0.9900");
        assert_eq!(quote.no_bid, "0.0100");
    }

    #[test]
    fn test_accept_quote() {
        let accept = AcceptQuoteRequest::new(Side::Yes);
        assert_eq!(accept.accepted_side, "yes");

        let accept = AcceptQuoteRequest::new(Side::No);
        assert_eq!(accept.accepted_side, "no");

        let accept = AcceptQuoteRequest::yes();
        assert_eq!(accept.accepted_side, "yes");

        let accept = AcceptQuoteRequest::no();
        assert_eq!(accept.accepted_side, "no");
    }

    #[test]
    fn test_rfq_target_cost_dollars() {
        let rfq = Rfq {
            id: "rfq123".to_string(),
            creator_id: "creator123".to_string(),
            market_ticker: "TICKER".to_string(),
            contracts: None,
            target_cost_centi_cents: Some(500000), // $50
            status: Some("open".to_string()),
            created_ts: "2024-01-01T00:00:00Z".to_string(),
            updated_ts: None,
            cancelled_ts: None,
            mve_collection_ticker: None,
            mve_selected_legs: None,
            rest_remainder: None,
            cancellation_reason: None,
            creator_user_id: None,
        };
        assert!((rfq.target_cost_dollars().unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quote_rfq_target_cost_dollars() {
        let quote = Quote {
            id: "quote123".to_string(),
            rfq_id: "rfq123".to_string(),
            creator_id: "creator123".to_string(),
            rfq_creator_id: "rfq_creator123".to_string(),
            market_ticker: "TICKER".to_string(),
            contracts: Some(10),
            yes_bid: Some(56),
            no_bid: Some(44),
            yes_bid_dollars: Some("0.5600".to_string()),
            no_bid_dollars: Some("0.4400".to_string()),
            created_ts: "2024-01-01T00:00:00Z".to_string(),
            updated_ts: None,
            status: Some("open".to_string()),
            accepted_side: None,
            accepted_ts: None,
            confirmed_ts: None,
            executed_ts: None,
            cancelled_ts: None,
            rest_remainder: None,
            cancellation_reason: None,
            creator_user_id: None,
            rfq_creator_user_id: None,
            rfq_target_cost_centi_cents: Some(500000), // $50
            rfq_creator_order_id: None,
            creator_order_id: None,
        };
        assert!((quote.rfq_target_cost_dollars().unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_communications_id_response_deserialize() {
        let json = r#"{"communications_id": "comms-abc123"}"#;
        let response: CommunicationsIdResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.communications_id, "comms-abc123");
    }
}
