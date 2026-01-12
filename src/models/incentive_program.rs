//! Incentive program models and response types.
//!
//! Incentive programs are rewards programs for trading activity on specific markets.

use serde::{Deserialize, Serialize};

/// An incentive program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveProgram {
    /// The unique identifier for the incentive program.
    #[serde(default)]
    pub id: Option<String>,
    /// The name of the incentive program.
    #[serde(default)]
    pub name: Option<String>,
    /// A description of the incentive program.
    #[serde(default)]
    pub description: Option<String>,
    /// The start date of the incentive program (RFC3339 timestamp).
    #[serde(default)]
    pub start_date: Option<String>,
    /// The end date of the incentive program (RFC3339 timestamp).
    #[serde(default)]
    pub end_date: Option<String>,
    /// The status of the incentive program.
    #[serde(default)]
    pub status: Option<String>,
    /// The associated series tickers.
    #[serde(default)]
    pub series_tickers: Option<Vec<String>>,
    /// The associated event tickers.
    #[serde(default)]
    pub event_tickers: Option<Vec<String>>,
    /// The associated market tickers.
    #[serde(default)]
    pub market_tickers: Option<Vec<String>>,
    /// The reward type (e.g., "volume_rebate", "maker_taker").
    #[serde(default)]
    pub reward_type: Option<String>,
    /// Additional program details/configuration.
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

/// Response from GET /incentive_programs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveProgramsResponse {
    /// The list of incentive programs.
    pub incentive_programs: Vec<IncentiveProgram>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_empty_response() {
        let json = r#"{"incentive_programs": []}"#;
        let response: IncentiveProgramsResponse = serde_json::from_str(json).unwrap();
        assert!(response.incentive_programs.is_empty());
    }

    #[test]
    fn test_deserialize_program() {
        let json = r#"{
            "incentive_programs": [{
                "id": "prog_123",
                "name": "Maker Rewards",
                "status": "active"
            }]
        }"#;
        let response: IncentiveProgramsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.incentive_programs.len(), 1);
        assert_eq!(response.incentive_programs[0].id, Some("prog_123".to_string()));
    }
}
