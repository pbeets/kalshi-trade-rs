//! API endpoint modules.
//!
//! These modules contain endpoint-specific logic. The public API is exposed
//! through flat methods on [`KalshiClient`](crate::KalshiClient).

pub(crate) mod communications;
pub(crate) mod events;
pub(crate) mod exchange;
pub(crate) mod live_data;
pub(crate) mod markets;
pub(crate) mod multivariate;
pub(crate) mod order_groups;
pub(crate) mod orders;
pub(crate) mod portfolio;
pub(crate) mod search;
pub(crate) mod series;
pub(crate) mod subaccounts;
