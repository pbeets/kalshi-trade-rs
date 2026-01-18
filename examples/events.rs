//! Events and Series API example
//!
//! Demonstrates Events API methods including listing events, fetching event details,
//! nested markets hierarchy, and event metadata. Also demonstrates Series API methods
//! including listing series and fetching series details.
//!
//! Run with: cargo run --example events

use std::time::{SystemTime, UNIX_EPOCH};

use kalshi_trade_rs::{
    EventStatus, GetEventParams, GetEventsParams, GetSeriesParams, KalshiClient, KalshiConfig,
    models::{
        CandlestickPeriod, ForecastPeriod, GetEventCandlesticksParams,
        GetEventForecastPercentileHistoryParams,
    },
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
    // Event Candlesticks & Forecast History
    // ==========================================================================

    // 10. Get event candlesticks (aggregated across all markets in an event)
    if let Some(first_event) = open_events.events.first() {
        let event_ticker = &first_event.event_ticker;
        let series_ticker = &first_event.series_ticker;

        println!("=== Event Candlesticks: {} ===", event_ticker);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let one_day_ago = now - 86400;

        let params = GetEventCandlesticksParams::new(one_day_ago, now, CandlestickPeriod::OneHour);
        let candles = client
            .get_event_candlesticks(series_ticker, event_ticker, params)
            .await?;

        println!("Markets in event: {}", candles.market_tickers.len());
        for ticker in candles.market_tickers.iter().take(5) {
            println!("  - {}", ticker);
        }
        if candles.market_tickers.len() > 5 {
            println!("  ... and {} more", candles.market_tickers.len() - 5);
        }

        println!(
            "Market candlestick arrays: {}",
            candles.market_candlesticks.len()
        );
        for (i, market_candles) in candles.market_candlesticks.iter().enumerate().take(3) {
            println!(
                "  Market {} has {} candles",
                candles.market_tickers.get(i).unwrap_or(&"?".to_string()),
                market_candles.len()
            );
        }
        if let Some(adjusted) = candles.adjusted_end_ts {
            println!("Adjusted end timestamp: {}", adjusted);
        }
        println!();

        // 11. Get event forecast percentile history
        // Note: This endpoint only works for events with numeric/forecast data.
        // Many events (like yes/no binary events) don't support this endpoint.
        println!(
            "=== Event Forecast Percentile History: {} ===",
            event_ticker
        );

        // Request median (50th percentile = 5000) and quartiles (25th = 2500, 75th = 7500)
        let percentiles = vec![2500, 5000, 7500];
        let params = GetEventForecastPercentileHistoryParams::new(
            percentiles.clone(),
            one_day_ago,
            now,
            ForecastPeriod::OneHour,
        );

        match client
            .get_event_forecast_percentile_history(series_ticker, event_ticker, params)
            .await
        {
            Ok(forecast) => {
                println!(
                    "Forecast history points: {}",
                    forecast.forecast_history.len()
                );

                for point in forecast.forecast_history.iter().take(3) {
                    println!(
                        "  Event: {} | ts: {} | period: {}min",
                        point.event_ticker, point.end_period_ts, point.period_interval
                    );
                    for pp in &point.percentile_points {
                        let formatted = pp.formatted_forecast.as_deref().unwrap_or("-");
                        let numerical = pp.numerical_forecast.map(|v| format!("{:.2}", v));
                        println!(
                            "    {}th percentile: {} ({})",
                            pp.percentile / 100,
                            formatted,
                            numerical.as_deref().unwrap_or("-")
                        );
                    }
                }
                if forecast.forecast_history.len() > 3 {
                    println!(
                        "  ... and {} more points",
                        forecast.forecast_history.len() - 3
                    );
                }
            }
            Err(e) => {
                println!(
                    "  (Forecast history not available for this event type: {})",
                    e
                );
            }
        }
        println!();
    }

    // ==========================================================================
    // Series API
    // ==========================================================================

    // 12. Get list of series
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

    // 13. Get series list with params
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

    // 14. Get a specific series
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
