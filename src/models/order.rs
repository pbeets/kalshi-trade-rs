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
    pub user_id: String,
    pub client_order_id: String,
    pub ticker: String,
    pub side: Side,
    pub action: Action,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub status: OrderStatus,
    /// Price in fixed-point dollars.
    pub yes_price_dollars: String,
    /// Price in fixed-point dollars.
    pub no_price_dollars: String,
    /// Fill count (fixed-point decimal string).
    pub fill_count_fp: String,
    /// Remaining count (fixed-point decimal string).
    pub remaining_count_fp: String,
    /// Initial count (fixed-point decimal string).
    pub initial_count_fp: String,
    pub taker_fill_cost_dollars: String,
    pub maker_fill_cost_dollars: String,
    #[serde(default)]
    pub taker_fees_dollars: Option<String>,
    #[serde(default)]
    pub maker_fees_dollars: Option<String>,
    /// Deprecated: always returns 0. Use the `get_order_queue_position` endpoint instead.
    #[serde(default)]
    pub queue_position: i64,
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
    /// Subaccount number this order belongs to (0 for primary, 1-32 for subaccounts).
    #[serde(default)]
    pub subaccount_number: Option<i32>,
}

/// Response from the get_orders endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
    pub cursor: String,
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
    /// Filter by subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
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
    /// Panics if `limit` is not in the range 1..=200.
    /// Use [`try_limit`](Self::try_limit) for fallible construction.
    #[must_use]
    pub fn limit(self, limit: i64) -> Self {
        self.try_limit(limit).expect("invalid limit")
    }

    /// Set the maximum number of results to return with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if `limit` is not in the range 1..=200.
    pub fn try_limit(mut self, limit: i64) -> crate::error::Result<Self> {
        if limit <= 0 || limit > 200 {
            return Err(crate::error::Error::InvalidLimit(limit, 1, 200));
        }
        self.limit = Some(limit);
        Ok(self)
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filter by subaccount number.
    ///
    /// Use 0 for the primary account, or 1-32 for numbered subaccounts.
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
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
        qb.push_opt("subaccount", self.subaccount);
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

    /// Deprecated: Use `reduce_only` instead. Only accepts value of 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_position_floor: Option<i64>,

    /// Number of contracts (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<String>,

    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
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
            sell_position_floor: None,
            count_fp: None,
            subaccount: None,
        }
    }

    #[must_use]
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Set yes price in cents (1-99).
    ///
    /// # Panics
    ///
    /// Panics if price is not between 1 and 99.
    /// Use [`try_yes_price`](Self::try_yes_price) for fallible construction.
    #[must_use]
    pub fn yes_price(self, price: i64) -> Self {
        self.try_yes_price(price).expect("invalid yes price")
    }

    /// Set yes price in cents (1-99) with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if price is not between 1 and 99.
    pub fn try_yes_price(mut self, price: i64) -> crate::error::Result<Self> {
        if !(1..=99).contains(&price) {
            return Err(crate::error::Error::InvalidPrice(price));
        }
        self.yes_price = Some(price);
        Ok(self)
    }

    /// Set no price in cents (1-99).
    ///
    /// # Panics
    ///
    /// Panics if price is not between 1 and 99.
    /// Use [`try_no_price`](Self::try_no_price) for fallible construction.
    #[must_use]
    pub fn no_price(self, price: i64) -> Self {
        self.try_no_price(price).expect("invalid no price")
    }

    /// Set no price in cents (1-99) with validation.
    ///
    /// # Errors
    ///
    /// Returns an error if price is not between 1 and 99.
    pub fn try_no_price(mut self, price: i64) -> crate::error::Result<Self> {
        if !(1..=99).contains(&price) {
            return Err(crate::error::Error::InvalidPrice(price));
        }
        self.no_price = Some(price);
        Ok(self)
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

    /// Deprecated: Use `reduce_only` instead. Only accepts value of 0.
    #[must_use]
    #[deprecated(note = "Use reduce_only instead. Only accepts value of 0.")]
    pub fn sell_position_floor(mut self, floor: i64) -> Self {
        self.sell_position_floor = Some(floor);
        self
    }

    /// Set the number of contracts as a fixed-point decimal string.
    #[must_use]
    pub fn count_fp(mut self, count_fp: impl Into<String>) -> Self {
        self.count_fp = Some(count_fp.into());
        self
    }

    /// Set the subaccount number (0 for primary, 1-32 for subaccounts).
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }
}

