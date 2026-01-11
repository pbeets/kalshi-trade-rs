//! Batch order management with rate limiting.
//!
//! This module provides a [`BatchManager`] that handles automatic chunking
//! and rate-limited submission of orders to the Kalshi API.
//!
//! # Example
//!
//! ```ignore
//! use kalshi_trade_rs::{BatchManager, RateLimitTier, CreateOrderRequest, Side, Action};
//!
//! let manager = BatchManager::new(&client, RateLimitTier::Basic);
//!
//! // Submit 50 orders - manager handles chunking and rate limiting
//! let orders: Vec<CreateOrderRequest> = (0..50)
//!     .map(|i| CreateOrderRequest::new("TICKER", Side::Yes, Action::Buy, 1))
//!     .collect();
//!
//! let results = manager.create_orders(orders).await?;
//! println!("Created {} orders, {} failed", results.success_count(), results.failure_count());
//! ```

use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{
    error::Result,
    models::{
        BatchCancelOrderResult, BatchCancelOrdersRequest, BatchCreateOrdersRequest,
        BatchOrderResult, CreateOrderRequest, Order,
    },
    KalshiClient,
};

/// Maximum orders per batch request (Kalshi API limit).
const MAX_BATCH_SIZE: usize = 20;

/// Write cost for each order in a batch create request.
const CREATE_ORDER_COST: f64 = 1.0;

/// Write cost for each order in a batch cancel request.
const CANCEL_ORDER_COST: f64 = 0.2;

/// Rate limit tiers for the Kalshi API.
///
/// Each tier defines the number of read and write operations allowed per second.
/// See <https://docs.kalshi.com/getting_started/rate_limits> for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitTier {
    /// 20 reads/sec, 10 writes/sec
    Basic,
    /// 30 reads/sec, 30 writes/sec
    Advanced,
    /// 100 reads/sec, 100 writes/sec
    Premier,
    /// 400 reads/sec, 400 writes/sec
    Prime,
}

impl RateLimitTier {
    /// Returns the writes per second limit for this tier.
    pub fn writes_per_second(&self) -> f64 {
        match self {
            RateLimitTier::Basic => 10.0,
            RateLimitTier::Advanced => 30.0,
            RateLimitTier::Premier => 100.0,
            RateLimitTier::Prime => 400.0,
        }
    }

    /// Returns the reads per second limit for this tier.
    pub fn reads_per_second(&self) -> f64 {
        match self {
            RateLimitTier::Basic => 20.0,
            RateLimitTier::Advanced => 30.0,
            RateLimitTier::Premier => 100.0,
            RateLimitTier::Prime => 400.0,
        }
    }
}

/// Token bucket rate limiter.
///
/// Implements a token bucket algorithm where tokens are consumed for each
/// operation and refill at a constant rate.
struct TokenBucket {
    /// Current number of available tokens.
    tokens: f64,
    /// Maximum token capacity.
    capacity: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last time tokens were refilled.
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate.
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate: capacity,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    /// Try to consume tokens. Returns true if successful, false if not enough tokens.
    #[allow(dead_code)]
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Calculate how long to wait until the given number of tokens are available.
    fn time_until_available(&mut self, tokens: f64) -> Duration {
        self.refill();
        if self.tokens >= tokens {
            Duration::ZERO
        } else {
            let needed = tokens - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }

    /// Wait until tokens are available, then consume them.
    async fn consume(&mut self, tokens: f64) {
        let wait_time = self.time_until_available(tokens);
        if !wait_time.is_zero() {
            tokio::time::sleep(wait_time).await;
        }
        self.refill();
        self.tokens -= tokens;
    }
}

/// Manages batch order operations with automatic rate limiting.
///
/// The `BatchManager` handles:
/// - Chunking large order lists into batches of 20 (API limit)
/// - Rate limiting requests based on your account tier
/// - Aggregating results from multiple batch requests
///
/// # Rate Limiting
///
/// The manager uses a token bucket algorithm to pace requests:
/// - Each order in a batch create costs 1 write token
/// - Each order in a batch cancel costs 0.2 write tokens
/// - Tokens refill at the tier's writes-per-second rate
///
/// # Example
///
/// ```ignore
/// let manager = BatchManager::new(&client, RateLimitTier::Basic);
///
/// // Create 50 orders (will be split into 3 batches: 20, 20, 10)
/// let results = manager.create_orders(orders).await?;
///
/// for order in results.successful_orders() {
///     println!("Created: {}", order.order_id);
/// }
/// ```
pub struct BatchManager<'a> {
    client: &'a KalshiClient,
    rate_limiter: Mutex<TokenBucket>,
}

