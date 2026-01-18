# Rust Kalshi Trading API Client

[![Crates.io](https://img.shields.io/crates/v/kalshi-trade-rs.svg)](https://crates.io/crates/kalshi-trade-rs)
[![docs.rs](https://img.shields.io/docsrs/kalshi-trade-rs)](https://docs.rs/kalshi-trade-rs)
[![CI](https://github.com/pbeets/kalshi-trade-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/pbeets/kalshi-trade-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An unofficial Rust client library for the [Kalshi](https://kalshi.com) prediction market, implementing the [Kalshi Trading API v2](https://trading-api.readme.io/reference).

## Key Features

This crate provides both REST API and WebSocket streaming capabilities:

- **REST Client**: Full coverage of the Kalshi API including portfolio management, order operations, market data, and exchange status
- **WebSocket Streaming**: Real-time ticker, trade, orderbook, and fill updates with channel-based message delivery

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

    // Subscribe to channels with market tickers
    let markets = &["INXD-25JAN17-B5955", "KXBTC-25DEC31-100000"];
    handle.subscribe(Channel::Ticker, markets).await?;
    handle.subscribe(Channel::Trade, markets).await?;

    loop {
        match handle.update_receiver.recv().await {
            Ok(update) => match &update.msg {
                StreamMessage::Ticker(t) => {
                    println!("[TICKER] {} @ {}¢", t.market_ticker, t.price);
                }
                StreamMessage::Trade(t) => {
                    println!("[TRADE] {} {} @ {}¢", t.market_ticker, t.count, t.yes_price);
                }
                StreamMessage::ConnectionLost { reason, .. } => {
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

Two connection strategies are available for WebSocket connections:

- **Simple**: Fast-fail on connection errors
- **Retry**: Exponential backoff with configurable attempts

The library requires implementing your own reconnection loop for handling disconnections. See the `stream_reconnect` example for the recommended pattern:

```rust
use kalshi_trade_rs::ws::ConnectStrategy;

let client = KalshiStreamClient::connect_with_strategy(&config, ConnectStrategy::Retry).await?;
```

## Examples

See the [`examples/`](examples/) directory for working examples covering REST API usage, WebSocket streaming, and reconnection patterns.

```bash
cargo run --example portfolio
```

## Error Handling

The library uses a unified `Error` type for all errors:

```rust
use kalshi_trade_rs::{KalshiClient, Error};

async fn example(client: &KalshiClient) {
    match client.get_balance().await {
        Ok(balance) => println!("Balance: {} cents", balance.balance),
        Err(Error::Auth(msg)) => eprintln!("Auth failed: {}", msg),
        Err(Error::Api(msg)) => eprintln!("API error: {}", msg),
        Err(Error::Http(e)) => eprintln!("HTTP error: {}", e),
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## Minimum Supported Rust Version

This crate requires **Rust 1.92** or later (uses Rust 2024 edition).

## Running Tests

Tests interact with the real Kalshi API. Set your credentials before running:

```bash
export KALSHI_ENV=demo
export KALSHI_API_KEY_ID=your_api_key_id
export KALSHI_PRIVATE_KEY_PATH=/path/to/your/private_key.pem

cargo test
```

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests for:

- Bug fixes
- New endpoint coverage
- Documentation improvements
- Additional examples

## Disclaimer

This is an **unofficial** client library and is not affiliated with or endorsed by Kalshi. Use at your own risk. The authors are not responsible for any financial losses incurred through the use of this software. Always test thoroughly with the demo environment before using in production.

## License

This project is licensed under the [MIT License](LICENSE).
