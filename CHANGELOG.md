# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New `trigger_order_group()` endpoint to manually trigger an order group's
  auto-cancel (PUT `/portfolio/order_groups/{id}/trigger`).
- New `update_order_group_limit()` endpoint to change an order group's contracts
  limit (PUT `/portfolio/order_groups/{id}/limit`).
- New `get_api_limits()` endpoint to retrieve API tier and rate limits
  (GET `/account/limits`).
- New `OrderGroupUpdates` WebSocket channel for order group lifecycle events
  (`OrderGroupUpdateData`, `OrderGroupEventType`).
- Subaccount support across order operations: `cancel_order_for_subaccount()`,
  `delete_order_group_for_subaccount()`, `reset_order_group_for_subaccount()`,
  `trigger_order_group_for_subaccount()` convenience methods on `KalshiClient`.
  Optional `subaccount` field added to `CreateOrderRequest`, `AmendOrderRequest`,
  `DecreaseOrderRequest`, and `CreateOrderGroupRequest`.
- Per-order subaccount support in batch cancels via new `BatchCancelOrderItem`
  type and `BatchCancelOrdersRequest::with_orders()`. New
  `cancel_orders_with_items()` method on `BatchManager`.
- `subaccount_number` field on `Order` and `Fill` response types.
- Subaccount support for RFQ/Quote operations: `subaccount` field on
  `CreateRfqRequest` and `CreateQuoteRequest`.
- `target_cost_dollars` field on `CreateRfqRequest`, `Rfq`, and `Quote` as a
  dollar-denominated fixed-point string (replaces deprecated
  `target_cost_centi_cents`).
- `side` and `expires_in_seconds` on `CreateRfqRequest`; `side` and `expires_ts`
  on `Rfq` and `Quote` response types.
- Partial acceptance support: `contracts` field on `AcceptQuoteRequest` and
  `CreateQuoteRequest`.
- Fixed-point `_fp` fields on REST API types: `count_fp` on `CreateOrderRequest`
  and `AmendOrderRequest`, `reduce_by_fp`/`reduce_to_fp` on
  `DecreaseOrderRequest`, `queue_position_fp` on `QueuePosition` and
  `OrderQueuePositionResponse`, `contracts_fp` on `CreateRfqRequest`/`Rfq`/`Quote`,
  `contracts_limit_fp` on `CreateOrderGroupRequest`/`UpdateOrderGroupLimitRequest`/
  `GetOrderGroupResponse`/`OrderGroupSummary`.
- Fixed-point `_fp` fields on WebSocket message types: `delta_fp` on
  `OrderbookDeltaData`, `volume_fp`/`open_interest_fp` on `TickerData`,
  `count_fp` on `TradeData`, `count_fp`/`post_position_fp` on `FillData`,
  and `position_fp`/`volume_fp` on `MarketPositionData`.
- `Settled` variant for `MarketStatus` enum and `Scalar` variant for
  `MarketResult` enum.
- Integer cent price fields on `Market`: `yes_bid`, `yes_ask`, `no_bid`,
  `no_ask`, `last_price`, `previous_yes_bid`, `previous_yes_ask`,
  `previous_price`, `notional_value`, `liquidity`.
- Market metadata fields: `category`, `response_price_units`, `risk_limit_cents`,
  `tick_size`, `updated_time`, `cap_count`, `fractional_trading_enabled`,
  `settlement_value`, `expiration_value`.
- `min_updated_ts` filter on `GetMarketsParams` for querying markets by metadata
  update time.
- `name` and `subaccount` fields on `CreateOrderGroupRequest`; `name`, `status`,
  `created_time` on `OrderGroupSummary`.
- `fee_cost` on `Fill` for exchange fee cost as a fixed-point dollar string.
- `market_result` and `total_cost` on `MarketPosition`.
- `status` field on `Event`, `Milestone`, and `StructuredTarget`.
- Series metadata fields: `category`, `status`, `tags`, `settlement_sources`
  (now typed as `Vec<SettlementSource>`), `contract_url`, `contract_terms_url`,
  `product_metadata`, `fee_type`, `fee_multiplier`, `additional_prohibitions`.
- `client_transfer_id` on `TransferBetweenSubaccountsRequest` for idempotent
  transfers.
- Multivariate collection enrichments: new `CollectionEvent` and
  `AssociatedEvent` types; `events`, `open_date`, `close_date`,
  `associated_events`, `is_ordered`, `size_min`, `size_max`,
  `functional_description` fields on `MultivariateEventCollection`.
- Selected-market support for multivariate lookups: `selected_markets` field and
  `with_selected_markets()` constructor on `CreateMarketInCollectionRequest` and
  `LookupTickersRequest`.
- `lookback_seconds` filter on `GetLookupHistoryParams`.

### Fixed

