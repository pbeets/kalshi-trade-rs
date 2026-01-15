//! Search API example
//!
//! Demonstrates Search API methods for discovering markets through tags
//! and sport-based filters. These are public endpoints that don't require
//! authentication.
//!
//! Run with: cargo run --example search

use kalshi_trade_rs::{KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Tags by Categories
    println!("=== Tags by Categories ===");
    match client.get_tags_by_categories().await {
        Ok(response) => {
            let categories: Vec<&String> = response.tags_by_categories.keys().collect();
            println!("Found {} categories", categories.len());
            println!();

            // Sort categories for consistent display
            let mut sorted_categories: Vec<_> = response.tags_by_categories.iter().collect();
            sorted_categories.sort_by_key(|(k, _)| k.as_str());

            for (category, tags_opt) in sorted_categories {
                match tags_opt {
                    Some(tags) if !tags.is_empty() => {
                        println!("  {}: {} tags", category, tags.len());
                        // Show first few tags as examples
                        let preview: Vec<&str> = tags.iter().take(5).map(|s| s.as_str()).collect();
                        println!("    Examples: {}", preview.join(", "));
                        if tags.len() > 5 {
                            println!("    ... and {} more", tags.len() - 5);
                        }
                    }
                    _ => {
                        println!("  {}: (no tags)", category);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to get tags by categories: {}", e);
        }
    }
    println!();

    // 2. Get Filters by Sport
    println!("=== Filters by Sport ===");
    match client.get_filters_by_sport().await {
        Ok(response) => {
            println!("Found {} sports", response.filters_by_sports.len());

            if !response.sport_ordering.is_empty() {
                println!("Display order: {}", response.sport_ordering.join(", "));
            }
            println!();

            // Use sport_ordering if available, otherwise sort alphabetically
            let sports: Vec<&String> = if !response.sport_ordering.is_empty() {
                response.sport_ordering.iter().collect()
            } else {
                let mut keys: Vec<_> = response.filters_by_sports.keys().collect();
                keys.sort();
                keys
            };

            for sport in sports {
                if let Some(filter) = response.filters_by_sports.get(sport) {
                    println!("  {}", sport);

                    if !filter.scopes.is_empty() {
                        println!("    Scopes: {}", filter.scopes.join(", "));
                    }

                    if !filter.competitions.is_empty() {
                        println!("    Competitions: {}", filter.competitions.len());
                        // Show first few competitions as examples
                        let mut comp_names: Vec<_> = filter.competitions.keys().collect();
                        comp_names.sort();
                        for comp_name in comp_names.iter().take(3) {
                            if let Some(comp) = filter.competitions.get(*comp_name) {
                                if !comp.scopes.is_empty() {
                                    println!(
                                        "      - {}: scopes=[{}]",
                                        comp_name,
                                        comp.scopes.join(", ")
                                    );
                                } else {
                                    println!("      - {}", comp_name);
                                }
                            }
                        }
                        if filter.competitions.len() > 3 {
                            println!("      ... and {} more", filter.competitions.len() - 3);
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Failed to get filters by sport: {}", e);
        }
    }

    println!("=== Done ===");
    Ok(())
}
