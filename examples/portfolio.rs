//! Portfolio API example
//!
//! Demonstrates all Portfolio API methods including balance, positions, fills, and orders.
//!
//! Run with: cargo run --example portfolio

use kalshi_trade_rs::{
    cents_to_dollars, GetFillsParams, GetOrdersParams, GetPositionsParams, KalshiClient,
    KalshiConfig, OrderStatus,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Balance
    println!("=== Balance ===");
    let balance = client.get_balance().await?;
    println!("Available: ${:.2}", cents_to_dollars(balance.balance));
    println!(
        "Portfolio Value: ${:.2}",
        cents_to_dollars(balance.portfolio_value)
    );
    println!();

    // 2. Get Positions
    println!("=== Positions ===");
    let positions = client.get_positions().await?;
    println!(
        "Market positions: {}, Event positions: {}",
        positions.market_positions.len(),
        positions.event_positions.len()
    );

    for pos in positions.market_positions.iter().take(5) {
        let side = if pos.position > 0 { "YES" } else { "NO" };
        println!(
            "  {} {} {} | exposure: ${:.2} | realized P&L: ${:.2}",
            pos.ticker,
            pos.position.abs(),
            side,
            cents_to_dollars(pos.market_exposure),
            cents_to_dollars(pos.realized_pnl)
        );
    }
    if positions.market_positions.len() > 5 {
        println!("  ... and {} more", positions.market_positions.len() - 5);
    }
    println!();

    // 3. Get Positions with params (filter example)
    println!("=== Positions (with limit) ===");
    let params = GetPositionsParams::new().limit(3);
    let limited_positions = client.get_positions_with_params(params).await?;
    println!(
        "Fetched {} positions (limited to 3)",
        limited_positions.market_positions.len()
    );
    println!();

    // 4. Get Fills
    println!("=== Recent Fills ===");
    let params = GetFillsParams::new().limit(5);
    let fills = client.get_fills_with_params(params).await?;
    println!("Recent fills: {}", fills.fills.len());

    for fill in &fills.fills {
        println!(
            "  {} {} {} {} @ ${:.2} ({})",
            fill.ticker,
            fill.action,
            fill.count,
            fill.side,
            cents_to_dollars(fill.yes_price),
            if fill.is_taker { "taker" } else { "maker" }
        );
    }
    println!();

    // 5. Get Orders
    println!("=== Orders ===");
    let orders = client.get_orders().await?;
    println!("Total orders: {}", orders.orders.len());

    // Count by status
    let resting = orders
        .orders
        .iter()
        .filter(|o| o.status == OrderStatus::Resting)
        .count();
    let executed = orders
        .orders
        .iter()
        .filter(|o| o.status == OrderStatus::Executed)
        .count();
    let canceled = orders
        .orders
        .iter()
        .filter(|o| o.status == OrderStatus::Canceled)
        .count();
    println!(
        "  Resting: {}, Executed: {}, Canceled: {}",
        resting, executed, canceled
    );

    // Show resting orders
    if resting > 0 {
        println!("\nResting orders:");
        let params = GetOrdersParams::new()
            .status(OrderStatus::Resting)
            .limit(5);
        let resting_orders = client.get_orders_with_params(params).await?;

        for order in &resting_orders.orders {
            println!(
                "  {} {} {} {} @ ${:.2} ({}/{})",
                order.ticker,
                order.action,
                order.remaining_count,
                order.side,
                cents_to_dollars(order.yes_price),
                order.fill_count,
                order.initial_count
            );
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
