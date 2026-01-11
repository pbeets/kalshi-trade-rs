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
//! let result = manager.create_orders(orders).await;
//!
//! // Always handle successfully created orders
//! println!("Created {} orders", result.completed.success_count());
//!
//! // Check if all batches completed
//! if let Some(err) = result.error {
//!     eprintln!("Batch processing stopped: {}", err);
//! }
//! ```
//!
//! # Builder Pattern
//!
//! For advanced configuration, use the builder:
//!
//! ```ignore
//! use kalshi_trade_rs::{BatchManager, RateLimitTier, RetryConfig};
//!
//! let manager = BatchManager::builder(&client)
//!     .tier(RateLimitTier::Advanced)
//!     .retry_config(RetryConfig::default())
//!     .build();
//! ```

use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{
    KalshiClient,
    error::{Error, MAX_BATCH_SIZE, Result},
    models::{
        BatchCancelOrderResult, BatchCancelOrdersRequest, BatchCreateOrdersRequest,
        BatchOrderResult, CreateOrderRequest, Order,
    },
};

/// Write cost for each order in a batch create request.
const CREATE_ORDER_COST: f64 = 1.0;

/// Write cost for each order in a batch cancel request.
const CANCEL_ORDER_COST: f64 = 0.2;

/// Default maximum retry attempts for transient errors.
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default base delay for exponential backoff.
const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Default maximum delay between retries.
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(10);

/// Rate limit tiers for the Kalshi API.
///
/// Each tier defines the number of read and write operations allowed per second.
/// See <https://docs.kalshi.com/getting_started/rate_limits> for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RateLimitTier {
    /// 20 reads/sec, 10 writes/sec
    #[default]
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
    ///
    /// Not used by `BatchManager` (which only handles write operations),
    /// but useful for implementing custom read rate limiting.
    pub fn reads_per_second(&self) -> f64 {
        match self {
            RateLimitTier::Basic => 20.0,
            RateLimitTier::Advanced => 30.0,
            RateLimitTier::Premier => 100.0,
            RateLimitTier::Prime => 400.0,
        }
    }
}

/// Configuration for retry behavior on transient errors.
///
/// Uses exponential backoff for retries.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
        }
    }
}

impl RetryConfig {
    /// Create a retry config with no retries.
    pub fn no_retries() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Create a retry config with the specified number of retries.
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Calculate the delay for the given attempt number (0-indexed).
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay = self.base_delay.saturating_mul(2u32.saturating_pow(attempt));
        delay.min(self.max_delay)
    }
}

/// Token bucket rate limiter.
///
/// Implements a token bucket algorithm where tokens are consumed for each
/// operation and refill at a constant rate. The bucket starts full, providing
/// burst capacity equal to one second of writes.
struct TokenBucket {
    /// Current number of available tokens.
    tokens: f64,
    /// Maximum token capacity (burst limit).
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

    /// Consume tokens, returning the wait time needed before they're available.
    ///
    /// This method immediately deducts the tokens (which may go negative) and returns
    /// how long the caller should wait before proceeding. This design allows the mutex
    /// to be released before sleeping.
    fn consume(&mut self, tokens: f64) -> Duration {
        self.refill();
        let wait_time = if self.tokens >= tokens {
            Duration::ZERO
        } else {
            let needed = tokens - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        };

        self.tokens -= tokens;

        wait_time
    }
}

/// Builder for [`BatchManager`].
///
/// Provides a fluent API for configuring batch manager options.
///
/// # Example
///
/// ```ignore
/// let manager = BatchManager::builder(&client)
///     .tier(RateLimitTier::Advanced)
///     .retry_config(RetryConfig::with_max_retries(5))
///     .build();
/// ```
pub struct BatchManagerBuilder<'a> {
    client: &'a KalshiClient,
    tier: RateLimitTier,
    retry_config: RetryConfig,
}

impl<'a> BatchManagerBuilder<'a> {
    fn new(client: &'a KalshiClient) -> Self {
        Self {
            client,
            tier: RateLimitTier::default(),
            retry_config: RetryConfig::no_retries(),
        }
    }

    /// Set the rate limit tier.
    pub fn tier(mut self, tier: RateLimitTier) -> Self {
        self.tier = tier;
        self
    }

    /// Set the retry configuration.
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Build the batch manager.
    pub fn build(self) -> BatchManager<'a> {
        BatchManager {
            client: self.client,
            rate_limiter: Mutex::new(TokenBucket::new(self.tier.writes_per_second())),
            retry_config: self.retry_config,
        }
    }
}

