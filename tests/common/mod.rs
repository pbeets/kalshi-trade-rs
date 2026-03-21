use kalshi_trade_rs::{KalshiClient, KalshiConfig};

/// Initialize a KalshiClient pointed at the demo environment.
///
/// Reads credentials from environment variables (loaded from .env via dotenvy).
/// Panics if credentials are missing — tests that call this require a valid
/// demo API key.
pub fn test_client() -> KalshiClient {
    // Load .env file if present (ignore errors — CI may use real env vars)
    let _ = dotenvy::dotenv();

    let config = KalshiConfig::from_env().expect(
        "Missing KALSHI_API_KEY_ID or KALSHI_PRIVATE_KEY_PATH. \
         Copy .env.blank to .env and fill in your demo credentials.",
    );

    assert_eq!(
        config.environment,
        kalshi_trade_rs::Environment::Demo,
        "Integration tests must run against the demo environment"
    );

    KalshiClient::new(config).expect("Failed to create KalshiClient")
}
