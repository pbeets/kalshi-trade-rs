//! Error types and API limit constants.

use thiserror::Error;

/// Maximum orders per batch request.
pub const MAX_BATCH_SIZE: usize = 20;

/// Maximum market tickers in batch candlesticks request.
pub const MAX_BATCH_CANDLESTICKS_TICKERS: usize = 100;

/// Maximum event tickers in comma-separated filter.
pub const MAX_EVENT_TICKERS: usize = 10;

/// Maximum percentiles in forecast history request.
pub const MAX_FORECAST_PERCENTILES: usize = 10;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

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
