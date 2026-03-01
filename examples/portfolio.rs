//! Portfolio API example
//!
//! Demonstrates all Portfolio API methods including balance, positions, fills, orders,
//! and settlements.
//!
//! Run with: cargo run --example portfolio

use kalshi_trade_rs::{
    GetBalanceParams, GetFillsParams, GetOrdersParams, GetPositionsParams, GetSettlementsParams,
    KalshiClient, KalshiConfig, OrderStatus, UpdateSubaccountNettingRequest, cents_to_dollars,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;

    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Balance (combined across all subaccounts)
    println!("=== Balance (all subaccounts) ===");

    let balance = client.get_balance().await?;

    println!("Available: ${:.2}", cents_to_dollars(balance.balance));

    println!(
        "Portfolio Value: ${:.2}",
        cents_to_dollars(balance.portfolio_value)
    );

    println!();

    // 1b. Get Balance for primary account only
    println!("=== Balance (primary account only) ===");

    let params = GetBalanceParams::new().subaccount(0);
    let primary_balance = client.get_balance_with_params(params).await?;

    println!(
        "Primary Available: ${:.2}",
        cents_to_dollars(primary_balance.balance)
    );
    println!(
        "Primary Portfolio Value: ${:.2}",
        cents_to_dollars(primary_balance.portfolio_value)
    );

    println!();

    // 1c. Get subaccount netting configuration
    println!("=== Subaccount Netting ===");

    match client.get_subaccount_netting().await {
        Ok(netting) => {
            for config in &netting.netting_configs {
                println!(
                    "  Subaccount {}: netting {}",
                    config.subaccount_number,
                    if config.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }

            // Example: enable netting on primary account (commented out to avoid side effects)
            // let request = UpdateSubaccountNettingRequest::new(0, true);
            // client.update_subaccount_netting(request).await?;
        }
        Err(e) => println!("  Netting not available: {}", e),
    }
    let _ = UpdateSubaccountNettingRequest::new(0, true); // suppress unused import warning

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
            "  {} {} {} {} @ {} ({}, fee: {})",
            fill.ticker,
            fill.action,
            fill.count_fp,
            fill.side,
            fill.yes_price_fixed,
            if fill.is_taker { "taker" } else { "maker" },
            fill.fee_cost,
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

        let params = GetOrdersParams::new().status(OrderStatus::Resting).limit(5);
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

    println!();

    // 6. Get Settlements
    println!("=== Settlements ===");

    let params = GetSettlementsParams::new().limit(10);
    let settlements = client.get_settlements_with_params(params).await?;

    println!("Recent settlements: {}", settlements.settlements.len());

    for settlement in settlements.settlements.iter().take(5) {
        let ticker = &settlement.ticker;
        let result = format!("{:?}", settlement.market_result);

        println!(
            "  {} | result: {} | P&L: ${:.2} | fee: {} | yes: {} no: {}",
            ticker,
            result,
            cents_to_dollars(settlement.revenue),
            settlement.fee_cost,
            settlement.yes_count_fp,
            settlement.no_count_fp,
        );
    }

    if settlements.settlements.len() > 5 {
        println!("  ... and {} more", settlements.settlements.len() - 5);
    }

    if settlements.settlements.is_empty() {
        println!("  (No settlements yet - markets must settle first)");
    }

    println!("\n=== Done ===");

    Ok(())
}
