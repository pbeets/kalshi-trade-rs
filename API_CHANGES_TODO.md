# Kalshi API Changes TODO

> Tracking Kalshi API changelog changes since kalshi-trade-rs v0.2.0 (Jan 18, 2026).
> Each item is self-contained so an LLM agent can pick it up independently.
> Items are ordered chronologically (earliest first). Implement in order.
>
> **Reference**: https://docs.kalshi.com/changelog

---

## BATCH 1: January 22, 2026

### 1.1 — Get API Tier Limits Endpoint

**Changelog**: Jan 22, 2026 — "Get api tier limits endpoint"

**What changed**: New endpoint `GET /account/limits` returns the user's API tier and rate limits.

**What to do**:
- Create new model types in `src/models/` (e.g., `account.rs` or add to existing):
  - `ApiLimitsResponse` with fields for tier name, read/write rate limits
- Add API function in `src/api/` (e.g., `account.rs`):
  - `get_api_limits(http) -> Result<ApiLimitsResponse>`
  - Calls `GET /account/limits`
- Wire up in `src/client.rs` as `client.get_api_limits()`
- Export new types from `src/models.rs` and `src/lib.rs`

**Files to modify**: `src/models.rs`, `src/lib.rs`, `src/client.rs`, new `src/models/account.rs`, new `src/api/account.rs`

---

### 1.2 — Order Group Trigger Endpoint

**Changelog**: Jan 28, 2026 (announced Jan 22) — subaccount support entry mentions `PUT /portfolio/order_groups/{order_group_id}/trigger`

**What changed**: New endpoint `PUT /portfolio/order_groups/{order_group_id}/trigger` manually triggers an order group.

**What to do**:
- Add API function in `src/api/order_groups.rs`:
  - `trigger_order_group(http, order_group_id, subaccount?) -> Result<()>`
  - Calls `PUT /portfolio/order_groups/{order_group_id}/trigger`
  - Accepts optional `subaccount` parameter (see item 3.1)
- Wire up in `src/client.rs` as `client.trigger_order_group(id)`

**Files to modify**: `src/api/order_groups.rs`, `src/client.rs`

---

### 1.3 — Order Group Limit Update Endpoint

**Changelog**: Jan 29, 2026 — "Order group limit update endpoint"

**What changed**: New endpoint `PUT /portfolio/order_groups/{order_group_id}/limit` updates the contracts limit. Response types for `GET /portfolio/order_groups` and `GET /portfolio/order_groups/{id}` now include `contracts_limit` and `contracts_limit_fp`.

**What to do**:
- Add request type (e.g., `UpdateOrderGroupLimitRequest`) with `contracts_limit` field
- Add API function in `src/api/order_groups.rs`:
  - `update_order_group_limit(http, order_group_id, request) -> Result<()>`
  - Calls `PUT /portfolio/order_groups/{order_group_id}/limit`
  - Accepts optional `subaccount` parameter
- Add `contracts_limit` and `contracts_limit_fp` fields to `GetOrderGroupResponse` in `src/models/order_group.rs`
- Wire up in `src/client.rs`

**Files to modify**: `src/models/order_group.rs`, `src/api/order_groups.rs`, `src/client.rs`

**Current state**: `GetOrderGroupResponse` in `src/models/order_group.rs` has `is_auto_cancel_enabled` and `orders` but no `contracts_limit` fields.

---

### 1.4 — Order Group Updates WebSocket Channel

**Changelog**: Jan 22, 2026 — "Order group updates WebSocket channel"

**What changed**: New WS channel `order_group_updates` streams lifecycle events (created, triggered, reset, deleted, limit_updated). Payloads include `contracts_limit_fp` for created and limit_updated events. Requires authentication.

**What to do**:
- Add `OrderGroupUpdates` variant to `Channel` enum in `src/ws/channel.rs`:
  - Wire name: `"order_group_updates"`
  - Requires auth: yes
  - Requires market ticker: no