- `OrderbookAggregator` now drops delta messages that arrive before a snapshot
  instead of creating ghost orderbook entries. Previously, early deltas would
  insert empty uninitialized books into the state map, causing `full_book()` and
  `tracked_markets()` to return stale or invalid data.

### Changed

- **Breaking:** `AmendOrderRequest::new()` no longer requires `client_order_id`
  and `updated_client_order_id` — these are now optional. Use
  `AmendOrderRequest::with_client_order_ids()` for the previous behavior.
- **Breaking:** `BatchCancelOrdersRequest::ids` is now `Option<Vec<String>>`.
  Existing code using `.ids` directly must unwrap. Use
  `BatchCancelOrdersRequest::with_orders()` for the new per-order subaccount
  format.
- **Breaking:** `Rfq::target_cost_dollars()` renamed to
  `Rfq::target_cost_as_dollars()` to avoid conflict with the new
  `target_cost_dollars` field.
- **Breaking:** `Quote::rfq_target_cost_dollars()` renamed to
  `Quote::rfq_target_cost_as_dollars()`.
- **Breaking:** `CreateMarketInCollectionRequest::variables` and
  `LookupTickersRequest::variables` changed from `HashMap` to `Option<HashMap>`
  to support the new `selected_markets` alternative.
- **Breaking:** Removed `initialized` field from `OrderbookSummary`. The
  aggregator now guarantees that summaries are only produced for fully
  initialized orderbooks, making the field redundant. Remove any
  `summary.initialized` checks from your code.
- `CreateRfqRequest::with_target_cost_dollars()` now sends the dollar amount via
  the `target_cost_dollars` field instead of converting to centi-cents.
- `Channel::requires_market_ticker()` now returns `true` only for
  `OrderbookDelta`. Other market data channels (`Ticker`, `Trade`,
  `MarketLifecycle`, `Multivariate`) support subscribing with an empty ticker
  list to receive updates for all markets.

### Deprecated

- `CreateOrderRequest::sell_position_floor()` — use `reduce_only` instead. Only
  accepts value of 0.
- `Rfq::target_cost_as_dollars()` and `Quote::rfq_target_cost_as_dollars()` —
  use the `target_cost_dollars` / `rfq_target_cost_dollars` fields directly.
- `BatchCancelOrdersRequest::new()` and `try_new()` — use `with_orders()` /
  `try_with_orders()` for per-order subaccount support.
- `target_cost_centi_cents` on RFQ/Quote types — use `target_cost_dollars`
  instead.
- `Order.queue_position` — always returns 0; use the `get_order_queue_position`
  endpoint instead.
- `Fill.price` and `Trade.price` — use `yes_price` / `no_price` instead.
- `Fill.trade_id`, `Fill.market_ticker`, `Fill.ts` — legacy field names.
- `Event.category` — use series-level category instead.
- `MarketPosition.resting_orders_count` — deprecated by the API.

## [0.2.0] - 2026-01-18

### Added

- `ConnectionLost` now includes a `subscriptions` field containing the channels and
  markets that were active at the time of disconnection. This enables automatic
  resubscription after reconnecting.
- Fixed-point decimal fields (`*_dollars` suffix) added throughout the API for
  prices and costs, avoiding floating-point precision issues.
- WebSocket sharding support via `CommunicationsSharding` for high-throughput
  market makers handling RFQ traffic across multiple connections.
- Exported `CommunicationData`, `RfqData`, `RfqDeletedData`, `QuoteData`, and
  `QuoteAcceptedData` types from the `ws` module for easier RFQ event handling.
- New `rfq_verify` example demonstrating read-only RFQ API verification.
- RFQ documentation in README with usage examples for creating RFQs, submitting
  quotes, and streaming events.

### Changed

- **Breaking:** `StreamMessage::ConnectionLost` variant now has two fields: `reason`
  and `subscriptions`. Update pattern matches from `ConnectionLost { reason }` to
  `ConnectionLost { reason, .. }` or `ConnectionLost { reason, subscriptions }`.
- Internal: `SubscriptionState` and `SharedSubscriptions` types moved from `client`
  module to `session` module and made public for internal sharing.
- Internal: Simplified `KalshiStreamSession` constructors by consolidating
  `connect()`, `connect_with_health()`, and `connect_with_ready()` into a single
  `connect()` method. This is not a breaking change as `KalshiStreamSession` is
  not part of the public API.

### Fixed

- Improved documentation for WebSocket reconnection patterns with examples showing
  how to use the new `subscriptions` field.

## [0.1.0] - 2026-01-15

### Added

- Initial release
- REST API client with full endpoint coverage
- WebSocket streaming client with subscription management
- RSA-PSS authentication
- Typed `DisconnectReason` enum for connection lifecycle events
- Health monitoring with configurable ping/pong timeouts
- Connection strategies: `Simple` (fast-fail) and `Retry` (exponential backoff)
- Support for all public and authenticated WebSocket channels
