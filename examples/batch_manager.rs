//! BatchManager example
//!
//! Demonstrates the BatchManager for rate-limited batch operations.
//! The BatchManager handles automatic chunking, rate limiting, and retry logic
//! for large-scale order operations.
//!
//! **WARNING**: This example places REAL orders on Kalshi! Use the demo environment
//! unless you understand the risks. Orders are placed at prices far from the market
//! to minimize execution risk.
//!
//! Run with: cargo run --example batch_manager

use kalshi_trade_rs::{
    Action, BatchManager, CreateOrderRequest, GetMarketsParams, KalshiClient, KalshiConfig,
    MarketFilterStatus, OrderType, RateLimitTier, RetryConfig, Side, TimeInForce,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment", config.environment);
    println!("**WARNING**: This example places REAL orders!\n");

    let client = KalshiClient::new(config)?;

    // Find an open market
    println!("=== Finding Open Market ===");
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(1);
    let markets = client.get_markets_with_params(params).await?;

    if markets.markets.is_empty() {
        println!("No open markets found");
        return Ok(());
    }

    let market = &markets.markets[0];
    println!("Using market: {}\n", market.ticker);

    // 1. Rate Limit Tiers
    println!("=== Rate Limit Tiers ===");
    println!("Kalshi has different rate limit tiers based on account type:\n");

    let tiers = [
        (RateLimitTier::Basic, "Basic", "Default tier"),
        (RateLimitTier::Advanced, "Advanced", "Increased limits"),
        (RateLimitTier::Premier, "Premier", "High-volume tier"),
        (RateLimitTier::Prime, "Prime", "Highest tier"),
    ];

    for (tier, name, desc) in &tiers {
        println!(
            "  {} ({}):",
            name, desc
        );
        println!(
            "    Reads:  {:>3}/sec",
            tier.reads_per_second() as i32
        );
        println!(
            "    Writes: {:>3}/sec",
            tier.writes_per_second() as i32
        );
    }
    println!();

    // 2. Basic BatchManager Usage
    println!("=== Basic BatchManager ===");
    println!("Creating a BatchManager with Basic tier...\n");

    let manager = BatchManager::new(&client, RateLimitTier::Basic);

    // Create a small batch of orders
    let base_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let orders: Vec<CreateOrderRequest> = (1..=5)
        .map(|i| {
            let client_order_id = format!("batch-mgr-{}-{}", i, base_ts + i as u128);
            CreateOrderRequest::new(&market.ticker, Side::Yes, Action::Buy, 1)
                .client_order_id(&client_order_id)
                .order_type(OrderType::Limit)
                .yes_price(i as i64) // 1-5 cents - far from market
                .post_only(true)
                .time_in_force(TimeInForce::GoodTillCanceled)
        })
        .collect();

    println!("Submitting {} orders with rate limiting...", orders.len());
    let result = manager.create_orders(orders).await;

    println!("Results:");
    println!("  Completed: {}", result.is_complete());
    println!("  Successes: {}", result.completed.success_count());
    println!("  Failures:  {}", result.completed.failure_count());

    if let Some(err) = &result.error {
        println!("  Error: {}", err);
    }

    // Collect order IDs for later cleanup
    let order_ids: Vec<String> = result
        .completed
        .successful_orders()
        .map(|o| o.order_id.clone())
        .collect();
    println!();

    // 3. Builder Pattern with Advanced Configuration
    println!("=== Builder Pattern ===");
    println!("Using the builder for advanced configuration...\n");

    let manager_advanced = BatchManager::builder(&client)
        .tier(RateLimitTier::Advanced)
        .retry_config(RetryConfig::default())
        .build();

    println!("Created manager with:");
    println!("  Tier: Advanced (30 writes/sec)");
    println!("  Retries: 3 (default)");
    println!();

    // 4. RetryConfig Options
    println!("=== Retry Configuration ===");

    let no_retries = RetryConfig::no_retries();
    println!("No retries: max_retries = {}", no_retries.max_retries);

    let default_retries = RetryConfig::default();
    println!(
        "Default: max_retries = {}, base_delay = {:?}, max_delay = {:?}",
        default_retries.max_retries, default_retries.base_delay, default_retries.max_delay
    );

    let custom_retries = RetryConfig::with_max_retries(5);
    println!("Custom: max_retries = {}", custom_retries.max_retries);

    let fully_custom = RetryConfig {
        max_retries: 10,
        base_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(30),
    };
    println!(
        "Fully custom: max_retries = {}, base_delay = {:?}, max_delay = {:?}",
        fully_custom.max_retries, fully_custom.base_delay, fully_custom.max_delay
    );
    println!();

    // 5. Handling Partial Success
    println!("=== Partial Success Handling ===");
    println!("BatchManager preserves partial progress when errors occur:\n");

    println!("// Using into_result() - discards partial progress");
    println!("let response = manager.create_orders(orders).await.into_result()?;");
    println!();

    println!("// Preserving partial progress");
    println!("let result = manager.create_orders(orders).await;");
    println!("for order in result.completed.successful_orders() {{");
    println!("    // Process successfully created orders");
    println!("}}");
    println!("if let Some(err) = result.error {{");
    println!("    // Handle the error that stopped processing");
    println!("}}");
    println!();

    // 6. Batch Cancel
    println!("=== Batch Cancel Orders ===");
    if !order_ids.is_empty() {
        println!("Canceling {} orders with rate limiting...", order_ids.len());

        let cancel_result = manager_advanced.cancel_orders(order_ids).await;

        println!("Cancel results:");
        println!("  Completed: {}", cancel_result.is_complete());
        println!("  Successes: {}", cancel_result.completed.success_count());
        println!("  Failures:  {}", cancel_result.completed.failure_count());
        println!(
            "  Total contracts reduced: {}",
            cancel_result.completed.total_reduced()
        );

        if let Some(err) = &cancel_result.error {
            println!("  Error: {}", err);
        }
    } else {
        println!("No orders to cancel.");
    }
    println!();

    // 7. Large Batch Example (conceptual)
    println!("=== Large Batch Operations ===");
    println!("The BatchManager automatically handles:");
    println!("  - Chunking orders into batches of 20 (API limit)");
    println!("  - Rate limiting based on your account tier");
    println!("  - Retry logic with exponential backoff");
    println!();

    println!("Example: Creating 100 orders with Premier tier:");
    println!("  let manager = BatchManager::builder(&client)");
    println!("      .tier(RateLimitTier::Premier)");
    println!("      .retry_config(RetryConfig::default())");
    println!("      .build();");
    println!();
    println!("  // Orders automatically chunked into 5 batches of 20");
    println!("  // Rate limited to 100 writes/sec");
    println!("  let result = manager.create_orders(orders_100).await;");
    println!();

    // 8. Cost Calculation
    println!("=== Operation Costs ===");
    println!("The BatchManager tracks write costs:");
    println!("  - Create order: 1.0 write token per order");
    println!("  - Cancel order: 0.2 write tokens per order");
    println!();
    println!("This means you can cancel 5x more orders than you can create");
    println!("within the same rate limit window.");
    println!();

    // Best Practices
    println!("=== Best Practices ===");
    println!("1. Choose the correct tier for your account");
    println!("2. Enable retries for production workloads");
    println!("3. Always handle partial success scenarios");
    println!("4. Use unique client_order_ids for tracking");
    println!("5. Monitor for rate limit errors in result.error");

    println!("\n=== Done ===");
    Ok(())
}