- Add `OrderGroupUpdateData` struct in `src/ws/message.rs` with fields:
  - `order_group_id: String`
  - `event_type: OrderGroupEventType` (enum: created, triggered, reset, deleted, limit_updated)
  - `contracts_limit_fp: Option<String>` (present for created, limit_updated)
- Add `OrderGroupUpdate(OrderGroupUpdateData)` variant to `StreamMessage`
- Update message parsing in `parse_stream_update()` for `"order_group_updates"` channel
- Update `Channel::requires_auth()` to include `OrderGroupUpdates`
- Export new types

**Files to modify**: `src/ws/channel.rs`, `src/ws/message.rs`, `src/ws/mod.rs` or `src/lib.rs`

---

### 1.5 — Fixed-Point Contract Count Fields (`*_fp`)

**Changelog**: Jan 22, 2026 — "Fixed-point contract count fields added to REST API"

**What changed**: `*_fp` string fields added across REST API for precise contract quantity representation. These are string-typed fixed-point decimal fields alongside existing integer fields.

**What to do**: Add optional `_fp` string fields to all structs that have contract count/quantity fields:

- `src/models/order.rs` — `Order` struct:
  - Add: `fill_count_fp`, `remaining_count_fp`, `initial_count_fp`, `taker_fill_count_fp`, `maker_fill_count_fp` (all `Option<String>`)
- `src/models/fill.rs` — `Fill` struct:
  - Add: `count_fp: Option<String>`
- `src/models/position.rs` — `MarketPosition` struct:
  - Add: `position_fp`, `volume_fp` (all `Option<String>`)
- `src/models/position.rs` — `EventPosition` struct:
  - Add: `total_cost_shares_fp: Option<String>` (if not present)
- `src/models/market.rs` — `Trade` struct:
  - Add: `count_fp: Option<String>`
- `src/models/market.rs` — `Orderbook` response:
  - Add `_fp` variants for size fields if applicable

Also add to WebSocket message types:
- `src/ws/message.rs` — `OrderbookDeltaData`: add `delta_fp: Option<String>`
- `src/ws/message.rs` — `TradeData`: add `count_fp: Option<String>`
- `src/ws/message.rs` — `FillData`: add `count_fp: Option<String>`
- `src/ws/message.rs` — `MarketPositionData`: add `position_fp`, `volume_fp` (all `Option<String>`)

**Files to modify**: `src/models/order.rs`, `src/models/fill.rs`, `src/models/position.rs`, `src/models/market.rs`, `src/ws/message.rs`

---

## BATCH 2: January 26–28, 2026

### 2.1 — Market `updated_time` Field and `min_updated_ts` Filter

**Changelog**: Jan 28, 2026 — "GetMarkets Min Updated Ts Filter"

**What changed**: `updated_time` field added to Market responses. `min_updated_ts` filter added to `GET /markets`.

**What to do**:
- Add `updated_time: Option<String>` to `Market` struct in `src/models/market.rs`
- Add `min_updated_ts: Option<i64>` to `GetMarketsParams` in `src/models/market.rs`
- Add builder method `min_updated_ts(mut self, ts: i64) -> Self`
- Add to query string builder

**Current state**: `Market` struct does NOT have `updated_time`. `GetMarketsParams` does NOT have `min_updated_ts`.

**Files to modify**: `src/models/market.rs`

---

### 2.2 — `MarketResult::Scalar` Variant

**Changelog**: Jan 28, 2026 — "Get markets may return scalar result"

**What changed**: Markets settled to a scalar result now return `"scalar"` instead of `""` in `market_result`.

**What to do**:
- Add `Scalar` variant to `MarketResult` enum in `src/models/market.rs`
- Currently the enum is: `Yes, No, None (#[serde(rename = "")]), Unknown (#[serde(other)])`
- Add: `Scalar` variant with `#[serde(rename = "scalar")]`
- Note: `Unknown` has `#[serde(other)]` so `"scalar"` currently deserializes as `Unknown`. Adding explicit `Scalar` variant will fix this.

