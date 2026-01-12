//! Batch Orders API example
//!
//! Demonstrates batch order operations: creating and canceling multiple orders
//! in a single request, and handling partial success scenarios.
//!
//! **WARNING**: This example places REAL orders on Kalshi! Use the demo environment
//! unless you understand the risks. Orders are placed with post_only=true at prices
//! far from the market to minimize execution risk.
//!
//! Run with: cargo run --example batch_orders

use kalshi_trade_rs::{
    Action, BatchCancelOrdersRequest, BatchCreateOrdersRequest, CreateOrderRequest,
    GetMarketsParams, KalshiClient, KalshiConfig, MarketFilterStatus, OrderType, Side,
    TimeInForce, cents_to_dollars,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment", config.environment);
    println!("**WARNING**: This example places REAL orders!\n");

    let client = KalshiClient::new(config)?;

    // Find open markets to trade
    println!("=== Finding Open Markets ===");
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(5);
    let markets = client.get_markets_with_params(params).await?;

    if markets.markets.is_empty() {
        println!("No open markets found");
        return Ok(());
    }

    println!("Found {} open markets:", markets.markets.len());
    for market in &markets.markets {
        println!("  {}", market.ticker);
    }
    println!();

    // 1. Batch Create Orders
    println!("=== Batch Create Orders ===");

    // Create orders at different price levels (all safe, low prices)
    let mut orders = Vec::new();
    let market = &markets.markets[0];

    let base_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    for i in 1..=3 {
        let client_order_id = format!("batch-{}-{}", i, base_ts + i as u128);
        let order = CreateOrderRequest::new(&market.ticker, Side::Yes, Action::Buy, 1)
            .client_order_id(&client_order_id)
            .order_type(OrderType::Limit)
            .yes_price(i as i64) // 1, 2, 3 cents
            .post_only(true)
            .time_in_force(TimeInForce::GoodTillCanceled);

        println!(
            "Order {}: {} YES @ {} cents (client_id: {}...)",
            i,
            market.ticker,
            i,
            &client_order_id[..20]
        );
        orders.push(order);
    }

    // Also create an order for a different market if available
    if markets.markets.len() > 1 {
        let market2 = &markets.markets[1];
        let client_order_id = format!("batch-4-{}", base_ts + 4);
        let order = CreateOrderRequest::new(&market2.ticker, Side::No, Action::Buy, 1)
            .client_order_id(&client_order_id)
            .order_type(OrderType::Limit)
            .no_price(1) // 1 cent
            .post_only(true)
            .time_in_force(TimeInForce::GoodTillCanceled);

        println!(
            "Order 4: {} NO @ 1 cents (client_id: {}...)",
            market2.ticker,
            &client_order_id[..20]
        );
        orders.push(order);
    }

    println!("\nSubmitting batch of {} orders...", orders.len());

    let batch_request = BatchCreateOrdersRequest::new(orders);
    let batch_response = client.batch_create_orders(batch_request).await?;

    println!("\nBatch create results:");
    let mut successful_order_ids = Vec::new();

    for (i, result) in batch_response.orders.iter().enumerate() {
        print!("  Order {}: ", i + 1);

        if let Some(order) = &result.order {
            println!("SUCCESS - ID: {} | status: {:?}", order.order_id, order.status);
            successful_order_ids.push(order.order_id.clone());
        } else if let Some(error) = &result.error {
            println!("FAILED - {}: {}", error.code, error.message);
            if let Some(details) = &error.details {
                println!("           Details: {}", details);
            }
        } else {
            println!("UNKNOWN RESULT");
        }
    }

    let success_count = successful_order_ids.len();
    let total_count = batch_response.orders.len();
    println!(
        "\nBatch summary: {}/{} orders succeeded",
        success_count, total_count
    );
    println!();

    // 2. Demonstrating partial success handling
    println!("=== Partial Success Handling ===");
    println!("When some orders in a batch fail, you need to:");
    println!("  1. Check each result individually");
    println!("  2. Handle successful orders (track IDs for management)");
    println!("  3. Log/handle failed orders (retry logic, error reporting)");
    println!();

    // Example of processing results
    let (succeeded, failed): (Vec<_>, Vec<_>) = batch_response
        .orders
        .iter()
        .partition(|r| r.order.is_some());

    println!("Successful orders: {}", succeeded.len());
    for result in &succeeded {
        if let Some(order) = &result.order {
            println!("  {} | {} {:?} @ ${:.2}",
                order.order_id,
                order.ticker,
                order.side,
                cents_to_dollars(order.yes_price)
            );
        }
    }

    if !failed.is_empty() {
        println!("Failed orders: {}", failed.len());
        for result in &failed {
            if let (Some(client_id), Some(error)) = (&result.client_order_id, &result.error) {
                println!("  Client ID: {} | Error: {}", client_id, error.message);
            }
        }
    }
    println!();

    // 3. Batch Cancel Orders
    if !successful_order_ids.is_empty() {
        println!("=== Batch Cancel Orders ===");
        println!("Canceling {} orders...", successful_order_ids.len());

        let cancel_request = BatchCancelOrdersRequest::new(successful_order_ids.clone());
        let cancel_response = client.batch_cancel_orders(cancel_request).await?;

        println!("\nBatch cancel results:");
        let mut canceled_count = 0;
        let mut failed_count = 0;

        for result in &cancel_response.orders {
            if let Some(order) = &result.order {
                println!(
                    "  {} | CANCELED | reduced by: {} contracts",
                    order.order_id,
                    result.reduced_by.unwrap_or(0)
                );
                canceled_count += 1;
            } else if let Some(error) = &result.error {
                println!(
                    "  {} | FAILED | {}: {}",
                    result.order_id.as_deref().unwrap_or("?"),
                    error.code,
                    error.message
                );
                failed_count += 1;
            }
        }

        println!(
            "\nCancel summary: {} canceled, {} failed",
            canceled_count, failed_count
        );
        println!();
    }

    // 4. Batch size limits
    println!("=== Batch Size Limits ===");
    println!("Maximum orders per batch: 20");
    println!();
    println!("Use try_new() for validation:");
    println!("  let request = BatchCreateOrdersRequest::try_new(orders)?;");
    println!("  let request = BatchCancelOrdersRequest::try_new(ids)?;");
    println!();

    // Demonstrate validation
    let too_many: Vec<CreateOrderRequest> = (0..21)
        .map(|i| CreateOrderRequest::new(format!("FAKE-{}", i), Side::Yes, Action::Buy, 1))
        .collect();

    match BatchCreateOrdersRequest::try_new(too_many) {
        Ok(_) => println!("Unexpected: batch should have been rejected"),
        Err(e) => println!("Validation caught: {}", e),
    }
    println!();

    // 5. Best practices
    println!("=== Best Practices ===");
    println!("1. Always use try_new() to validate batch size");
    println!("2. Use unique client_order_id for each order (for tracking)");
    println!("3. Handle partial failures - some orders may succeed while others fail");
    println!("4. Use post_only=true for market making to avoid taking liquidity");
    println!("5. Consider rate limits when submitting many batches");
    println!();

    println!("=== Advanced: Using BatchManager ===");
    println!("For large-scale operations, use the BatchManager:");
    println!();
    println!("  use kalshi_trade_rs::{{BatchManager, RateLimitTier}};");
    println!();
    println!("  let manager = BatchManager::builder()");
    println!("      .rate_limit(RateLimitTier::Standard)");
    println!("      .max_retries(3)");
    println!("      .build();");
    println!();
    println!("  // Create many orders with automatic batching");
    println!("  let results = manager.create_orders(&client, orders).await?;");
    println!();

    println!("=== Done ===");
    Ok(())
}
