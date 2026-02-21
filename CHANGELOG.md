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
- `contracts_limit_fp` fixed-point string field on `CreateOrderGroupRequest`,
  `UpdateOrderGroupLimitRequest`, `GetOrderGroupResponse`, and `OrderGroupSummary`.
  The integer `contracts_limit` field is now optional (provide one or both).
- New `OrderGroupUpdates` WebSocket channel for order group lifecycle events
  (`OrderGroupUpdateData`, `OrderGroupEventType`).
- Fixed-point `_fp` fields added to WebSocket message types: `delta_fp` on
  `OrderbookDeltaData`, `volume_fp`/`open_interest_fp` on `TickerData`,
  `count_fp` on `TradeData`, `count_fp`/`post_position_fp` on `FillData`,
  and `position_fp`/`volume_fp` on `MarketPositionData`.

### Fixed

- `OrderbookAggregator` now drops delta messages that arrive before a snapshot
  instead of creating ghost orderbook entries. Previously, early deltas would
  insert empty uninitialized books into the state map, causing `full_book()` and
  `tracked_markets()` to return stale or invalid data.

### Changed

- **Breaking:** Removed `initialized` field from `OrderbookSummary`. The
  aggregator now guarantees that summaries are only produced for fully
  initialized orderbooks, making the field redundant. Remove any
  `summary.initialized` checks from your code.
- `Channel::requires_market_ticker()` now returns `true` only for
  `OrderbookDelta`. Other market data channels (`Ticker`, `Trade`,
  `MarketLifecycle`, `Multivariate`) support subscribing with an empty ticker
  list to receive updates for all markets.

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
