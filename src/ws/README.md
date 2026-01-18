# Kalshi WebSocket API Reference

Complete reference for WebSocket streaming supported by this library.

**Official Documentation**: [docs.kalshi.com/reference/websocket-overview](https://docs.kalshi.com/reference/websocket-overview)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and verified |

---

## Summary

| Category | Channels | Coverage |
|----------|----------|----------|
| Public Channels | 4 | 100% |
| Authenticated Channels | 3 | 100% |
| Other Channels | 1 | 100% |
| **Total** | **8** | **100%** |

---

## Connection Features

| Feature | Status | Notes |
|---------|--------|-------|
| RSA-PSS Authentication | ✅ | Headers: KALSHI-ACCESS-KEY, KALSHI-ACCESS-SIGNATURE, KALSHI-ACCESS-TIMESTAMP |
| Connection Strategies | ✅ | `Simple` (fast-fail) and `Retry` (exponential backoff) |
| Health Monitoring | ✅ | Bidirectional ping/pong with configurable timeouts |
| Heartbeat Response | ✅ | Auto-responds to Kalshi's 10-second ping frames |
| Graceful Shutdown | ✅ | Clean close with subscriber notification |
| Reconnection Support | ✅ | Via application-level pattern (see examples) |

---

## Public Channels

| Status | Channel | Rust Type | Message Type | Notes |
|--------|---------|-----------|--------------|-------|
| ✅ | `orderbook_delta` | `Channel::OrderbookDelta` | `OrderbookSnapshotData`, `OrderbookDeltaData` | Sends snapshot first, then deltas |
| ✅ | `ticker` | `Channel::Ticker` | `TickerData` | Price, volume, open interest updates |
| ✅ | `trade` | `Channel::Trade` | `TradeData` | Public trade notifications |
| ✅ | `market_lifecycle_v2` | `Channel::MarketLifecycle` | `MarketLifecycleData` | Market state changes |

**Source**: `src/ws/channel.rs`

---

## Authenticated Channels

These channels require valid API credentials to subscribe.

| Status | Channel | Rust Type | Message Type | Notes |
|--------|---------|-----------|--------------|-------|
| ✅ | `fill` | `Channel::Fill` | `FillData` | User order fill notifications |
| ✅ | `market_positions` | `Channel::MarketPositions` | `MarketPositionData` | Real-time position updates |
| ✅ | `communications` | `Channel::Communications` | `CommunicationData` | RFQ and quote notifications |

**Source**: `src/ws/channel.rs`

---

## Other Channels

| Status | Channel | Rust Type | Notes |
|--------|---------|-----------|-------|
| ✅ | `multivariate` | `Channel::Multivariate` | Multivariate collection lookup notifications |

---

## Message Types

All message types are defined in `src/ws/message.rs`:

| Type | Description | Key Fields |
|------|-------------|------------|
| `StreamUpdate` | Wrapper for all updates | `channel`, `sid`, `seq`, `msg` |
| `StreamMessage` | Enum of all message variants | See below |
| `OrderbookSnapshotData` | Full orderbook state | `market_ticker`, `yes`, `no` |
| `OrderbookDeltaData` | Incremental orderbook update | `market_ticker`, `price`, `delta`, `side` |
| `TickerData` | Market ticker data | `market_ticker`, `price`, `yes_bid`, `yes_ask`, `volume`, `open_interest` |
| `TradeData` | Public trade info | `market_ticker`, `yes_price`, `no_price`, `count`, `taker_side`, `ts` |
| `FillData` | User fill notification | `trade_id`, `order_id`, `market_ticker`, `is_taker`, `side`, `yes_price`, `count` |
| `MarketPositionData` | Position update | `user_id`, `market_ticker`, `position`, `position_cost`, `realized_pnl` |
| `MarketLifecycleData` | Lifecycle event | `event_type`, `market_ticker`, timestamps, `result` |
| `CommunicationData` | RFQ/Quote events | Tagged enum: `RfqCreated`, `RfqDeleted`, `QuoteCreated`, `QuoteAccepted` |

### System Messages

Locally-generated messages (not from server):

| Type | Description | When Sent |
|------|-------------|-----------|
| `StreamMessage::Closed` | Clean connection close | User-requested or server close frame |
| `StreamMessage::ConnectionLost` | Unexpected disconnection | Error, timeout, network failure |
| `StreamMessage::Unsubscribed` | Channel unsubscribed | After successful unsubscribe |

---

## Usage Examples

### Basic Connection

```rust
use kalshi_trade_rs::auth::KalshiConfig;
use kalshi_trade_rs::ws::{Channel, KalshiStreamClient};

let config = KalshiConfig::from_env()?;
let client = KalshiStreamClient::connect(&config).await?;
let mut handle = client.handle();

// Subscribe to ticker updates for specific markets
handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;

// Process updates
while let Ok(update) = handle.update_receiver.recv().await {
    println!("{:?}", update.msg);
}
```

### Adding Markets to Existing Subscription

```rust
// Initial subscription
handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;

// Add more markets later (automatically uses add_markets under the hood)
handle.subscribe(Channel::Ticker, &["KXBTC-25DEC31-100000"]).await?;

// Check what markets we're subscribed to
println!("Ticker markets: {:?}", handle.markets(Channel::Ticker));
// ["INXD-25JAN17-B5955", "KXBTC-25DEC31-100000"]
```

### Reconnection Pattern

```rust
use kalshi_trade_rs::ws::{ConnectStrategy, KalshiStreamClient, StreamMessage};

let client = KalshiStreamClient::connect_with_strategy(
    &config,
    ConnectStrategy::Retry, // Retries with exponential backoff
).await?;

let mut handle = client.handle();

loop {
    match handle.update_receiver.recv().await {
        Ok(update) => match &update.msg {
            StreamMessage::ConnectionLost { reason } => {
                // Reconnect logic here
                break;
            }
            _ => { /* process update */ }
        },
        Err(_) => break,
    }
}
```

### Multi-Channel Subscription

```rust
// Subscribe to different channels for same markets
handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
handle.subscribe(Channel::OrderbookDelta, &["INXD-25JAN17-B5955"]).await?;
handle.subscribe(Channel::Trade, &["INXD-25JAN17-B5955"]).await?;

// Check all subscriptions
let subs = handle.subscriptions();
for (channel, markets) in subs {
    println!("{:?} → {:?}", channel, markets);
}
```

### Unsubscribing

```rust
// Remove specific markets from a channel
handle.unsubscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;

// Unsubscribe from entire channel
handle.unsubscribe_all(Channel::OrderbookDelta).await?;
```

---

## Protocol Details

### Subscribe Request

```json
{
    "id": 1,
    "cmd": "subscribe",
    "params": {
        "channels": ["ticker", "trade"],
        "market_ticker": "INXD-25JAN17-B5955"
    }
}
```

### Unsubscribe Request

```json
{
    "id": 2,
    "cmd": "unsubscribe",
    "params": {
        "sids": [123, 456]
    }
}
```

### Response Format

```json
{
    "id": 1,
    "type": "subscribed",
    "msg": {
        "channel": "ticker",
        "sid": 123
    }
}
```

### Update Format

```json
{
    "type": "ticker",
    "sid": 123,
    "seq": 42,
    "msg": {
        "market_ticker": "INXD-25JAN17-B5955",
        "price": 65,
        "yes_bid": 64,
        "yes_ask": 66,
        "volume": 1000,
        "open_interest": 500,
        "dollar_volume": 65000,
        "dollar_open_interest": 32500,
        "ts": 1704067200
    }
}
```

---

## Important Notes

1. **Market ticker requirements**: Channels `orderbook_delta`, `ticker`, `trade`, and `market_lifecycle_v2` require at least one market ticker. The client validates this before sending.

2. **Authentication-only channels**: `fill`, `market_positions`, and `communications` are user-scoped and don't require market tickers.

3. **Broadcast channel lag**: If a subscriber falls behind, they will receive a `RecvError::Lagged(n)` indicating dropped messages. Increase `buffer_size` via `connect_with_options()` if needed.

4. **Multi-channel responses**: When subscribing to N channels, Kalshi sends N separate responses (all with the same request ID but different SIDs). The implementation collects all responses before returning.

---

## References

- [Kalshi WebSocket Overview](https://docs.kalshi.com/reference/websocket-overview)
- [Kalshi WebSocket Subscriptions](https://docs.kalshi.com/reference/ws-subscriptions)
- Examples: `examples/stream_ticker.rs`, `examples/stream_reconnect.rs`, `examples/multi_channel_subscribe.rs`
