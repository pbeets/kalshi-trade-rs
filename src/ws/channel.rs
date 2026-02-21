//! WebSocket channel definitions for Kalshi streaming API.

use serde::{Deserialize, Serialize};

/// Available WebSocket channels for streaming data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// Real-time orderbook price level changes (delta updates)
    OrderbookDelta,
    /// Market price, volume, and open interest updates
    Ticker,
    /// Public trade notifications
    Trade,
    /// Personal order fill notifications (requires authentication)
    Fill,
    /// Real-time portfolio position updates (requires authentication)
    MarketPositions,
    /// RFQ and quote notifications (requires authentication)
    Communications,
    /// Market state changes and event lifecycle
    #[serde(rename = "market_lifecycle_v2")]
    MarketLifecycle,
    /// Order group lifecycle events (requires authentication)
    OrderGroupUpdates,
    /// Multivariate collection lookup notifications
    Multivariate,
}

impl Channel {
    /// Returns true if this channel requires authentication.
    pub fn requires_auth(&self) -> bool {
        matches!(
            self,
            Self::Fill | Self::MarketPositions | Self::Communications | Self::OrderGroupUpdates
        )
    }

    /// Returns true if this channel requires at least one market ticker.
    ///
    /// Only [`OrderbookDelta`](Self::OrderbookDelta) requires market tickers.
    /// Other market data channels (`Ticker`, `Trade`, `MarketLifecycle`,
    /// `Multivariate`) support subscribing with an empty ticker list to
    /// receive updates for all markets. User-scoped channels (`Fill`,
    /// `MarketPositions`, `Communications`) never require tickers.
    pub fn requires_market_ticker(&self) -> bool {
        matches!(self, Self::OrderbookDelta)
    }

    /// Returns the wire format name for this channel.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OrderbookDelta => "orderbook_delta",
            Self::Ticker => "ticker",
            Self::Trade => "trade",
            Self::Fill => "fill",
            Self::MarketPositions => "market_positions",
            Self::Communications => "communications",
            Self::MarketLifecycle => "market_lifecycle_v2",
            Self::OrderGroupUpdates => "order_group_updates",
            Self::Multivariate => "multivariate",
        }
    }
}
