//! Internal orderbook state for a single market.

use std::collections::BTreeMap;

use crate::models::Side;
use crate::ws::{OrderbookDeltaData, OrderbookSnapshotData, PriceLevel};

/// Internal orderbook state for a single market.
///
/// Uses `BTreeMap` for O(log n) best price queries.
#[derive(Debug, Clone, Default)]
pub(crate) struct OrderbookState {
    /// YES side price levels: price_cents -> quantity
    yes_levels: BTreeMap<i64, i64>,
    /// NO side price levels: price_cents -> quantity
    no_levels: BTreeMap<i64, i64>,
    /// Last sequence number seen
    last_seq: Option<i64>,
    /// Whether we've received the initial snapshot
    initialized: bool,
}

impl OrderbookState {
    /// Create a new empty orderbook state.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize from a snapshot.
    pub fn from_snapshot(snapshot: &OrderbookSnapshotData) -> Self {
        let yes_levels = snapshot
            .yes
            .as_ref()
            .map(|levels| levels_to_btree(levels))
            .unwrap_or_default();

        let no_levels = snapshot
            .no
            .as_ref()
            .map(|levels| levels_to_btree(levels))
            .unwrap_or_default();

        Self {
            yes_levels,
            no_levels,
            last_seq: None,
            initialized: true,
        }
    }

    /// Apply a delta update.
    ///
    /// Returns the new quantity at the price level after applying the delta.
    pub fn apply_delta(&mut self, delta: &OrderbookDeltaData) -> i64 {
        let levels = match delta.side {
            Side::Yes => &mut self.yes_levels,
            Side::No => &mut self.no_levels,
        };

        let current = levels.get(&delta.price).copied().unwrap_or(0);
        let new_qty = current + delta.delta;

        if new_qty <= 0 {
            levels.remove(&delta.price);
            0
        } else {
            levels.insert(delta.price, new_qty);
            new_qty
        }
    }

    /// Update the last sequence number.
    pub fn update_seq(&mut self, seq: Option<i64>) {
        if let Some(s) = seq {
            self.last_seq = Some(s);
        }
    }

    /// Get the last sequence number.
    pub fn last_seq(&self) -> Option<i64> {
        self.last_seq
    }

    /// Whether the orderbook has been initialized with a snapshot.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the best YES bid (highest price someone will pay for YES).
    ///
    /// Returns (price, quantity) or None if no bids.
    pub fn best_yes_bid(&self) -> Option<(i64, i64)> {
        self.yes_levels
            .iter()
            .next_back()
            .map(|(&price, &qty)| (price, qty))
    }

    /// Get the best YES ask (lowest price to buy YES).
    ///
    /// In Kalshi, the best YES ask is derived from the best NO bid:
    /// YES ask = 100 - NO bid price
    ///
    /// Returns (price, quantity) or None if no asks.
    pub fn best_yes_ask(&self) -> Option<(i64, i64)> {
        // Best NO bid = highest NO bid price
        // Someone bidding 55 for NO means they'll sell YES at 45
        self.no_levels
            .iter()
            .next_back()
            .map(|(&no_price, &qty)| (100 - no_price, qty))
    }

    /// Get the best NO bid (highest price someone will pay for NO).
    ///
    /// Returns (price, quantity) or None if no bids.
    #[allow(dead_code)]
    pub fn best_no_bid(&self) -> Option<(i64, i64)> {
        self.no_levels
            .iter()
            .next_back()
            .map(|(&price, &qty)| (price, qty))
    }

    /// Get the best NO ask (lowest price to buy NO).
    ///
    /// In Kalshi, the best NO ask is derived from the best YES bid:
    /// NO ask = 100 - YES bid price
    ///
    /// Returns (price, quantity) or None if no asks.
    #[allow(dead_code)]
    pub fn best_no_ask(&self) -> Option<(i64, i64)> {
        self.yes_levels
            .iter()
            .next_back()
            .map(|(&yes_price, &qty)| (100 - yes_price, qty))
    }

    /// Get the YES spread (ask - bid) in cents.
    ///
    /// Returns None if either bid or ask is unavailable.
    pub fn spread(&self) -> Option<i64> {
        let bid = self.best_yes_bid()?.0;
        let ask = self.best_yes_ask()?.0;
        Some(ask - bid)
    }

    /// Get the YES midpoint price.
    ///
    /// Returns None if either bid or ask is unavailable.
    pub fn midpoint(&self) -> Option<f64> {
        let bid = self.best_yes_bid()?.0 as f64;
        let ask = self.best_yes_ask()?.0 as f64;
        Some((bid + ask) / 2.0)
    }

    /// Get the quantity at a specific price level.
    pub fn depth_at_price(&self, side: Side, price: i64) -> i64 {
        let levels = match side {
            Side::Yes => &self.yes_levels,
            Side::No => &self.no_levels,
        };
        levels.get(&price).copied().unwrap_or(0)
    }

    /// Get total YES liquidity (sum of all YES bid quantities).
    pub fn total_yes_liquidity(&self) -> i64 {
        self.yes_levels.values().sum()
    }

    /// Get total NO liquidity (sum of all NO bid quantities).
    pub fn total_no_liquidity(&self) -> i64 {
        self.no_levels.values().sum()
    }

    /// Get all YES levels (price -> quantity).
    #[allow(dead_code)]
    pub fn yes_levels(&self) -> &BTreeMap<i64, i64> {
        &self.yes_levels
    }

