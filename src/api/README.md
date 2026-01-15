# Kalshi API Implementation Status

Comprehensive tracking of Kalshi API endpoints: implementation status, verification status, and gaps.

**Last updated**: 2026-01-14
**Reference**: [Official Kalshi API Documentation](https://docs.kalshi.com)
**Python SDK**: [kalshi-python](https://pypi.org/project/kalshi-python/) (v2.1.4, auto-generated from OpenAPI spec)

---

## Legend

### Status Column
| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and working |
| ⚠️ | Implemented with caveats (see notes) |
| 🔲 | Implemented but not yet tested |
| ❌ | Not implemented |

### Verified Column
| Symbol | Meaning |
|--------|---------|
| ✅ | Has working example in `examples/` |
| (empty) | No example yet |

---

## Summary

| Category | Implemented | Verified | Total | Coverage |
|----------|-------------|----------|-------|----------|
| Exchange | 5 | 5 | 5 | 100% |
| Orders | 10 | 9 | 10 | 100% |
| Order Groups | 6 | 5 | 6 | 100% |
| Portfolio | 5 | 4 | 5 | 100% |
| Subaccounts | 5 | 0 | 5 | 100% |
| Markets | 6 | 6 | 6 | 100% |
| Events | 6 | 4 | 6 | 100% |
| Series | 2 | 2 | 2 | 100% |
| Communications (RFQ/Quotes) | 11 | 0 | 11 | 100% |
| Search | 2 | 2 | 2 | 100% |
| Live Data | 2 | 0 | 2 | 100% |
| Multivariate Collections | 5 | 0 | 5 | 100% |
| API Keys | 4 | 0 | 4 | 100% |
| Milestones | 2 | 2 | 2 | 100% |
| Structured Targets | 2 | 2 | 2 | 100% |
| Incentive Programs | 1 | 1 | 1 | 100% |
| FCM | 2 | 0 | 2 | 100% |
| **TOTAL** | **76** | **42** | **76** | **100%** |

---

## Exchange API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/exchange/status` | `get_exchange_status()` | Public endpoint |
| ✅ | ✅ | GET | `/exchange/schedule` | `get_exchange_schedule()` | Public endpoint |
| ✅ | ✅ | GET | `/exchange/announcements` | `get_exchange_announcements()` | Public endpoint |
| ✅ | ✅ | GET | `/exchange/user_data_timestamp` | `get_user_data_timestamp()` | Requires auth |
| ✅ | ✅ | GET | `/series/fee_changes` | `get_fee_changes()`, `get_fee_changes_with_params()` | Fee change notifications |

**Source files**: `src/api/exchange.rs`, `src/api/series.rs`
**Verified in**: `examples/exchange_status.rs`

---

## Orders API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | POST | `/portfolio/orders` | `create_order()` | |
| ⚠️ | ✅ | GET | `/portfolio/orders/{id}` | `get_order()` | 404 in demo for new orders |
| ✅ | ✅ | DELETE | `/portfolio/orders/{id}` | `cancel_order()` | |
| ✅ | ✅ | POST | `/portfolio/orders/{id}/amend` | `amend_order()` | |
| 🔲 | | POST | `/portfolio/orders/{id}/decrease` | `decrease_order()` | |
| ✅ | ✅ | GET | `/portfolio/orders` | `get_orders()`, `get_orders_with_params()` | |
| ✅ | ✅ | POST | `/portfolio/orders/batched` | `batch_create_orders()` | Max 20 orders |
| ✅ | ✅ | DELETE | `/portfolio/orders/batched` | `batch_cancel_orders()` | Max 20 orders |
| ⚠️ | ✅ | GET | `/portfolio/orders/queue_positions` | `get_queue_positions()` | Requires market_tickers param |
| ⚠️ | ✅ | GET | `/portfolio/orders/{id}/queue_position` | `get_order_queue_position()` | 404 in demo for new orders |

**Source file**: `src/api/orders.rs`
**Verified in**: `examples/trading.rs`, `examples/batch_orders.rs`, `examples/batch_manager.rs`

**Known Issues**:
- Demo environment has eventual consistency: newly created orders may return 404 on immediate fetch
- `get_queue_positions()` requires `market_tickers` or `event_ticker` parameter (returns 400 without)

---

## Order Groups API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | ✅ | POST | `/portfolio/order_groups` | `create_order_group()` | |
| 🔲 | ✅ | GET | `/portfolio/order_groups/{id}` | `get_order_group()` | |
| 🔲 | | PUT | `/portfolio/order_groups/{id}` | `update_order_group()` | |
| 🔲 | ✅ | GET | `/portfolio/order_groups` | `list_order_groups()` | |
| 🔲 | ✅ | DELETE | `/portfolio/order_groups/{id}` | `delete_order_group()` | Cancels all orders |
| 🔲 | ✅ | PUT | `/portfolio/order_groups/{id}/reset` | `reset_order_group()` | |

**Source file**: `src/api/order_groups.rs`
**Verified in**: `examples/order_groups.rs`

---

## Portfolio API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/portfolio/balance` | `get_balance()` | |
| ✅ | ✅ | GET | `/portfolio/positions` | `get_positions()`, `get_positions_with_params()` | |
| ✅ | ✅ | GET | `/portfolio/fills` | `get_fills()`, `get_fills_with_params()` | |
| ✅ | ✅ | GET | `/portfolio/orders` | `get_orders()` | Listed under Orders |
| 🔲 | | GET | `/portfolio/settlements` | `get_settlements()`, `get_settlements_with_params()` | |

**Source file**: `src/api/portfolio.rs`
**Verified in**: `examples/portfolio.rs`, `examples/test_auth.rs`

---

## Subaccounts API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | | POST | `/portfolio/subaccounts` | `create_subaccount()` | Max 32 subaccounts |
| 🔲 | | POST | `/portfolio/subaccounts/transfer` | `transfer_between_subaccounts()` | |
| 🔲 | | GET | `/portfolio/subaccounts/balances` | `get_subaccount_balances()` | |
| 🔲 | | GET | `/portfolio/subaccounts/transfers` | `get_subaccount_transfers()` | |
| 🔲 | | GET | `/portfolio/summary/total_resting_order_value` | `get_resting_order_value()` | For FCM members |

**Source file**: `src/api/subaccounts.rs`

---

## Markets API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/markets` | `get_markets()`, `get_markets_with_params()` | |
| ✅ | ✅ | GET | `/markets/{ticker}` | `get_market()` | |
| ✅ | ✅ | GET | `/markets/{ticker}/orderbook` | `get_orderbook()`, `get_orderbook_with_params()` | |
| ✅ | ✅ | GET | `/markets/trades` | `get_trades()`, `get_trades_with_params()` | |
| 🔲 | ✅ | GET | `/series/{series}/markets/{ticker}/candlesticks` | `get_candlesticks()` | |
| 🔲 | ✅ | GET | `/markets/candlesticks` | `get_batch_candlesticks()` | Max 100 tickers |

**Source file**: `src/api/markets.rs`
**Verified in**: `examples/markets.rs`, `examples/candlesticks.rs`

---

## Events API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/events` | `get_events()`, `get_events_with_params()` | Excludes multivariate |
| ✅ | ✅ | GET | `/events/{ticker}` | `get_event()`, `get_event_with_params()` | |
| ✅ | ✅ | GET | `/events/{ticker}/metadata` | `get_event_metadata()` | |
| ✅ | ✅ | GET | `/events/multivariate` | `get_multivariate_events()` | |
| 🔲 | | GET | `/series/{s}/events/{e}/candlesticks` | `get_event_candlesticks()` | |
| 🔲 | | GET | `/series/{s}/events/{e}/forecast_percentile_history` | `get_event_forecast_percentile_history()` | |

**Source file**: `src/api/events.rs`
**Verified in**: `examples/events.rs`

---

## Series API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/series/{ticker}` | `get_series()` | |
| ✅ | ✅ | GET | `/series` | `get_series_list()`, `get_series_list_with_params()` | |

**Source file**: `src/api/series.rs`
**Verified in**: `examples/events.rs`

---

## Communications API (RFQ/Quotes)

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | | POST | `/communications/rfqs` | `create_rfq()` | |
| 🔲 | | GET | `/communications/rfqs` | `list_rfqs()`, `list_rfqs_with_params()` | |
| 🔲 | | GET | `/communications/rfqs/{id}` | `get_rfq()` | |
| 🔲 | | DELETE | `/communications/rfqs/{id}` | `cancel_rfq()` | |
| 🔲 | | POST | `/communications/quotes` | `create_quote()` | |
| 🔲 | | GET | `/communications/quotes` | `list_quotes()`, `list_quotes_with_params()` | |
| 🔲 | | GET | `/communications/quotes/{id}` | `get_quote()` | |
| 🔲 | | DELETE | `/communications/quotes/{id}` | `cancel_quote()` | |
| 🔲 | | PUT | `/communications/quotes/{id}/accept` | `accept_quote()` | |
| 🔲 | | PUT | `/communications/quotes/{id}/confirm` | `confirm_quote()` | Starts order execution timer |
| 🔲 | | GET | `/communications/id` | `get_communications_id()` | Get user's comms ID |

**Source file**: `src/api/communications.rs`

---

## Search API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/search/tags_by_categories` | `get_tags_by_categories()` | Some categories have null tags |
| ✅ | ✅ | GET | `/search/filters_by_sport` | `get_filters_by_sport()` | |

**Source file**: `src/api/search.rs`
**Verified in**: `examples/search.rs`

---

## Live Data API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | | GET | `/live_data/{type}/milestone/{id}` | `get_live_data()` | |
| 🔲 | | GET | `/live_data/batch` | `get_batch_live_data()` | |

**Source file**: `src/api/live_data.rs`

---

## Multivariate Collections API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | | GET | `/multivariate_event_collections` | `get_multivariate_collections()` | |
| 🔲 | | GET | `/multivariate_event_collections/{ticker}` | `get_multivariate_collection()` | |
| 🔲 | | POST | `/multivariate_event_collections/{ticker}` | `create_market_in_collection()` | |
| 🔲 | | GET | `/multivariate_event_collections/{ticker}/lookup` | `get_lookup_history()` | |
| 🔲 | | PUT | `/multivariate_event_collections/{ticker}/lookup` | `lookup_tickers()` | |

**Source file**: `src/api/multivariate.rs`

---

## API Keys API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | | GET | `/api_keys` | `get_api_keys()` | List all API keys |
| ✅ | | POST | `/api_keys` | `create_api_key()` | Premier/Market Maker tier |
| ✅ | | POST | `/api_keys/generate` | `generate_api_key()` | Auto-generates keypair |
| ✅ | | DELETE | `/api_keys/{api_key}` | `delete_api_key()` | Permanent deletion |

**Source file**: `src/api/api_keys.rs`

**Notes**: API keys are typically managed via the Kalshi web UI, but these endpoints allow programmatic management.

---

## Milestones API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ⚠️ | ✅ | GET | `/milestones` | `get_milestones()`, `get_milestones_with_params()` | API requires limit param |
| ✅ | ✅ | GET | `/milestones/{id}` | `get_milestone()` | |

**Source file**: `src/api/milestones.rs`
**Verified in**: `examples/milestones.rs`

---

## Structured Targets API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/structured_targets` | `get_structured_targets()`, `get_structured_targets_with_params()` | Pagination (1-2000) |
| ✅ | ✅ | GET | `/structured_targets/{id}` | `get_structured_target()` | |

**Source file**: `src/api/structured_targets.rs`
**Verified in**: `examples/structured_targets.rs`

---

## Incentive Programs API

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| ✅ | ✅ | GET | `/incentive_programs` | `get_incentive_programs()` | Rewards programs info |

**Source file**: `src/api/incentive_programs.rs`
**Verified in**: `examples/exchange_status.rs`

---

## FCM API (Futures Commission Merchant)

Specialized for FCM members only.

| Status | Verified | Method | Endpoint | Rust Function | Notes |
|--------|----------|--------|----------|---------------|-------|
| 🔲 | | GET | `/fcm/orders` | `get_fcm_orders()` | Requires FCM access |
| 🔲 | | GET | `/fcm/positions` | `get_fcm_positions()` | Requires FCM access |

**Source file**: `src/api/fcm.rs`

**Notes**: These endpoints require FCM member access level. They allow filtering orders and positions by subtrader ID.

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
| `series_and_search.rs` | Series, Fee Changes, Tags, Filters | ✅ Verified |

### Not Yet Tested

The following implemented endpoints need verification examples:

- **Candlesticks**: `get_candlesticks()`, `get_batch_candlesticks()`, `get_event_candlesticks()`
- **Forecast**: `get_event_forecast_percentile_history()`
- **Settlements**: `get_settlements()`
- **Order Groups**: All 6 endpoints
- **Subaccounts**: All 5 endpoints
- **Communications**: All 12 endpoints (including `confirm_quote()`, `get_communications_id()`)
- **Live Data**: `get_live_data()`, `get_batch_live_data()`
- **Multivariate Collections**: All 5 endpoints
- **API Keys**: All 4 endpoints (`get_api_keys()`, `create_api_key()`, `generate_api_key()`, `delete_api_key()`)
- **Milestones**: All 2 endpoints (`get_milestones()`, `get_milestone()`)
- **Structured Targets**: All 2 endpoints (`get_structured_targets()`, `get_structured_target()`)
- **Incentive Programs**: `get_incentive_programs()`
- **FCM**: All 2 endpoints (`get_fcm_orders()`, `get_fcm_positions()`) - requires FCM member access

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
