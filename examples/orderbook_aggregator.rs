//! Example: Orderbook aggregator for live orderbook state.
//!
//! This example demonstrates how to use the OrderbookAggregator to maintain
//! live orderbook state from WebSocket delta updates.
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
//! cargo run --example orderbook_aggregator
//! ```

use std::time::Duration;
use tokio::time::timeout;

use kalshi_trade_rs::{
    GetMarketsParams, KalshiClient, MarketFilterStatus, OrderbookAggregator,
    auth::KalshiConfig,
    ws::{Channel, KalshiStreamClient},
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

    // Sort by volume (descending) and take the top 3 most active markets
    let mut markets = markets_response.markets;

    if markets.is_empty() {
        println!("No open markets found!");
        return Ok(());
    }

    markets.sort_by(|a, b| b.volume.unwrap_or(0).cmp(&a.volume.unwrap_or(0)));

    let selected_markets: Vec<_> = markets.into_iter().take(3).collect();

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
    let handle = client.handle();

    println!("\nConnected! Subscribing to orderbook updates...");

    // Subscribe to orderbook delta channel
    let ticker_refs: Vec<&str> = market_tickers.iter().map(|s| s.as_str()).collect();

    let mut sub_handle = handle.clone();
    sub_handle
        .subscribe(Channel::OrderbookDelta, &ticker_refs)
        .await?;

    println!(
        "Subscribed to {} markets",
        sub_handle.markets(Channel::OrderbookDelta).len()
    );

    // Create orderbook aggregator
    let aggregator = OrderbookAggregator::new();

    // Get update receiver for push-based updates (before spawning processor)
    let mut update_receiver = aggregator.update_receiver();
    let mut gap_receiver = aggregator.gap_receiver();

    // Spawn aggregator processor with cloned handle
    let agg_clone = aggregator.clone();
    let process_handle = handle.clone();
    tokio::spawn(async move {
        agg_clone.process_updates(process_handle).await;
    });

    // Spawn gap monitor
    tokio::spawn(async move {
        while let Ok(gap) = gap_receiver.recv().await {
            println!(
                "[GAP] {:?}: expected seq {}, got {}",
                gap.ticker.as_deref().unwrap_or("global"),
                gap.expected,
                gap.received
            );
        }
    });

    println!("Waiting for updates (Ctrl+C to exit)...\n");

    // Demonstrate both pull and push patterns
    let deadline = Duration::from_secs(60);
    let start = std::time::Instant::now();
    let mut last_summary_time = std::time::Instant::now();

    loop {
        if start.elapsed() > deadline {
            println!("\nReached time limit, shutting down...");
            break;
        }

        // Push-based: React to orderbook changes
        match timeout(Duration::from_millis(500), update_receiver.recv()).await {
            Ok(Ok(update)) => {
                if let Some(delta) = &update.delta {
                    println!(
                        "[DELTA] {} | {:?} @ {}¢: {:+} -> {} | spread: {:?}¢ | mid: {:.1}¢",
                        update.ticker,
                        delta.side,
                        delta.price,
                        delta.quantity_change,
                        delta.new_quantity,
                        update.summary.spread,
                        update.summary.midpoint.unwrap_or(0.0)
                    );
                } else {
                    // Snapshot received
                    println!(
                        "[SNAPSHOT] {} | bid: {:?} | ask: {:?} | spread: {:?}¢",
                        update.ticker,
                        update.summary.best_bid,
                        update.summary.best_ask,
                        update.summary.spread,
                    );
                }
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                println!("[WARN] Dropped {} orderbook updates (slow consumer)", n);
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                println!("[ERROR] Update channel closed");
                break;
            }
            Err(_) => {
                // Timeout - no updates in 500ms
            }
        }

        // Pull-based: Print summary every 5 seconds
        if last_summary_time.elapsed() > Duration::from_secs(5) {
            last_summary_time = std::time::Instant::now();
            println!("\n--- Orderbook Summary ---");
            for ticker in &market_tickers {
                if let Some(summary) = aggregator.summary(ticker) {
                    if summary.initialized {
                        println!(
                            "{}: bid={:?} ask={:?} spread={:?}¢ mid={:.1}¢ yes_liq={} no_liq={}",
                            ticker,
                            summary.best_bid,
                            summary.best_ask,
                            summary.spread,
                            summary.midpoint.unwrap_or(0.0),
                            summary.total_yes_liquidity,
                            summary.total_no_liquidity
                        );
                    } else {
                        println!("{}: waiting for snapshot...", ticker);
                    }
                } else {
                    println!("{}: not tracked", ticker);
                }
            }
            println!("-------------------------\n");
        }
    }

    // Unsubscribe and shut down
    println!("Unsubscribing...");
    sub_handle.unsubscribe_all(Channel::OrderbookDelta).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
