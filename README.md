# Rust Kalshi Trading API Client

[![Crates.io](https://img.shields.io/crates/v/kalshi-trade-rs.svg)](https://crates.io/crates/kalshi-trade-rs)
[![docs.rs](https://img.shields.io/docsrs/kalshi-trade-rs)](https://docs.rs/kalshi-trade-rs)
[![CI](https://github.com/pbeets/kalshi-trade-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/pbeets/kalshi-trade-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An unofficial Rust client library for the [Kalshi](https://kalshi.com) prediction market, implementing the [Kalshi Trading API v2](https://trading-api.readme.io/reference).

## Key Features

- **REST Client**: Full coverage of 86 Kalshi API endpoints including portfolio management, order operations, market data, exchange status, historical data, and RFQ (Request for Quote) communications
- **WebSocket Streaming**: 10 real-time channels — ticker, trade, orderbook, fill, order updates, position, RFQ/quote, order groups, market lifecycle, and multivariate
- **Batch Operations**: Rate-limited `BatchManager` with automatic chunking, retry, and per-order subaccount support
- **Orderbook Aggregation**: Live orderbook state from WebSocket delta streams with gap detection
- **Subaccount Support**: Full subaccount filtering on orders, fills, positions, settlements, and balance queries
- **Fixed-Point Fields**: `_fp` and `_dollars` fields throughout for precise decimal arithmetic without floating-point issues

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
kalshi-trade-rs = "0.3.0"
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

## RFQ (Request for Quote) Support

The library provides full support for Kalshi's RFQ system, enabling large trades and combo/parlay bets:

```rust
use kalshi_trade_rs::{KalshiClient, CreateRfqRequest, CreateQuoteRequest, AcceptQuoteRequest};

// Create an RFQ (as requester)
let rfq = CreateRfqRequest::with_target_cost_dollars("TICKER", 100.0, true);
let response = client.create_rfq(rfq).await?;

// Submit a quote (as market maker)
let quote = CreateQuoteRequest::from_cents("rfq-id", 55, true);  // YES at 55¢
let response = client.create_quote(quote).await?;

// Accept a quote
client.accept_quote("quote-id", AcceptQuoteRequest::yes()).await?;

// Confirm (as quoter, within 30 seconds)
client.confirm_quote("quote-id").await?;
```

Stream RFQ events via WebSocket:

```rust
handle.subscribe(Channel::Communications, &[]).await?;
// Receive: RfqCreated, RfqDeleted, QuoteCreated, QuoteAccepted, QuoteExecuted
```

See [`examples/rfq_verify.rs`](examples/rfq_verify.rs) for a complete verification example.

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

See the [`examples/`](examples/) directory for 25 working examples:

### REST API

| Example | Description |
|---------|-------------|
| `portfolio` | Account balance, positions, fills, settlements |
| `trading` | Order lifecycle: create, amend, decrease, cancel |
| `markets` | Market queries, orderbook, trades, pagination |
| `events` | Events, series, metadata, multivariate, candlesticks, forecasts |
| `candlesticks` | OHLCV data with multiple periods and batch queries |
| `historical` | Archived data: cutoff, markets, candlesticks, fills, orders |
| `batch_orders` | Batch create/cancel with partial success handling |
| `batch_manager` | Rate-limited batch operations with `BatchManager` |
| `order_groups` | Order group creation, management, triggering |
| `exchange_status` | Exchange status, schedule, announcements, fee changes |
| `live_data` | Live milestone data (single and batch) |
| `milestones` | Milestone metadata queries |
| `structured_targets` | Structured target lookups with type breakdown |
| `rfq_verify` | RFQ system verification (read-only) |
| `search` | Search filters by category and sport |
| `series_and_search` | Series metadata and search combined |

### WebSocket Streaming

| Example | Description |
|---------|-------------|
| `stream_ticker` | Real-time ticker and trade updates |
| `stream_user_channels` | Fills, positions, RFQ communications |
| `stream_reconnect` | Reconnection with resubscription |
| `stream_firehose` | High-volume streaming pattern |
| `stream_lifecycle` | Market lifecycle events |
| `stream_user_orders` | Real-time order update notifications |
| `multi_channel_subscribe` | Multi-channel subscription management |
| `orderbook_aggregator` | Live orderbook state from delta streams |
| `test_auth` | Basic authentication verification |

```bash
cargo run --example portfolio
cargo run --example trading
cargo run --example historical
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