**Current state** (line 58 of `src/models/market.rs`):
```rust
pub enum MarketResult {
    Yes,
    No,
    #[serde(rename = "")]
    None,
    #[serde(other)]
    Unknown,
}
```

**Files to modify**: `src/models/market.rs`

---

### 2.3 — Amend Order: Optional Client Order ID Fields

**Changelog**: Jan 22/28, 2026 — "Amend order endpoint: client_order_id fields now optional"

**What changed**: `client_order_id` and `updated_client_order_id` in amend order requests are now optional. Orders can be identified by `order_id` alone.

**What to do**:
- Check `AmendOrderRequest` in `src/models/order.rs` — ensure both fields are `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`
- If they are already optional, this may be a no-op

**Files to modify**: `src/models/order.rs` (verify)

---

### 2.4 — Subaccount Support for Cancel, Amend, Decrease, Order Group Operations

**Changelog**: Jan 28, 2026 — "Subaccount support for cancel, amend, decrease order, and order group operations"

**What changed**: These endpoints now accept optional `subaccount` parameter:
- `DELETE /portfolio/orders/{order_id}` (cancel)
- `POST /portfolio/orders/{order_id}/amend`
- `POST /portfolio/orders/{order_id}/decrease`
- `POST /portfolio/order_groups` (create)
- `PUT /portfolio/order_groups/{order_group_id}/limit`
- `PUT /portfolio/order_groups/{order_group_id}/trigger`
- `DELETE /portfolio/order_groups/{order_group_id}`

**What to do**:
- Add optional `subaccount: Option<i32>` parameter to the API functions in:
  - `src/api/orders.rs` — `cancel_order`, `amend_order`, `decrease_order`
  - `src/api/order_groups.rs` — `create_order_group`, `delete_order_group`, `reset_order_group`, plus the new `trigger` and `limit` endpoints
- For body-based requests, add `subaccount` to the request struct
- For delete/path-based requests, add as query parameter
- Update `src/client.rs` method signatures accordingly

**Current state**: `src/api/orders.rs` does NOT have subaccount params on cancel/amend/decrease. `src/api/order_groups.rs` does NOT have subaccount support.

**Files to modify**: `src/api/orders.rs`, `src/api/order_groups.rs`, `src/models/order.rs`, `src/models/order_group.rs`, `src/client.rs`

---

### 2.5 — Per-Order Subaccount in Batch Cancels

**Changelog**: Jan 28, 2026 — "Per-order subaccount support in batch cancels"

**What changed**: Batch cancel now supports per-order subaccounts. New shape: `orders: [{ order_id, subaccount? }]` where subaccount defaults to 0. Legacy `ids` array still accepted.

**What to do**:
- Check current batch cancel request type in `src/models/order.rs`
- Add new struct for per-order cancel items: `BatchCancelOrderItem { order_id: String, subaccount: Option<i32> }`
- Update `BatchCancelOrdersRequest` to support both legacy `ids` format and new `orders` format
- Or replace with the new format (since legacy is backwards compatible)

**Files to modify**: `src/models/order.rs`, possibly `src/api/orders.rs`

---

### 2.6 — Dollar-Denominated Target Cost for RFQs

**Changelog**: Jan 28, 2026 — "Dollar-denominated target cost for RFQs"

**What changed**: RFQ and Quote endpoints support `target_cost_dollars` as fixed-point dollar string. `target_cost_centi_cents` is deprecated.

**What to do**:
- Add `target_cost_dollars: Option<String>` to:
  - `CreateRfqRequest` in `src/models/communications.rs`
  - `Rfq` response struct
  - `Quote` response struct (if it has target cost)
- Mark `target_cost_centi_cents` as deprecated in docs