/// Response from DELETE /portfolio/orders/{order_id} (cancel order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub order: Order,
    /// Number of contracts that were canceled.
    pub reduced_by: i64,
    /// Number of contracts that were canceled (fixed-point decimal string).
    pub reduced_by_fp: String,
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

    /// Original client-specified order ID to be amended (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// New client-specified order ID after amendment (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_client_order_id: Option<String>,

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

    /// Updated quantity (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count_fp: Option<String>,

    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl AmendOrderRequest {
    /// Create a new amend order request with required fields only.
    ///
    /// Orders can be identified by `order_id` alone; `client_order_id` and
    /// `updated_client_order_id` are optional.
    #[must_use]
    pub fn new(ticker: impl Into<String>, side: Side, action: Action) -> Self {
        Self {
            ticker: ticker.into(),
            side,
            action,
            client_order_id: None,
            updated_client_order_id: None,
            yes_price: None,
            no_price: None,
            yes_price_dollars: None,
            no_price_dollars: None,
            count: None,
            count_fp: None,
            subaccount: None,
        }
    }

    /// Create a new amend order request with client order IDs.
    #[must_use]
    pub fn with_client_order_ids(
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
            client_order_id: Some(client_order_id.into()),
            updated_client_order_id: Some(updated_client_order_id.into()),
            yes_price: None,
            no_price: None,
            yes_price_dollars: None,
            no_price_dollars: None,
            count: None,
            count_fp: None,
            subaccount: None,
        }
    }

    /// Set the original client-specified order ID.
    #[must_use]
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Set the new client-specified order ID after amendment.
    #[must_use]
    pub fn updated_client_order_id(mut self, id: impl Into<String>) -> Self {
        self.updated_client_order_id = Some(id.into());
        self
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

    /// Set the updated quantity as a fixed-point decimal string.
    #[must_use]
    pub fn count_fp(mut self, count_fp: impl Into<String>) -> Self {
        self.count_fp = Some(count_fp.into());
        self
    }

    /// Set the subaccount number (0 for primary, 1-32 for subaccounts).
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
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

    /// Amount to reduce by (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_by_fp: Option<String>,

    /// Target remaining quantity (fixed-point decimal string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_to_fp: Option<String>,

    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl DecreaseOrderRequest {
    /// Create a request to reduce by a specific amount.
    #[must_use]
    pub fn reduce_by(amount: i64) -> Self {
        Self {
            reduce_by: Some(amount),
            reduce_to: None,
            reduce_by_fp: None,
            reduce_to_fp: None,
            subaccount: None,
        }
    }

    /// Create a request to reduce to a target quantity.
    #[must_use]
    pub fn reduce_to(target: i64) -> Self {
        Self {
            reduce_by: None,
            reduce_to: Some(target),
            reduce_by_fp: None,
            reduce_to_fp: None,
            subaccount: None,
        }
    }

    /// Set the subaccount number (0 for primary, 1-32 for subaccounts).
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
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
    /// Panics if `orders.len() > 20`. Use [`try_new`](Self::try_new)
    /// for fallible construction.
    #[must_use]
    pub fn new(orders: Vec<CreateOrderRequest>) -> Self {
        Self::try_new(orders).expect("batch size exceeded")
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

/// A single order to cancel with optional subaccount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrderItem {
    /// Order ID to cancel.
    pub order_id: String,
    /// Subaccount number (0 for primary, 1-32 for subaccounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
}

impl BatchCancelOrderItem {
    /// Create a new batch cancel order item.
    #[must_use]
    pub fn new(order_id: impl Into<String>) -> Self {
        Self {
            order_id: order_id.into(),
            subaccount: None,
        }
    }

    /// Set the subaccount number.
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }
}

/// Request body for DELETE /portfolio/orders/batched (batch cancel).
///
/// Supports two formats:
/// - Legacy: `ids` array of order ID strings
/// - New: `orders` array with per-order subaccount support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrdersRequest {
    /// Array of order IDs to cancel (legacy format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    /// Array of orders to cancel with per-order subaccount support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<BatchCancelOrderItem>>,
}

