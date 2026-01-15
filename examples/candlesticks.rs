//! Candlesticks API example
//!
//! Demonstrates the Candlesticks API for fetching historical OHLCV data.
//! Candlesticks are useful for charting, backtesting, and technical analysis.
//!
//! NOTE: The demo environment typically has low/no trading activity, so candlestick
//! data may be empty. For real data, use the production environment with active markets.
//!
//! Run with: cargo run --example candlesticks

use kalshi_trade_rs::{
    CandlestickPeriod, GetBatchCandlesticksParams, GetCandlesticksParams, GetMarketsParams,
    KalshiClient, KalshiConfig, MarketFilterStatus, cents_to_dollars,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // Find open markets - try to find one with volume
    println!("=== Finding Markets with Activity ===");
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(50);
    let markets = client.get_markets_with_params(params).await?;

    if markets.markets.is_empty() {
        println!("No open markets found");
        return Ok(());
    }

    // Try to find a market with volume > 0, fall back to first market
    let market = markets
        .markets
        .iter()
        .find(|m| m.volume.unwrap_or(0) > 0)
        .unwrap_or(&markets.markets[0]);

    let ticker = &market.ticker;
    let series_ticker = &market.event_ticker;
    let volume = market.volume.unwrap_or(0);

    println!("Selected market: {}", ticker);
    println!("Series: {}", series_ticker);
    println!("Volume: {}", volume);

    if volume == 0 {
        println!("\nNote: This market has no trading activity.");
        println!("Candlestick data requires trades to generate OHLCV data.");
        println!("Demo environment markets typically have zero volume.");
    }
    println!();

    // Calculate time range: last 24 hours
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_day_ago = now - 86400;
    let one_hour_ago = now - 3600;

    // 1. Get Candlesticks - Hourly
    println!("=== Hourly Candlesticks (Last 24h) ===");

    let params = GetCandlesticksParams::new(one_day_ago, now, CandlestickPeriod::OneHour);
    let response = client
        .get_candlesticks(series_ticker, ticker, params)
        .await?;

    println!("Ticker: {}", response.ticker);
    println!("Candlesticks: {}", response.candlesticks.len());

    if response.candlesticks.is_empty() {
        println!("  (No data - market has no trading activity in this period)");
    } else {
        println!();
        println!(
            "{:<20} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "Timestamp", "Open", "High", "Low", "Close", "Volume"
        );
        println!("{}", "-".repeat(76));

        for candle in response.candlesticks.iter().take(10) {
            let timestamp = chrono::DateTime::from_timestamp(candle.end_period_ts, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| candle.end_period_ts.to_string());

            let (open, high, low, close) = if let Some(price) = &candle.price {
                (
                    price.open.map(|p| format!("${:.2}", cents_to_dollars(p))),
                    price.high.map(|p| format!("${:.2}", cents_to_dollars(p))),
                    price.low.map(|p| format!("${:.2}", cents_to_dollars(p))),
                    price.close.map(|p| format!("${:.2}", cents_to_dollars(p))),
                )
            } else {
                (None, None, None, None)
            };

            let volume = candle.volume.map(|v| v.to_string());

            println!(
                "{:<20} {:>8} {:>8} {:>8} {:>8} {:>8}",
                timestamp,
                open.as_deref().unwrap_or("-"),
                high.as_deref().unwrap_or("-"),
                low.as_deref().unwrap_or("-"),
                close.as_deref().unwrap_or("-"),
                volume.as_deref().unwrap_or("-")
            );
        }

        if response.candlesticks.len() > 10 {
            println!("... and {} more candles", response.candlesticks.len() - 10);
        }
    }
    println!();

    // 2. Get Candlesticks - 1 Minute (Last Hour)
    println!("=== 1-Minute Candlesticks (Last Hour) ===");

    let params = GetCandlesticksParams::new(one_hour_ago, now, CandlestickPeriod::OneMinute);
    let response = client
        .get_candlesticks(series_ticker, ticker, params)
        .await?;

    println!("Found {} 1-minute candles", response.candlesticks.len());

    if !response.candlesticks.is_empty() {
        println!("\nRecent bid/ask spreads:");
        println!("{:<20} {:>12} {:>12}", "Timestamp", "Yes Bid", "Yes Ask");
        println!("{}", "-".repeat(46));

        for candle in response.candlesticks.iter().rev().take(5) {
            let timestamp = chrono::DateTime::from_timestamp(candle.end_period_ts, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| candle.end_period_ts.to_string());

            let yes_bid = candle
                .yes_bid
                .as_ref()
                .and_then(|b| b.close)
                .map(|p| format!("${:.2}", cents_to_dollars(p)));
            let yes_ask = candle
                .yes_ask
                .as_ref()
                .and_then(|a| a.close)
                .map(|p| format!("${:.2}", cents_to_dollars(p)));

            println!(
                "{:<20} {:>12} {:>12}",
                timestamp,
                yes_bid.as_deref().unwrap_or("-"),
                yes_ask.as_deref().unwrap_or("-")
            );
        }
    }
    println!();

    // 3. Get Candlesticks - Daily
    println!("=== Daily Candlesticks (Last 7 Days) ===");

    let one_week_ago = now - (7 * 86400);
    let params = GetCandlesticksParams::new(one_week_ago, now, CandlestickPeriod::OneDay);
    let response = client
        .get_candlesticks(series_ticker, ticker, params)
        .await?;

    println!("Found {} daily candles", response.candlesticks.len());

    for candle in &response.candlesticks {
        let date = chrono::DateTime::from_timestamp(candle.end_period_ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| candle.end_period_ts.to_string());

        let vol = candle.volume.unwrap_or(0);
        let oi = candle.open_interest.unwrap_or(0);

        println!("  {} | Volume: {} | Open Interest: {}", date, vol, oi);
    }
    println!();

    // 4. Batch Candlesticks (Multiple Markets)
    if markets.markets.len() >= 2 {
        println!("=== Batch Candlesticks (Multiple Markets) ===");

        let tickers: Vec<&str> = markets
            .markets
            .iter()
            .take(3)
            .map(|m| m.ticker.as_str())
            .collect();

        println!(
            "Fetching hourly candlesticks for {} markets:",
            tickers.len()
        );
        for t in &tickers {
            println!("  - {}", t);
        }
        println!();

        let params = GetBatchCandlesticksParams::new(
            tickers.join(","),
            one_day_ago,
            now,
            CandlestickPeriod::OneHour,
        );

        let batch_response = client.get_batch_candlesticks(params).await?;

        println!("Results:");
        for market_data in &batch_response.markets {
            println!(
                "  {} - {} candles",
                market_data.market_ticker,
                market_data.candlesticks.len()
            );
        }
    }
    println!();

    // 5. Candlestick Period Options
    println!("=== Candlestick Periods ===");
    println!("Available periods:");
    println!(
        "  CandlestickPeriod::OneMinute - {} minute(s)",
        CandlestickPeriod::OneMinute.as_minutes()
    );
    println!(
        "  CandlestickPeriod::OneHour   - {} minute(s)",
        CandlestickPeriod::OneHour.as_minutes()
    );
    println!(
        "  CandlestickPeriod::OneDay    - {} minute(s)",
        CandlestickPeriod::OneDay.as_minutes()
    );
    println!();

    // 6. Use Cases
    println!("=== Common Use Cases ===");
    println!("1. Technical Analysis: Calculate moving averages, RSI, etc.");
    println!("2. Backtesting: Test trading strategies on historical data");
    println!("3. Charting: Build price charts for visualization");
    println!("4. Volatility Analysis: Study price ranges and volume patterns");
    println!("5. Market Making: Analyze bid/ask spreads over time");
    println!();

    // 7. Data Fields Explanation
    println!("=== Candlestick Data Fields ===");
    println!("Each candlestick contains:");
    println!("  end_period_ts  - End timestamp of the candle period");
    println!("  yes_bid        - OHLC data for YES bid prices");
    println!("  yes_ask        - OHLC data for YES ask prices");
    println!("  price          - OHLC data for trade prices (with yes/no)");
    println!("  volume         - Number of contracts traded");
    println!("  open_interest  - Open interest at end of period");

    println!("\n=== Done ===");
    Ok(())
}
