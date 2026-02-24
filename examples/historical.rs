//! Historical Data API example
//!
//! Demonstrates the Historical Data API for fetching archived market data,
//! including cutoff timestamps, historical markets, candlesticks, fills, and orders.
//!
//! Run with: cargo run --example historical

use kalshi_trade_rs::{
    CandlestickPeriod, GetHistoricalCandlesticksParams, GetHistoricalFillsParams,
    GetHistoricalMarketsParams, GetHistoricalOrdersParams, KalshiClient, KalshiConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Historical Cutoff Timestamps
    println!("=== Historical Cutoff ===");
    let cutoff = client.get_historical_cutoff().await?;
    println!("Markets archived through: {}", cutoff.market_settled_ts);
    println!("Trades archived through:  {}", cutoff.trades_created_ts);
    println!("Orders archived through:  {}", cutoff.orders_updated_ts);
    println!();

    // 2. Get Historical Markets (default params)
    println!("=== Historical Markets (first 5) ===");
    let params = GetHistoricalMarketsParams::new().limit(5);
    let markets = client.get_historical_markets_with_params(params).await?;
    println!("Returned {} historical markets", markets.markets.len());

    for market in &markets.markets {
        println!(
            "  {} | {:?} | result: {:?} | vol: {}",
            market.ticker, market.status, market.result, market.volume
        );
    }

    if !markets.cursor.is_empty() {
        println!(
            "  Next cursor: {}...",
            &markets.cursor[..markets.cursor.len().min(20)]
        );
    }
    println!();

    // 3. Get a specific historical market and its candlesticks
    if let Some(market) = markets.markets.first() {
        let ticker = &market.ticker;

        println!("=== Historical Market Detail: {} ===", ticker);
        let detail = client.get_historical_market(ticker).await?;
        let m = &detail.market;

        println!("  Title: {}", m.title);
        println!("  Event: {}", m.event_ticker);
        println!("  Type: {:?}", m.market_type);
        println!("  Status: {:?}", m.status);
        if let Some(sv) = &m.settlement_value_dollars {
            println!("  Settlement value: ${}", sv);
        }
        println!();

        // 4. Get Historical Candlesticks
        // Use a broad time range to catch data for archived markets
        println!("=== Historical Candlesticks: {} ===", ticker);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let one_year_ago = now - 365 * 86400;

        let params =
            GetHistoricalCandlesticksParams::new(one_year_ago, now, CandlestickPeriod::OneDay);
        match client.get_historical_candlesticks(ticker, params).await {
            Ok(candles) => {
                println!("  {} candlesticks returned", candles.candlesticks.len());
                for candle in candles.candlesticks.iter().take(3) {
                    let close = candle.price.close.as_deref().unwrap_or("N/A");
                    println!(
                        "    ts={} close=${} vol={}",
                        candle.end_period_ts, close, candle.volume
                    );
                }
                if candles.candlesticks.len() > 3 {
                    println!("    ... and {} more", candles.candlesticks.len() - 3);
                }
            }
            Err(e) => println!("  Could not fetch candlesticks: {}", e),
        }
        println!();
    }

    // 5. Get Historical Fills (requires auth)
    println!("=== Historical Fills (first 5) ===");
    let params = GetHistoricalFillsParams::new().limit(5);
    let fills = client.get_historical_fills_with_params(params).await?;
    println!("Returned {} historical fills", fills.fills.len());

    for fill in &fills.fills {
        println!(
            "  {} {} {} {} @ {} ({})",
            fill.ticker,
            fill.action,
            fill.count_fp,
            fill.side,
            fill.yes_price_fixed,
            if fill.is_taker { "taker" } else { "maker" },
        );
    }

    if fills.fills.is_empty() {
        println!("  (No historical fills found)");
    }
    println!();

    // 6. Get Historical Orders (requires auth)
    println!("=== Historical Orders (first 5) ===");
    let params = GetHistoricalOrdersParams::new().limit(5);
    let orders = client.get_historical_orders_with_params(params).await?;
    println!("Returned {} historical orders", orders.orders.len());

    for order in &orders.orders {
        println!(
            "  {} {} {} {} | {:?} | filled: {}/{}",
            order.ticker,
            order.action,
            order.initial_count,
            order.side,
            order.status,
            order.fill_count,
            order.initial_count,
        );
    }

    if orders.orders.is_empty() {
        println!("  (No historical orders found)");
    }

    println!("\n=== Done ===");
    Ok(())
}