/// Manages batch order operations with automatic rate limiting.
///
/// The `BatchManager` handles:
/// - Chunking large order lists into batches of 20 (API limit)
/// - Rate limiting requests based on your account tier
/// - Automatic retry with exponential backoff for transient errors
/// - Aggregating results from multiple batch requests
/// - Preserving partial progress when errors occur
///
/// # Supported Operations
///
/// - **Batch Create**: Submit multiple orders at once
/// - **Batch Cancel**: Cancel multiple orders at once
///
/// Note: The Kalshi API does not support batch amend operations.
/// Individual order amendments must use [`KalshiClient::amend_order`].
///
/// # Rate Limiting
///
/// The manager uses a token bucket algorithm to pace requests:
/// - Each order in a batch create costs 1 write token
/// - Each order in a batch cancel costs 0.2 write tokens
/// - Tokens refill at the tier's writes-per-second rate
/// - Initial bucket is full, allowing burst capacity of 1 second
///
/// # Retry Behavior
///
/// By default, retries are disabled. Enable them via the builder:
///
/// ```ignore
/// let manager = BatchManager::builder(&client)
///     .retry_config(RetryConfig::default())  // 3 retries with exponential backoff
///     .build();
/// ```
///
/// Retries use exponential backoff and only apply to transient errors
/// (network timeouts, rate limit responses, server errors).
///
/// # Empty Input Handling
///
/// Passing an empty vector to `create_orders` or `cancel_orders` returns
/// immediately with an empty successful response. No API calls are made.
///
/// # Partial Failure Handling
///
/// When processing multiple batches, if an error occurs mid-way through,
/// the manager preserves successfully completed work. Check both the
/// `completed` field and `error` field of the result.
///
/// # Example
///
/// ```ignore
/// let manager = BatchManager::new(&client, RateLimitTier::Basic);
///
/// // Create 50 orders (will be split into 3 batches: 20, 20, 10)
/// let result = manager.create_orders(orders).await;
///
/// // Process all successful orders
/// for order in result.completed.successful_orders() {
///     println!("Created: {}", order.order_id);
/// }
///
/// // Handle any error that stopped processing
/// if let Some(err) = result.error {
///     eprintln!("Stopped early: {}", err);
/// }
///
/// // Or convert to Result if you don't need partial results
/// let response = manager.create_orders(orders).await.into_result()?;
/// ```
pub struct BatchManager<'a> {
    client: &'a KalshiClient,
    rate_limiter: Mutex<TokenBucket>,
    retry_config: RetryConfig,
}

impl<'a> BatchManager<'a> {
    /// Create a new batch manager with the specified rate limit tier.
    ///
    /// For advanced configuration (retries), use [`BatchManager::builder`].
    pub fn new(client: &'a KalshiClient, tier: RateLimitTier) -> Self {
        Self {
            client,
            rate_limiter: Mutex::new(TokenBucket::new(tier.writes_per_second())),
            retry_config: RetryConfig::no_retries(),
        }
    }