impl<'a> BatchManager<'a> {
    /// Create a new batch manager with the specified rate limit tier.
    pub fn new(client: &'a KalshiClient, tier: RateLimitTier) -> Self {
        Self {
            client,
            rate_limiter: Mutex::new(TokenBucket::new(tier.writes_per_second())),
        }
    }

    /// Create multiple orders with automatic batching and rate limiting.
    ///
    /// Orders are split into chunks of 20 (the API maximum) and submitted
    /// with appropriate delays to respect rate limits.
    ///
    /// # Arguments
    ///
    /// * `orders` - The orders to create (can be any number)
    ///
    /// # Returns
    ///
    /// An aggregated response containing results from all batches.
    pub async fn create_orders(
        &self,
        orders: Vec<CreateOrderRequest>,
    ) -> Result<AggregatedCreateResponse> {
        let mut all_results = Vec::with_capacity(orders.len());

        for chunk in orders.chunks(MAX_BATCH_SIZE) {
            let cost = chunk.len() as f64 * CREATE_ORDER_COST;

            // Wait for rate limit capacity
            {
                let mut limiter = self.rate_limiter.lock().await;
                limiter.consume(cost).await;
            }

            // Send the batch
            let request = BatchCreateOrdersRequest::new(chunk.to_vec());
            let response = self.client.batch_create_orders(request).await?;
            all_results.extend(response.orders);
        }

        Ok(AggregatedCreateResponse {
            orders: all_results,
        })
    }

    /// Cancel multiple orders with automatic batching and rate limiting.
    ///
    /// Order IDs are split into chunks of 20 (the API maximum) and submitted
    /// with appropriate delays to respect rate limits.
    ///
    /// # Arguments
    ///
    /// * `order_ids` - The order IDs to cancel (can be any number)
    ///
    /// # Returns
    ///
    /// An aggregated response containing results from all batches.
    pub async fn cancel_orders(
        &self,
        order_ids: Vec<String>,
    ) -> Result<AggregatedCancelResponse> {
        let mut all_results = Vec::with_capacity(order_ids.len());

        for chunk in order_ids.chunks(MAX_BATCH_SIZE) {
            let cost = chunk.len() as f64 * CANCEL_ORDER_COST;

            // Wait for rate limit capacity
            {
                let mut limiter = self.rate_limiter.lock().await;
                limiter.consume(cost).await;
            }

            // Send the batch
            let request = BatchCancelOrdersRequest::new(chunk.to_vec());
            let response = self.client.batch_cancel_orders(request).await?;
            all_results.extend(response.orders);
        }

        Ok(AggregatedCancelResponse {
            orders: all_results,
        })
    }
}

/// Aggregated response from multiple batch create requests.
#[derive(Debug, Clone)]
pub struct AggregatedCreateResponse {
    /// All order results from all batches.
    pub orders: Vec<BatchOrderResult>,
}

impl AggregatedCreateResponse {
    /// Returns an iterator over successfully created orders.
    pub fn successful_orders(&self) -> impl Iterator<Item = &Order> {
        self.orders.iter().filter_map(|r| r.order.as_ref())
    }

    /// Returns an iterator over failed orders with their client_order_id and error.
    pub fn failed_orders(
        &self,
    ) -> impl Iterator<Item = (Option<&str>, &crate::models::BatchOrderError)> {
        self.orders
            .iter()
            .filter_map(|r| r.error.as_ref().map(|e| (r.client_order_id.as_deref(), e)))
    }

    /// Returns the number of successfully created orders.
    pub fn success_count(&self) -> usize {
        self.orders.iter().filter(|r| r.order.is_some()).count()
    }

    /// Returns the number of failed orders.
    pub fn failure_count(&self) -> usize {
        self.orders.iter().filter(|r| r.error.is_some()).count()
    }

    /// Returns the total number of orders processed.
    pub fn total_count(&self) -> usize {
        self.orders.len()
    }
}

/// Aggregated response from multiple batch cancel requests.
#[derive(Debug, Clone)]
pub struct AggregatedCancelResponse {
    /// All order results from all batches.
    pub orders: Vec<BatchCancelOrderResult>,
}

impl AggregatedCancelResponse {
    /// Returns an iterator over successfully canceled orders.
    pub fn successful_orders(&self) -> impl Iterator<Item = &Order> {
        self.orders.iter().filter_map(|r| r.order.as_ref())
    }

