//! Test authentication by fetching portfolio balance
//!
//! Run with: cargo run --example test_auth

use kalshi_trade_rs::{KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("Loading configuration from environment...");

    let config = KalshiConfig::from_env()?;

    println!("Environment: {:?}", config.environment);
    println!("API Key ID: {}", config.api_key_id);

    println!("\nCreating Kalshi client...");
    let client = KalshiClient::new(config)?;

    println!("Fetching portfolio balance...");
    let balance = client.get_balance().await?;

    println!("\n=== Success! ===");
    println!(
        "Balance: {} cents (${:.2})",
        balance.balance,
        balance.balance as f64 / 100.0
    );
    println!(
        "Portfolio Value: {} cents (${:.2})",
        balance.portfolio_value,
        balance.portfolio_value as f64 / 100.0
    );

    // Also test fetching positions
    println!("\nFetching positions...");
    let positions = client.get_positions().await?;
    println!(
        "Found {} market positions, {} event positions",
        positions.market_positions.len(),
        positions.event_positions.len()
    );

    for pos in &positions.market_positions {
        println!(
            "  {}: {} contracts, exposure: {} cents",
            pos.ticker, pos.position, pos.market_exposure
        );
    }

    Ok(())
}