    /// Get all NO levels (price -> quantity).
    #[allow(dead_code)]
    pub fn no_levels(&self) -> &BTreeMap<i64, i64> {
        &self.no_levels
    }

    /// Clear the orderbook state.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.yes_levels.clear();
        self.no_levels.clear();
        self.last_seq = None;
        self.initialized = false;
    }
}

/// Convert price levels array to BTreeMap.
fn levels_to_btree(levels: &[PriceLevel]) -> BTreeMap<i64, i64> {
    levels
        .iter()
        .filter(|[_, qty]| *qty > 0)
        .map(|[price, qty]| (*price, *qty))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_snapshot() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200]]),
            yes_dollars: None,
            no: Some(vec![[55, 150], [56, 250]]),
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        assert!(state.is_initialized());
        assert_eq!(state.depth_at_price(Side::Yes, 45), 100);
        assert_eq!(state.depth_at_price(Side::Yes, 44), 200);
        assert_eq!(state.depth_at_price(Side::No, 55), 150);
        assert_eq!(state.depth_at_price(Side::No, 56), 250);
    }

    #[test]
    fn test_apply_delta_add() {
        let mut state = OrderbookState::new();

        let delta = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };

        let new_qty = state.apply_delta(&delta);
        assert_eq!(new_qty, 100);
        assert_eq!(state.depth_at_price(Side::Yes, 45), 100);
    }

    #[test]
    fn test_apply_delta_increase() {
        let mut state = OrderbookState::new();

        // Add initial
        let delta1 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        state.apply_delta(&delta1);

        // Increase
        let delta2 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 50,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        let new_qty = state.apply_delta(&delta2);
        assert_eq!(new_qty, 150);
    }

    #[test]
    fn test_apply_delta_decrease() {
        let mut state = OrderbookState::new();

        // Add initial
        let delta1 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        state.apply_delta(&delta1);

        // Decrease
        let delta2 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: -30,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        let new_qty = state.apply_delta(&delta2);
        assert_eq!(new_qty, 70);
    }

    #[test]
    fn test_apply_delta_remove_level() {
        let mut state = OrderbookState::new();

        // Add initial
        let delta1 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        state.apply_delta(&delta1);

        // Remove (delta brings to zero)
        let delta2 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: -100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        let new_qty = state.apply_delta(&delta2);
        assert_eq!(new_qty, 0);
        assert_eq!(state.depth_at_price(Side::Yes, 45), 0);
    }

    #[test]
    fn test_apply_delta_negative_removes() {
        let mut state = OrderbookState::new();

        // Add initial
        let delta1 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: 50,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        state.apply_delta(&delta1);

        // Remove more than exists (should remove level)
        let delta2 = OrderbookDeltaData {
            market_ticker: "TEST".to_string(),
            price: 45,
            delta: -100,
            side: Side::Yes,
            price_dollars: None,
            client_order_id: None,
        };
        let new_qty = state.apply_delta(&delta2);
        assert_eq!(new_qty, 0);
        assert_eq!(state.depth_at_price(Side::Yes, 45), 0);
    }

    #[test]
    fn test_best_yes_bid() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200], [43, 50]]),
            yes_dollars: None,
            no: None,
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        // Best bid is highest price
        assert_eq!(state.best_yes_bid(), Some((45, 100)));
    }

    #[test]
    fn test_best_yes_ask() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: None,
            yes_dollars: None,
            no: Some(vec![[55, 150], [56, 250]]),
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        // Best NO bid at 56 means YES ask at 100-56=44
        assert_eq!(state.best_yes_ask(), Some((44, 250)));
    }

    #[test]
    fn test_spread() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]), // Best bid at 45
            yes_dollars: None,
            no: Some(vec![[53, 150]]), // Best NO bid at 53 -> YES ask at 47
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        // Spread = ask - bid = 47 - 45 = 2
        assert_eq!(state.spread(), Some(2));
    }

    #[test]
    fn test_midpoint() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]), // Best bid at 45
            yes_dollars: None,
            no: Some(vec![[53, 150]]), // Best NO bid at 53 -> YES ask at 47
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        // Midpoint = (45 + 47) / 2 = 46
        assert_eq!(state.midpoint(), Some(46.0));
    }

    #[test]
    fn test_total_liquidity() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100], [44, 200]]),
            yes_dollars: None,
            no: Some(vec![[55, 150], [56, 250]]),
            no_dollars: None,
        };

        let state = OrderbookState::from_snapshot(&snapshot);

        assert_eq!(state.total_yes_liquidity(), 300);
        assert_eq!(state.total_no_liquidity(), 400);
    }

    #[test]
    fn test_clear() {
        let snapshot = OrderbookSnapshotData {
            market_ticker: "TEST".to_string(),
            yes: Some(vec![[45, 100]]),
            yes_dollars: None,
            no: Some(vec![[55, 150]]),
            no_dollars: None,
        };

        let mut state = OrderbookState::from_snapshot(&snapshot);
        state.update_seq(Some(42));
        assert!(state.is_initialized());

        state.clear();

        assert!(!state.is_initialized());
        assert_eq!(state.last_seq(), None);
        assert_eq!(state.total_yes_liquidity(), 0);
        assert_eq!(state.total_no_liquidity(), 0);
    }

    #[test]
    fn test_empty_orderbook() {
        let state = OrderbookState::new();

        assert!(!state.is_initialized());
        assert_eq!(state.best_yes_bid(), None);
        assert_eq!(state.best_yes_ask(), None);
        assert_eq!(state.spread(), None);
        assert_eq!(state.midpoint(), None);
    }
}
