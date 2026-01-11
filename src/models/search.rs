//! Search models for market discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from GET /search/tags_by_categories.
///
/// Contains a mapping of series categories to their associated tags,
/// which can be used for filtering and search functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsByCategoriesResponse {
    /// Mapping of series categories to their associated tags.
    pub tags_by_categories: HashMap<String, Vec<String>>,
}

/// Competition filter details within a sport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionFilter {
    /// Available scopes for this competition.
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Sport filter details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportFilter {
    /// Available scopes for this sport.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Competitions within this sport.
    #[serde(default)]
    pub competitions: HashMap<String, CompetitionFilter>,
}

/// Response from GET /search/filters_by_sport.
///
/// Contains filtering options organized by sport for market discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiltersBySportResponse {
    /// Mapping of sports to their filter details.
    pub filters_by_sports: HashMap<String, SportFilter>,

    /// Ordered list of sports for display purposes.
    #[serde(default)]
    pub sport_ordering: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_by_categories_deserialize() {
        let json = r#"{
            "tags_by_categories": {
                "Politics": ["US Elections", "Congress", "Presidential"],
                "Sports": ["NFL", "NBA", "MLB"]
            }
        }"#;
        let response: TagsByCategoriesResponse = serde_json::from_str(json).unwrap();
        assert!(response.tags_by_categories.contains_key("Politics"));
        assert_eq!(response.tags_by_categories["Sports"].len(), 3);
    }

    #[test]
    fn test_filters_by_sport_deserialize() {
        let json = r#"{
            "filters_by_sports": {
                "NFL": {
                    "scopes": ["regular_season", "playoffs"],
                    "competitions": {
                        "Super Bowl": {
                            "scopes": ["game"]
                        }
                    }
                }
            },
            "sport_ordering": ["NFL", "NBA", "MLB"]
        }"#;
        let response: FiltersBySportResponse = serde_json::from_str(json).unwrap();
        assert!(response.filters_by_sports.contains_key("NFL"));
        assert_eq!(response.sport_ordering.len(), 3);
    }
}
