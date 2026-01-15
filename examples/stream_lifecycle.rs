//! Example: Stream lifecycle channels (MarketLifecycle, Multivariate).
//!
//! This example demonstrates subscribing to event-driven channels that track
//! market state changes and multivariate collection updates.
//!
//! # Channels
//!
//! - `MarketLifecycle`: Market creation, activation, deactivation, determination, settlement
//! - `Multivariate`: Multivariate collection lookup notifications
//!
//! # Usage
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//!     cargo run --example stream_lifecycle
//! ```
//!
//! # Notes
//!
//! - These are public channels but events are infrequent
//! - MarketLifecycle events occur when markets change state (open, close, settle)
//! - Best tested during active market hours when new markets are created

use std::time::Duration;
use tokio::time::timeout;

use kalshi_trade_rs::{
    auth::KalshiConfig,
    ws::{Channel, KalshiStreamClient, StreamMessage},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs=info".parse()?),
        )
        .init();

    let config = KalshiConfig::from_env()?;
    println!(
        "Connecting to Kalshi {:?} environment...",
        config.environment
    );

    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();
    println!("Connected!\n");

    // Collect all subscription IDs
    let mut all_sids: Vec<i64> = Vec::new();

    // MarketLifecycle requires market tickers, Multivariate does not
    // We'll subscribe to them separately to handle this correctly

    // First, subscribe to Multivariate (no tickers needed)
    println!("Subscribing to Multivariate channel (all collections)...");
    let mv_result = handle.subscribe(&[Channel::Multivariate], None).await?;

    for sub in &mv_result.successful {
        println!("  {} -> sid={}", sub.channel, sub.sid);
    }
    for err in &mv_result.failed {
        println!("  {:?} FAILED: {} - {}", err.channel, err.code, err.message);
    }
    all_sids.extend(mv_result.sids());

    // For MarketLifecycle, we need to specify market tickers
    println!("\nSubscribing to MarketLifecycle channel...");

    // MarketLifecycle requires tickers - fetch some active markets
    let rest_client = kalshi_trade_rs::KalshiClient::new(config.clone())?;
    let params = kalshi_trade_rs::GetMarketsParams::new()
        .status(kalshi_trade_rs::MarketFilterStatus::Open)
        .limit(5);
    let markets = rest_client.get_markets_with_params(params).await?;

    if markets.markets.is_empty() {
        println!("  No active markets found - skipping MarketLifecycle subscription");
    } else {
        let tickers: Vec<&str> = markets.markets.iter().map(|m| m.ticker.as_str()).collect();
        println!("  Subscribing to {} markets: {:?}", tickers.len(), tickers);

        let lc_result = handle
            .subscribe(&[Channel::MarketLifecycle], Some(&tickers))
            .await?;

        for sub in &lc_result.successful {
            println!("  {} -> sid={}", sub.channel, sub.sid);
        }
        for err in &lc_result.failed {
            println!("  {:?} FAILED: {} - {}", err.channel, err.code, err.message);
        }
        all_sids.extend(lc_result.sids());
    }

    if all_sids.is_empty() {
        println!("\nNo channels subscribed successfully. Exiting.");
        client.shutdown().await?;
        return Ok(());
    }

    println!("\nWaiting for lifecycle events (120 seconds)...");
    println!("Note: These events are infrequent - markets don't change state often\n");

    let deadline = Duration::from_secs(120);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > deadline {
            println!("\nReached time limit, shutting down...");
            break;
        }

        match timeout(Duration::from_secs(30), handle.update_receiver.recv()).await {
            Ok(Ok(update)) => match &update.msg {
                StreamMessage::Closed { reason } => {
                    println!("[CLOSED] {}", reason);
                    break;
                }
                StreamMessage::ConnectionLost { reason } => {
                    println!("[CONNECTION LOST] {}", reason);
                    break;
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
                StreamMessage::Unsubscribed => {
                    println!("[UNSUBSCRIBED] sid={}", update.sid);
                }
                _ => {
                    // Other updates
                    println!(
                        "[{}] sid={} | {:?}",
                        update.channel.to_uppercase(),
                        update.sid,
                        update.msg
                    );
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
                println!("[INFO] No lifecycle events in last 30 seconds (this is normal)...");
            }
        }
    }

    println!("Unsubscribing...");
    if !all_sids.is_empty() {
        handle.unsubscribe(&all_sids).await?;
    }

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
