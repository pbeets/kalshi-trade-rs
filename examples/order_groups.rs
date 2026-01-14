//! Order Groups API example
//!
//! Demonstrates Order Groups API methods for managing position limits.
//! Order groups allow you to set a contracts limit that auto-cancels all orders
//! when the limit is hit - useful for position size management.
//!
//! **WARNING**: This example places REAL orders on Kalshi! Use the demo environment
//! unless you understand the risks.
//!
//! Run with: cargo run --example order_groups

use kalshi_trade_rs::{
    Action, CreateOrderGroupRequest, CreateOrderRequest, GetMarketsParams, GetOrderGroupsParams,
    KalshiClient, KalshiConfig, MarketFilterStatus, OrderType, Side, TimeInForce,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment", config.environment);
    println!("**WARNING**: This example places REAL orders!\n");

    let client = KalshiClient::new(config)?;

    // Find open markets
    println!("=== Finding Open Markets ===");
    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(1);
    let markets = client.get_markets_with_params(params).await?;

    if markets.markets.is_empty() {
        println!("No open markets found");
        return Ok(());
    }

    let market = &markets.markets[0];
    println!("Using market: {}", market.ticker);
    println!();

    // 1. Create an Order Group
    println!("=== Create Order Group ===");
    println!("Creating a group with contracts_limit=10...\n");

    let request = CreateOrderGroupRequest::new(10);
    let create_response = client.create_order_group(request).await?;
    let order_group_id = &create_response.order_group_id;

    println!("Order Group ID: {}", order_group_id);
    println!();

    // 2. Create orders associated with the group
    println!("=== Create Orders in Group ===");
    println!("Creating an order with order_group_id...\n");

    let order = CreateOrderRequest::new(&market.ticker, Side::Yes, Action::Buy, 1)
        .order_type(OrderType::Limit)
        .yes_price(1) // 1 cent - far from market
        .post_only(true)
        .time_in_force(TimeInForce::GoodTillCanceled)
        .order_group_id(order_group_id);

    let order_response = client.create_order(order).await?;
    println!("Created order: {}", order_response.order.order_id);
    println!(
        "Order is in group: {:?}",
        order_response.order.order_group_id
    );
    println!();

    // 3. Get Order Group by ID
    println!("=== Get Order Group ===");
    let get_response = client.get_order_group(order_group_id).await?;

    println!("Order Group ID: {}", order_group_id);
    println!(
        "Auto-cancel enabled: {}",
        get_response.is_auto_cancel_enabled
    );
    println!("Order IDs in group: {}", get_response.orders.len());
    for oid in &get_response.orders {
        println!("  - {}", oid);
    }
    println!();

    // 4. List Order Groups
    println!("=== List Order Groups ===");
    let list_response = client.list_order_groups().await?;

    println!("Total order groups: {}", list_response.order_groups.len());
    for (i, group) in list_response.order_groups.iter().take(5).enumerate() {
        println!(
            "  {}. {} | auto_cancel={}",
            i + 1,
            group.id,
            group.is_auto_cancel_enabled
        );
    }
    println!();

    // 5. List with pagination
    println!("=== List with Pagination ===");
    let params = GetOrderGroupsParams::new().limit(2);
    let page1 = client.list_order_groups_with_params(params).await?;
    println!("Page 1: {} groups", page1.order_groups.len());
    println!();

    // 6. Reset Order Group
    println!("=== Reset Order Group ===");
    println!("Resetting matched contracts counter...\n");
    client.reset_order_group(order_group_id).await?;
    println!("Reset complete.");
    println!();

    // 7. Delete Order Group
    println!("=== Delete Order Group ===");
    println!("Deleting the order group (this cancels all orders)...\n");
    client.delete_order_group(order_group_id).await?;
    println!("Order group deleted successfully.");
    println!();

    // Best Practices
    println!("=== Order Group Best Practices ===");
    println!("1. Use order groups to limit total position size");
    println!("2. Set contracts_limit to your maximum desired exposure");
    println!("3. All orders auto-cancel when the limit is hit");
    println!("4. Reset the group to restart after hitting limits");
    println!("5. Delete groups when done to clean up");
    println!();

    // Workflow
    println!("=== Typical Workflow ===");
    println!("1. Create order group with contracts_limit");
    println!("2. Create orders with order_group_id parameter");
    println!("3. Orders auto-cancel when limit reached");
    println!("4. Reset group to continue trading");
    println!("5. Delete group when position is closed");
    println!();

    // Available API Methods
    println!("=== Available Order Group Methods ===");
    println!("  create_order_group()            - Create new order group");
    println!("  get_order_group(id)             - Get order group details");
    println!("  list_order_groups()             - List all order groups");
    println!("  list_order_groups_with_params() - List with pagination");
    println!("  reset_order_group(id)           - Reset matched contracts counter");
    println!("  delete_order_group(id)          - Delete group and cancel orders");
    println!();

    println!("=== Done ===");
    Ok(())
}
