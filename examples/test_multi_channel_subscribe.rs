//! Test: Multi-channel subscription behavior
//!
//! This example tests our multi-channel subscription implementation:
//! 1. Sends ONE request with multiple channels
//! 2. Verifies we receive multiple SIDs (one per channel)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example test_multi_channel_subscribe
//! ```

use std::time::Duration;

use kalshi_trade_rs::{
    auth::KalshiConfig,
    ws::{Channel, KalshiStreamClient},
};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize tracing so we can see debug output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs::ws::actor=debug".parse().unwrap()),
        )
        .init();

    let config = KalshiConfig::from_env()?;

    println!("Connecting to Kalshi ({:?})...", config.environment);
    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();
    println!("Connected!\n");

    // Subscribe to multiple channels at once
    let channels = [Channel::Ticker, Channel::Trade, Channel::OrderbookDelta];
    println!(
        "Subscribing to {} channels: {:?}",
        channels.len(),
        channels.iter().map(|c| c.as_str()).collect::<Vec<_>>()
    );

    // Use timeout to avoid hanging forever
    let result = match timeout(Duration::from_secs(10), handle.subscribe(&channels, None)).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            println!("Subscribe error: {}", e);
            client.shutdown().await?;
            return Err(e.into());
        }
        Err(_) => {
            println!("Subscribe timed out after 10 seconds!");
            client.shutdown().await?;
            return Err("Timeout".into());
        }
    };

    println!("\nResult:");
    println!("  Successful: {}", result.successful.len());
    for sub in &result.successful {
        println!("    - {}: sid={}", sub.channel, sub.sid);
    }
    if !result.failed.is_empty() {
        println!("  Failed: {}", result.failed.len());
        for err in &result.failed {
            println!("    - {:?}: {} - {}", err.channel, err.code, err.message);
        }
    }

    println!();
    if result.successful.len() == channels.len() {
        println!(
            "PASS: All {} channels subscribed successfully!",
            channels.len()
        );
    } else {
        println!(
            "FAIL: Expected {} subscriptions, got {}",
            channels.len(),
            result.successful.len()
        );
    }

    // Wait briefly for any updates to confirm connection is working
    println!("\nWaiting for updates (2 seconds)...");
    let mut update_count = 0;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        match timeout(Duration::from_millis(100), handle.update_receiver.recv()).await {
            Ok(Ok(_)) => update_count += 1,
            _ => {}
        }
    }
    println!("Received {} updates", update_count);

    // Cleanup
    let sids = result.sids();
    if !sids.is_empty() {
        handle.unsubscribe(&sids).await?;
    }
    client.shutdown().await?;

    println!("\nDone!");
    Ok(())
}
