//! Orderbook aggregator for maintaining live orderbook state.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;

use crate::models::Side;
use crate::ws::{KalshiStreamHandle, StreamMessage};

use super::state::OrderbookState;

/// Summary of an orderbook's current state.
#[derive(Debug, Clone)]
pub struct OrderbookSummary {
    /// Market ticker.
    pub ticker: String,
    /// Best YES bid (price, quantity) - highest price to buy YES.
    pub best_bid: Option<(i64, i64)>,
    /// Best YES ask (price, quantity) - lowest price to sell YES.
    pub best_ask: Option<(i64, i64)>,
    /// Spread in cents (ask - bid).
    pub spread: Option<i64>,
    /// Midpoint price.
    pub midpoint: Option<f64>,
    /// Total YES side liquidity.
    pub total_yes_liquidity: i64,
    /// Total NO side liquidity.
    pub total_no_liquidity: i64,
    /// Whether the orderbook has been initialized with a snapshot.
    pub initialized: bool,
}

/// What changed in an orderbook update.
#[derive(Debug, Clone)]
pub struct OrderbookDelta {
    /// Side that was updated (Yes or No).
    pub side: Side,
    /// Price level that changed.
    pub price: i64,
    /// Change in quantity (positive or negative).
    pub quantity_change: i64,
    /// Quantity at this level after the change.
    pub new_quantity: i64,
}

/// An orderbook update event.
///
/// Emitted when the orderbook changes.
#[derive(Debug, Clone)]
pub struct OrderbookUpdate {
    /// Market ticker.
    pub ticker: String,
    /// Current orderbook summary after this update.
    pub summary: OrderbookSummary,
    /// What changed (only present for delta updates, not snapshots).
    pub delta: Option<OrderbookDelta>,
}

/// Notification of a sequence gap.
///
/// Indicates that messages may have been missed.
#[derive(Debug, Clone)]
pub struct SequenceGap {
    /// Market ticker (if known).
    pub ticker: Option<String>,
    /// Expected sequence number.
    pub expected: i64,
    /// Received sequence number.
    pub received: i64,
}

/// Default channel capacity for update broadcasts.
const DEFAULT_UPDATE_CAPACITY: usize = 1024;

/// Default channel capacity for gap notifications.
const DEFAULT_GAP_CAPACITY: usize = 64;

/// Aggregator that maintains live orderbook state from WebSocket updates.
///
/// The aggregator processes orderbook snapshots and deltas to maintain
/// the current state of all subscribed markets' orderbooks. It supports
/// both pull-based (polling) and push-based (streaming) access patterns.
///
/// # Example - Pull-based (polling)
///
/// ```no_run
/// use kalshi_trade_rs::orderbook::OrderbookAggregator;
/// use std::time::Duration;
///
/// # async fn example(handle: kalshi_trade_rs::ws::KalshiStreamHandle) {
/// let aggregator = OrderbookAggregator::new();
/// let agg_clone = aggregator.clone();
///
/// // Spawn background processor
/// tokio::spawn(async move {
///     agg_clone.process_updates(handle).await;
/// });
///
/// // Poll at your cadence
/// loop {
///     if let Some(summary) = aggregator.summary("TICKER-1") {
///         println!("spread={:?}", summary.spread);
///     }
///     tokio::time::sleep(Duration::from_millis(100)).await;
/// }
/// # }
/// ```
///
/// # Example - Push-based (streaming)
///
/// ```no_run
/// use kalshi_trade_rs::orderbook::OrderbookAggregator;
///
/// # async fn example(handle: kalshi_trade_rs::ws::KalshiStreamHandle) {
/// let aggregator = OrderbookAggregator::new();
/// let mut updates = aggregator.update_receiver();
/// let agg_clone = aggregator.clone();
///
/// // Spawn background processor
/// tokio::spawn(async move {
///     agg_clone.process_updates(handle).await;
/// });
///
/// // React to every change
/// while let Ok(update) = updates.recv().await {
///     println!("{}: spread={:?}", update.ticker, update.summary.spread);
///     if let Some(delta) = &update.delta {
///         println!("  changed {:?} @ {}", delta.side, delta.price);
///     }
/// }
/// # }
/// ```
#[derive(Clone)]
pub struct OrderbookAggregator {
    state: Arc<RwLock<HashMap<String, OrderbookState>>>,
    update_sender: broadcast::Sender<OrderbookUpdate>,
    gap_sender: broadcast::Sender<SequenceGap>,
}

