//! Exchange Status API example
//!
//! Demonstrates Exchange API methods for checking exchange status, schedule,
//! and announcements. These are public endpoints that don't require authentication.
//!
//! Run with: cargo run --example exchange_status

use kalshi_trade_rs::{KalshiClient, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize client
    let config = KalshiConfig::from_env()?;
    println!("Connected to {:?} environment\n", config.environment);

    let client = KalshiClient::new(config)?;

    // 1. Get Exchange Status
    println!("=== Exchange Status ===");
    let status = client.get_exchange_status().await?;

    println!("Exchange active: {}", status.exchange_active);
    println!("Trading active:  {}", status.trading_active);

    if !status.exchange_active {
        println!("\nThe exchange is currently down for maintenance.");
        if let Some(resume_time) = &status.exchange_estimated_resume_time {
            println!("Estimated resume time: {}", resume_time);
        }
    } else if !status.trading_active {
        println!("\nThe exchange is active but trading is paused.");
        println!("This typically happens outside of trading hours.");
    } else {
        println!("\nAll systems operational - trading is live!");
    }
    println!();

    // 2. Get Exchange Schedule
    println!("=== Exchange Schedule ===");
    let schedule_response = client.get_exchange_schedule().await?;
    let schedule = &schedule_response.schedule;

    // Display standard hours for today
    println!("Weekly Trading Schedule (all times in Eastern Time):");
    println!();

    // Helper to print daily sessions
    fn print_day_sessions(day_name: &str, sessions: &[kalshi_trade_rs::TradingSession]) {
        if sessions.is_empty() {
            println!("  {}: Closed", day_name);
        } else {
            let sessions_str: Vec<String> = sessions
                .iter()
                .map(|s| format!("{}-{}", s.open_time, s.close_time))
                .collect();
            println!("  {}: {}", day_name, sessions_str.join(", "));
        }
    }

    // Display each day's schedule from the first (current) period
    if let Some(period) = schedule.standard_hours.first() {
        if let Some(start) = &period.start_time {
            println!("Effective from: {}", start);
        }
        if let Some(end) = &period.end_time {
            println!("Until: {}", end);
        }
        println!();

        print_day_sessions("Monday   ", &period.monday);
        print_day_sessions("Tuesday  ", &period.tuesday);
        print_day_sessions("Wednesday", &period.wednesday);
        print_day_sessions("Thursday ", &period.thursday);
        print_day_sessions("Friday   ", &period.friday);
        print_day_sessions("Saturday ", &period.saturday);
        print_day_sessions("Sunday   ", &period.sunday);
    }
    println!();

    // Display maintenance windows if any
    if !schedule.maintenance_windows.is_empty() {
        println!("Scheduled Maintenance Windows:");
        for window in &schedule.maintenance_windows {
            println!("  {} to {}", window.start_datetime, window.end_datetime);
        }
    } else {
        println!("No scheduled maintenance windows.");
    }
    println!();

    // 3. Get Exchange Announcements
    println!("=== Exchange Announcements ===");
    let announcements_response = client.get_exchange_announcements().await?;

    if announcements_response.announcements.is_empty() {
        println!("No current announcements.");
    } else {
        println!(
            "Found {} announcement(s):",
            announcements_response.announcements.len()
        );
        println!();

        for (i, announcement) in announcements_response.announcements.iter().enumerate() {
            let type_emoji = match announcement.announcement_type {
                kalshi_trade_rs::AnnouncementType::Info => "[INFO]",
                kalshi_trade_rs::AnnouncementType::Warning => "[WARN]",
                kalshi_trade_rs::AnnouncementType::Error => "[ERROR]",
                _ => "[???]",
            };

            let status_str = match announcement.status {
                kalshi_trade_rs::AnnouncementStatus::Active => "Active",
                kalshi_trade_rs::AnnouncementStatus::Inactive => "Inactive",
                _ => "Unknown",
            };

            println!("{}. {} ({})", i + 1, type_emoji, status_str);
            println!("   Time: {}", announcement.delivery_time);
            println!("   Message: {}", announcement.message);
            println!();
        }
    }

    // 4. Practical usage example: Check before trading
    println!("=== Pre-Trade Check Example ===");
    let status = client.get_exchange_status().await?;

    if status.exchange_active && status.trading_active {
        println!("Ready to trade!");
        // In production, you would proceed with trading operations here
    } else {
        println!("Trading is currently unavailable.");
        if !status.exchange_active {
            if let Some(resume_time) = &status.exchange_estimated_resume_time {
                println!("Check back after: {}", resume_time);
            }
        }
        // In production, you might schedule a retry or notify the user
    }

    println!("\n=== Done ===");
    Ok(())
}
