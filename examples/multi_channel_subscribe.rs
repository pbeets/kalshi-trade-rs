//! Test: Multi-channel subscription behavior.
//!
//! Demonstrates subscribing to multiple channels with a dynamically fetched market ticker.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example test_multi_channel_subscribe
//! ```

use std::time::Duration;

use kalshi_trade_rs::{
    GetMarketsParams, KalshiClient, KalshiConfig, MarketFilterStatus,
    ws::{Channel, KalshiStreamClient},
};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs::ws::actor=info".parse().unwrap()),
        )
        .init();

    let config = KalshiConfig::from_env()?;
    println!("Environment: {:?}\n", config.environment);

    // Fetch an active market ticker to subscribe to
    let rest_client = KalshiClient::new(config.clone())?;
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(1);
    let markets = rest_client.get_markets_with_params(params).await?;

    let ticker = markets
        .markets
        .first()
        .map(|m| m.ticker.clone())
        .ok_or("No active markets found")?;

    println!("Using market ticker: {ticker}\n");

    // Connect to WebSocket
    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();
    println!("Connected to WebSocket\n");

    // Subscribe to multiple channels for the same ticker
    let channels = [Channel::Ticker, Channel::Trade, Channel::OrderbookDelta];
    println!(
        "Subscribing to {} channels: {:?}",
        channels.len(),
        channels.iter().map(|c| c.as_str()).collect::<Vec<_>>()
    );

    for channel in &channels {
        match timeout(
            Duration::from_secs(10),
            handle.subscribe(*channel, &[ticker.as_str()]),
        )
        .await
        {
            Ok(Ok(())) => {
                println!("  {} -> subscribed", channel.as_str());
            }
            Ok(Err(e)) => {
                println!("  {} -> FAILED: {}", channel.as_str(), e);
            }
            Err(_) => {
                println!("  {} -> TIMEOUT", channel.as_str());
            }
        }
    }

    // Report results
    let subs = handle.subscriptions();
    println!("\nResult:");
    println!("  Subscribed channels: {}", subs.len());
    for (channel, markets) in &subs {
        println!("    - {:?}: {:?}", channel, markets);
    }

    let status = if subs.len() == channels.len() {
        "PASS"
    } else {
        "FAIL"
    };
    println!(
        "\n{status}: {}/{} channels subscribed",
        subs.len(),
        channels.len()
    );

    // Brief wait for updates
    println!("\nWaiting for updates (2 seconds)...");
    let mut count = 0;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if timeout(Duration::from_millis(100), handle.update_receiver.recv())
            .await
            .is_ok()
        {
            count += 1;
        }
    }
    println!("Received {count} updates");

    // Cleanup
    for channel in &channels {
        if handle.is_subscribed(*channel) {
            handle.unsubscribe_all(*channel).await?;
        }
    }
    client.shutdown().await?;

    println!("\nDone!");
    Ok(())
}