**Current state**: `CreateRfqRequest` has `target_cost_centi_cents: Option<i64>` but likely no `target_cost_dollars`.

**Files to modify**: `src/models/communications.rs`

---

## BATCH 3: January 29, 2026

### 3.1 — Orders Return `subaccount_number`

**Changelog**: Jan 29, 2026 — "Orders return subaccount number"

**What changed**: Order responses include `subaccount_number` (0 for primary, 1-32 for subaccounts).

**What to do**:
- Add `subaccount_number: Option<i32>` to `Order` struct in `src/models/order.rs`

**Current state**: `Order` struct does NOT have `subaccount_number`.

**Files to modify**: `src/models/order.rs`

---

### 3.2 — Subaccount Filter Behavior Change

**Changelog**: Jan 29, 2026 — "Subaccount filter behavior change for orders, fills, and settlements"

**What changed**: When `subaccount` param is omitted, results return across ALL subaccounts. When provided (including 0), results filter to that specific subaccount.

**What to do**:
- Verify `GetOrdersParams` already has `subaccount: Option<i32>` — YES it does (line 118-120 of `src/models/order.rs`)
- Add `subaccount: Option<i32>` to `GetFillsParams` in `src/models/fill.rs` (if not present)
- Add `subaccount: Option<i32>` to `GetSettlementsParams` in `src/models/settlement.rs` (if not present)
- Update documentation to reflect new behavior

**Files to modify**: `src/models/fill.rs`, `src/models/settlement.rs` (verify each)

---

### 3.3 — `fee_cost` on Fills API

**Changelog**: Jan 27/29, 2026 — "Exchange Fee available on Fills API"

**What changed**: `GET /portfolio/fills` now returns `fee_cost` field.

**What to do**:
- Add `fee_cost: Option<String>` to `Fill` struct in `src/models/fill.rs`

**Current state**: `Fill` struct does NOT have `fee_cost`.

**Files to modify**: `src/models/fill.rs`

---

### 3.4 — Fee Cost in Fill WebSocket Messages

**Changelog**: Jan 29, 2026 — "Fee cost added to fill WebSocket messages"

**What changed**: Fill WS messages now include `fee_cost` as fixed-point dollars string.

**What to do**:
- Add `fee_cost: Option<String>` (with `skip_serializing_if`) to `FillData` in `src/ws/message.rs`

**Current state** (line 194): `FillData` does NOT have `fee_cost`.

**Files to modify**: `src/ws/message.rs`

---

### 3.5 — Subaccount Balance String Dollars

**Changelog**: Jan 27, 2026 — "Subaccount Balance returns string dollars representation"

**What changed**: Subaccount balance field is now represented as fixed-point dollars string instead of centicent integer.

**What to do**:
- Check `SubaccountBalancesResponse` and related types in `src/models/subaccount.rs`
- Ensure balance fields include dollar-string representations or update field types

**Files to modify**: `src/models/subaccount.rs`

---

## ~~BATCH 4: January 30 – February 5, 2026~~ ✅ DONE

### ~~4.1 — Queue Position Fixed-Point Field~~ ✅

Already implemented in batch 1 (`queue_position_fp` on `QueuePosition` and `OrderQueuePositionResponse`).

### ~~4.2 — Subaccount Support for RFQs~~ ✅

`CreateRfqRequest.subaccount` was implemented in batch 2. Added `subaccount` filter to `ListRfqsParams`.

### ~~4.3 — Subaccount Support for RFQ Quotes~~ ✅

Already implemented in batch 2 (`CreateQuoteRequest.subaccount`).

### ~~4.4 — User Orders WebSocket Channel~~ ✅

Added `UserOrders` channel, `UserOrderData`, `UserOrderEventType`, message parsing.

### ~~4.5 — Order Group Read Endpoints Subaccount Parameter~~ ✅

Added `subaccount` to `GetOrderGroupsParams` and `get_order_group()` API function. Added `get_order_group_for_subaccount()` to client.

