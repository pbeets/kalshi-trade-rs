//! Error types for the Kalshi API client.
//!
//! This module provides the unified [`enum@Error`] type used throughout the crate,
//! along with API limit constants.
//!
//! # Error Categories
//!
//! Errors fall into several categories:
//!
//! - **Network errors**: [`Error::Http`], [`Error::WebSocket`] - connection failures,
//!   timeouts, TLS errors
//! - **Authentication errors**: [`Error::Auth`], [`Error::InvalidPrivateKey`] - credential
//!   issues, signature failures
//! - **API errors**: [`Error::Api`] - server-side rejections (invalid tickers, insufficient
//!   balance, rate limits)
//! - **Validation errors**: [`Error::InvalidPrice`], [`Error::BatchSizeExceeded`], etc. -
//!   client-side validation before requests are sent
//! - **Configuration errors**: [`Error::MissingEnvVar`], [`Error::PrivateKeyFileError`] -
//!   setup and initialization issues
//!
//! # Example
//!
//! ```no_run
//! use kalshi_trade_rs::{KalshiClient, KalshiConfig, Error};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = KalshiConfig::from_env()?;
//!     let client = KalshiClient::new(config)?;
//!
//!     match client.get_balance().await {
//!         Ok(balance) => println!("Balance: {} cents", balance.balance),
//!         Err(Error::Auth(msg)) => {
//!             // Authentication failed - check credentials
//!             eprintln!("Authentication error: {}", msg);
//!         }
//!         Err(Error::Http(e)) => {
//!             // Network error - may be transient, consider retry
//!             eprintln!("Network error: {}", e);
//!         }
//!         Err(Error::Api(msg)) => {
//!             // Server rejected request - check the message for details
//!             eprintln!("API error: {}", msg);
//!         }
//!         Err(e) => {
//!             eprintln!("Other error: {}", e);
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Handling WebSocket Errors
//!
//! For WebSocket connections, errors are delivered via [`crate::ws::StreamMessage::ConnectionLost`]
//! on the update receiver. See the [`ws`](crate::ws) module for reconnection patterns.

use std::fmt;
use thiserror::Error;

/// Maximum orders per batch request.
pub const MAX_BATCH_SIZE: usize = 20;

/// Maximum market tickers in batch candlesticks request.
pub const MAX_BATCH_CANDLESTICKS_TICKERS: usize = 100;

/// Maximum event tickers in comma-separated filter.
pub const MAX_EVENT_TICKERS: usize = 10;

/// Maximum percentiles in forecast history request.
pub const MAX_FORECAST_PERCENTILES: usize = 10;

/// Reason why a WebSocket connection was lost or couldn't be established.
///
/// This enum provides detailed information about disconnection causes,
/// allowing clients to handle different scenarios appropriately (e.g.,
/// retry on transient errors, alert on auth failures).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason {
    // === Connection Establishment Failures ===
    /// Session failed to start (crashed before signaling ready).
    SessionStartupFailed,
    /// Session didn't signal ready within the timeout period.
    SessionStartupTimeout,

    // === Clean Shutdowns (expected) ===
    /// Client requested close via shutdown().
    ClientClosed,
    /// Server sent a WebSocket close frame.
    ServerClosed,
    /// All subscription channels were closed.
    AllChannelsClosed,

    // === Connection Lost (unexpected) ===
    /// Session's command channel closed (session died unexpectedly).
    SessionDied,
    /// No response to our ping within the configured timeout.
    PingTimeout,
    /// No heartbeat received from Kalshi server within timeout.
    ServerHeartbeatTimeout,
    /// Failed to send health check message (ping or pong).
    HealthCheckFailed(String),
    /// WebSocket I/O error during operation.
    IoError(String),
    /// WebSocket protocol error.
    ProtocolError(String),
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionStartupFailed => write!(f, "session failed to start"),
            Self::SessionStartupTimeout => write!(f, "session startup timeout"),
            Self::ClientClosed => write!(f, "client requested close"),
            Self::ServerClosed => write!(f, "server closed connection"),
            Self::AllChannelsClosed => write!(f, "all channels closed"),
            Self::SessionDied => write!(f, "session died unexpectedly"),
            Self::PingTimeout => write!(f, "ping timeout"),
            Self::ServerHeartbeatTimeout => write!(f, "server heartbeat timeout"),
            Self::HealthCheckFailed(msg) => write!(f, "health check failed: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::ProtocolError(msg) => write!(f, "protocol error: {}", msg),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    #[error("WebSocket disconnected: {0}")]
    Disconnected(DisconnectReason),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Failed to read private key file '{0}': {1}")]
    PrivateKeyFileError(String, String),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),

    #[error("Batch size {0} exceeds maximum of {MAX_BATCH_SIZE}")]
    BatchSizeExceeded(usize),

    #[error("Market tickers count {0} exceeds maximum of {MAX_BATCH_CANDLESTICKS_TICKERS}")]
    TooManyMarketTickers(usize),

    #[error("Event tickers count {0} exceeds maximum of {MAX_EVENT_TICKERS}")]
    TooManyEventTickers(usize),

    #[error("Percentiles count {0} exceeds maximum of {MAX_FORECAST_PERCENTILES}")]
    TooManyPercentiles(usize),

    #[error("Percentile value {0} out of range: must be between 0 and 10000")]
    PercentileOutOfRange(i32),

    #[error("Cannot specify both series_ticker and collection_ticker")]
    MutuallyExclusiveParams,

    #[error("Invalid timestamp range: start_ts ({0}) must be less than end_ts ({1})")]
    InvalidTimestampRange(i64, i64),

    #[error("Invalid price {0}: must be between 1 and 99")]
    InvalidPrice(i64),

    #[error("Invalid quantity {0}: must be positive")]
    InvalidQuantity(i64),

    #[error("Market tickers required for channels: {0}")]
    MissingMarketTickers(String),

    #[error("Invalid subaccount ID {0}: must be between 0 and 32")]
    InvalidSubaccountId(i32),

    #[error("Cannot transfer between same subaccount")]
    SameSubaccountTransfer,

    #[error("Transfer amount must be positive, got {0}")]
    InvalidTransferAmount(i64),

    #[error("Invalid limit {0}: must be between {1} and {2}")]
    InvalidLimit(i64, i64, i64),

    #[error("Invalid contracts limit {0}: must be at least 1")]
    InvalidContractsLimit(i64),

    #[error("Invalid contracts {0}: must be positive")]
    InvalidContracts(i64),

    #[error("Invalid target cost {0}: must be positive")]
    InvalidTargetCost(i64),

    #[error("Invalid target cost dollars {0}: must be positive")]
    InvalidTargetCostDollars(f64),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(Box::new(err))
    }
}
