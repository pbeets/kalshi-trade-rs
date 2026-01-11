//! Order models and query parameters.

use serde::{Deserialize, Serialize};

use super::common::{Action, OrderStatus, OrderType, SelfTradePreventionType, Side};
use super::query::QueryBuilder;

/// Time in force for an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    /// Fill or kill - entire order must fill immediately or cancel.
    FillOrKill,
    /// Good till canceled - order remains until filled or explicitly canceled.
    GoodTillCanceled,
    /// Immediate or cancel - fill what's possible immediately, cancel the rest.
    ImmediateOrCancel,
}

/// An order in the Kalshi exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub client_order_id: Option<String>,
    pub ticker: String,
    pub side: Side,
    pub action: Action,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    /// Price in cents.
    pub yes_price: i64,
    /// Price in cents.
    pub no_price: i64,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub yes_price_dollars: Option<String>,
    /// Price in fixed-point dollars.
    #[serde(default)]
    pub no_price_dollars: Option<String>,
    pub fill_count: i64,
    pub remaining_count: i64,
    pub initial_count: i64,
    /// Fees in cents.
    #[serde(default)]
    pub taker_fees: Option<i64>,
    /// Fees in cents.
    #[serde(default)]
    pub maker_fees: Option<i64>,
    /// Cost in cents.
    #[serde(default)]
    pub taker_fill_cost: Option<i64>,
    /// Cost in cents.
    #[serde(default)]
    pub maker_fill_cost: Option<i64>,
    #[serde(default)]
    pub taker_fill_cost_dollars: Option<String>,
    #[serde(default)]
    pub maker_fill_cost_dollars: Option<String>,
    #[serde(default)]
    pub taker_fees_dollars: Option<String>,
    #[serde(default)]
    pub maker_fees_dollars: Option<String>,
    /// Deprecated.
    #[serde(default)]
    pub queue_position: Option<i64>,
    #[serde(default)]
    pub expiration_time: Option<String>,
    #[serde(default)]
    pub created_time: Option<String>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,
    #[serde(default)]
    pub order_group_id: Option<String>,
    #[serde(default)]
    pub cancel_order_on_pause: Option<bool>,
}

/// Response from the get_orders endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query parameters for the get_orders endpoint.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetOrdersParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OrderStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl GetOrdersParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn event_ticker(mut self, event_ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(event_ticker.into());
        self
    }

    #[must_use]
    pub fn min_ts(mut self, ts: i64) -> Self {
        self.min_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn max_ts(mut self, ts: i64) -> Self {
        self.max_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn status(mut self, status: OrderStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the maximum number of results to return.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `limit` is not in the range 1..=1000.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        debug_assert!(
            limit > 0 && limit <= 1000,
            "limit must be between 1 and 1000, got {}",
            limit
        );
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("ticker", self.ticker.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("min_ts", self.min_ts);
        qb.push_opt("max_ts", self.max_ts);
        qb.push_opt("status", self.status.map(|s| s.as_str()));
        qb.push_opt("limit", self.limit);
        qb.push_opt("cursor", self.cursor.as_ref());
        qb.build()
    }
}

/// Response from GET /portfolio/orders/{order_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order: Order,
}

/// Request body for POST /portfolio/orders (create order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    /// Market ticker.
    pub ticker: String,

    /// Side of the order (yes or no).
    pub side: Side,

    /// Action (buy or sell).
    pub action: Action,

    /// Number of contracts. Must be >= 1.
    pub count: i64,

    /// Client-assigned order identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Order type (limit or market).
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,

    /// Yes price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,

    /// No price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,

    /// Yes price in fixed-point dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,

    /// No price in fixed-point dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,

    /// Order expiration timestamp (Unix seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,

    /// Time in force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,

    /// Maximum cost in cents. Enables fill-or-kill behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i64>,

    /// Post-only flag (maker only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,

    /// Reduce-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,

    /// Self-trade prevention type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_type: Option<SelfTradePreventionType>,

    /// Associated order group ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_group_id: Option<String>,

    /// Auto-cancel if exchange trading pauses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,
}

