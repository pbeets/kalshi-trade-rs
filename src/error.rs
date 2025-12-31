use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

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
}

pub type Result<T> = std::result::Result<T, Error>;
