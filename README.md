# Rust Kalshi Trading API Client

An unofficial Rust client library for connecting to the Kalshi prediction market.

## Key Features

The library provides both REST API and WebSocket streaming capabilities:

- **REST Client**: Full coverage of the Kalshi API including portfolio management, order operations, market data, and exchange status
- **WebSocket Streaming**: Real-time ticker, trade, orderbook, and fill updates with channel-based message delivery
- **Batch Operations**: Rate-limited batch order creation and cancellation with automatic chunking and retry logic

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
kalshi-trade-rs = "0.1.0"
```

Configure environment variables:

```bash
export KALSHI_ENV=demo          # or "prod" for production
export KALSHI_API_KEY_ID=your_api_key_id
export KALSHI_PRIVATE_KEY_PATH=/path/to/your/private_key.pem
```

Quick start example:

```rust
use kalshi_trade_rs::{KalshiClient, KalshiConfig, cents_to_dollars};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = KalshiConfig::from_env()?;
    let client = KalshiClient::new(config)?;

    // Get account balance
    let balance = client.get_balance().await?;
    println!("Balance: ${:.2}", cents_to_dollars(balance.balance));

    // Get positions
    let positions = client.get_positions().await?;
    for pos in positions.market_positions {
        println!("{}: {} contracts", pos.ticker, pos.position);
    }

    Ok(())
}
```

WebSocket streaming example:

```rust
use kalshi_trade_rs::{KalshiConfig, KalshiStreamClient, Channel, StreamMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = KalshiConfig::from_env()?;
    let client = KalshiStreamClient::connect(&config).await?;
    let mut handle = client.handle();

    handle.subscribe(&[Channel::Ticker, Channel::Trade], None).await?;

    loop {
        match handle.update_receiver.recv().await {
            Ok(update) => match &update.msg {
                StreamMessage::Ticker(t) => {
                    println!("[TICKER] {} @ {}¢", t.market_ticker, t.price);
                }
                StreamMessage::Trade(t) => {
                    println!("[TRADE] {} {} @ {}¢", t.market_ticker, t.count, t.yes_price);
                }
                StreamMessage::ConnectionLost { reason } => {
                    println!("Connection lost: {}", reason);
                    break;
                }
                _ => {}
            },
            Err(_) => break,
        }
    }

    Ok(())
}
```

## Connection and Reconnection

Three connection strategies are available for WebSocket connections:

- **Simple**: Fast-fail on connection errors
- **Retry**: Exponential backoff with configurable attempts

The library requires implementing your own reconnection loop for handling disconnections. See the `stream_reconnect` example for the recommended pattern:

```rust
use kalshi_trade_rs::ws::ConnectStrategy;

let client = KalshiStreamClient::connect_with_strategy(&config, ConnectStrategy::Retry).await?;
```

## Licensing

This project is licensed under the MIT License.
