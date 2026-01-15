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
    GetMarketsParams, KalshiClient, MarketFilterStatus,
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

    // First, fetch open markets via REST API and select the most active ones
    println!("Fetching open markets...");

    let rest_client = KalshiClient::new(config.clone())?;

    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(200);

    let markets_response = rest_client.get_markets_with_params(params).await?;

    // Sort by volume (descending) and take the top 5 most active markets
    let mut markets = markets_response.markets;

    if markets.is_empty() {
        println!("No open markets found!");
        return Ok(());
    }

    markets.sort_by(|a, b| b.volume.unwrap_or(0).cmp(&a.volume.unwrap_or(0)));

    let selected_markets: Vec<_> = markets.into_iter().take(5).collect();

    println!(
        "Selected {} markets to subscribe to:",
        selected_markets.len()
    );

    for market in &selected_markets {
        println!(
            "  - {} (vol: {})",
            market.ticker,
            market.volume.unwrap_or(0)
        );
    }

    let market_tickers: Vec<String> = selected_markets.iter().map(|m| m.ticker.clone()).collect();

    // Connect to WebSocket
    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();

    println!("\nConnected! Subscribing to ticker updates...");

    // Subscribe to ticker and trade channels for the markets we found
    let ticker_refs: Vec<&str> = market_tickers.iter().map(|s| s.as_str()).collect();

    handle.subscribe(Channel::Ticker, &ticker_refs).await?;
    handle.subscribe(Channel::Trade, &ticker_refs).await?;

    println!(
        "Subscribed to {} markets",
        handle.markets(Channel::Ticker).len()
    );
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
    handle.unsubscribe_all(Channel::Ticker).await?;
    handle.unsubscribe_all(Channel::Trade).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
