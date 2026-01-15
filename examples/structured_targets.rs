//! Structured Targets API example
//!
//! Demonstrates Structured Targets API methods for listing and retrieving
//! structured targets. These are public endpoints that don't require authentication.
//!
//! Run with: cargo run --example structured_targets

use kalshi_trade_rs::{GetStructuredTargetsParams, KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. List Structured Targets (with pagination)
    println!("=== Structured Targets (first 20) ===");
    let params = GetStructuredTargetsParams::new().limit(20);
    let response = client.get_structured_targets_with_params(params).await?;

    if response.structured_targets.is_empty() {
        println!("No structured targets found.");
        println!("\n=== Done ===");
        return Ok(());
    }

    println!(
        "Found {} structured target(s):",
        response.structured_targets.len()
    );
    println!();

    for (i, target) in response.structured_targets.iter().take(10).enumerate() {
        let id = target.id.as_deref().unwrap_or("(no id)");
        let name = target.name.as_deref().unwrap_or("(unnamed)");
        let target_type = target.target_type.as_deref().unwrap_or("unknown");

        println!("{}. {} [{}]", i + 1, name, target_type);
        println!("   ID: {}", id);

        if let Some(source_id) = &target.source_id {
            println!("   Source: {}", source_id);
        }
        if let Some(details) = &target.details {
            // Show a few key fields from details if available
            if let Some(obj) = details.as_object() {
                let keys: Vec<&str> = obj.keys().take(3).map(|s| s.as_str()).collect();
                if !keys.is_empty() {
                    println!(
                        "   Details: {} fields ({}, ...)",
                        obj.len(),
                        keys.join(", ")
                    );
                }
            }
        }
        println!();
    }

    if response.structured_targets.len() > 10 {
        println!("... and {} more", response.structured_targets.len() - 10);
        println!();
    }

    // Show pagination cursor if present
    if let Some(cursor) = &response.cursor {
        println!(
            "Pagination cursor available ({}...)",
            &cursor[..cursor.len().min(20)]
        );
        println!();
    }

    // 2. Get a specific Structured Target by ID
    println!("=== Get Specific Structured Target ===");

    // Use the first target from our list
    let first_target = &response.structured_targets[0];
    if let Some(target_id) = &first_target.id {
        println!("Fetching details for: {}", target_id);
        println!();

        match client.get_structured_target(target_id).await {
            Ok(detail_response) => {
                let target = &detail_response.structured_target;

                println!("Name: {}", target.name.as_deref().unwrap_or("(unnamed)"));
                println!(
                    "Type: {}",
                    target.target_type.as_deref().unwrap_or("unknown")
                );
                println!("ID: {}", target.id.as_deref().unwrap_or("(no id)"));

                if let Some(source_id) = &target.source_id {
                    println!("Source ID: {}", source_id);
                }
                if let Some(updated) = &target.last_updated_ts {
                    println!("Last Updated: {}", updated);
                }
                if let Some(details) = &target.details {
                    println!("Details: {}", serde_json::to_string_pretty(details)?);
                }
            }
            Err(e) => {
                println!("Error fetching target details: {}", e);
            }
        }
    } else {
        println!("First target has no ID, skipping detail fetch.");
    }

    // 3. Count by type
    println!("\n=== Target Type Summary ===");
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for target in &response.structured_targets {
        let target_type = target
            .target_type
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        *type_counts.entry(target_type).or_insert(0) += 1;
    }

    if !type_counts.is_empty() {
        println!(
            "Breakdown by type (from first {} targets):",
            response.structured_targets.len()
        );
        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));
        for (target_type, count) in sorted_types.iter().take(10) {
            println!("  {}: {}", target_type, count);
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
