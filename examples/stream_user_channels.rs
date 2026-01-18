//! Example: Stream user-scoped channels (Fill, MarketPositions, Communications).
//!
//! This example demonstrates subscribing to authenticated channels that track
//! user-specific activity. These channels don't require market tickers - they
//! automatically receive all events for the authenticated user.
//!
//! # Channels
//!
//! - `Fill`: Notifications when your orders are filled
//! - `MarketPositions`: Real-time position updates
//! - `Communications`: RFQ and quote notifications
//!
//! # Usage
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//!     cargo run --example stream_user_channels
//! ```
//!
//! # Notes
//!
//! - These channels require valid API credentials
//! - You'll only see updates when there's actual activity (fills, position changes, etc.)
//! - For testing fills, run the `trading.rs` example in another terminal

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

    // Subscribe to all user-scoped channels
    // These don't require market tickers - they receive all user events
    println!("Subscribing to user channels...");

    handle.subscribe(Channel::Fill, &[]).await?;
    println!("  Fill -> subscribed");

    handle.subscribe(Channel::MarketPositions, &[]).await?;
    println!("  MarketPositions -> subscribed");

    handle.subscribe(Channel::Communications, &[]).await?;
    println!("  Communications -> subscribed");

    println!("\nWaiting for updates (60 seconds)...");
    println!("Tip: Create/fill orders in another terminal to see Fill updates\n");

    let deadline = Duration::from_secs(60);
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
                StreamMessage::Fill(fill) => {
                    println!(
                        "[FILL] {} | order={} | {} {} @ {}c | {} contracts | taker={}",
                        fill.market_ticker,
                        fill.order_id,
                        format!("{:?}", fill.action).to_uppercase(),
                        format!("{:?}", fill.side).to_uppercase(),
                        fill.yes_price,
                        fill.count,
                        fill.is_taker
                    );
                }
                StreamMessage::MarketPosition(pos) => {
                    println!(
                        "[POSITION] {} | position={:?} | cost={:?} | pnl={:?}",
                        pos.market_ticker.as_deref().unwrap_or("?"),
                        pos.position,
                        pos.position_cost,
                        pos.realized_pnl
                    );
                }
                StreamMessage::Communication(comm) => {
                    println!("[COMMUNICATION] {:?}", comm);
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
    handle.unsubscribe_all(Channel::Fill).await?;
    handle.unsubscribe_all(Channel::MarketPositions).await?;
    handle.unsubscribe_all(Channel::Communications).await?;

    println!("Shutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
