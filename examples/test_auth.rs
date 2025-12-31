//! Test authentication by fetching portfolio balance
//!
//! Run with: cargo run --example test_auth

use kalshi_trade_rs::{auth::KalshiConfig, client::HttpClient};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    balance: i64,
    payout: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("Loading configuration from environment...");

    let config = KalshiConfig::from_env()?;

    println!("Environment: {:?}", config.environment);
    println!("API Key ID: {}", config.api_key_id);

    println!("\nCreating HTTP client...");
    let client = HttpClient::new(config)?;

    println!("Fetching portfolio balance...");
    let balance: BalanceResponse = client.get("/portfolio/balance").await?;

    println!("\n=== Success! ===");
    println!(
        "Balance: {} cents (${:.2})",
        balance.balance,
        balance.balance as f64 / 100.0
    );

    if let Some(payout) = balance.payout {
        println!("Payout: {} cents (${:.2})", payout, payout as f64 / 100.0);
    }

    Ok(())
}
