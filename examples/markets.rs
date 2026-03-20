//! Markets API example
//!
//! Demonstrates Markets API methods including listing markets, fetching individual
//! market details, orderbooks, trades, and cursor-based pagination.
//!
//! Run with: cargo run --example markets

use kalshi_trade_rs::{
    GetMarketsParams, GetOrderbookParams, GetTradesParams, KalshiClient, KalshiConfig,
    MarketFilterStatus,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Markets (default parameters)
    println!("=== Get Markets (default) ===");
    let markets = client.get_markets().await?;
    println!("Total markets returned: {}", markets.markets.len());
    if !markets.cursor.is_empty() {
        println!(
            "Next cursor: {}...",
            &markets.cursor[..markets.cursor.len().min(20)]
        );
    }
    println!();

    // 2. Get Markets with filters
    println!("=== Get Open Markets (limit 5) ===");
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(5);
    let open_markets = client.get_markets_with_params(params).await?;

    for market in &open_markets.markets {
        println!(
            "  {} | {} | Vol: {}",
            market.ticker, market.title, market.volume_fp
        );
        print!(
            "    YES bid/ask: ${} / ${}",
            market.yes_bid_dollars, market.yes_ask_dollars
        );
        // Show top-of-book sizes when available
        match (&market.yes_bid_size_fp, &market.yes_ask_size_fp) {
            (Some(bid_size), Some(ask_size)) => println!("  (size: {} / {})", bid_size, ask_size),
            _ => println!(),
        }
    }
    println!();

    // 3. Pagination example
    println!("=== Pagination Example ===");
    let mut all_tickers = Vec::new();
    let mut cursor: Option<String> = None;
    let mut page = 0;
    const PAGE_SIZE: i64 = 10;
    const MAX_PAGES: usize = 3;

    loop {
        let mut params = GetMarketsParams::new()
            .status(MarketFilterStatus::Open)
            .limit(PAGE_SIZE);

        if let Some(c) = cursor.take() {
            params = params.cursor(c);
        }

        let response = client.get_markets_with_params(params).await?;
        let fetched = response.markets.len();
        all_tickers.extend(response.markets.into_iter().map(|m| m.ticker));

        page += 1;
        println!("Page {}: fetched {} markets", page, fetched);

        // Check if there's more data
        if !response.cursor.is_empty() {
            if page < MAX_PAGES {
                cursor = Some(response.cursor);
            } else {
                println!("Stopping after {} pages (demo limit)", MAX_PAGES);
                break;
            }
        } else {
            println!("No more pages");
            break;
        }
    }
    println!("Total collected: {} market tickers\n", all_tickers.len());

    // 4. Get a specific market
    if let Some(ticker) = all_tickers.first() {
        println!("=== Get Single Market ===");
        let market_response = client.get_market(ticker).await?;
        let market = &market_response.market;

        println!("Ticker: {}", market.ticker);
        println!("Event: {}", market.event_ticker);
        println!("Type: {:?}", market.market_type);
        println!("Status: {:?}", market.status);
        println!("Title: {}", market.title);
        println!("Total Volume: {} contracts", market.volume_fp);
        println!("Open Interest: {} contracts", market.open_interest_fp);
        println!();

        // 5. Get Orderbook
        println!("=== Get Orderbook ===");
        let orderbook_response = client.get_orderbook(ticker).await?;

        // Prefer orderbook_fp (v2 spec), fall back to orderbook.yes_dollars
        if let Some(ref fp) = orderbook_response.orderbook_fp {
            let yes = fp.yes_dollars.as_deref().unwrap_or(&[]);
            let no = fp.no_dollars.as_deref().unwrap_or(&[]);
            println!("YES levels: {}", yes.len());
            for (i, level) in yes.iter().take(3).enumerate() {
                println!("  Level {}: {} @ ${}", i + 1, level.quantity, level.price);
            }
            if yes.len() > 3 {
                println!("  ... and {} more levels", yes.len() - 3);
            }
            println!("NO levels: {}", no.len());
            for (i, level) in no.iter().take(3).enumerate() {
                println!("  Level {}: {} @ ${}", i + 1, level.quantity, level.price);
            }
            if no.len() > 3 {
                println!("  ... and {} more levels", no.len() - 3);
            }
        } else {
            let orderbook = &orderbook_response.orderbook;
            let yes = orderbook.yes_dollars.as_deref().unwrap_or(&[]);
            let no = orderbook.no_dollars.as_deref().unwrap_or(&[]);
            println!("YES levels: {}", yes.len());
            for (i, level) in yes.iter().take(3).enumerate() {
                println!("  Level {}: {} @ ${}", i + 1, level.quantity, level.price);
            }
            println!("NO levels: {}", no.len());
            for (i, level) in no.iter().take(3).enumerate() {
                println!("  Level {}: {} @ ${}", i + 1, level.quantity, level.price);
            }
        }
        println!();

        // 6. Get Orderbook with depth limit
        println!("=== Get Orderbook (depth=3) ===");
        let params = GetOrderbookParams::new().depth(3);
        let limited = client.get_orderbook_with_params(ticker, params).await?;
        if let Some(ref fp) = limited.orderbook_fp {
            let yes = fp.yes_dollars.as_deref().unwrap_or(&[]);
            let no = fp.no_dollars.as_deref().unwrap_or(&[]);
            println!("YES levels (limited): {}", yes.len());
            println!("NO levels (limited): {}", no.len());
        } else {
            let ob = &limited.orderbook;
            println!(
                "YES levels (limited): {}",
                ob.yes_dollars.as_deref().unwrap_or(&[]).len()
            );
            println!(
                "NO levels (limited): {}",
                ob.no_dollars.as_deref().unwrap_or(&[]).len()
            );
        }
        println!();
    }

    // 7. Get Recent Trades (all markets)
    println!("=== Recent Trades (all markets) ===");
    let params = GetTradesParams::new().limit(5);
    let trades = client.get_trades_with_params(params).await?;
    println!("Recent trades: {}", trades.trades.len());

    for trade in &trades.trades {
        println!(
            "  {} | {} contracts @ {} YES | taker: {:?} | {}",
            trade.ticker,
            trade.count_fp,
            trade.yes_price_dollars,
            trade.taker_side,
            trade.created_time
        );
    }
    println!();

    // 8. Get Trades for specific market
    if let Some(ticker) = all_tickers.first() {
        println!("=== Trades for {} ===", ticker);
        let params = GetTradesParams::new().ticker(ticker).limit(5);
        let market_trades = client.get_trades_with_params(params).await?;

        if market_trades.trades.is_empty() {
            println!("  No recent trades for this market");
        } else {
            for trade in &market_trades.trades {
                println!(
                    "  {} contracts @ {} YES ({} NO) | {:?}",
                    trade.count_fp,
                    trade.yes_price_dollars,
                    trade.no_price_dollars,
                    trade.taker_side
                );
            }
        }
        println!();
    }

    // 9. Filter markets by event ticker
    if let Some(first_market) = all_tickers.first() {
        // Get the event ticker from the first market
        let market_response = client.get_market(first_market).await?;
        let event_ticker = &market_response.market.event_ticker;

        println!("=== Markets in Event {} ===", event_ticker);
        let params = GetMarketsParams::new().event_ticker(event_ticker).limit(10);
        let event_markets = client.get_markets_with_params(params).await?;

        println!(
            "Found {} markets in this event:",
            event_markets.markets.len()
        );
        for market in &event_markets.markets {
            let status = format!("{:?}", market.status);
            println!("  {} | {}", market.ticker, status);
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
