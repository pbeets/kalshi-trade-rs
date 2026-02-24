//! Example: Stream user order updates via the `user_orders` channel.
//!
//! This example demonstrates subscribing to the authenticated `user_orders`
//! channel, which delivers real-time notifications whenever your orders are
//! created, filled, canceled, or otherwise updated.
//!
//! # Usage
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//!     cargo run --example stream_user_orders
//! ```
//!
//! To filter updates to specific markets, pass tickers:
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//! MARKET_TICKERS="INXD-25JAN17-B5955,KXBTC-25DEC31-100000" \
//!     cargo run --example stream_user_orders
//! ```
//!
//! # Notes
//!
//! - Requires valid API credentials
//! - Omit `MARKET_TICKERS` to receive updates for all your orders
//! - For testing, run the `trading.rs` example in another terminal

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
                .add_directive("kalshi_trade_rs=debug".parse()?),
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

    // Optionally filter to specific markets via MARKET_TICKERS env var.
    // Leave unset to receive updates for all your orders.
    let tickers_env = std::env::var("MARKET_TICKERS").unwrap_or_default();
    let markets: Vec<&str> = if tickers_env.is_empty() {
        vec![]
    } else {
        tickers_env.split(',').map(str::trim).collect()
    };

    if markets.is_empty() {
        println!("Subscribing to user_orders (all markets)...");
    } else {
        println!("Subscribing to user_orders for: {:?}", markets);
    }

    handle.subscribe(Channel::UserOrders, &markets).await?;
    println!("Subscribed!\n");

    println!("Waiting for order updates (60 seconds)...");
    println!("Tip: Place or cancel orders in another terminal to see updates\n");

    let deadline = Duration::from_secs(300);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > deadline {
            println!("\nReached time limit, shutting down...");
            break;
        }

        match timeout(Duration::from_secs(10), handle.update_receiver.recv()).await {
            Ok(Ok(update)) => match &update.msg {
                StreamMessage::Closed { reason } => {
                    println!("[CLOSED] {}", reason);
                    break;
                }
                StreamMessage::ConnectionLost { reason, .. } => {
                    println!("[CONNECTION LOST] {}", reason);
                    break;
                }
                StreamMessage::UserOrder(order) => {
                    println!(
                        "[ORDER] {} | id={} | status={:?} | side={:?} | price={} | initial={} remaining={} filled={}",
                        order.ticker.as_deref().unwrap_or("?"),
                        order.order_id,
                        order.status,
                        order.side,
                        order.yes_price_dollars.as_deref().unwrap_or("?"),
                        order.initial_count_fp.as_deref().unwrap_or("?"),
                        order.remaining_count_fp.as_deref().unwrap_or("?"),
                        order.fill_count_fp.as_deref().unwrap_or("?"),
                    );
                    if let Some(client_id) = &order.client_order_id {
                        println!("         client_order_id={}", client_id);
                    }
                }
                StreamMessage::Unsubscribed => {
                    println!("[UNSUBSCRIBED] sid={}", update.sid);
                }
                _ => {
                    println!("[OTHER] channel={} sid={}", update.channel, update.sid);
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
                println!("[INFO] No updates in last 10 seconds (waiting for activity)...");
            }
        }
    }

    println!("Unsubscribing...");
    handle.unsubscribe_all(Channel::UserOrders).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