impl BatchCancelOrdersRequest {
    /// Create a new batch cancel request using order IDs (legacy format).
    ///
    /// # Panics
    ///
    /// Panics if `ids.len() > 20`. Use [`try_new`](Self::try_new)
    /// for fallible construction.
    #[must_use]
    #[deprecated(
        since = "0.3.0",
        note = "Uses legacy `ids` format. Use `with_orders()` for per-order subaccount support."
    )]
    #[allow(deprecated)]
    pub fn new(ids: Vec<String>) -> Self {
        Self::try_new(ids).expect("batch size exceeded")
    }

    /// Create a new batch cancel request with validation (legacy format).
    ///
    /// Returns an error if the batch exceeds the maximum size of 20 orders.
    #[deprecated(
        since = "0.3.0",
        note = "Uses legacy `ids` format. Use `try_with_orders()` for per-order subaccount support."
    )]
    pub fn try_new(ids: Vec<String>) -> crate::error::Result<Self> {
        if ids.len() > crate::error::MAX_BATCH_SIZE {
            return Err(crate::error::Error::BatchSizeExceeded(ids.len()));
        }
        Ok(Self {
            ids: Some(ids),
            orders: None,
        })
    }

    /// Create a new batch cancel request with per-order subaccount support.
    ///
    /// # Panics
    ///
    /// Panics if `orders.len() > 20`. Use [`try_with_orders`](Self::try_with_orders)
    /// for fallible construction.
    #[must_use]
    pub fn with_orders(orders: Vec<BatchCancelOrderItem>) -> Self {
        Self::try_with_orders(orders).expect("batch size exceeded")
    }

    /// Create a new batch cancel request with per-order subaccount support and validation.
    ///
    /// Returns an error if the batch exceeds the maximum size of 20 orders.
    pub fn try_with_orders(orders: Vec<BatchCancelOrderItem>) -> crate::error::Result<Self> {
        if orders.len() > crate::error::MAX_BATCH_SIZE {
            return Err(crate::error::Error::BatchSizeExceeded(orders.len()));
        }
        Ok(Self {
            ids: None,
            orders: Some(orders),
        })
    }
}

impl TryFrom<Vec<String>> for BatchCancelOrdersRequest {
    type Error = crate::error::Error;

    #[allow(deprecated)]
    fn try_from(ids: Vec<String>) -> crate::error::Result<Self> {
        Self::try_new(ids)
    }
}

/// Result for a single order in a batch cancel operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCancelOrderResult {
    /// Order ID.
    pub order_id: String,

    /// Number of contracts canceled.
    pub reduced_by: i64,

    /// Number of contracts canceled (fixed-point decimal string).
    pub reduced_by_fp: String,

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
    /// Queue position (fixed-point decimal string).
    #[serde(default)]
    pub queue_position_fp: Option<String>,
}

/// Response from GET /portfolio/orders/queue_positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuePositionsResponse {
    /// Queue positions for the requested orders.
    #[serde(default, deserialize_with = "null_as_empty_vec::deserialize")]
    pub queue_positions: Vec<QueuePosition>,
}

/// Response from GET /portfolio/orders/{order_id}/queue_position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderQueuePositionResponse {
    /// Queue position - number of contracts ahead in the queue.
    pub queue_position: i64,
    /// Queue position (fixed-point decimal string).
    #[serde(default)]
    pub queue_position_fp: Option<String>,
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
    /// Filter by subaccount number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subaccount: Option<i32>,
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

    /// Filter by subaccount number.
    #[must_use]
    pub fn subaccount(mut self, subaccount: i32) -> Self {
        self.subaccount = Some(subaccount);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut qb = QueryBuilder::new();
        qb.push_opt("market_tickers", self.market_tickers.as_ref());
        qb.push_opt("event_ticker", self.event_ticker.as_ref());
        qb.push_opt("subaccount", self.subaccount);
        qb.build()
    }
}

use super::common::null_as_empty_vec;

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
    #[allow(deprecated)]
    fn test_batch_cancel_validation() {
        let ids: Vec<String> = (0..20).map(|i| format!("order-{}", i)).collect();
        assert!(BatchCancelOrdersRequest::try_new(ids).is_ok());

        let too_many: Vec<String> = (0..21).map(|i| format!("order-{}", i)).collect();
        assert!(matches!(
            BatchCancelOrdersRequest::try_new(too_many),
            Err(crate::error::Error::BatchSizeExceeded(21))
        ));
    }

    #[test]
    fn test_batch_cancel_with_orders() {
        let orders = vec![
            BatchCancelOrderItem::new("order-1").subaccount(1),
            BatchCancelOrderItem::new("order-2"),
        ];
        let req = BatchCancelOrdersRequest::with_orders(orders);
        assert!(req.ids.is_none());
        assert_eq!(req.orders.as_ref().unwrap().len(), 2);
        assert_eq!(req.orders.as_ref().unwrap()[0].subaccount, Some(1));
        assert!(req.orders.as_ref().unwrap()[1].subaccount.is_none());
    }

    #[test]
    fn test_amend_order_optional_client_ids() {
        let req = AmendOrderRequest::new("TICKER", Side::Yes, Action::Buy)
            .yes_price(55)
            .count(10);
        assert!(req.client_order_id.is_none());
        assert!(req.updated_client_order_id.is_none());
        assert_eq!(req.count, Some(10));

        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("client_order_id"));
    }

    #[test]
    fn test_amend_order_with_client_ids() {
        let req = AmendOrderRequest::with_client_order_ids(
            "TICKER",
            Side::Yes,
            Action::Buy,
            "old-id",
            "new-id",
        );
        assert_eq!(req.client_order_id.as_deref(), Some("old-id"));
        assert_eq!(req.updated_client_order_id.as_deref(), Some("new-id"));
    }
}
