# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-18

### Added

- `ConnectionLost` now includes a `subscriptions` field containing the channels and
  markets that were active at the time of disconnection. This enables automatic
  resubscription after reconnecting.

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
