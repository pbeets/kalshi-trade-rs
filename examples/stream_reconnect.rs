//! Example: Automatic reconnection handling for WebSocket streams.
//!
//! This example demonstrates the recommended pattern for production applications
//! that need to maintain a persistent connection with automatic reconnection.
//!
//! # Usage
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//!     cargo run --example stream_reconnect
//! ```

use std::time::Duration;

use kalshi_trade_rs::{
    auth::KalshiConfig,
    ws::{Channel, ConnectStrategy, KalshiStreamClient, StreamMessage},
};

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = KalshiConfig::from_env()?;

    let mut reconnect_attempts = 0;

    loop {
        match run_stream(&config).await {
            Ok(reason) => {
                println!("Stream closed: {reason}");
                break; // Clean exit - don't reconnect
            }
            Err(e) => {
                reconnect_attempts += 1;
                if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                    return Err(e);
                }
                println!("Error: {e}. Reconnecting in {RECONNECT_DELAY:?}...");
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        }
    }

    Ok(())
}

/// Run stream until disconnection or error.
async fn run_stream(config: &KalshiConfig) -> Result<String, Box<dyn std::error::Error>> {
    let client = KalshiStreamClient::connect_with_strategy(config, ConnectStrategy::Retry).await?;
    let mut handle = client.handle();

    // Subscribe to Fill channel (authenticated, no market ticker required)
    handle.subscribe(Channel::Fill, &[]).await?;

    loop {
        match handle.update_receiver.recv().await {
            Ok(update) => match &update.msg {
                StreamMessage::Closed { reason } => return Ok(reason.to_string()),
                StreamMessage::ConnectionLost {
                    reason,
                    subscriptions,
                } => {
                    eprintln!("Connection lost: {reason}");
                    if !subscriptions.is_empty() {
                        eprintln!("Lost subscriptions: {subscriptions:?}");
                    }
                    return Err(format!("Connection lost: {reason}").into());
                }
                StreamMessage::Fill(f) => {
                    println!("[FILL] {} @ {}¢ x{}", f.market_ticker, f.yes_price, f.count);
                }
                _ => {}
            },
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                println!("[WARN] Dropped {n} messages");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                return Err("Channel closed".into());
            }
        }
    }
}
