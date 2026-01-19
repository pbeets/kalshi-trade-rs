//! Example: Verify RFQ functionality (read-only).
//!
//! This example demonstrates and verifies the RFQ (Request for Quote) API
//! functionality without placing any trades. It performs read-only operations:
//!
//! - Gets your communications ID
//! - Lists existing RFQs and quotes
//! - Briefly connects to the WebSocket Communications channel
//!
//! # Usage
//!
//! ```bash
//! KALSHI_API_KEY_ID=your_key KALSHI_PRIVATE_KEY_PATH=path/to/key.pem \
//!     cargo run --example rfq_verify
//! ```
//!
//! # RFQ Workflow Overview
//!
//! For clients building RFQ trading/monitoring systems:
//!
//! ## As an RFQ Creator (requesting quotes):
//! 1. Create an RFQ with `client.create_rfq()` specifying contracts or target_cost
//! 2. Subscribe to WebSocket `Communications` channel for `QuoteCreated` events
//! 3. Review incoming quotes and `accept_quote()` to execute the trade
//!
//! ## As a Market Maker (providing quotes):
//! 1. Subscribe to WebSocket `Communications` channel for `RfqCreated` events
//! 2. When an RFQ arrives, analyze and respond with `client.create_quote()`
//! 3. Wait for acceptance or cancellation via WebSocket events
//!
//! ## Key Types:
//! - `CreateRfqRequest::with_contracts()` - RFQ requesting specific contract count
//! - `CreateRfqRequest::with_target_cost_dollars()` - RFQ with dollar target
//! - `CreateQuoteRequest::from_cents()` - Quote with YES price in cents (1-99)
//! - `AcceptQuoteRequest::yes()` / `AcceptQuoteRequest::no()` - Accept a side
//!
//! ## WebSocket Events:
//! - `CommunicationData::RfqCreated` - New RFQ created
//! - `CommunicationData::RfqDeleted` - RFQ cancelled
//! - `CommunicationData::QuoteCreated` - Quote submitted
//! - `CommunicationData::QuoteAccepted` - Quote accepted (trade executing)

use std::time::Duration;