impl Default for OrderbookAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderbookAggregator {
    /// Create a new orderbook aggregator.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_UPDATE_CAPACITY, DEFAULT_GAP_CAPACITY)
    }

    /// Create a new orderbook aggregator with custom channel capacities.
    pub fn with_capacity(update_capacity: usize, gap_capacity: usize) -> Self {
        let (update_sender, _) = broadcast::channel(update_capacity);
        let (gap_sender, _) = broadcast::channel(gap_capacity);

        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            update_sender,
            gap_sender,
        }
    }

    /// Process updates from a WebSocket handle.
    ///
    /// This method runs in a loop, processing orderbook updates until
    /// the connection is closed or lost. Run this in a spawned task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::orderbook::OrderbookAggregator;
    /// # async fn example(handle: kalshi_trade_rs::ws::KalshiStreamHandle) {
    /// let aggregator = OrderbookAggregator::new();
    /// let agg_clone = aggregator.clone();
    ///
    /// tokio::spawn(async move {
    ///     agg_clone.process_updates(handle).await;
    /// });
    /// # }
    /// ```
    pub async fn process_updates(&self, mut handle: KalshiStreamHandle) {
        loop {
            match handle.update_receiver.recv().await {
                Ok(update) => {
                    // Check for sequence gaps
                    if let Some(seq) = update.seq {
                        self.check_sequence_gap(seq);
                    }

                    match &update.msg {
                        StreamMessage::OrderbookSnapshot(snapshot) => {
                            self.handle_snapshot(snapshot);
                        }
                        StreamMessage::OrderbookDelta(delta) => {
                            self.handle_delta(delta, update.seq);
                        }
                        StreamMessage::Closed { .. } | StreamMessage::ConnectionLost { .. } => {
                            // Connection ended, exit the loop
                            break;
                        }
                        _ => {
                            // Ignore other message types
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We missed messages - notify via gap channel
                    let _ = self.gap_sender.send(SequenceGap {
                        ticker: None,
                        expected: 0,
                        received: n as i64,
                    });
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }

    /// Handle an orderbook snapshot.
    fn handle_snapshot(&self, snapshot: &crate::ws::OrderbookSnapshotData) {
        let ticker = snapshot.market_ticker.clone();
        let new_state = OrderbookState::from_snapshot(snapshot);

        // Update state
        {
            let mut state = self.state.write().expect("state lock poisoned");
            state.insert(ticker.clone(), new_state);
        }

        // Emit update
        if let Some(summary) = self.summary(&ticker) {
            let _ = self.update_sender.send(OrderbookUpdate {
                ticker,
                summary,
                delta: None,
            });
        }
    }

    /// Handle an orderbook delta.
    fn handle_delta(&self, delta: &crate::ws::OrderbookDeltaData, seq: Option<i64>) {
        let ticker = delta.market_ticker.clone();

        let (new_qty, should_emit) = {
            let mut state = self.state.write().expect("state lock poisoned");
            let orderbook = state.entry(ticker.clone()).or_default();

            // Check sequence gap for this specific market
            if let (Some(last), Some(new)) = (orderbook.last_seq(), seq)
                && new != last + 1
            {
                // Gap detected
                let _ = self.gap_sender.send(SequenceGap {
                    ticker: Some(ticker.clone()),
                    expected: last + 1,
                    received: new,
                });
            }

            orderbook.update_seq(seq);
            let new_qty = orderbook.apply_delta(delta);

            (new_qty, orderbook.is_initialized())
        };

        // Only emit updates if we've seen a snapshot
        if should_emit && let Some(summary) = self.summary(&ticker) {
            let _ = self.update_sender.send(OrderbookUpdate {
                ticker,
                summary,
                delta: Some(OrderbookDelta {
                    side: delta.side,
                    price: delta.price,
                    quantity_change: delta.delta,
                    new_quantity: new_qty,
                }),
            });
        }
    }

    /// Check for global sequence gaps.
    fn check_sequence_gap(&self, _seq: i64) {
        // Global sequence tracking would require additional state.
        // For now, we rely on per-market sequence tracking in handle_delta.
    }

    /// Clear all orderbook state.
    ///
    /// Call this on reconnection to reset state before receiving new snapshots.
    pub fn clear(&self) {
        let mut state = self.state.write().expect("state lock poisoned");
        state.clear();
    }

    /// Clear state for a specific market.
    pub fn clear_market(&self, ticker: &str) {
        let mut state = self.state.write().expect("state lock poisoned");
        state.remove(ticker);
    }

    /// Get a summary of the orderbook for a market.
    ///
    /// Returns `None` if the market is not being tracked.
    pub fn summary(&self, ticker: &str) -> Option<OrderbookSummary> {
        let state = self.state.read().expect("state lock poisoned");
        let orderbook = state.get(ticker)?;

        Some(OrderbookSummary {
            ticker: ticker.to_string(),
            best_bid: orderbook.best_yes_bid(),
            best_ask: orderbook.best_yes_ask(),
            spread: orderbook.spread(),
            midpoint: orderbook.midpoint(),
            total_yes_liquidity: orderbook.total_yes_liquidity(),
            total_no_liquidity: orderbook.total_no_liquidity(),
            initialized: orderbook.is_initialized(),
        })
    }

    /// Get the best YES bid for a market.
    ///
    /// Returns (price, quantity) or None.
    pub fn best_bid(&self, ticker: &str) -> Option<(i64, i64)> {
        let state = self.state.read().expect("state lock poisoned");
        state.get(ticker)?.best_yes_bid()
    }

    /// Get the best YES ask for a market.
    ///
    /// Returns (price, quantity) or None.
    pub fn best_ask(&self, ticker: &str) -> Option<(i64, i64)> {
        let state = self.state.read().expect("state lock poisoned");
        state.get(ticker)?.best_yes_ask()
    }

    /// Get the spread for a market in cents.
    pub fn spread(&self, ticker: &str) -> Option<i64> {
        let state = self.state.read().expect("state lock poisoned");
        state.get(ticker)?.spread()
    }

    /// Get the midpoint price for a market.
    pub fn midpoint(&self, ticker: &str) -> Option<f64> {
        let state = self.state.read().expect("state lock poisoned");
        state.get(ticker)?.midpoint()
    }

    /// Get the quantity at a specific price level.
    pub fn depth_at_price(&self, ticker: &str, side: Side, price: i64) -> i64 {
        let state = self.state.read().expect("state lock poisoned");
        state
            .get(ticker)
            .map(|ob| ob.depth_at_price(side, price))
            .unwrap_or(0)
    }

    /// Get the list of tracked markets.
    pub fn tracked_markets(&self) -> Vec<String> {
        let state = self.state.read().expect("state lock poisoned");
        state.keys().cloned().collect()
    }

    /// Check if a market has been initialized with a snapshot.
    pub fn is_initialized(&self, ticker: &str) -> bool {
        let state = self.state.read().expect("state lock poisoned");
        state.get(ticker).is_some_and(|ob| ob.is_initialized())
    }

    /// Subscribe to orderbook updates.
    ///
    /// Returns a receiver that will receive updates for all tracked markets.
    pub fn update_receiver(&self) -> broadcast::Receiver<OrderbookUpdate> {
        self.update_sender.subscribe()
    }

    /// Subscribe to sequence gap notifications.
    ///
    /// Returns a receiver that will receive notifications when sequence
    /// gaps are detected, indicating potentially missed messages.
    pub fn gap_receiver(&self) -> broadcast::Receiver<SequenceGap> {
        self.gap_sender.subscribe()
    }
}

impl std::fmt::Debug for OrderbookAggregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.read().expect("state lock poisoned");
        f.debug_struct("OrderbookAggregator")
            .field("tracked_markets", &state.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::{OrderbookDeltaData, OrderbookSnapshotData};

    #[test]
    fn test_aggregator_new() {
        let agg = OrderbookAggregator::new();
        assert!(agg.tracked_markets().is_empty());
    }

    #[test]
    fn test_handle_snapshot() {
        let agg = OrderbookAggregator::new();

        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200]]),
            yes_dollars: None,
            no: Some(vec![[55, 150]]),
            no_dollars: None,
        };

        agg.handle_snapshot(&snapshot);

        assert!(agg.is_initialized("TEST"));
        assert_eq!(agg.best_bid("TEST"), Some((45, 100)));
        assert_eq!(agg.best_ask("TEST"), Some((45, 150))); // 100 - 55 = 45
    }

    #[test]
    fn test_handle_delta() {
        let agg = OrderbookAggregator::new();

        // First add a snapshot
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]),
            yes_dollars: None,
            no: Some(vec![[55, 150]]),
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot);

        // Then apply a delta
        let delta = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 46,
            delta: 50,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        agg.handle_delta(&delta, Some(1));

        // Best bid should now be 46
        assert_eq!(agg.best_bid("TEST"), Some((46, 50)));
    }

    #[test]
    fn test_clear() {
        let agg = OrderbookAggregator::new();

        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]),
            yes_dollars: None,
            no: None,
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot);

        assert!(!agg.tracked_markets().is_empty());

        agg.clear();

        assert!(agg.tracked_markets().is_empty());
    }

    #[test]
    fn test_clear_market() {
        let agg = OrderbookAggregator::new();

        let snapshot1 = OrderbookSnapshotData {
            market_ticker: "TEST1".to_string(),
            yes: Some(vec![[45, 100]]),
            yes_dollars: None,
            no: None,
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot1);

        let snapshot2 = OrderbookSnapshotData {
            market_ticker: "TEST2".to_string(),
            yes: Some(vec![[50, 200]]),
            yes_dollars: None,
            no: None,
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot2);

        assert_eq!(agg.tracked_markets().len(), 2);

        agg.clear_market("TEST1");

        assert_eq!(agg.tracked_markets().len(), 1);
        assert!(agg.summary("TEST2").is_some());
        assert!(agg.summary("TEST1").is_none());
    }

    #[test]
    fn test_summary() {
        let agg = OrderbookAggregator::new();

        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200]]),
            yes_dollars: None,
            no: Some(vec![[53, 150]]), // YES ask at 47
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot);

        let summary = agg.summary("TEST").unwrap();

        assert_eq!(summary.ticker, "TEST");
        assert_eq!(summary.best_bid, Some((45, 100)));
        assert_eq!(summary.best_ask, Some((47, 150)));
        assert_eq!(summary.spread, Some(2));
        assert_eq!(summary.midpoint, Some(46.0));
        assert_eq!(summary.total_yes_liquidity, 300);
        assert_eq!(summary.total_no_liquidity, 150);
        assert!(summary.initialized);
    }

    #[test]
    fn test_unknown_market_returns_none() {
        let agg = OrderbookAggregator::new();

        assert!(agg.summary("UNKNOWN").is_none());
        assert!(agg.best_bid("UNKNOWN").is_none());
        assert!(agg.best_ask("UNKNOWN").is_none());
        assert!(agg.spread("UNKNOWN").is_none());
        assert!(agg.midpoint("UNKNOWN").is_none());
        assert!(!agg.is_initialized("UNKNOWN"));
    }

    #[test]
    fn test_depth_at_price() {
        let agg = OrderbookAggregator::new();

        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200]]),
            yes_dollars: None,
            no: Some(vec![[55, 150]]),
            no_dollars: None,
        };
        agg.handle_snapshot(&snapshot);

        assert_eq!(agg.depth_at_price("TEST", Side::Yes, 45), 100);
        assert_eq!(agg.depth_at_price("TEST", Side::Yes, 44), 200);
        assert_eq!(agg.depth_at_price("TEST", Side::No, 55), 150);
        assert_eq!(agg.depth_at_price("TEST", Side::Yes, 99), 0); // No level
        assert_eq!(agg.depth_at_price("UNKNOWN", Side::Yes, 45), 0); // Unknown market
    }

    #[test]
    fn test_clone_shares_state() {
        let agg1 = OrderbookAggregator::new();
        let agg2 = agg1.clone();

        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]),
            yes_dollars: None,
            no: None,
            no_dollars: None,
        };
        agg1.handle_snapshot(&snapshot);

        // agg2 should see the same state
        assert!(agg2.is_initialized("TEST"));
        assert_eq!(agg2.best_bid("TEST"), Some((45, 100)));
    }
}
