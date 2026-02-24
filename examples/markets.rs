//! Markets API example
//!
//! Demonstrates Markets API methods including listing markets, fetching individual
//! market details, orderbooks, trades, and cursor-based pagination.
//!
//! Run with: cargo run --example markets

use kalshi_trade_rs::{
    GetMarketsParams, GetOrderbookParams, GetTradesParams, KalshiClient, KalshiConfig,
    MarketFilterStatus, cents_to_dollars,
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
        println!("Next cursor: {}...", &markets.cursor[..markets.cursor.len().min(20)]);
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
            market.ticker,
            market.title,
            market.volume
        );
        println!("    YES bid/ask: ${} / ${}", market.yes_bid_dollars, market.yes_ask_dollars);
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
        println!("Total Volume: {} contracts", market.volume);
        println!("Open Interest: {} contracts", market.open_interest);
        println!();

        // 5. Get Orderbook
        println!("=== Get Orderbook ===");
        let orderbook_response = client.get_orderbook(ticker).await?;
        let orderbook = &orderbook_response.orderbook;

        println!("YES levels: {}", orderbook.yes.len());
        for (i, level) in orderbook.yes.iter().take(3).enumerate() {
            if level.len() >= 2 {
                println!(
                    "  Level {}: {} @ ${:.2}",
                    i + 1,
                    level[1],
                    cents_to_dollars(level[0])
                );
            }
        }
        if orderbook.yes.len() > 3 {
            println!("  ... and {} more levels", orderbook.yes.len() - 3);
        }

        println!("NO levels: {}", orderbook.no.len());
        for (i, level) in orderbook.no.iter().take(3).enumerate() {
            if level.len() >= 2 {
                println!(
                    "  Level {}: {} @ ${:.2}",
                    i + 1,
                    level[1],
                    cents_to_dollars(level[0])
                );
            }
        }
        if orderbook.no.len() > 3 {
            println!("  ... and {} more levels", orderbook.no.len() - 3);
        }
        println!();

        // 6. Get Orderbook with depth limit
        println!("=== Get Orderbook (depth=3) ===");
        let params = GetOrderbookParams::new().depth(3);
        let limited_orderbook = client.get_orderbook_with_params(ticker, params).await?;
        println!(
            "YES levels (limited): {}",
            limited_orderbook.orderbook.yes.len()
        );
        println!(
            "NO levels (limited): {}",
            limited_orderbook.orderbook.no.len()
        );
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
