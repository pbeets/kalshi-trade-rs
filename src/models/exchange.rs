//! Exchange models and response types.
//!
//! Types for exchange status, schedule, announcements, and data timestamps.

use serde::{Deserialize, Serialize};

/// Response from the GET /exchange/status endpoint.
///
/// Indicates whether the exchange and trading are currently active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeStatusResponse {
    /// Indicates if the core exchange accepts state changes.
    /// Returns false during maintenance; true otherwise.
    /// Covers trading, new users, and transfers.
    pub exchange_active: bool,

    /// Indicates if trading is currently permitted.
    /// True during exchange hours; false outside trading hours or during pauses.
    pub trading_active: bool,

    /// RFC3339 timestamp indicating estimated maintenance completion.
    /// Not guaranteed and subject to extension.
    #[serde(default)]
    pub exchange_estimated_resume_time: Option<String>,
}

/// Response from the GET /exchange/schedule endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeScheduleResponse {
    /// The exchange schedule configuration.
    pub schedule: ExchangeSchedule,
}

/// Exchange schedule containing standard hours and maintenance windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeSchedule {
    /// Weekly trading schedule periods with effective date ranges.
    pub standard_hours: Vec<StandardHoursPeriod>,

    /// Scheduled maintenance periods when the exchange may be unavailable.
    #[serde(default)]
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

/// A period of standard trading hours with its effective date range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardHoursPeriod {
    /// Start of the period when these hours are effective (RFC3339).
    #[serde(default)]
    pub start_time: Option<String>,

    /// End of the period when these hours are effective (RFC3339).
    #[serde(default)]
    pub end_time: Option<String>,

    /// Monday trading sessions.
    #[serde(default)]
    pub monday: Vec<TradingSession>,

    /// Tuesday trading sessions.
    #[serde(default)]
    pub tuesday: Vec<TradingSession>,

    /// Wednesday trading sessions.
    #[serde(default)]
    pub wednesday: Vec<TradingSession>,

    /// Thursday trading sessions.
    #[serde(default)]
    pub thursday: Vec<TradingSession>,

    /// Friday trading sessions.
    #[serde(default)]
    pub friday: Vec<TradingSession>,

    /// Saturday trading sessions.
    #[serde(default)]
    pub saturday: Vec<TradingSession>,

    /// Sunday trading sessions.
    #[serde(default)]
    pub sunday: Vec<TradingSession>,
}

/// A trading session with open and close times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSession {
    /// Session open time in HH:MM format (ET).
    pub open_time: String,

    /// Session close time in HH:MM format (ET).
    pub close_time: String,
}

/// A scheduled maintenance window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// Start of maintenance window (RFC3339).
    pub start_datetime: String,

    /// End of maintenance window (RFC3339).
    pub end_datetime: String,
}

/// Response from the GET /exchange/announcements endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAnnouncementsResponse {
    /// A list of exchange-wide announcements.
    pub announcements: Vec<Announcement>,
}

/// An exchange-wide announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    /// The type of the announcement.
    #[serde(rename = "type")]
    pub announcement_type: AnnouncementType,

    /// The message contained within the announcement.
    pub message: String,

    /// The time the announcement was delivered (RFC3339).
    pub delivery_time: String,

    /// The current status of this announcement.
    pub status: AnnouncementStatus,
}

/// Type of an announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnouncementType {
    /// Informational announcement.
    Info,
    /// Warning announcement.
    Warning,
    /// Error/critical announcement.
    Error,
}

/// Status of an announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnouncementStatus {
    /// Announcement is currently active.
    Active,
    /// Announcement is no longer active.
    Inactive,
}

/// Response from the GET /exchange/user_data_timestamp endpoint.
///
/// Provides an approximate indication of when user portfolio data
/// (balance, orders, fills, positions) was last validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataTimestampResponse {
    /// Timestamp when user data was last updated (RFC3339).
    pub as_of_time: String,
}
