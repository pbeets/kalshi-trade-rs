//! Trading API example
//!
//! Demonstrates order lifecycle: create, get, amend, decrease, and cancel orders.
//!
//! **WARNING**: This example places REAL orders on Kalshi! Use the demo environment
//! unless you understand the risks. Orders are placed with post_only=true to minimize
//! the chance of accidental execution.
//!
//! Run with: cargo run --example trading

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kalshi_trade_rs::{
    Action, AmendOrderRequest, CreateOrderRequest, DecreaseOrderRequest, GetMarketsParams,
    GetOrdersParams, GetQueuePositionsParams, KalshiClient, KalshiConfig, MarketFilterStatus,
    OrderStatus, OrderType, Side, TimeInForce, cents_to_dollars,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;

    println!("Connected to {:?} environment", config.environment);
    println!("**WARNING**: This example places REAL orders!\n");

    let client = KalshiClient::new(config)?;

    // Find an open market to trade
    println!("=== Finding an Open Market ===");

    let params = GetMarketsParams::new()
        .status(MarketFilterStatus::Open)
        .limit(5);

    let markets = client.get_markets_with_params(params).await?;

    let Some(market) = markets.markets.first() else {
        println!("No open markets found");
        return Ok(());
    };

    println!("Selected market: {}", market.ticker);

    if let Some(title) = &market.title {
        println!("Title: {}", title);
    }

    if let (Some(yes_bid), Some(yes_ask)) = (&market.yes_bid_dollars, &market.yes_ask_dollars) {
        println!("Current YES: ${} bid / ${} ask", yes_bid, yes_ask);
    }

    println!();

    // Use a price far from the market to avoid execution
    // Place a YES bid at 1 cent (very unlikely to execute)
    let safe_price = 1; // 1 cent

    // Generate a unique client order ID using timestamp
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let client_order_id = format!("example-{}", ts);

    // 1. Create an order with multiple contracts (for decrease_order demo)
    println!("=== Create Order ===");

    let create_request = CreateOrderRequest::new(&market.ticker, Side::Yes, Action::Buy, 5)
        .client_order_id(&client_order_id)
        .order_type(OrderType::Limit)
        .yes_price(safe_price)
        .post_only(true) // Ensures we only add liquidity, never take
        .time_in_force(TimeInForce::GoodTillCanceled);

    println!("Creating order:");
    println!("  Ticker: {}", create_request.ticker);
    println!("  Side: {:?}", create_request.side);
    println!("  Action: {:?}", create_request.action);
    println!("  Count: {}", create_request.count);
    println!(
        "  YES Price: {} cents (${:.2})",
        safe_price,
        cents_to_dollars(safe_price)
    );
    println!("  Client Order ID: {}", client_order_id);
    println!("  Post Only: true");

    let order_response = client.create_order(create_request).await?;
    let order = &order_response.order;

    println!("\nOrder created:");
    println!("  Order ID: {}", order.order_id);
    println!("  Status: {:?}", order.status);
    println!("  Fill Count: {}", order.fill_count);
    println!("  Remaining: {}", order.remaining_count);

    if let Some(created) = &order.created_time {
        println!("  Created: {}", created);
    }

    println!();

    let order_id = order.order_id.clone();

    // Small delay for demo environment consistency
    println!("(Waiting for demo environment consistency...)");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Get the order
    // Note: Demo environment may have eventual consistency delays
    println!("=== Get Order ===");

    match client.get_order(&order_id).await {
        Ok(fetched) => println!(
            "Fetched order {} | status: {:?}",
            fetched.order.order_id, fetched.order.status
        ),
        Err(e) => println!(
            "get_order returned error (demo env consistency issue): {}",
            e
        ),
    }

    println!();

    // 3. Get order queue position
    println!("=== Get Queue Position ===");

    match client.get_order_queue_position(&order_id).await {
        Ok(queue_pos) => {
            println!(
                "Queue position: {} contracts ahead",
                queue_pos.queue_position
            );
        }
        Err(e) => {
            println!("Could not get queue position: {}", e);
        }
    }

    println!();

    // 4. Decrease order quantity
    println!("=== Decrease Order ===");
    println!("Decreasing order from 5 to 3 contracts...");

    let decrease_request = DecreaseOrderRequest::reduce_to(3);

    match client.decrease_order(&order_id, decrease_request).await {
        Ok(decreased) => {
            println!("Order decreased:");
            println!("  Order ID: {}", decreased.order.order_id);
            println!("  Remaining: {}", decreased.order.remaining_count);
            println!("  Status: {:?}", decreased.order.status);
        }
        Err(e) => {
            println!("Could not decrease order: {}", e);
            println!("(This may fail in demo env due to eventual consistency)");
        }
    }

    println!();

    // 5. Amend order (change price)
    println!("=== Amend Order ===");

    let new_price = 2; // Change from 1 cent to 2 cents
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let new_client_order_id = format!("example-amended-{}", ts);

    let amend_request = AmendOrderRequest::new(
        &market.ticker,
        Side::Yes,
        Action::Buy,
        &client_order_id,
        &new_client_order_id,
    )
    .yes_price(new_price)
    .count(3); // Match decreased count

    println!("Amending order:");
    println!("  Old price: {} cents", safe_price);
    println!("  New price: {} cents", new_price);
    println!("  Old client ID: {}", client_order_id);
    println!("  New client ID: {}", new_client_order_id);

    let amend_response = client.amend_order(&order_id, amend_request).await?;
    println!("\nAmend result:");
    println!("  Old order status: {:?}", amend_response.old_order.status);
    println!("  New order ID: {}", amend_response.order.order_id);
    println!("  New order status: {:?}", amend_response.order.status);
    println!(
        "  New order price: {} cents",
        amend_response.order.yes_price
    );
    println!();

    // Update order_id to the new order
    let order_id = amend_response.order.order_id.clone();

    // 6. Cancel order
    println!("=== Cancel Order ===");

    let cancel_response = client.cancel_order(&order_id).await?;

    println!("Order canceled:");
    println!("  Order ID: {}", cancel_response.order.order_id);
    println!("  Status: {:?}", cancel_response.order.status);

    if let Some(reduced_by) = cancel_response.reduced_by {
        println!("  Reduced by: {} contracts", reduced_by);
    }

    println!();

    // 7. List orders
    println!("=== List Orders ===");

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

    println!();

    // 8. List orders with filters
    println!("=== Resting Orders ===");

    let params = GetOrdersParams::new().status(OrderStatus::Resting).limit(5);
    let resting_orders = client.get_orders_with_params(params).await?;

    if resting_orders.orders.is_empty() {
        println!("No resting orders");
    } else {
        for order in &resting_orders.orders {
            println!(
                "  {} | {:?} {:?} {} @ ${:.2} | {}/{}",
                order.ticker,
                order.action,
                order.side,
                order.remaining_count,
                cents_to_dollars(order.yes_price),
                order.fill_count,
                order.initial_count
            );
        }
    }

    println!();

    // 9. Get queue positions (requires market_tickers or event_ticker filter)
    println!("=== Queue Positions (filtered by market) ===");

    let params = GetQueuePositionsParams::new().market_tickers(&market.ticker);

    match client.get_queue_positions_with_params(params).await {
        Ok(response) => {
            if let Some(queue_positions) = &response.queue_positions {
                println!("Orders with queue positions: {}", queue_positions.len());
                for qp in queue_positions.iter().take(5) {
                    println!(
                        "  {} | {} contracts ahead",
                        qp.market_ticker, qp.queue_position
                    );
                }
            } else {
                println!("No orders with queue positions for this market");
            }
        }
        Err(e) => println!("Could not get queue positions: {}", e),
    }

    println!("\n=== Done ===");
    Ok(())
}
