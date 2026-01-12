# Kalshi API Implementation Status

This document tracks all implemented API endpoints and their verification status.

**Last verified**: 2026-01-12 (Demo environment)

## Verification Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Verified working |
| ⚠️ | Works with caveats (see notes) |
| ❌ | Not working / has issues |
| 🔲 | Not yet verified |
| ➖ | Not implemented |

---

## Markets API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_markets()` | GET /markets | ✅ | markets.rs | |
| `get_markets_with_params()` | GET /markets | ✅ | markets.rs | Pagination, filters |
| `get_market(ticker)` | GET /markets/{ticker} | ✅ | markets.rs | |
| `get_orderbook(ticker)` | GET /markets/{ticker}/orderbook | ✅ | markets.rs | |
| `get_orderbook_with_params()` | GET /markets/{ticker}/orderbook | ✅ | markets.rs | Depth parameter |
| `get_trades()` | GET /markets/trades | ✅ | markets.rs | |
| `get_trades_with_params()` | GET /markets/trades | ✅ | markets.rs | Ticker filter, pagination |
| `get_candlesticks()` | GET /series/.../candlesticks | 🔲 | - | |
| `get_batch_candlesticks()` | GET /markets/candlesticks | 🔲 | - | |

## Events API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_events()` | GET /events | ✅ | events.rs | |
| `get_events_with_params()` | GET /events | ✅ | events.rs | Filters, nested markets |
| `get_event(ticker)` | GET /events/{ticker} | ✅ | events.rs | |
| `get_event_with_params()` | GET /events/{ticker} | ✅ | events.rs | with_nested_markets |
| `get_event_metadata()` | GET /events/{ticker}/metadata | ✅ | events.rs | |
| `get_multivariate_events()` | GET /events/multivariate | ✅ | events.rs | |
| `get_multivariate_events_with_params()` | GET /events/multivariate | 🔲 | - | |
| `get_event_candlesticks()` | GET /series/.../events/.../candlesticks | 🔲 | - | |
| `get_event_forecast_percentile_history()` | GET /series/.../events/.../forecast | 🔲 | - | |

## Portfolio API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_balance()` | GET /portfolio/balance | ✅ | portfolio.rs | |
| `get_positions()` | GET /portfolio/positions | ✅ | portfolio.rs | |
| `get_positions_with_params()` | GET /portfolio/positions | ✅ | portfolio.rs | |
| `get_fills()` | GET /portfolio/fills | ✅ | portfolio.rs | |
| `get_fills_with_params()` | GET /portfolio/fills | ✅ | portfolio.rs | |
| `get_settlements()` | GET /portfolio/settlements | 🔲 | - | |
| `get_settlements_with_params()` | GET /portfolio/settlements | 🔲 | - | |

## Orders API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `create_order()` | POST /portfolio/orders | ✅ | trading.rs | |
| `get_order(id)` | GET /portfolio/orders/{id} | ⚠️ | trading.rs | 404 in demo env for new orders |
| `cancel_order(id)` | DELETE /portfolio/orders/{id} | ✅ | trading.rs | |
| `amend_order(id)` | POST /portfolio/orders/{id}/amend | ✅ | trading.rs | |
| `decrease_order(id)` | POST /portfolio/orders/{id}/decrease | 🔲 | - | Documented in trading.rs |
| `get_orders()` | GET /portfolio/orders | ✅ | trading.rs | |
| `get_orders_with_params()` | GET /portfolio/orders | ✅ | trading.rs | Status filter |
| `batch_create_orders()` | POST /portfolio/orders/batched | ✅ | batch_orders.rs | |
| `batch_cancel_orders()` | DELETE /portfolio/orders/batched | ✅ | batch_orders.rs | |
| `get_queue_positions()` | GET /portfolio/orders/queue_positions | ⚠️ | - | Requires market_tickers param |
| `get_queue_positions_with_params()` | GET /portfolio/orders/queue_positions | ✅ | trading.rs | |
| `get_order_queue_position(id)` | GET /portfolio/orders/{id}/queue_position | ⚠️ | trading.rs | 404 in demo env for new orders |

## Exchange API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_exchange_status()` | GET /exchange/status | ✅ | - | Tested separately |
| `get_exchange_schedule()` | GET /exchange/schedule | 🔲 | - | |
| `get_exchange_announcements()` | GET /exchange/announcements | 🔲 | - | |
| `get_user_data_timestamp()` | GET /exchange/user_data_timestamp | 🔲 | - | |

## Series API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_series(ticker)` | GET /series/{ticker} | 🔲 | - | |
| `get_series_list()` | GET /series | 🔲 | - | |
| `get_series_list_with_params()` | GET /series | 🔲 | - | |

## Order Groups API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `create_order_group()` | POST /portfolio/order_groups | 🔲 | - | |
| `get_order_group(id)` | GET /portfolio/order_groups/{id} | 🔲 | - | |
| `update_order_group(id)` | PUT /portfolio/order_groups/{id} | 🔲 | - | |

## Communications API (RFQ/Quotes)

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `create_rfq()` | POST /rfqs | 🔲 | - | |
| `get_rfq(id)` | GET /rfqs/{id} | 🔲 | - | |
| `list_rfqs()` | GET /rfqs | 🔲 | - | |
| `cancel_rfq(id)` | DELETE /rfqs/{id} | 🔲 | - | |
| `create_quote()` | POST /quotes | 🔲 | - | |
| `get_quote(id)` | GET /quotes/{id} | 🔲 | - | |
| `list_quotes()` | GET /quotes | 🔲 | - | |
| `accept_quote(id)` | POST /quotes/{id}/accept | 🔲 | - | |
| `cancel_quote(id)` | DELETE /quotes/{id} | 🔲 | - | |

## Search API

| Endpoint | Method | Status | Example | Notes |
|----------|--------|--------|---------|-------|
| `get_tags_by_categories()` | GET /search/tags | 🔲 | - | |
| `get_filters_by_sport()` | GET /search/filters | 🔲 | - | |

---

## Known Issues

### Demo Environment Consistency

The demo environment has eventual consistency issues where newly created orders cannot be immediately fetched:

- `get_order(id)` returns 404 for orders just created
- `get_order_queue_position(id)` returns 404 for orders just created
- `get_orders()` list may not include newly created orders

**Workaround**: The order IS created (confirmed by successful `cancel_order()`). Use list endpoints or add delays if needed.

### API Parameter Requirements

- `get_queue_positions()` requires `market_tickers` or `event_ticker` parameter (returns 400 without)

---

## Examples Summary

| Example | APIs Covered | Status |
|---------|--------------|--------|
| `markets.rs` | Markets, Orderbook, Trades, Pagination | ✅ All working |
| `events.rs` | Events, Nested Markets, Metadata, Multivariate | ✅ All working |
| `portfolio.rs` | Balance, Positions, Fills, Orders list | ✅ All working |
| `trading.rs` | Create/Get/Amend/Cancel Order, Queue Positions | ⚠️ get_order has demo env issue |
| `batch_orders.rs` | Batch Create/Cancel, Partial Success | ✅ All working |

---

## Running Examples

```bash
# Set up environment variables
export KALSHI_ENV=demo
export KALSHI_API_KEY_ID=your_key_id
export KALSHI_PRIVATE_KEY_PATH=/path/to/private_key.pem

# Or use .env file
cp .env.example .env
# Edit .env with your credentials

# Run examples
cargo run --example markets
cargo run --example events
cargo run --example portfolio
cargo run --example trading      # Places real orders!
cargo run --example batch_orders # Places real orders!
```
