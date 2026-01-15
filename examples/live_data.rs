//! Live Data API example
//!
//! Demonstrates retrieving real-time data for milestones without WebSocket connections.
//! Live data provides current values for tracked metrics like prices, scores, and other
//! data points that drive market resolution.
//!
//! Run with: cargo run --example live_data

use kalshi_trade_rs::{GetBatchLiveDataParams, GetMilestonesParams, KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // First, get some milestones to use for live data queries
    println!("=== Finding Milestones ===");
    let params = GetMilestonesParams::new().limit(10);
    let milestones_response = client.get_milestones_with_params(params).await?;

    if milestones_response.milestones.is_empty() {
        println!("No milestones found in this environment.");
        println!("Live data requires milestones to be available.");
        println!("\n=== Done ===");
        return Ok(());
    }

    println!(
        "Found {} milestone(s) to query for live data\n",
        milestones_response.milestones.len()
    );

    // 1. Get live data for a single milestone
    println!("=== Get Live Data (Single Milestone) ===");

    let first_milestone = &milestones_response.milestones[0];
    let milestone_id = first_milestone.id.as_deref().unwrap_or("");
    let milestone_type = first_milestone.milestone_type.as_deref().unwrap_or("");
    let milestone_title = first_milestone.title.as_deref().unwrap_or("(untitled)");

    if milestone_id.is_empty() || milestone_type.is_empty() {
        println!("First milestone missing ID or type, skipping single query.");
    } else {
        println!("Querying live data for: {}", milestone_title);
        println!("  ID: {}", milestone_id);
        println!("  Type: {}", milestone_type);

        match client.get_live_data(milestone_type, milestone_id).await {
            Ok(response) => {
                let data = &response.live_data;
                println!();
                println!("Live Data Response:");
                println!("  Milestone ID: {}", data.milestone_id);
                if let Some(t) = &data.milestone_type {
                    println!("  Type: {}", t);
                }
                if let Some(v) = data.value {
                    println!("  Value: {}", v);
                }
                if let Some(vs) = &data.value_string {
                    println!("  Value (string): {}", vs);
                }
                if let Some(ts) = data.updated_ts {
                    println!("  Updated: {} (unix timestamp)", ts);
                }
                if let Some(event) = &data.event_ticker {
                    println!("  Event: {}", event);
                }
                if let Some(series) = &data.series_ticker {
                    println!("  Series: {}", series);
                }
            }
            Err(e) => {
                println!("Failed to get live data: {}", e);
                println!("(This is expected if the milestone doesn't have active live data)");
            }
        }
    }
    println!();

    // 2. Get live data in batch
    println!("=== Get Batch Live Data ===");

    // Collect milestone IDs that have both ID and type
    let valid_milestones: Vec<_> = milestones_response
        .milestones
        .iter()
        .filter(|m| m.id.is_some() && m.milestone_type.is_some())
        .take(5)
        .collect();

    if valid_milestones.is_empty() {
        println!("No valid milestones found for batch query.");
    } else {
        let ids: Vec<&str> = valid_milestones
            .iter()
            .map(|m| m.id.as_deref().unwrap())
            .collect();

        println!("Querying batch live data for {} milestones:", ids.len());
        for (i, m) in valid_milestones.iter().enumerate() {
            println!(
                "  {}. {} ({})",
                i + 1,
                m.title.as_deref().unwrap_or("(untitled)"),
                m.id.as_deref().unwrap_or("")
            );
        }
        println!();

        let params = GetBatchLiveDataParams::from_ids(&ids);
        match client.get_batch_live_data(params).await {
            Ok(response) => {
                println!(
                    "Batch Response: {} live data entries",
                    response.live_data.len()
                );
                println!();

                for (i, data) in response.live_data.iter().enumerate() {
                    println!("{}. Milestone: {}", i + 1, data.milestone_id);
                    if let Some(t) = &data.milestone_type {
                        println!("   Type: {}", t);
                    }
                    if let Some(v) = data.value {
                        println!("   Value: {}", v);
                    } else if let Some(vs) = &data.value_string {
                        println!("   Value: {}", vs);
                    }
                    if let Some(event) = &data.event_ticker {
                        println!("   Event: {}", event);
                    }
                }
            }
            Err(e) => {
                println!("Batch query failed: {}", e);
                println!("(This is expected if milestones don't have active live data)");
            }
        }
    }
    println!();

    // 3. Summary of milestone types available
    println!("=== Milestone Types Summary ===");

    let types: std::collections::HashSet<_> = milestones_response
        .milestones
        .iter()
        .filter_map(|m| m.milestone_type.as_deref())
        .collect();

    if types.is_empty() {
        println!("No milestone types found.");
    } else {
        println!("Available milestone types in this environment:");
        for t in &types {
            println!("  - {}", t);
        }
        println!();
        println!("Note: Live data availability depends on active events.");
        println!("The demo environment may not have active live data feeds.");
    }

    println!("\n=== Done ===");
    Ok(())
}