impl CreateOrderRequest {
    /// Create a new order request with required fields.
    #[must_use]
    pub fn new(ticker: impl Into<String>, side: Side, action: Action, count: i64) -> Self {
        Self {
            ticker: ticker.into(),
            side,
            action,
            count,
            client_order_id: None,
            order_type: None,
            yes_price: None,
            no_price: None,
            yes_price_dollars: None,
            no_price_dollars: None,
            expiration_ts: None,
            time_in_force: None,
            buy_max_cost: None,
            post_only: None,
            reduce_only: None,
            self_trade_prevention_type: None,
            order_group_id: None,
            cancel_order_on_pause: None,
        }
    }

    #[must_use]
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    /// Set yes price in cents (1-99).
    #[must_use]
    pub fn yes_price(mut self, price: i64) -> Self {
        debug_assert!(
            (1..=99).contains(&price),
            "yes_price must be between 1 and 99, got {}",
            price
        );
        self.yes_price = Some(price);
        self
    }

    /// Set no price in cents (1-99).
    #[must_use]
    pub fn no_price(mut self, price: i64) -> Self {
        debug_assert!(
            (1..=99).contains(&price),
            "no_price must be between 1 and 99, got {}",
            price
        );
        self.no_price = Some(price);
        self
    }

    /// Set yes price in fixed-point dollars (e.g., "0.56").
    #[must_use]
    pub fn yes_price_dollars(mut self, price: impl Into<String>) -> Self {
        self.yes_price_dollars = Some(price.into());
        self
    }

    /// Set no price in fixed-point dollars (e.g., "0.56").
    #[must_use]
    pub fn no_price_dollars(mut self, price: impl Into<String>) -> Self {
        self.no_price_dollars = Some(price.into());
        self
    }

    #[must_use]
    pub fn expiration_ts(mut self, ts: i64) -> Self {
        self.expiration_ts = Some(ts);
        self
    }

    #[must_use]
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    #[must_use]
    pub fn buy_max_cost(mut self, cost: i64) -> Self {
        self.buy_max_cost = Some(cost);
        self
    }

    #[must_use]
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }

    #[must_use]
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }

    #[must_use]
    pub fn self_trade_prevention_type(mut self, stp: SelfTradePreventionType) -> Self {
        self.self_trade_prevention_type = Some(stp);
        self
    }

    #[must_use]
    pub fn order_group_id(mut self, id: impl Into<String>) -> Self {
        self.order_group_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn cancel_order_on_pause(mut self, cancel: bool) -> Self {
        self.cancel_order_on_pause = Some(cancel);
        self
    }
}

/// Response from DELETE /portfolio/orders/{order_id} (cancel order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub order: Order,
    /// Number of contracts that were canceled.
    #[serde(default)]
    pub reduced_by: Option<i64>,
}

/// Request body for POST /portfolio/orders/{order_id}/amend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendOrderRequest {
    /// Market ticker.
    pub ticker: String,

    /// Side of the order.
    pub side: Side,

    /// Action of the order.
    pub action: Action,

    /// Original client-specified order ID to be amended.
    pub client_order_id: String,

    /// New client-specified order ID after amendment.
    pub updated_client_order_id: String,

    /// Updated yes price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i64>,

    /// Updated no price in cents (1-99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i64>,

    /// Updated yes price in fixed-point dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price_dollars: Option<String>,

    /// Updated no price in fixed-point dollars.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price_dollars: Option<String>,

    /// Updated quantity for the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

impl AmendOrderRequest {
    /// Create a new amend order request.
    #[must_use]
    pub fn new(
        ticker: impl Into<String>,
        side: Side,
        action: Action,
        client_order_id: impl Into<String>,
        updated_client_order_id: impl Into<String>,
    ) -> Self {
        Self {
            ticker: ticker.into(),
            side,
            action,
            client_order_id: client_order_id.into(),
            updated_client_order_id: updated_client_order_id.into(),
            yes_price: None,
            no_price: None,
            yes_price_dollars: None,
            no_price_dollars: None,
            count: None,
        }
    }

    #[must_use]
    pub fn yes_price(mut self, price: i64) -> Self {
        self.yes_price = Some(price);
        self
    }

    #[must_use]
    pub fn no_price(mut self, price: i64) -> Self {
        self.no_price = Some(price);
        self
    }

    #[must_use]
    pub fn yes_price_dollars(mut self, price: impl Into<String>) -> Self {
        self.yes_price_dollars = Some(price.into());
        self
    }

