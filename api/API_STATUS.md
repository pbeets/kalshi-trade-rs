# Kalshi API Implementation Status

Comprehensive tracking of Kalshi API endpoints: implementation status, verification status, and gaps.

**Last updated**: 2026-01-11
**Reference**: [Official Kalshi API Documentation](https://docs.kalshi.com)
**Python SDK**: [kalshi-python](https://pypi.org/project/kalshi-python/) (v2.1.4, auto-generated from OpenAPI spec)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and verified working |
| ⚠️ | Implemented with caveats (see notes) |
| 🔲 | Implemented but not yet verified |
| ❌ | Not implemented |

---

## Summary

| Category | Implemented | Total | Coverage |
|----------|-------------|-------|----------|
| Exchange | 4 | 5 | 80% |
| Orders | 10 | 10 | 100% |
| Order Groups | 6 | 6 | 100% |
| Portfolio | 5 | 5 | 100% |
| Subaccounts | 5 | 5 | 100% |
| Markets | 6 | 6 | 100% |
| Events | 6 | 6 | 100% |
| Series | 2 | 2 | 100% |
| Communications (RFQ/Quotes) | 10 | 12 | 83% |
| Search | 2 | 2 | 100% |
| Live Data | 2 | 2 | 100% |
| Multivariate Collections | 5 | 5 | 100% |
| API Keys | 0 | 4 | 0% |
| FCM | 0 | 2 | 0% |
| Structured Targets | 0 | 2 | 0% |
| Milestones | 0 | 2 | 0% |
| Incentive Programs | 0 | 1 | 0% |
| **TOTAL** | **63** | **77** | **82%** |

---

## Exchange API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/exchange/status` | `get_exchange_status()` | Public endpoint |
| ✅ | GET | `/exchange/schedule` | `get_exchange_schedule()` | Public endpoint |
| ✅ | GET | `/exchange/announcements` | `get_exchange_announcements()` | Public endpoint |
| ✅ | GET | `/exchange/user_data_timestamp` | `get_user_data_timestamp()` | Requires auth |
| ❌ | GET | `/series/fee_changes` | - | Fee change notifications |

**Source file**: `src/api/exchange.rs`

---

## Orders API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | POST | `/portfolio/orders` | `create_order()` | |
| ⚠️ | GET | `/portfolio/orders/{id}` | `get_order()` | 404 in demo for new orders |
| ✅ | DELETE | `/portfolio/orders/{id}` | `cancel_order()` | |
| ✅ | POST | `/portfolio/orders/{id}/amend` | `amend_order()` | |
| 🔲 | POST | `/portfolio/orders/{id}/decrease` | `decrease_order()` | |
| ✅ | GET | `/portfolio/orders` | `get_orders()`, `get_orders_with_params()` | |
| ✅ | POST | `/portfolio/orders/batched` | `batch_create_orders()` | Max 20 orders |
| ✅ | DELETE | `/portfolio/orders/batched` | `batch_cancel_orders()` | Max 20 orders |
| ⚠️ | GET | `/portfolio/orders/queue_positions` | `get_queue_positions()` | Requires market_tickers param |
| ⚠️ | GET | `/portfolio/orders/{id}/queue_position` | `get_order_queue_position()` | 404 in demo for new orders |

**Source file**: `src/api/orders.rs`

**Known Issues**:
- Demo environment has eventual consistency: newly created orders may return 404 on immediate fetch
- `get_queue_positions()` requires `market_tickers` or `event_ticker` parameter (returns 400 without)

---

## Order Groups API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/portfolio/order_groups` | `create_order_group()` | |
| 🔲 | GET | `/portfolio/order_groups/{id}` | `get_order_group()` | |
| 🔲 | PUT | `/portfolio/order_groups/{id}` | `update_order_group()` | |
| 🔲 | GET | `/portfolio/order_groups` | `list_order_groups()` | |
| 🔲 | DELETE | `/portfolio/order_groups/{id}` | `delete_order_group()` | Cancels all orders |
| 🔲 | PUT | `/portfolio/order_groups/{id}/reset` | `reset_order_group()` | |

**Source file**: `src/api/order_groups.rs`

---

## Portfolio API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/portfolio/balance` | `get_balance()` | |
| ✅ | GET | `/portfolio/positions` | `get_positions()`, `get_positions_with_params()` | |
| ✅ | GET | `/portfolio/fills` | `get_fills()`, `get_fills_with_params()` | |
| ✅ | GET | `/portfolio/orders` | `get_orders()` | Listed under Orders |
| 🔲 | GET | `/portfolio/settlements` | `get_settlements()`, `get_settlements_with_params()` | |

**Source file**: `src/api/portfolio.rs`

---

## Subaccounts API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/portfolio/subaccounts` | `create_subaccount()` | Max 32 subaccounts |
| 🔲 | POST | `/portfolio/subaccounts/transfer` | `transfer_between_subaccounts()` | |
| 🔲 | GET | `/portfolio/subaccounts/balances` | `get_subaccount_balances()` | |
| 🔲 | GET | `/portfolio/subaccounts/transfers` | `get_subaccount_transfers()` | |
| 🔲 | GET | `/portfolio/summary/total_resting_order_value` | `get_resting_order_value()` | For FCM members |

**Source file**: `src/api/subaccounts.rs`

---

## Markets API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/markets` | `get_markets()`, `get_markets_with_params()` | |
| ✅ | GET | `/markets/{ticker}` | `get_market()` | |
| ✅ | GET | `/markets/{ticker}/orderbook` | `get_orderbook()`, `get_orderbook_with_params()` | |
| ✅ | GET | `/markets/trades` | `get_trades()`, `get_trades_with_params()` | |
| 🔲 | GET | `/series/{series}/markets/{ticker}/candlesticks` | `get_candlesticks()` | |
| 🔲 | GET | `/markets/candlesticks` | `get_batch_candlesticks()` | Max 100 tickers |

**Source file**: `src/api/markets.rs`

---

## Events API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/events` | `get_events()`, `get_events_with_params()` | Excludes multivariate |
| ✅ | GET | `/events/{ticker}` | `get_event()`, `get_event_with_params()` | |
| ✅ | GET | `/events/{ticker}/metadata` | `get_event_metadata()` | |
| ✅ | GET | `/events/multivariate` | `get_multivariate_events()` | |
| 🔲 | GET | `/series/{s}/events/{e}/candlesticks` | `get_event_candlesticks()` | |
| 🔲 | GET | `/series/{s}/events/{e}/forecast_percentile_history` | `get_event_forecast_percentile_history()` | |

**Source file**: `src/api/events.rs`

---

## Series API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/series/{ticker}` | `get_series()` | |
| 🔲 | GET | `/series` | `get_series_list()`, `get_series_list_with_params()` | |

**Source file**: `src/api/series.rs`

---

## Communications API (RFQ/Quotes)

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/communications/rfqs` | `create_rfq()` | |
| 🔲 | GET | `/communications/rfqs` | `list_rfqs()`, `list_rfqs_with_params()` | |
| 🔲 | GET | `/communications/rfqs/{id}` | `get_rfq()` | |
| 🔲 | DELETE | `/communications/rfqs/{id}` | `cancel_rfq()` | |
| 🔲 | POST | `/communications/quotes` | `create_quote()` | |
| 🔲 | GET | `/communications/quotes` | `list_quotes()`, `list_quotes_with_params()` | |
| 🔲 | GET | `/communications/quotes/{id}` | `get_quote()` | |
| 🔲 | DELETE | `/communications/quotes/{id}` | `cancel_quote()` | |
| 🔲 | PUT | `/communications/quotes/{id}/accept` | `accept_quote()` | |
| ❌ | PUT | `/communications/quotes/{id}/confirm` | - | Quote confirmation |
| ❌ | GET | `/communications/id` | - | Get user's comms ID |

**Source file**: `src/api/communications.rs`

---

## Search API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/search/tags_by_categories` | `get_tags_by_categories()` | |
| 🔲 | GET | `/search/filters_by_sport` | `get_filters_by_sport()` | |

**Source file**: `src/api/search.rs`

---

## Live Data API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/live_data/{type}/milestone/{id}` | `get_live_data()` | |
| 🔲 | GET | `/live_data/batch` | `get_batch_live_data()` | |

**Source file**: `src/api/live_data.rs`

---

## Multivariate Collections API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/multivariate_event_collections` | `get_multivariate_collections()` | |
| 🔲 | GET | `/multivariate_event_collections/{ticker}` | `get_multivariate_collection()` | |
| 🔲 | POST | `/multivariate_event_collections/{ticker}` | `create_market_in_collection()` | |
| 🔲 | GET | `/multivariate_event_collections/{ticker}/lookup` | `get_lookup_history()` | |
| 🔲 | PUT | `/multivariate_event_collections/{ticker}/lookup` | `lookup_tickers()` | |

**Source file**: `src/api/multivariate.rs`

---

## NOT IMPLEMENTED

### API Keys Management (Low Priority)
Usually managed via Kalshi web UI.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api_keys` | List all API keys |
| POST | `/api_keys` | Create API key with public key |
| POST | `/api_keys/generate` | Generate API key (auto-creates keypair) |
| DELETE | `/api_keys/{api_key}` | Delete an API key |

### FCM (Futures Commission Merchant) (Low Priority)
Specialized for FCM members only.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/fcm/orders` | Get FCM orders by subtrader ID |
| GET | `/fcm/positions` | Get FCM positions by subtrader ID |

### Structured Targets (Low Priority)
Specialized use case.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/structured_targets` | List structured targets |
| GET | `/structured_targets/{id}` | Get specific structured target |

### Milestones (Low Priority)
Specialized use case.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/milestones` | List milestones with date filtering |
| GET | `/milestones/{id}` | Get specific milestone |

### Incentive Programs (Low Priority)
Read-only info.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/incentive_programs` | List available incentive programs |

### Exchange (Partial)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/series/fee_changes` | Get series fee change history |

### Communications (Partial)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/communications/id` | Get user's communications identifier |
| PUT | `/communications/quotes/{id}/confirm` | Confirm a quote |

---

## Test Coverage

### Verified Examples

| Example | APIs Covered | Status |
|---------|--------------|--------|
| `markets.rs` | Markets, Orderbook, Trades | ✅ Verified |
| `events.rs` | Events, Metadata, Multivariate | ✅ Verified |
| `portfolio.rs` | Balance, Positions, Fills | ✅ Verified |
| `trading.rs` | Create/Get/Amend/Cancel Order | ⚠️ Demo env issues |
| `batch_orders.rs` | Batch Create/Cancel | ✅ Verified |

### Not Yet Tested

The following implemented endpoints need verification examples:

- **Candlesticks**: `get_candlesticks()`, `get_batch_candlesticks()`, `get_event_candlesticks()`
- **Forecast**: `get_event_forecast_percentile_history()`
- **Settlements**: `get_settlements()`
- **Series**: `get_series()`, `get_series_list()`
- **Order Groups**: All 6 endpoints
- **Subaccounts**: All 5 endpoints
- **Communications**: All 10 endpoints
- **Search**: `get_tags_by_categories()`, `get_filters_by_sport()`
- **Live Data**: `get_live_data()`, `get_batch_live_data()`
- **Multivariate Collections**: All 5 endpoints

---

## Implementation Notes

### Adding New Endpoints

1. Create/update module in `src/api/`
2. Add request/response models in `src/models/`
3. Expose method on `KalshiClient` in `src/client.rs`
4. Re-export models in `src/lib.rs`
5. Add example in `examples/`
6. Update this document

### Validation Patterns

- All request parameters use builder patterns with `Default`
- Query strings generated via `to_query_string()` methods
- Path parameters URL-encoded via `form_urlencoded`

### Error Handling

- All endpoints return `Result<T, Error>`
- API errors mapped to structured `Error::Api` variant
- Rate limiting handled in `BatchManager` for bulk operations

---

## WebSocket API

This document covers the REST API. WebSocket streaming is implemented separately in `src/ws/`.

Supported WebSocket channels:
- Order book updates
- Trade feed
- Ticker updates
- Fill notifications

See `examples/stream_ticker.rs` and `examples/multi_channel_subscribe.rs`.

---

## References

- [Official Kalshi API Documentation](https://docs.kalshi.com)
- [Kalshi Python SDK (PyPI)](https://pypi.org/project/kalshi-python/)
- [Kalshi Developer Discord](https://discord.gg/kalshi) - #dev channel
