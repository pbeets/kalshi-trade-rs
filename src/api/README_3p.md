# Kalshi REST API Reference

Complete reference for all Kalshi REST API endpoints supported by this library.

**Official Documentation**: [docs.kalshi.com](https://docs.kalshi.com)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and verified |
| ⚠️ | Implemented with caveats (see notes) |
| 🔲 | Implemented, not yet verified |

---

## Summary

| Category | Endpoints | Coverage |
|----------|-----------|----------|
| Exchange | 5 | 100% |
| Orders | 10 | 100% |
| Order Groups | 5 | 100% |
| Portfolio | 5 | 100% |
| Subaccounts | 5 | 100% |
| Markets | 6 | 100% |
| Events | 6 | 100% |
| Series | 3 | 100% |
| Communications (RFQ/Quotes) | 11 | 100% |
| Search | 2 | 100% |
| Live Data | 2 | 100% |
| Multivariate Collections | 5 | 100% |
| API Keys | 4 | 100% |
| Milestones | 2 | 100% |
| Structured Targets | 2 | 100% |
| Incentive Programs | 1 | 100% |
| FCM | 2 | 100% |
| **Total** | **76** | **100%** |

---

## Exchange API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/exchange/status` | `get_exchange_status()` | Public endpoint |
| ✅ | GET | `/exchange/schedule` | `get_exchange_schedule()` | Public endpoint |
| ✅ | GET | `/exchange/announcements` | `get_exchange_announcements()` | Public endpoint |
| ✅ | GET | `/exchange/user_data_timestamp` | `get_user_data_timestamp()` | Requires auth |
| 🔲 | GET | `/series/fee_changes` | `get_fee_changes()` | Fee change notifications |

**Source**: `src/api/exchange.rs`, `src/api/series.rs`

---

## Orders API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | POST | `/portfolio/orders` | `create_order()` | |
| ⚠️ | GET | `/portfolio/orders/{id}` | `get_order()` | Demo env: eventual consistency |
| ✅ | DELETE | `/portfolio/orders/{id}` | `cancel_order()` | |
| ✅ | POST | `/portfolio/orders/{id}/amend` | `amend_order()` | |
| 🔲 | POST | `/portfolio/orders/{id}/decrease` | `decrease_order()` | |
| ✅ | GET | `/portfolio/orders` | `get_orders()` | |
| ✅ | POST | `/portfolio/orders/batched` | `batch_create_orders()` | Max 20 orders |
| ✅ | DELETE | `/portfolio/orders/batched` | `batch_cancel_orders()` | Max 20 orders |
| ⚠️ | GET | `/portfolio/orders/queue_positions` | `get_queue_positions()` | Requires ticker param |
| ⚠️ | GET | `/portfolio/orders/{id}/queue_position` | `get_order_queue_position()` | Demo env: eventual consistency |

**Source**: `src/api/orders.rs`

**Notes**:
- Demo environment has eventual consistency for newly created orders
- `get_queue_positions()` requires `market_tickers` or `event_ticker` parameter

---

## Order Groups API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/portfolio/order_groups/create` | `create_order_group()` | |
| 🔲 | GET | `/portfolio/order_groups/{id}` | `get_order_group()` | |
| 🔲 | GET | `/portfolio/order_groups` | `list_order_groups()` | |
| 🔲 | DELETE | `/portfolio/order_groups/{id}` | `delete_order_group()` | Cancels all orders in group |
| 🔲 | PUT | `/portfolio/order_groups/{id}/reset` | `reset_order_group()` | Resets contracts counter |

**Source**: `src/api/order_groups.rs`

---

## Portfolio API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/portfolio/balance` | `get_balance()` | |
| ✅ | GET | `/portfolio/positions` | `get_positions()` | |
| ✅ | GET | `/portfolio/fills` | `get_fills()` | |
| ✅ | GET | `/portfolio/orders` | `get_orders()` | See Orders API |
| 🔲 | GET | `/portfolio/settlements` | `get_settlements()` | |

**Source**: `src/api/portfolio.rs`

---

## Subaccounts API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/portfolio/subaccounts` | `create_subaccount()` | Max 32 subaccounts |
| 🔲 | POST | `/portfolio/subaccounts/transfer` | `transfer_between_subaccounts()` | |
| 🔲 | GET | `/portfolio/subaccounts/balances` | `get_subaccount_balances()` | |
| 🔲 | GET | `/portfolio/subaccounts/transfers` | `get_subaccount_transfers()` | |
| 🔲 | GET | `/portfolio/summary/total_resting_order_value` | `get_resting_order_value()` | FCM members only |

**Source**: `src/api/subaccounts.rs`

---

## Markets API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/markets` | `get_markets()` | |
| ✅ | GET | `/markets/{ticker}` | `get_market()` | |
| ✅ | GET | `/markets/{ticker}/orderbook` | `get_orderbook()` | |
| ✅ | GET | `/markets/trades` | `get_trades()` | |
| 🔲 | GET | `/series/{s}/markets/{t}/candlesticks` | `get_candlesticks()` | |
| 🔲 | GET | `/markets/candlesticks` | `get_batch_candlesticks()` | Max 100 tickers |

**Source**: `src/api/markets.rs`

---

## Events API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/events` | `get_events()` | Excludes multivariate |
| ✅ | GET | `/events/{ticker}` | `get_event()` | |
| ✅ | GET | `/events/{ticker}/metadata` | `get_event_metadata()` | |
| ✅ | GET | `/events/multivariate` | `get_multivariate_events()` | |
| 🔲 | GET | `/series/{s}/events/{e}/candlesticks` | `get_event_candlesticks()` | |
| 🔲 | GET | `/series/{s}/events/{e}/forecast_percentile_history` | `get_event_forecast_percentile_history()` | |

**Source**: `src/api/events.rs`

---

## Series API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/series/{ticker}` | `get_series()` | |
| 🔲 | GET | `/series` | `get_series_list()` | |
| 🔲 | GET | `/series/fee_changes` | `get_fee_changes()` | |

**Source**: `src/api/series.rs`

---

## Communications API (RFQ/Quotes)

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | POST | `/communications/rfqs` | `create_rfq()` | |
| 🔲 | GET | `/communications/rfqs` | `list_rfqs()` | |
| 🔲 | GET | `/communications/rfqs/{id}` | `get_rfq()` | |
| 🔲 | DELETE | `/communications/rfqs/{id}` | `cancel_rfq()` | |
| 🔲 | POST | `/communications/quotes` | `create_quote()` | |
| 🔲 | GET | `/communications/quotes` | `list_quotes()` | |
| 🔲 | GET | `/communications/quotes/{id}` | `get_quote()` | |
| 🔲 | DELETE | `/communications/quotes/{id}` | `cancel_quote()` | |
| 🔲 | PUT | `/communications/quotes/{id}/accept` | `accept_quote()` | |
| 🔲 | PUT | `/communications/quotes/{id}/confirm` | `confirm_quote()` | Starts execution timer |
| 🔲 | GET | `/communications/id` | `get_communications_id()` | Get user's comms ID |

**Source**: `src/api/communications.rs`

---

## Search API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/search/tags_by_categories` | `get_tags_by_categories()` | |
| 🔲 | GET | `/search/filters_by_sport` | `get_filters_by_sport()` | |

**Source**: `src/api/search.rs`

---

## Live Data API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/live_data/{type}/milestone/{id}` | `get_live_data()` | |
| 🔲 | GET | `/live_data/batch` | `get_batch_live_data()` | |

**Source**: `src/api/live_data.rs`

---

## Multivariate Collections API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/multivariate_event_collections` | `get_multivariate_collections()` | |
| 🔲 | GET | `/multivariate_event_collections/{ticker}` | `get_multivariate_collection()` | |
| 🔲 | POST | `/multivariate_event_collections/{ticker}` | `create_market_in_collection()` | |
| 🔲 | GET | `/multivariate_event_collections/{ticker}/lookup` | `get_lookup_history()` | |
| 🔲 | PUT | `/multivariate_event_collections/{ticker}/lookup` | `lookup_tickers()` | |

**Source**: `src/api/multivariate.rs`

---

## API Keys API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/api_keys` | `get_api_keys()` | List all API keys |
| ✅ | POST | `/api_keys` | `create_api_key()` | Premier/Market Maker tier |
| ✅ | POST | `/api_keys/generate` | `generate_api_key()` | Auto-generates keypair |
| ✅ | DELETE | `/api_keys/{api_key}` | `delete_api_key()` | Permanent deletion |

**Source**: `src/api/api_keys.rs`

---

## Milestones API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/milestones` | `get_milestones()` | Date filtering support |
| ✅ | GET | `/milestones/{id}` | `get_milestone()` | |

**Source**: `src/api/milestones.rs`

---

## Structured Targets API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| ✅ | GET | `/structured_targets` | `get_structured_targets()` | Pagination (1-2000) |
| ✅ | GET | `/structured_targets/{id}` | `get_structured_target()` | |

**Source**: `src/api/structured_targets.rs`

---

## Incentive Programs API

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/incentive_programs` | `get_incentive_programs()` | Supports filtering |

**Source**: `src/api/incentive_programs.rs`

**Query Parameters**:
- `status`: "all", "active", "upcoming", "closed", "paid_out"
- `type`: "all", "liquidity", "volume"
- `limit`: Max results (default 100, max 10000)
- `cursor`: Pagination cursor

---

## FCM API

Specialized endpoints for FCM (Futures Commission Merchant) members.

| Status | Method | Endpoint | Rust Function | Notes |
|--------|--------|----------|---------------|-------|
| 🔲 | GET | `/fcm/orders` | `get_fcm_orders()` | FCM access required |
| 🔲 | GET | `/fcm/positions` | `get_fcm_positions()` | FCM access required |

**Source**: `src/api/fcm.rs`

---

## Base URLs

| Environment | REST API | WebSocket |
|-------------|----------|-----------|
| Demo | `https://demo-api.kalshi.co/trade-api/v2` | `wss://demo-api.kalshi.co/trade-api/ws/v2` |
| Production | `https://trading-api.kalshi.com/trade-api/v2` | `wss://trading-api.kalshi.com/trade-api/ws/v2` |

---

## Validation

Request parameters are validated before sending to the API. Methods with validation provide both:

- **Infallible constructors** (e.g., `new()`, `limit()`) - panic on invalid input
- **Fallible constructors** (e.g., `try_new()`, `try_limit()`) - return `Result` for error handling

Example:
```rust
// Panics if limit is invalid
let params = GetOrdersParams::new().limit(100);

// Returns Result for error handling
let params = GetOrdersParams::new().try_limit(100)?;
```

---

## Error Handling

All API methods return `Result<T, Error>`. Common error types:

| Error | Description |
|-------|-------------|
| `Error::Http` | Network or HTTP errors |
| `Error::Api` | Kalshi API errors (with message) |
| `Error::Auth` | Authentication failures |
| `Error::InvalidPrice` | Price outside valid range (1-99 cents) |
| `Error::InvalidLimit` | Limit outside valid range |
| `Error::BatchSizeExceeded` | Batch request exceeds max size |

---

## References

- [Official Kalshi API Documentation](https://docs.kalshi.com)
- [API Changelog](https://docs.kalshi.com/changelog)