    #[must_use]
    pub fn no_price_dollars(mut self, price: impl Into<String>) -> Self {
        self.no_price_dollars = Some(price.into());
        self
    }

    #[must_use]
    pub fn count(mut self, count: i64) -> Self {
        self.count = Some(count);
        self
    }
}

/// Response from POST /portfolio/orders/{order_id}/amend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendOrderResponse {
    /// Order state before amendment.
    pub old_order: Order,
    /// Order state after amendment.
    pub order: Order,
}

/// Request body for POST /portfolio/orders/{order_id}/decrease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreaseOrderRequest {
    /// Amount to decrease order by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by: Option<i64>,

    /// Target remaining quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_to: Option<i64>,
}

impl DecreaseOrderRequest {
    /// Create a request to reduce by a specific amount.
    #[must_use]
    pub fn reduce_by(amount: i64) -> Self {
        Self {
            reduce_by: Some(amount),
            reduce_to: None,
        }
    }

    /// Create a request to reduce to a target quantity.
    #[must_use]
    pub fn reduce_to(target: i64) -> Self {
        Self {
            reduce_by: None,
            reduce_to: Some(target),
        }
    }
}

/// Request body for POST /portfolio/orders/batched (batch create).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateOrdersRequest {
    /// Array of orders to create (max 20).
    pub orders: Vec<CreateOrderRequest>,
}

impl BatchCreateOrdersRequest {
    /// Create a new batch create request.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `orders.len() > 20`. In release builds,
    /// oversized batches will be rejected by the API. Use [`try_new`](Self::try_new)
    /// for fallible construction.
    #[must_use]
    pub fn new(orders: Vec<CreateOrderRequest>) -> Self {
        debug_assert!(
            orders.len() <= crate::error::MAX_BATCH_SIZE,
            "batch create supports max {} orders, got {}",
            crate::error::MAX_BATCH_SIZE,
            orders.len()
        );
        Self { orders }
    }

    /// Create a new batch create request with validation.
    ///
    /// Returns an error if the batch exceeds the maximum size of 20 orders.
    pub fn try_new(orders: Vec<CreateOrderRequest>) -> crate::error::Result<Self> {
        if orders.len() > crate::error::MAX_BATCH_SIZE {
            return Err(crate::error::Error::BatchSizeExceeded(orders.len()));
        }
        Ok(Self { orders })
    }
}

impl TryFrom<Vec<CreateOrderRequest>> for BatchCreateOrdersRequest {
    type Error = crate::error::Error;

    fn try_from(orders: Vec<CreateOrderRequest>) -> crate::error::Result<Self> {
        Self::try_new(orders)
    }
}

/// Result for a single order in a batch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOrderResult {
    /// Echo of submitted client_order_id.
    #[serde(default)]
    pub client_order_id: Option<String>,

    /// Confirmed order details (present on success).
    #[serde(default)]
    pub order: Option<Order>,

    /// Error details (present on failure).
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

/// Error details for a failed batch order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOrderError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
}

/// Response from POST /portfolio/orders/batched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateOrdersResponse {
    pub orders: Vec<BatchOrderResult>,
}

/// Request body for DELETE /portfolio/orders/batched (batch cancel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrdersRequest {
    /// Array of order IDs to cancel.
    pub ids: Vec<String>,
}

impl BatchCancelOrdersRequest {
    /// Create a new batch cancel request.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `ids.len() > 20`. In release builds,
    /// oversized batches will be rejected by the API. Use [`try_new`](Self::try_new)
    /// for fallible construction.
    #[must_use]
    pub fn new(ids: Vec<String>) -> Self {
        debug_assert!(
            ids.len() <= crate::error::MAX_BATCH_SIZE,
            "batch cancel supports max {} orders, got {}",
            crate::error::MAX_BATCH_SIZE,
            ids.len()
        );
        Self { ids }
    }

    /// Create a new batch cancel request with validation.
    ///
    /// Returns an error if the batch exceeds the maximum size of 20 orders.
    pub fn try_new(ids: Vec<String>) -> crate::error::Result<Self> {
        if ids.len() > crate::error::MAX_BATCH_SIZE {
            return Err(crate::error::Error::BatchSizeExceeded(ids.len()));
        }
        Ok(Self { ids })
    }
}

