//! Events and Series API example
//!
//! Demonstrates Events API methods including listing events, fetching event details,
//! nested markets hierarchy, and event metadata. Also demonstrates Series API methods
//! including listing series and fetching series details.
//!
//! Run with: cargo run --example events

use kalshi_trade_rs::{
    EventStatus, GetEventParams, GetEventsParams, GetSeriesParams, KalshiClient, KalshiConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Events (default parameters)
    println!("=== Get Events (default) ===");
    let events = client.get_events().await?;
    println!("Total events returned: {}", events.events.len());
    if let Some(cursor) = &events.cursor {
        println!("Next cursor: {}...", &cursor[..cursor.len().min(20)]);
    }
    println!();

    // 2. Get Events with filters
    println!("=== Get Open Events (limit 5) ===");
    let params = GetEventsParams::new().status(EventStatus::Open).limit(5);
    let open_events = client.get_events_with_params(params).await?;

    for event in &open_events.events {
        println!("  {} | {}", event.event_ticker, event.title);
        if let Some(subtitle) = &event.sub_title {
            println!("    Subtitle: {}", subtitle);
        }
        println!("    Series: {}", event.series_ticker);
        if let Some(category) = &event.category {
            println!("    Category: {}", category);
        }
    }
    println!();

    // 3. Get Events with nested markets
    println!("=== Get Events with Nested Markets ===");
    let params = GetEventsParams::new()
        .status(EventStatus::Open)
        .with_nested_markets(true)
        .limit(3);
    let events_with_markets = client.get_events_with_params(params).await?;

    for event in &events_with_markets.events {
        println!("Event: {} | {}", event.event_ticker, event.title);

        if let Some(markets) = &event.markets {
            println!("  Markets ({}):", markets.len());
            for market in markets.iter().take(5) {
                let title = market.title.as_deref().unwrap_or("(no title)");
                let yes_ask = market.yes_ask_dollars.as_deref().unwrap_or("N/A");
                println!("    {} | {} | YES ask: ${}", market.ticker, title, yes_ask);
            }
            if markets.len() > 5 {
                println!("    ... and {} more markets", markets.len() - 5);
            }
        } else {
            println!("  (no nested markets)");
        }
        println!();
    }

    // 4. Get a specific event
    if let Some(first_event) = open_events.events.first() {
        let event_ticker = &first_event.event_ticker;

        println!("=== Get Single Event: {} ===", event_ticker);
        let event_response = client.get_event(event_ticker).await?;
        let event = &event_response.event;

        println!("Ticker: {}", event.event_ticker);
        println!("Title: {}", event.title);
        println!("Series: {}", event.series_ticker);
        if let Some(subtitle) = &event.sub_title {
            println!("Subtitle: {}", subtitle);
        }
        if let Some(category) = &event.category {
            println!("Category: {}", category);
        }
        if let Some(mutually_exclusive) = event.mutually_exclusive {
            println!("Mutually Exclusive: {}", mutually_exclusive);
        }
        if let Some(strike_date) = &event.strike_date {
            println!("Strike Date: {}", strike_date);
        }
        if let Some(strike_period) = &event.strike_period {
            println!("Strike Period: {}", strike_period);
        }

        // Check for markets in the deprecated field
        if let Some(markets) = &event_response.markets {
            println!("Markets (deprecated field): {}", markets.len());
        }
        println!();

        // 5. Get event with nested markets using params
        println!("=== Get Event with Nested Markets ===");
        let params = GetEventParams::new().with_nested_markets(true);
        let event_with_markets = client.get_event_with_params(event_ticker, params).await?;

        if let Some(markets) = &event_with_markets.event.markets {
            println!(
                "Event {} has {} nested markets:",
                event_ticker,
                markets.len()
            );
            for market in markets {
                let status = format!("{:?}", market.status);
                let volume = market.volume.unwrap_or(0);
                let oi = market.open_interest.unwrap_or(0);

                println!(
                    "  {} | {} | vol: {} | oi: {}",
                    market.ticker, status, volume, oi
                );

                // Show pricing if available
                if let (Some(yes_bid), Some(yes_ask)) =
                    (&market.yes_bid_dollars, &market.yes_ask_dollars)
                {
                    println!("    YES: ${} bid / ${} ask", yes_bid, yes_ask);
                }
            }
        }
        println!();

        // 6. Get event metadata
        println!("=== Get Event Metadata ===");
        let metadata = client.get_event_metadata(event_ticker).await?;

        println!("Image URL: {}", metadata.image_url);
        if let Some(featured_img) = &metadata.featured_image_url {
            println!("Featured Image: {}", featured_img);
        }
        if let Some(competition) = &metadata.competition {
            println!("Competition: {}", competition);
        }
        if let Some(scope) = &metadata.competition_scope {
            println!("Competition Scope: {}", scope);
        }

        println!("Market Details ({}):", metadata.market_details.len());
        for detail in metadata.market_details.iter().take(5) {
            println!(
                "  {} | color: {} | img: {}",
                detail.market_ticker, detail.color_code, detail.image_url
            );
        }
        if metadata.market_details.len() > 5 {
            println!("  ... and {} more", metadata.market_details.len() - 5);
        }

        println!(
            "Settlement Sources ({}):",
            metadata.settlement_sources.len()
        );
        for source in &metadata.settlement_sources {
            let name = source.name.as_deref().unwrap_or("(unnamed)");
            let url = source.url.as_deref().unwrap_or("(no url)");
            println!("  {} | {}", name, url);
        }
        println!();
    }

    // 7. Pagination through events
    println!("=== Pagination Example ===");
    let mut cursor: Option<String> = None;
    let mut total_events = 0;
    let mut page = 0;
    const PAGE_SIZE: i64 = 10;
    const MAX_PAGES: usize = 3;

    loop {
        let mut params = GetEventsParams::new()
            .status(EventStatus::Open)
            .limit(PAGE_SIZE);

        if let Some(c) = cursor.take() {
            params = params.cursor(c);
        }

        let response = client.get_events_with_params(params).await?;
        let fetched = response.events.len();
        total_events += fetched;
        page += 1;

        println!("Page {}: fetched {} events", page, fetched);

        if let Some(next_cursor) = response.cursor {
            if page < MAX_PAGES {
                cursor = Some(next_cursor);
            } else {
                println!("Stopping after {} pages (demo limit)", MAX_PAGES);
                break;
            }
        } else {
            println!("No more pages");
            break;
        }
    }
    println!("Total collected: {} events\n", total_events);

    // 8. Filter events by series
    if let Some(first_event) = open_events.events.first() {
        let series_ticker = &first_event.series_ticker;

        println!("=== Events in Series {} ===", series_ticker);
        let params = GetEventsParams::new().series_ticker(series_ticker).limit(5);
        let series_events = client.get_events_with_params(params).await?;

        println!(
            "Found {} events in this series:",
            series_events.events.len()
        );
        for event in &series_events.events {
            println!("  {} | {}", event.event_ticker, event.title);
        }
        println!();
    }

    // 9. Get multivariate events (combo events)
    println!("=== Multivariate Events ===");
    let mve_response = client.get_multivariate_events().await?;
    println!("Multivariate events: {}", mve_response.events.len());

    for event in mve_response.events.iter().take(3) {
        println!("  {} | {}", event.event_ticker, event.title);
        println!("    Series: {}", event.series_ticker);
    }
    if mve_response.events.len() > 3 {
        println!("  ... and {} more", mve_response.events.len() - 3);
    }
    println!();

    // ==========================================================================
    // Series API
    // ==========================================================================

    // 10. Get list of series
    println!("=== Get Series List ===");
    let series_list = client.get_series_list().await?;
    println!("Total series returned: {}", series_list.series.len());

    for series in series_list.series.iter().take(5) {
        println!(
            "  {} | {} | frequency: {}",
            series.ticker, series.title, series.frequency
        );
    }
    if series_list.series.len() > 5 {
        println!("  ... and {} more series", series_list.series.len() - 5);
    }
    println!();

    // 11. Get series list with params
    println!("=== Get Series List with Params ===");
    let params = GetSeriesParams::new().include_volume(true);
    let series_with_volume = client.get_series_list_with_params(params).await?;
    println!(
        "Series with volume info: {} returned",
        series_with_volume.series.len()
    );

    for series in series_with_volume.series.iter().take(3) {
        println!(
            "  {} | {} | frequency: {}",
            series.ticker, series.title, series.frequency
        );
    }
    println!();

    // 12. Get a specific series
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

    println!("=== Done ===");
    Ok(())
}
