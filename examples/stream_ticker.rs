//! Example: Stream ticker data from Kalshi WebSocket API.
//!
//! This example demonstrates how to connect to the Kalshi WebSocket API
//! and subscribe to ticker updates for specific markets.
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
//! cargo run --example stream_ticker
//! ```

use std::time::Duration;
use tokio::time::timeout;

use kalshi_trade_rs::{
    auth::KalshiConfig,
    ws::{Channel, KalshiStreamClient, StreamMessage},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing for logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs=debug".parse()?),
        )
        .init();

    // Load configuration from environment
    let config = KalshiConfig::from_env()?;

    println!(
        "Connecting to Kalshi {:?} environment...",
        config.environment
    );

    // Connect to WebSocket
    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();

    println!("Connected! Subscribing to ticker updates...");

    // Subscribe to ticker and trade channels
    // You can specify market tickers or omit for all markets
    let result = handle
        .subscribe(
            &[Channel::Ticker, Channel::Trade],
            None, // Subscribe to all markets
        )
        .await?;

    println!("Subscribed with SIDs: {:?}", result.sids);
    println!("Waiting for updates (Ctrl+C to exit)...\n");

    // Process updates for 60 seconds or until interrupted
    let deadline = Duration::from_secs(60);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > deadline {
            println!("\nReached time limit, shutting down...");
            break;
        }

        match timeout(Duration::from_secs(5), handle.update_receiver.recv()).await {
            Ok(Ok(update)) => match &update.msg {
                StreamMessage::Closed { reason } => {
                    println!("[CLOSED] {}", reason);
                    break;
                }
                StreamMessage::ConnectionLost { reason } => {
                    println!("[CONNECTION LOST] {}", reason);
                    break;
                }
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
                StreamMessage::Trade(trade) => {
                    println!(
                        "[TRADE]  {} | {} contracts @ {}¢ | taker: {:?}",
                        trade.market_ticker, trade.count, trade.yes_price, trade.taker_side
                    );
                }
                _ => {
                    println!("[OTHER]  {:?}", update.msg);
                }
            },
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                println!("[WARN] Dropped {} messages (slow consumer)", n);
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                println!("[ERROR] Channel closed");
                break;
            }
            Err(_) => {
                // Timeout - no messages in 5 seconds
                println!("[INFO] No updates in last 5 seconds...");
            }
        }
    }

    // Unsubscribe and shut down
    println!("Unsubscribing...");
    handle.unsubscribe(&result.sids).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
