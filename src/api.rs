//! API endpoint modules.
//!
//! These modules contain endpoint-specific logic. The public API is exposed
//! through flat methods on [`KalshiClient`](crate::KalshiClient).

pub(crate) mod events;
pub(crate) mod exchange;
pub(crate) mod markets;
mod orders;
pub(crate) mod portfolio;
