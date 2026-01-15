//! Series and Search API verification example
//!
//! Demonstrates and verifies the Series and Search API endpoints.
//!
//! Run with: cargo run --example series_and_search

use kalshi_trade_rs::{GetFeeChangesParams, GetSeriesParams, KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // =========================================================================
    // Search API
    // =========================================================================

    // 1. Get tags by categories
    println!("=== Get Tags by Categories ===");
    let tags_response = client.get_tags_by_categories().await?;
    println!(
        "Categories found: {}",
        tags_response.tags_by_categories.len()
    );

    for (category, tags) in tags_response.tags_by_categories.iter().take(5) {
        match tags {
            Some(tag_list) => {
                let preview: Vec<_> = tag_list.iter().take(3).collect();
                let suffix = if tag_list.len() > 3 {
                    format!(" ... and {} more", tag_list.len() - 3)
                } else {
                    String::new()
                };
                println!("  {}: {:?}{}", category, preview, suffix);
            }
            None => {
                println!("  {}: (no tags)", category);
            }
        }
    }
    if tags_response.tags_by_categories.len() > 5 {
        println!(
            "  ... and {} more categories",
            tags_response.tags_by_categories.len() - 5
        );
    }
    println!();

    // 2. Get filters by sport
    println!("=== Get Filters by Sport ===");
    let filters_response = client.get_filters_by_sport().await?;
    println!(
        "Sports found: {} (ordering: {} items)",
        filters_response.filters_by_sports.len(),
        filters_response.sport_ordering.len()
    );

    println!("Sport ordering: {:?}", filters_response.sport_ordering);

    for (sport, filter) in filters_response.filters_by_sports.iter().take(3) {
        println!("  {}:", sport);
        println!("    Scopes: {:?}", filter.scopes);
        println!("    Competitions: {} total", filter.competitions.len());
        for (comp_name, comp_filter) in filter.competitions.iter().take(2) {
            println!("      {} -> scopes: {:?}", comp_name, comp_filter.scopes);
        }
        if filter.competitions.len() > 2 {
            println!(
                "      ... and {} more competitions",
                filter.competitions.len() - 2
            );
        }
    }
    println!();

    // =========================================================================
    // Series API
    // =========================================================================

    // 3. Get series list (default parameters)
    println!("=== Get Series List (default) ===");
    let series_list = client.get_series_list().await?;
    println!("Series returned: {}", series_list.series.len());
    if let Some(cursor) = &series_list.cursor {
        println!("Next cursor: {}...", &cursor[..cursor.len().min(20)]);
    }

    for series in series_list.series.iter().take(5) {
        println!(
            "  {} | {} | freq: {}",
            series.ticker, series.title, series.frequency
        );
    }
    if series_list.series.len() > 5 {
        println!("  ... and {} more series", series_list.series.len() - 5);
    }
    println!();

    // 4. Get series list with parameters
    println!("=== Get Series List (with params) ===");
    let params = GetSeriesParams::new().include_volume(true);
    let series_with_volume = client.get_series_list_with_params(params).await?;
    println!(
        "Series with volume data: {}",
        series_with_volume.series.len()
    );

    for series in series_with_volume.series.iter().take(3) {
        println!(
            "  {} | {} | freq: {}",
            series.ticker, series.title, series.frequency
        );
    }
    println!();

    // 5. Get a specific series
    if let Some(first_series) = series_list.series.first() {
        let series_ticker = &first_series.ticker;

        println!("=== Get Single Series: {} ===", series_ticker);
        let series_response = client.get_series(series_ticker).await?;
        let series = &series_response.series;

        println!("Ticker: {}", series.ticker);
        println!("Title: {}", series.title);
        println!("Frequency: {}", series.frequency);
        println!();
    }

    // 6. Get fee changes (default - upcoming only)
    println!("=== Get Fee Changes (upcoming) ===");
    let fee_changes = client.get_fee_changes().await?;
    println!(
        "Upcoming fee changes: {}",
        fee_changes.series_fee_change_arr.len()
    );

    for change in fee_changes.series_fee_change_arr.iter().take(5) {
        println!(
            "  {} | {} | {:?} | multiplier: {} | scheduled: {}",
            change.id,
            change.series_ticker,
            change.fee_type,
            change.fee_multiplier,
            change.scheduled_ts
        );
    }
    if fee_changes.series_fee_change_arr.len() > 5 {
        println!(
            "  ... and {} more fee changes",
            fee_changes.series_fee_change_arr.len() - 5
        );
    }
    println!();

    // 7. Get fee changes with parameters (include historical)
    println!("=== Get Fee Changes (with historical) ===");
    let params = GetFeeChangesParams::new().show_historical(true);
    let historical_fee_changes = client.get_fee_changes_with_params(params).await?;
    println!(
        "Total fee changes (including historical): {}",
        historical_fee_changes.series_fee_change_arr.len()
    );

    for change in historical_fee_changes.series_fee_change_arr.iter().take(3) {
        println!(
            "  {} | {} | {:?} | {}",
            change.id, change.series_ticker, change.fee_type, change.scheduled_ts
        );
    }
    println!();

    // 8. Get fee changes for a specific series
    if let Some(first_series) = series_list.series.first() {
        println!("=== Get Fee Changes for Series {} ===", first_series.ticker);
        let params = GetFeeChangesParams::new()
            .series_ticker(&first_series.ticker)
            .show_historical(true);
        let series_fee_changes = client.get_fee_changes_with_params(params).await?;
        println!(
            "Fee changes for {}: {}",
            first_series.ticker,
            series_fee_changes.series_fee_change_arr.len()
        );
    }

    println!("\n=== All endpoints verified successfully! ===");
    Ok(())
}
