//! Example: Stream with automatic reconnection handling.
//!
//! This example demonstrates how to handle disconnections and reconnect
//! to the Kalshi WebSocket API. It shows the recommended pattern for
//! production applications that need to maintain a persistent connection.
//!
//! # Usage
//!
//! Set the following environment variables:
//! - `KALSHI_API_KEY_ID`: Your Kalshi API key ID
//! - `KALSHI_PRIVATE_KEY_PATH`: Path to your RSA private key PEM file
//! - `KALSHI_ENV`: "demo" or "prod" (defaults to "demo")
//!
//! Then run:
//! ```bash
//! cargo run --example stream_reconnect
//! ```

use std::time::Duration;

use kalshi_trade_rs::{
    auth::KalshiConfig,
    ws::{Channel, ConnectStrategy, KalshiStreamClient, StreamMessage},
};

/// Channels we want to subscribe to.
const CHANNELS: &[Channel] = &[Channel::Ticker, Channel::Trade];

/// Maximum number of reconnection attempts before giving up.
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Delay between reconnection attempts.
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing for logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs=info".parse()?),
        )
        .init();

    // Load configuration from environment
    let config = KalshiConfig::from_env()?;
    println!("Kalshi {:?} environment", config.environment);

    let mut reconnect_attempts = 0;

    // Main reconnection loop
    loop {
        println!("\n--- Connecting to Kalshi WebSocket ---");

        match run_stream(&config).await {
            Ok(reason) => {
                println!("Stream ended cleanly: {}", reason);
                // Clean exit requested - don't reconnect
                break;
            }
            Err(e) => {
                println!("Stream error: {}", e);
                reconnect_attempts += 1;

                if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                    println!("Max reconnection attempts reached, exiting");
                    return Err(e);
                }

                println!(
                    "Reconnecting in {:?} (attempt {}/{})",
                    RECONNECT_DELAY, reconnect_attempts, MAX_RECONNECT_ATTEMPTS
                );
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        }
    }

    Ok(())
}

/// Run the stream until disconnection or error.
///
/// Returns Ok(reason) for clean exits, Err for errors that should trigger reconnection.
async fn run_stream(config: &KalshiConfig) -> Result<String, Box<dyn std::error::Error>> {
    // Connect with retry strategy - this handles initial connection failures
    let client = KalshiStreamClient::connect_with_strategy(config, ConnectStrategy::Retry).await?;

    let mut handle = client.handle();
    println!("Connected!");

    // Subscribe to channels
    let result = handle.subscribe(CHANNELS, None).await?;
    println!("Subscribed with SIDs: {:?}", result.sids);
    println!("Waiting for updates (Ctrl+C to exit)...\n");

    // Track subscription IDs for potential resubscription
    let _subscription_ids = result.sids.clone();

    // Process updates
    loop {
        match handle.update_receiver.recv().await {
            Ok(update) => {
                match &update.msg {
                    // Handle clean close - don't reconnect
                    StreamMessage::Closed { reason } => {
                        return Ok(reason.clone());
                    }

                    // Handle connection loss - trigger reconnection
                    StreamMessage::ConnectionLost { reason } => {
                        return Err(format!("Connection lost: {}", reason).into());
                    }

                    // Handle ticker updates
                    StreamMessage::Ticker(ticker) => {
                        println!(
                            "[TICKER] {} | price: {}¢ | bid: {}¢ | ask: {}¢ | vol: {}",
                            ticker.market_ticker,
                            ticker.price,
                            ticker.yes_bid,
                            ticker.yes_ask,
                            ticker.volume
                        );
                    }

                    // Handle trade updates
                    StreamMessage::Trade(trade) => {
                        println!(
                            "[TRADE]  {} | {} contracts @ {}¢ | taker: {:?}",
                            trade.market_ticker, trade.count, trade.yes_price, trade.taker_side
                        );
                    }

                    // Handle other message types
                    _ => {
                        println!("[OTHER]  {:?}", update.msg);
                    }
                }
            }

            // Handle broadcast channel errors
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                println!("[WARN] Dropped {} messages (slow consumer)", n);
                // Continue processing - this is not fatal
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                // Channel closed without disconnect event
                return Err("Broadcast channel closed unexpectedly".into());
            }
        }
    }
}