    /// Create a builder for advanced configuration.
    pub fn builder(client: &'a KalshiClient) -> BatchManagerBuilder<'a> {
        BatchManagerBuilder::new(client)
    }

    /// Execute a batch operation with retry logic.
    async fn execute_with_retry<T, F, Fut>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if self.should_retry(&e, attempt) => {
                    let delay = self.retry_config.delay_for_attempt(attempt);
                    tracing::debug!(
                        attempt = attempt + 1,
                        max_retries = self.retry_config.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Retrying batch operation"
                    );
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Determine if an error is retryable and we haven't exceeded max attempts.
    fn should_retry(&self, error: &Error, attempt: u32) -> bool {
        if attempt >= self.retry_config.max_retries {
            return false;
        }

        match error {
            // HTTP errors (network issues, timeouts) are always retryable
            Error::Http(_) => true,
            // API errors are only retryable if they indicate a transient issue
            Error::Api(msg) => is_transient_api_error(msg),
            // Other error types are not retryable
            _ => false,
        }
    }

    /// Create multiple orders with automatic batching and rate limiting.
    ///
    /// Orders are split into chunks of 20 (the API maximum) and submitted
    /// with appropriate delays to respect rate limits.
    ///
    /// # Arguments
    ///
    /// * `orders` - The orders to create. An empty vector returns immediately
    ///   with an empty successful response.
    ///
    /// # Returns
    ///
    /// A result containing successfully processed orders and any error that stopped processing.
    /// If an error occurs mid-batch, successfully created orders are still returned.
    pub async fn create_orders(
        &self,
        orders: Vec<CreateOrderRequest>,
    ) -> BatchOperationResult<AggregatedCreateResponse> {
        // Handle empty input - return immediately without API calls
        if orders.is_empty() {
            return BatchOperationResult {
                completed: AggregatedCreateResponse { orders: vec![] },
                error: None,
            };
        }

        let mut all_results = Vec::with_capacity(orders.len());

        for chunk in orders.chunks(MAX_BATCH_SIZE) {
            let cost = chunk.len() as f64 * CREATE_ORDER_COST;

            // Get wait time and consume tokens, then release lock before sleeping
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().await;
                limiter.consume(cost)
            };

            if !wait_time.is_zero() {
                tokio::time::sleep(wait_time).await;
            }

            // Send the batch with retry logic
            let request = BatchCreateOrdersRequest::new(chunk.to_vec());
            let client = self.client;
            match self
                .execute_with_retry(|| {
                    let req = request.clone();
                    async move { client.batch_create_orders(req).await }
                })
                .await
            {
                Ok(response) => all_results.extend(response.orders),
                Err(e) => {
                    return BatchOperationResult {
                        completed: AggregatedCreateResponse {
                            orders: all_results,
                        },
                        error: Some(e),
                    };
                }
            }
        }

        BatchOperationResult {
            completed: AggregatedCreateResponse {
                orders: all_results,
            },
            error: None,
        }
    }

    /// Cancel multiple orders with automatic batching and rate limiting.
    ///
    /// Order IDs are split into chunks of 20 (the API maximum) and submitted
    /// with appropriate delays to respect rate limits.
    ///
    /// # Arguments
    ///
    /// * `order_ids` - The order IDs to cancel. An empty vector returns immediately
    ///   with an empty successful response.
    ///
    /// # Returns
    ///
    /// A result containing successfully processed cancellations and any error that stopped processing.
    /// If an error occurs mid-batch, successfully canceled orders are still returned.
    pub async fn cancel_orders(
        &self,
        order_ids: Vec<String>,
    ) -> BatchOperationResult<AggregatedCancelResponse> {
        // Handle empty input - return immediately without API calls
        if order_ids.is_empty() {
            return BatchOperationResult {
                completed: AggregatedCancelResponse { orders: vec![] },
                error: None,
            };
        }

        let mut all_results = Vec::with_capacity(order_ids.len());

        for chunk in order_ids.chunks(MAX_BATCH_SIZE) {
            let cost = chunk.len() as f64 * CANCEL_ORDER_COST;

            // Get wait time and consume tokens, then release lock before sleeping
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().await;
                limiter.consume(cost)
            };

            if !wait_time.is_zero() {
                tokio::time::sleep(wait_time).await;
            }

            // Send the batch with retry logic
            let request = BatchCancelOrdersRequest::new(chunk.to_vec());
            let client = self.client;
            match self
                .execute_with_retry(|| {
                    let req = request.clone();
                    async move { client.batch_cancel_orders(req).await }
                })
                .await
            {
                Ok(response) => all_results.extend(response.orders),
                Err(e) => {
                    return BatchOperationResult {
                        completed: AggregatedCancelResponse {
                            orders: all_results,
                        },
                        error: Some(e),
                    };
                }
            }
        }

        BatchOperationResult {
            completed: AggregatedCancelResponse {
                orders: all_results,
            },
            error: None,
        }
    }
}

/// Check if an API error message indicates a transient/retryable error.
fn is_transient_api_error(msg: &str) -> bool {
    let msg_lower = msg.to_lowercase();
    msg_lower.contains("rate limit")
        || msg_lower.contains("timeout")
        || msg_lower.contains("temporarily unavailable")
        || msg_lower.contains("503")
        || msg_lower.contains("502")
        || msg_lower.contains("504")
}

/// Result of a batch operation that may have partially succeeded.
///
/// When processing multiple batches, if an error occurs mid-way through,
/// this struct preserves the successfully completed work along with the error.
///
/// # Example
///
/// ```ignore
/// let result = manager.create_orders(orders).await;
///
/// // Always process successful orders
/// for order in result.completed.successful_orders() {
///     println!("Created: {}", order.order_id);
/// }
///
/// // Check if there was an error
/// if let Some(err) = result.error {
///     eprintln!("Batch stopped early due to: {}", err);
/// }
/// ```
#[derive(Debug)]
pub struct BatchOperationResult<T> {
    /// Successfully completed portion of the operation.
    pub completed: T,
    /// Error that stopped processing, if any.
    pub error: Option<crate::error::Error>,
}