    /// Returns an iterator over failed cancellations with their order_id and error.
    pub fn failed_orders(
        &self,
    ) -> impl Iterator<Item = (Option<&str>, &crate::models::BatchOrderError)> {
        self.orders
            .iter()
            .filter_map(|r| r.error.as_ref().map(|e| (r.order_id.as_deref(), e)))
    }

    /// Returns the number of successfully canceled orders.
    pub fn success_count(&self) -> usize {
        self.orders.iter().filter(|r| r.order.is_some()).count()
    }

    /// Returns the number of failed cancellations.
    pub fn failure_count(&self) -> usize {
        self.orders.iter().filter(|r| r.error.is_some()).count()
    }

    /// Returns the total number of contracts canceled.
    pub fn total_reduced(&self) -> i64 {
        self.orders.iter().filter_map(|r| r.reduced_by).sum()
    }

    /// Returns the total number of orders processed.
    pub fn total_count(&self) -> usize {
        self.orders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_writes() {
        assert_eq!(RateLimitTier::Basic.writes_per_second(), 10.0);
        assert_eq!(RateLimitTier::Advanced.writes_per_second(), 30.0);
        assert_eq!(RateLimitTier::Premier.writes_per_second(), 100.0);
        assert_eq!(RateLimitTier::Prime.writes_per_second(), 400.0);
    }

    #[test]
    fn test_rate_limit_tier_reads() {
        assert_eq!(RateLimitTier::Basic.reads_per_second(), 20.0);
        assert_eq!(RateLimitTier::Advanced.reads_per_second(), 30.0);
        assert_eq!(RateLimitTier::Premier.reads_per_second(), 100.0);
        assert_eq!(RateLimitTier::Prime.reads_per_second(), 400.0);
    }

    #[test]
    fn test_token_bucket_initial_capacity() {
        let bucket = TokenBucket::new(10.0);
        assert_eq!(bucket.tokens, 10.0);
        assert_eq!(bucket.capacity, 10.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(10.0);
        assert!(bucket.try_consume(5.0));
        // Allow small floating point drift from refill
        assert!(bucket.tokens <= 5.1 && bucket.tokens >= 4.9);
        assert!(bucket.try_consume(5.0));
        // After consuming all tokens, should be near zero (may have tiny refill)
        assert!(bucket.tokens < 0.1);
        assert!(!bucket.try_consume(1.0)); // Should fail, not enough tokens
    }

    #[test]
    fn test_token_bucket_time_until_available() {
        let mut bucket = TokenBucket::new(10.0);
        bucket.tokens = 0.0; // Drain the bucket

        let wait_time = bucket.time_until_available(5.0);
        // Should need to wait 0.5 seconds for 5 tokens at 10 tokens/sec
        assert!((wait_time.as_secs_f64() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_create_response() {
        use crate::models::{
            Action, BatchOrderError, BatchOrderResult, Order, OrderStatus, OrderType, Side,
        };

        fn make_order(id: &str) -> Order {
            Order {
                order_id: id.to_string(),
                user_id: None,
                client_order_id: Some(format!("client-{}", id)),
                ticker: "TEST".to_string(),
                side: Side::Yes,
                action: Action::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::Resting,
                yes_price: 50,
                no_price: 50,
                yes_price_dollars: None,
                no_price_dollars: None,
                fill_count: 0,
                remaining_count: 10,
                initial_count: 10,
                taker_fees: None,
                maker_fees: None,
                taker_fill_cost: None,
                maker_fill_cost: None,
                taker_fill_cost_dollars: None,
                maker_fill_cost_dollars: None,
                taker_fees_dollars: None,
                maker_fees_dollars: None,
                expiration_time: None,
                created_time: None,
                last_update_time: None,
                self_trade_prevention_type: None,
                order_group_id: None,
                cancel_order_on_pause: None,
            }
        }

        let response = AggregatedCreateResponse {
            orders: vec![
                BatchOrderResult {
                    client_order_id: Some("order1".to_string()),
                    order: Some(make_order("1")),
                    error: None,
                },
                BatchOrderResult {
                    client_order_id: Some("order2".to_string()),
                    order: None,
                    error: Some(BatchOrderError {
                        code: "ERROR".to_string(),
                        message: "Failed".to_string(),
                        details: None,
                        service: None,
                    }),
                },
            ],
        };

        assert_eq!(response.success_count(), 1);
        assert_eq!(response.failure_count(), 1);
        assert_eq!(response.total_count(), 2);
    }
}