### ~~4.6 — `market_id` on Incentive Programs API~~ ✅

Added `market_id: String` (required) and `target_size_fp: Option<String>` to `IncentiveProgram`. Tightened all required fields (`id`, `market_id`, `market_ticker`, `incentive_type`, `start_date`, `end_date`, `period_reward`, `paid_out`) to non-optional per the official OpenAPI spec.

---

## ~~BATCH 5: February 11–12, 2026~~ ✅ DONE

### ~~5.1 — CreateOrder Removes `type` Field~~ ✅

Removed `order_type` field from `CreateOrderRequest`. Deprecated `OrderType::Market` variant (kept for historical data on `Order` response).

### ~~5.2 — `fractional_trading_enabled` on Market~~ ✅

Already implemented (`fractional_trading_enabled` on `Market` struct).

### ~~5.3 — WebSocket QoL: Ticker High-Precision Time and `skip_ticker_ack`~~ ✅

Added `time: Option<String>` to `TickerData`. Added `skip_ticker_ack` flag threading through `SubscribeOptions` → `StreamCommand::Subscribe` → `build_subscribe` → wire protocol. Added `subscribe_with_options()` public API.

### ~~5.4 — L1 Orderbook Sizes on Ticker WebSocket~~ ✅

Added `yes_bid_size_fp`, `yes_ask_size_fp`, `bid_size_fp`, `ask_size_fp`, `last_trade_size_fp` to `TickerData`.

---

## ~~BATCH 6: February 13–19, 2026~~ ✅ DONE

### ~~6.1 — Market Liquidity Fields Deprecated~~ ✅

Added `#[deprecated]` attributes to `liquidity` and `liquidity_dollars` fields on `Market` struct. Added `#[allow(deprecated)]` on the struct itself to suppress serde deserialization warnings.

### ~~6.2 — Subaccount Filtering on `GET /portfolio/balance`~~ ✅

Added `GetBalanceParams` with `subaccount` builder. Updated `get_balance()` API function to accept params. Added `get_balance_with_params()` to client. Omitting subaccount returns combined balance across all subaccounts; `subaccount(0)` filters to primary only.

### ~~6.3 — `settlement_value` on `market_lifecycle_v2` Determined Events~~ ✅

Added `settlement_value: Option<String>` to `MarketLifecycleData` in WS message types.

### ~~6.4 — Historical Data Endpoints~~ ✅

Created `src/models/historical.rs` with `HistoricalCutoffResponse`, `GetHistoricalMarketsParams`, `GetHistoricalCandlesticksParams`, `GetHistoricalFillsParams`, `GetHistoricalOrdersParams`. Created `src/api/historical.rs` with 6 endpoint functions. Wired up in client with 10 methods (default + parameterized variants). Cutoff timestamps are ISO 8601 strings per API docs. Response types reuse existing `MarketsResponse`, `MarketResponse`, `CandlesticksResponse`, `FillsResponse`, `OrdersResponse`.

---

## SUMMARY

| Batch | Date Range | Items | Theme |
|-------|-----------|-------|-------|
| 1 | Jan 22 | 1.1–1.5 | New endpoints, WS channel, `_fp` fields |
| 2 | Jan 26–28 | 2.1–2.6 | Market fields, subaccount expansion, RFQ dollars |
| 3 | Jan 29 | 3.1–3.5 | Order subaccount, fee_cost, fill WS, balance |
| 4 | Jan 30–Feb 5 | 4.1–4.6 | Queue FP, subaccount RFQ, user_orders WS, order groups |
| 5 | Feb 11–12 | 5.1–5.4 | Remove market type, fractional trading, ticker WS enrichment |
| ~~6~~ | ~~Feb 13–19~~ | ~~6.1–6.4~~ | ~~Deprecations, balance subaccount, lifecycle, historical~~ ✅ |

**Total: 25 work items across 6 batches.**
