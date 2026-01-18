//! Milestones API example
//!
//! Demonstrates the Milestones API for retrieving data points that are tracked
//! and used for market resolution. Milestones represent real-world data such as
//! game scores, events, or other metrics that determine market outcomes.
//!
//! Run with: cargo run --example milestones

use kalshi_trade_rs::{GetMilestonesParams, KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Milestones (limit is required by API)
    println!("=== List Milestones ===");
    let params = GetMilestonesParams::new().limit(20);
    let response = client.get_milestones_with_params(params).await?;

    println!("Found {} milestone(s)", response.milestones.len());

    if response.milestones.is_empty() {
        println!("No milestones found in this environment.");
        println!("Milestones are typically available for markets with real-time data feeds.");
        println!("\n=== Done ===");
        return Ok(());
    }

    println!();

    // Display milestones
    for (i, milestone) in response.milestones.iter().take(10).enumerate() {
        let id = milestone.id.as_deref().unwrap_or("(no id)");
        let milestone_type = milestone.milestone_type.as_deref().unwrap_or("(unknown)");
        let title = milestone.title.as_deref().unwrap_or("(untitled)");
        let category = milestone.category.as_deref().unwrap_or("(no category)");

        println!("{}. {} [{}]", i + 1, title, milestone_type);
        println!("   ID: {}", id);
        println!("   Category: {}", category);

        if let Some(msg) = &milestone.notification_message {
            println!("   Message: {}", msg);
        }
        if let Some(start) = &milestone.start_date {
            println!("   Start: {}", start);
        }
        if let Some(tickers) = &milestone.primary_event_tickers
            && !tickers.is_empty()
        {
            println!("   Events: {}", tickers.join(", "));
        }
        println!();
    }

    if response.milestones.len() > 10 {
        println!("... and {} more milestones", response.milestones.len() - 10);
    }
    println!();

    // 2. Get milestones with different parameters
    println!("=== Milestones with Parameters ===");
    let params = GetMilestonesParams::new().limit(5);
    let filtered_response = client.get_milestones_with_params(params).await?;

    println!(
        "Fetched {} milestone(s) with limit=5",
        filtered_response.milestones.len()
    );

    if let Some(cursor) = &filtered_response.cursor {
        println!("Pagination cursor: {}...", &cursor[..cursor.len().min(30)]);
    }
    println!();

    // 3. Get a specific milestone by ID
    println!("=== Get Specific Milestone ===");

    // Use the first milestone from our list
    let first_milestone = &response.milestones[0];
    if let Some(milestone_id) = &first_milestone.id {
        println!("Fetching milestone: {}", milestone_id);

        match client.get_milestone(milestone_id).await {
            Ok(milestone_response) => {
                let milestone = &milestone_response.milestone;
                println!();
                println!("Milestone Details:");
                println!(
                    "  ID:       {}",
                    milestone.id.as_deref().unwrap_or("(none)")
                );
                println!(
                    "  Type:     {}",
                    milestone.milestone_type.as_deref().unwrap_or("(none)")
                );
                println!(
                    "  Title:    {}",
                    milestone.title.as_deref().unwrap_or("(none)")
                );
                println!(
                    "  Category: {}",
                    milestone.category.as_deref().unwrap_or("(none)")
                );

                if let Some(msg) = &milestone.notification_message {
                    println!("  Message:  {}", msg);
                }
                if let Some(start) = &milestone.start_date {
                    println!("  Start:    {}", start);
                }
                if let Some(end) = &milestone.end_date {
                    println!("  End:      {}", end);
                }
                if let Some(tickers) = &milestone.primary_event_tickers
                    && !tickers.is_empty()
                {
                    println!("  Primary Events: {}", tickers.join(", "));
                }
                if let Some(tickers) = &milestone.related_event_tickers
                    && !tickers.is_empty()
                {
                    println!("  Related Events: {}", tickers.join(", "));
                }
                if let Some(updated) = &milestone.last_updated_ts {
                    println!("  Updated:  {}", updated);
                }
                if let Some(details) = &milestone.details
                    && let Some(obj) = details.as_object()
                    && !obj.is_empty()
                {
                    println!("  Details:  {} fields", obj.len());
                    for (key, value) in obj.iter().take(5) {
                        println!("    {}: {}", key, value);
                    }
                }
            }
            Err(e) => {
                println!("Failed to fetch milestone: {}", e);
            }
        }
    } else {
        println!("First milestone has no ID, skipping individual fetch.");
    }
    println!();

    // 4. Pagination example
    if response.milestones.len() >= 3 {
        println!("=== Pagination Example ===");

        // First page
        let params = GetMilestonesParams::new().limit(3);
        let page1 = client.get_milestones_with_params(params).await?;

        println!("Page 1: {} milestones", page1.milestones.len());
        for m in &page1.milestones {
            println!(
                "  - {} ({})",
                m.title.as_deref().unwrap_or("(untitled)"),
                m.id.as_deref().unwrap_or("no-id")
            );
        }

        // Second page (if cursor available)
        if let Some(cursor) = page1.cursor {
            let params = GetMilestonesParams::new().limit(3).cursor(cursor);
            let page2 = client.get_milestones_with_params(params).await?;

            println!("Page 2: {} milestones", page2.milestones.len());
            for m in &page2.milestones {
                println!(
                    "  - {} ({})",
                    m.title.as_deref().unwrap_or("(untitled)"),
                    m.id.as_deref().unwrap_or("no-id")
                );
            }
        } else {
            println!("No more pages available.");
        }
        println!();
    }

    // 5. Category summary
    println!("=== Milestone Categories ===");
    let mut category_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for milestone in &response.milestones {
        let category = milestone
            .category
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        *category_counts.entry(category).or_insert(0) += 1;
    }

    if !category_counts.is_empty() {
        println!("Categories found:");
        let mut sorted: Vec<_> = category_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (category, count) in sorted {
            println!("  {}: {}", category, count);
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