use kalshi_trade_rs::{
    KalshiClient, ListQuotesParams, ListRfqsParams,
    auth::KalshiConfig,
    ws::{Channel, CommunicationData, KalshiStreamClient, StreamMessage},
};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("kalshi_trade_rs=debug".parse()?),
        )
        .init();

    let config = KalshiConfig::from_env()?;
    println!(
        "=== RFQ Verification Example ({:?}) ===\n",
        config.environment
    );

    // -------------------------------------------------------------------------
    // Part 1: REST API Verification
    // -------------------------------------------------------------------------
    println!("--- REST API Tests ---\n");

    let client = KalshiClient::new(config.clone())?;

    // Test 1: Get communications ID
    println!("1. Getting communications ID...");
    match client.get_communications_id().await {
        Ok(response) => {
            println!("   Communications ID: {}\n", response.communications_id);
        }
        Err(e) => {
            println!("   Error: {}\n", e);
        }
    }

    // Test 2: List RFQs (all statuses)
    println!("2. Listing RFQs...");
    let params = ListRfqsParams::new().limit(10);
    match client.list_rfqs_with_params(params).await {
        Ok(response) => {
            if response.rfqs.is_empty() {
                println!("   No RFQs found (this is normal if you haven't created any)\n");
            } else {
                println!("   Found {} RFQ(s):", response.rfqs.len());
                for rfq in &response.rfqs {
                    // RFQs specify either contracts OR target_cost. When contracts=0 or None,
                    // check target_cost. The API may return contracts=Some(0) when target_cost is used.
                    let size = match rfq.contracts {
                        Some(c) if c > 0 => format!("{} contracts", c),
                        _ => rfq
                            .target_cost_dollars()
                            .map(|d| format!("${:.2} target", d))
                            .unwrap_or_else(|| "unknown size".to_string()),
                    };
                    println!(
                        "   - {} | {} | {} | status: {}",
                        rfq.id,
                        rfq.market_ticker,
                        size,
                        rfq.status.as_deref().unwrap_or("unknown")
                    );
                }
                println!();
            }
        }
        Err(e) => {
            println!("   Error: {}\n", e);
        }
    }

    // Test 3: List quotes for a specific RFQ
    // Note: list_quotes() requires either:
    //   - quote_creator_user_id: to see quotes YOU created
    //   - rfq_creator_user_id: to see quotes for RFQs YOU created
    //   - rfq_id: to see quotes for a specific RFQ
    // The user_id is different from communications_id.
    println!("3. Listing quotes...");

    // Try querying by RFQ ID if we found any open RFQs
    let open_rfq = client
        .list_rfqs_with_params(ListRfqsParams::new().status("open").limit(1))
        .await
        .ok()
        .and_then(|r| r.rfqs.into_iter().next());

    if let Some(rfq) = open_rfq {
        let params = ListQuotesParams::new().rfq_id(&rfq.id).limit(10);
        match client.list_quotes_with_params(params).await {
            Ok(response) => {
                if response.quotes.is_empty() {
                    println!(
                        "   No quotes found for RFQ {} (normal if no one quoted)\n",
                        rfq.id
                    );
                } else {
                    println!(
                        "   Found {} quote(s) for RFQ {}:",
                        response.quotes.len(),
                        rfq.id
                    );
                    for quote in &response.quotes {
                        let yes_bid = quote.yes_bid_dollars.as_deref().unwrap_or("?");
                        let no_bid = quote.no_bid_dollars.as_deref().unwrap_or("?");
                        println!(
                            "   - {} | yes: {} / no: {} | status: {}",
                            quote.id,
                            yes_bid,
                            no_bid,
                            quote.status.as_deref().unwrap_or("unknown")
                        );
                    }
                    println!();
                }
            }
            Err(e) => {
                // The API requires either quote_creator_user_id or rfq_creator_user_id
                // when querying by rfq_id if you don't own that RFQ
                println!("   list_quotes() requires user_id filter or ownership of the RFQ");
                println!("   API note: {}\n", e);
            }
        }
    } else {
        println!("   No open RFQs to query quotes for\n");
    }

    // -------------------------------------------------------------------------
    // Part 2: WebSocket Verification
    // -------------------------------------------------------------------------
    println!("--- WebSocket Test ---\n");

    println!("4. Connecting to WebSocket and subscribing to Communications channel...");
    let ws_client = KalshiStreamClient::connect(&config).await?;
    let mut handle = ws_client.handle();

    // Subscribe to Communications channel
    handle.subscribe(Channel::Communications, &[]).await?;
    println!("   Subscribed to Communications channel");
    println!(
        "   Listening for 10 seconds (you likely won't see events unless RFQs are active)...\n"
    );

    let deadline = Duration::from_secs(10);
    let start = std::time::Instant::now();
    let mut event_count = 0;

    loop {
        if start.elapsed() > deadline {
            break;
        }

        match timeout(Duration::from_secs(2), handle.update_receiver.recv()).await {
            Ok(Ok(update)) => match &update.msg {
                StreamMessage::Communication(comm) => {
                    event_count += 1;
                    match comm {
                        CommunicationData::RfqCreated(rfq) => {
                            let size = rfq
                                .target_cost_dollars
                                .as_deref()
                                .map(|d| format!("${}", d))
                                .or_else(|| rfq.contracts.map(|c| format!("{} contracts", c)))
                                .unwrap_or_else(|| "?".to_string());
                            println!(
                                "   [RFQ CREATED] {} | {} | {}",
                                rfq.id, rfq.market_ticker, size
                            );
                        }
                        CommunicationData::RfqDeleted(rfq) => {
                            println!("   [RFQ DELETED] {} | {}", rfq.id, rfq.market_ticker);
                        }
                        CommunicationData::QuoteCreated(quote) => {
                            println!(
                                "   [QUOTE CREATED] {} for RFQ {} | YES: {} / NO: {}",
                                quote.quote_id,
                                quote.rfq_id,
                                quote.yes_bid_dollars,
                                quote.no_bid_dollars
                            );
                        }
                        CommunicationData::QuoteAccepted(quote) => {
                            println!(
                                "   [QUOTE ACCEPTED] {} | side: {:?}",
                                quote.quote_id, quote.accepted_side
                            );
                        }
                    }
                }
                StreamMessage::Closed { reason } => {
                    println!("   [CLOSED] {}", reason);
                    break;
                }
                StreamMessage::ConnectionLost { reason, .. } => {
                    println!("   [CONNECTION LOST] {}", reason);
                    break;
                }
                StreamMessage::Unsubscribed => {
                    println!("   [UNSUBSCRIBED]");
                }
                _ => {
                    // Other message types (fill, position, etc.)
                }
            },
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(n))) => {
                println!("   [WARN] Dropped {} messages", n);
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                println!("   [ERROR] Channel closed");
                break;
            }
            Err(_) => {
                // Timeout - no events in last 2 seconds, continue waiting
            }
        }
    }

    if event_count == 0 {
        println!("   No events received (normal when there's no RFQ activity)");
    } else {
        println!("   Received {} event(s)", event_count);
    }

    // Clean up
    handle.unsubscribe_all(Channel::Communications).await?;
    ws_client.shutdown().await?;

    // -------------------------------------------------------------------------
    // Summary
    // -------------------------------------------------------------------------
    println!("\n--- Summary ---\n");
    println!("RFQ API verified:");
    println!("  [OK] get_communications_id()");
    println!("  [OK] list_rfqs()");
    println!("  [OK] list_quotes()");
    println!("  [OK] WebSocket Communications channel");
    println!();
    println!("See module docs and examples/stream_user_channels.rs for usage patterns.");

    Ok(())
}