impl<T> BatchOperationResult<T> {
    /// Returns true if the operation completed without errors.
    pub fn is_complete(&self) -> bool {
        self.error.is_none()
    }

    /// Returns true if the operation was interrupted by an error.
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Converts to a Result, discarding partial progress on error.
    ///
    /// Use this when you don't care about partial results.
    pub fn into_result(self) -> Result<T> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(self.completed),
        }
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
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        };

        // Exponential backoff: 100ms, 200ms, 400ms, 800ms, 1600ms...
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
        assert_eq!(config.delay_for_attempt(4), Duration::from_millis(1600));

        // Should be capped at max_delay
        let config_with_low_max = RetryConfig {
            max_retries: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
        };
        assert_eq!(
            config_with_low_max.delay_for_attempt(5),
            Duration::from_secs(5)
        );
        assert_eq!(
            config_with_low_max.delay_for_attempt(10),
            Duration::from_secs(5)
        );
    }

    #[test]
    fn test_token_bucket_rate_limiting() {
        let mut bucket = TokenBucket::new(10.0);

        // First consume should have no wait (bucket starts full)
        let wait1 = bucket.consume(10.0);
        assert!(wait1.is_zero());

        // Second consume creates debt and returns wait time
        let wait2 = bucket.consume(5.0);
        assert!((wait2.as_secs_f64() - 0.5).abs() < 0.1);

        // Third consume adds more debt (now need to wait for 10 tokens)
        let wait3 = bucket.consume(5.0);
        assert!((wait3.as_secs_f64() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_is_transient_api_error() {
        // Transient errors should be retried
        assert!(is_transient_api_error("rate limit exceeded"));
        assert!(is_transient_api_error("Rate Limit Exceeded"));
        assert!(is_transient_api_error("request timeout"));
        assert!(is_transient_api_error("service temporarily unavailable"));
        assert!(is_transient_api_error("503 Service Unavailable"));
        assert!(is_transient_api_error("502 Bad Gateway"));
        assert!(is_transient_api_error("504 Gateway Timeout"));

        // Business errors should NOT be retried
        assert!(!is_transient_api_error("invalid order"));
        assert!(!is_transient_api_error("insufficient balance"));
        assert!(!is_transient_api_error("order not found"));
    }

    #[test]
    fn test_aggregated_cancel_response_total_reduced() {
        use crate::models::{
            Action, BatchCancelOrderResult, BatchOrderError, Order, OrderStatus, OrderType, Side,
        };

        fn make_order(id: &str) -> Order {
            Order {
                order_id: id.to_string(),
                user_id: None,
                client_order_id: None,
                ticker: "TEST".to_string(),
                side: Side::Yes,
                action: Action::Buy,
                order_type: OrderType::Limit,
                status: OrderStatus::Canceled,
                yes_price: 50,
                no_price: 50,
                yes_price_dollars: None,
                no_price_dollars: None,
                fill_count: 0,
                remaining_count: 0,
                initial_count: 10,
                taker_fees: None,
                maker_fees: None,
                taker_fill_cost: None,
                maker_fill_cost: None,
                taker_fill_cost_dollars: None,
                maker_fill_cost_dollars: None,
                taker_fees_dollars: None,
                maker_fees_dollars: None,
                queue_position: None,
                expiration_time: None,
                created_time: None,
                last_update_time: None,
                self_trade_prevention_type: None,
                order_group_id: None,
                cancel_order_on_pause: None,
            }
        }

        let response = AggregatedCancelResponse {
            orders: vec![
                BatchCancelOrderResult {
                    order_id: Some("order1".to_string()),
                    reduced_by: Some(5),
                    order: Some(make_order("order1")),
                    error: None,
                },
                BatchCancelOrderResult {
                    order_id: Some("order2".to_string()),
                    reduced_by: Some(10),
                    order: Some(make_order("order2")),
                    error: None,
                },
                BatchCancelOrderResult {
                    order_id: Some("order3".to_string()),
                    reduced_by: None,
                    order: None,
                    error: Some(BatchOrderError {
                        code: "NOT_FOUND".to_string(),
                        message: "Order not found".to_string(),
                        details: None,
                        service: None,
                    }),
                },
            ],
        };

        assert_eq!(response.success_count(), 2);
        assert_eq!(response.failure_count(), 1);
        assert_eq!(response.total_reduced(), 15);
    }
}
