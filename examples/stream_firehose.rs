//! Example: Stream ticker data from Kalshi WebSocket API.
//!
//! This example demonstrates how to connect to the Kalshi WebSocket API
//! and subscribe to all Ticker, Trade, MarketLifecycle, and Multivariate markets
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
//! cargo run --example stream_firehose
//! ```

use kalshi_trade_rs::{Channel, KalshiConfig, KalshiStreamClient, StreamMessage};
use std::time::Duration;
use tokio::time::timeout;

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

    println!("\nConnected! Subscribing to ticker updates...");

    handle.subscribe(Channel::Ticker, &[]).await?;
    handle.subscribe(Channel::Trade, &[]).await?;
    handle.subscribe(Channel::MarketLifecycle, &[]).await?;
    handle.subscribe(Channel::Multivariate, &[]).await?;

    println!("Subscribed to all Ticker, Trade, MarketLifecycle, and Multivariate markets",);

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
                StreamMessage::ConnectionLost { reason, .. } => {
                    println!("[CONNECTION LOST] {}", reason);
                    break;
                }
                StreamMessage::Ticker(ticker) => {
                    println!(
                        "[TICKER] {} | price: {} | bid: {} | ask: {} | vol: {}",
                        ticker.market_ticker,
                        ticker.price_dollars,
                        ticker.yes_bid_dollars,
                        ticker.yes_ask_dollars,
                        ticker.volume_fp
                    );
                }
                StreamMessage::Trade(trade) => {
                    println!(
                        "[TRADE]  {} | {} contracts @ {} | taker: {:?}",
                        trade.market_ticker,
                        trade.count_fp,
                        trade.yes_price_dollars,
                        trade.taker_side
                    );
                }
                StreamMessage::MarketLifecycle(lifecycle) => {
                    println!(
                        "[LIFECYCLE] {} | event={:?} | open={:?} | close={:?} | result={:?}",
                        lifecycle.market_ticker,
                        lifecycle.event_type,
                        lifecycle.open_ts,
                        lifecycle.close_ts,
                        lifecycle.result
                    );
                    if let Some(meta) = &lifecycle.additional_metadata {
                        println!("            name={:?} | title={:?}", meta.name, meta.title);
                    }
                }
                StreamMessage::MultivariateLookup(mv) => {
                    println!(
                        "[MULTIVARIATE] collection={} | event={} | market={}",
                        mv.collection_ticker, mv.event_ticker, mv.market_ticker
                    );
                    for leg in &mv.selected_markets {
                        println!(
                            "  leg: market={:?} | side={:?}",
                            leg.market_ticker, leg.side
                        );
                    }
                }
                StreamMessage::MarketPosition(pos) => {
                    println!(
                        "[POSITION] {} | position={:?} | cost={} | pnl={}",
                        pos.market_ticker, pos.position, pos.position_cost, pos.realized_pnl
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
    handle.unsubscribe_all(Channel::MarketLifecycle).await?;
    handle.unsubscribe_all(Channel::Multivariate).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