impl TryFrom<Vec<String>> for BatchCancelOrdersRequest {
    type Error = crate::error::Error;

    fn try_from(ids: Vec<String>) -> crate::error::Result<Self> {
        Self::try_new(ids)
    }
}

/// Result for a single order in a batch cancel operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrderResult {
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,

    /// Number of contracts canceled.
    #[serde(default)]
    pub reduced_by: Option<i64>,

    /// Order details after cancellation.
    #[serde(default)]
    pub order: Option<Order>,

    /// Error details (present on failure).
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

/// Response from DELETE /portfolio/orders/batched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrdersResponse {
    pub orders: Vec<BatchCancelOrderResult>,
}

/// Queue position for a single order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePosition {
    /// Order ID.
    pub order_id: String,
    /// Market ticker.
    pub market_ticker: String,
    /// Queue position - number of contracts ahead in the queue.
    pub queue_position: i64,
}

/// Response from GET /portfolio/orders/queue_positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePositionsResponse {
    pub queue_positions: Vec<QueuePosition>,
}

/// Response from GET /portfolio/orders/{order_id}/queue_position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderQueuePositionResponse {
    /// Queue position - number of contracts ahead in the queue.
    pub queue_position: i64,
}

/// Query parameters for GET /portfolio/orders/queue_positions.
#[derive(Debug, Default, Clone, Serialize)]
pub struct GetQueuePositionsParams {
    /// Comma-separated list of market tickers to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<String>,
    /// Event ticker to filter by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
}

impl GetQueuePositionsParams {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by market tickers (comma-separated).
    #[must_use]
    pub fn market_tickers(mut self, tickers: impl Into<String>) -> Self {
        self.market_tickers = Some(tickers.into());
        self
    }

    /// Filter by event ticker.
    #[must_use]
    pub fn event_ticker(mut self, ticker: impl Into<String>) -> Self {
        self.event_ticker = Some(ticker.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("market_tickers", self.market_tickers.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_with_status() {
        let params = GetOrdersParams::new().status(OrderStatus::Resting);
        assert_eq!(params.to_query_string(), "?status=resting");
    }

    #[test]
    fn test_queue_positions_params() {
        let params = GetQueuePositionsParams::new()
            .market_tickers("AAPL,GOOG")
            .event_ticker("EVENT-123");
        let qs = params.to_query_string();
        assert!(qs.contains("market_tickers=AAPL%2CGOOG"));
        assert!(qs.contains("event_ticker=EVENT-123"));
    }

    #[test]
    fn test_create_order_request() {
        let req = CreateOrderRequest::new("KXBTC-25JAN", Side::Yes, Action::Buy, 10)
            .yes_price(50)
            .post_only(true);
        assert_eq!(req.ticker, "KXBTC-25JAN");
        assert_eq!(req.count, 10);
        assert_eq!(req.yes_price, Some(50));
        assert_eq!(req.post_only, Some(true));
    }

    #[test]
    fn test_batch_create_validation() {
        let orders: Vec<CreateOrderRequest> = (0..20)
            .map(|i| CreateOrderRequest::new(format!("TICKER-{}", i), Side::Yes, Action::Buy, 1))
            .collect();
        assert!(BatchCreateOrdersRequest::try_new(orders).is_ok());

        let too_many: Vec<CreateOrderRequest> = (0..21)
            .map(|i| CreateOrderRequest::new(format!("TICKER-{}", i), Side::Yes, Action::Buy, 1))
            .collect();
        assert!(matches!(
            BatchCreateOrdersRequest::try_new(too_many),
            Err(crate::error::Error::BatchSizeExceeded(21))
        ));
    }

    #[test]
    fn test_batch_cancel_validation() {
        let ids: Vec<String> = (0..20).map(|i| format!("order-{}", i)).collect();
        assert!(BatchCancelOrdersRequest::try_new(ids).is_ok());

        let too_many: Vec<String> = (0..21).map(|i| format!("order-{}", i)).collect();
        assert!(matches!(
            BatchCancelOrdersRequest::try_new(too_many),
            Err(crate::error::Error::BatchSizeExceeded(21))
        ));
    }
}
