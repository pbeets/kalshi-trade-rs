//! API endpoint modules.
//!
//! These modules contain endpoint-specific logic. The public API is exposed
//! through flat methods on [`KalshiClient`](crate::KalshiClient).

mod events;
pub(crate) mod exchange;
mod markets;
mod orders;
pub(crate) mod portfolio;
