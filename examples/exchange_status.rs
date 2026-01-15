//! Exchange Status API example
//!
//! Demonstrates Exchange API methods for checking exchange status, schedule,
//! announcements, user data timestamps, fee changes, and incentive programs.
//!
//! Run with: cargo run --example exchange_status

use kalshi_trade_rs::{
    FeeType, GetFeeChangesParams, GetIncentiveProgramsParams, KalshiClient, KalshiConfig,
};

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

    // 4. Get User Data Timestamp (requires authentication)
    println!("=== User Data Timestamp ===");
    match client.get_user_data_timestamp().await {
        Ok(timestamp) => {
            println!("User data last validated: {}", timestamp.as_of_time);
            println!("This indicates when portfolio data (balance, orders, fills, positions)");
            println!("was last confirmed accurate by the exchange.");
        }
        Err(e) => {
            // This endpoint requires authentication - handle gracefully if not authenticated
            let err_str = e.to_string();
            if err_str.contains("401") || err_str.contains("Unauthorized") {
                println!("Skipped: Requires authentication (no credentials configured)");
            } else {
                println!("Error: {}", e);
            }
        }
    }
    println!();

    // 5. Get Fee Changes
    println!("=== Fee Changes ===");
    let fee_changes = client.get_fee_changes().await?;

    if fee_changes.series_fee_change_arr.is_empty() {
        println!("No upcoming fee changes scheduled.");
    } else {
        println!(
            "Found {} upcoming fee change(s):",
            fee_changes.series_fee_change_arr.len()
        );
        println!();

        for change in &fee_changes.series_fee_change_arr {
            let fee_type_str = match change.fee_type {
                FeeType::Quadratic => "Quadratic",
                FeeType::QuadraticWithMakerFees => "Quadratic w/ Maker",
                FeeType::Flat => "Flat",
                _ => "Unknown",
            };
            println!("  Series: {}", change.series_ticker);
            println!("    Fee Type: {}", fee_type_str);
            println!("    Multiplier: {:.2}x", change.fee_multiplier);
            println!("    Effective: {}", change.scheduled_ts);
            println!();
        }
    }

    // Also show historical fee changes
    println!("=== Historical Fee Changes ===");
    let params = GetFeeChangesParams::new().show_historical(true);
    let historical = client.get_fee_changes_with_params(params).await?;

    if historical.series_fee_change_arr.is_empty() {
        println!("No historical fee changes found.");
    } else {
        println!(
            "Found {} total fee change(s) (including historical):",
            historical.series_fee_change_arr.len()
        );
        for change in historical.series_fee_change_arr.iter().take(5) {
            println!(
                "  {} - {:?} @ {} ({})",
                change.series_ticker, change.fee_type, change.fee_multiplier, change.scheduled_ts
            );
        }
        if historical.series_fee_change_arr.len() > 5 {
            println!(
                "  ... and {} more",
                historical.series_fee_change_arr.len() - 5
            );
        }
    }
    println!();

    // 6. Get Incentive Programs
    println!("=== Incentive Programs ===");
    let programs = client.get_incentive_programs().await?;

    if programs.incentive_programs.is_empty() {
        println!("No incentive programs found.");
    } else {
        println!(
            "Found {} incentive program(s):",
            programs.incentive_programs.len()
        );
        println!();

        for program in programs.incentive_programs.iter().take(10) {
            let incentive_type = program.incentive_type.as_deref().unwrap_or("unknown");
            let market = program.market_ticker.as_deref().unwrap_or("(no market)");
            let paid = program.paid_out.unwrap_or(false);

            println!(
                "  {} [{}] {}",
                market,
                incentive_type,
                if paid { "(paid)" } else { "" }
            );

            if let Some(reward) = program.period_reward {
                // Convert cents to dollars
                println!("    Reward: ${:.2}", reward as f64 / 100.0);
            }

            if let Some(start) = &program.start_date {
                print!("    Period: {}", start);
                if let Some(end) = &program.end_date {
                    println!(" to {}", end);
                } else {
                    println!(" (ongoing)");
                }
            }

            if let Some(discount) = program.discount_factor_bps {
                println!("    Discount: {} bps", discount);
            }
            println!();
        }

        if programs.incentive_programs.len() > 10 {
            println!("  ... and {} more", programs.incentive_programs.len() - 10);
            println!();
        }
    }

    // Show only volume-type programs
    println!("=== Volume Incentive Programs ===");
    let params = GetIncentiveProgramsParams::new().program_type("volume");
    let volume_programs = client.get_incentive_programs_with_params(params).await?;

    if volume_programs.incentive_programs.is_empty() {
        println!("No volume incentive programs at this time.");
    } else {
        println!(
            "Found {} volume program(s):",
            volume_programs.incentive_programs.len()
        );
        for program in volume_programs.incentive_programs.iter().take(5) {
            let market = program.market_ticker.as_deref().unwrap_or("(unknown)");
            let reward = program
                .period_reward
                .map(|r| format!("${:.2}", r as f64 / 100.0))
                .unwrap_or_default();
            println!("  - {} {}", market, reward);
        }
    }
    println!();

    // 7. Practical usage example: Check before trading
    println!("=== Pre-Trade Check Example ===");
    let status = client.get_exchange_status().await?;

    if status.exchange_active && status.trading_active {
        println!("Ready to trade!");
        // In production, you would proceed with trading operations here
    } else {
        println!("Trading is currently unavailable.");
        if !status.exchange_active
            && let Some(resume_time) = &status.exchange_estimated_resume_time
        {
            println!("Check back after: {}", resume_time);
        }
        // In production, you might schedule a retry or notify the user
    }

    println!("\n=== Done ===");
    Ok(())
}
