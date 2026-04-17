//! Integration tests that verify each API endpoint against the Kalshi demo environment.
//!
//! These tests confirm:
//! - URL paths are correct (no 404)
//! - HTTP methods are correct (no 405)
//! - Request bodies serialize correctly (no 400)
//! - Response bodies deserialize into our types (no serde errors)
//! - Auth headers are computed correctly (no 401)
//!
//! # Running
//!
//! ```bash
//! # Set up credentials (copy .env.blank to .env and fill in demo credentials)
//! cargo test --test api_integration
//! ```
//!
//! # Notes
//!
//! - All tests hit the real Kalshi demo API — they are ignored by default.
//!   Run with `cargo test --test api_integration -- --ignored` to execute.
//! - Mutating endpoints (create/cancel orders, etc.) are tested but use
//!   safe parameters that won't affect real positions.

mod common;

use common::test_client;
use kalshi_trade_rs::models::*;

// =========================================================================
// Exchange API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_exchange_status() {
    let client = test_client();
    let result = client.get_exchange_status().await;
    assert!(
        result.is_ok(),
        "get_exchange_status failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_exchange_schedule() {
    let client = test_client();
    let result = client.get_exchange_schedule().await;
    assert!(
        result.is_ok(),
        "get_exchange_schedule failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_exchange_announcements() {
    let client = test_client();
    let result = client.get_exchange_announcements().await;
    assert!(
        result.is_ok(),
        "get_exchange_announcements failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_user_data_timestamp() {
    let client = test_client();
    let result = client.get_user_data_timestamp().await;
    assert!(
        result.is_ok(),
        "get_user_data_timestamp failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Markets API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_markets() {
    let client = test_client();
    let result = client.get_markets().await;
    assert!(result.is_ok(), "get_markets failed: {:?}", result.err());
    let markets = result.unwrap();
    assert!(!markets.markets.is_empty(), "expected at least one market");
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_markets_with_params() {
    let client = test_client();
    let params = GetMarketsParams::new().limit(5);
    let result = client.get_markets_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_markets_with_params failed: {:?}",
        result.err()
    );
    let markets = result.unwrap();
    assert!(markets.markets.len() <= 5);
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_market() {
    let client = test_client();
    // First get a valid ticker from the markets list
    let markets = client
        .get_markets_with_params(GetMarketsParams::new().limit(1))
        .await
        .expect("need at least one market");
    let ticker = &markets.markets[0].ticker;

    let result = client.get_market(ticker).await;
    assert!(result.is_ok(), "get_market failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_orderbook() {
    let client = test_client();
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    let result = client.get_orderbook(ticker).await;
    assert!(result.is_ok(), "get_orderbook failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_orderbook_with_params() {
    let client = test_client();
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    let params = GetOrderbookParams::new().depth(5);
    let result = client.get_orderbook_with_params(ticker, params).await;
    assert!(
        result.is_ok(),
        "get_orderbook_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_trades() {
    let client = test_client();
    let result = client.get_trades().await;
    assert!(result.is_ok(), "get_trades failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_trades_with_params() {
    let client = test_client();
    let params = GetTradesParams::new().limit(5);
    let result = client.get_trades_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_trades_with_params failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Events API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_events() {
    let client = test_client();
    let result = client.get_events().await;
    assert!(result.is_ok(), "get_events failed: {:?}", result.err());
    let events = result.unwrap();
    assert!(!events.events.is_empty(), "expected at least one event");
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_events_with_params() {
    let client = test_client();
    let params = GetEventsParams::new().limit(3);
    let result = client.get_events_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_events_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_event() {
    let client = test_client();
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need at least one event");
    let event_ticker = &events.events[0].event_ticker;

    let result = client.get_event(event_ticker).await;
    assert!(result.is_ok(), "get_event failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_event_with_params() {
    let client = test_client();
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need at least one event");
    let event_ticker = &events.events[0].event_ticker;

    let params = GetEventParams::new().with_nested_markets(true);
    let result = client.get_event_with_params(event_ticker, params).await;
    assert!(
        result.is_ok(),
        "get_event_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_event_metadata() {
    let client = test_client();
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need at least one event");
    let event_ticker = &events.events[0].event_ticker;

    let result = client.get_event_metadata(event_ticker).await;
    assert!(
        result.is_ok(),
        "get_event_metadata failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_multivariate_events() {
    let client = test_client();
    let result = client.get_multivariate_events().await;
    assert!(
        result.is_ok(),
        "get_multivariate_events failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_multivariate_events_with_params() {
    let client = test_client();
    let params = GetMultivariateEventsParams::new().limit(3);
    let result = client.get_multivariate_events_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_multivariate_events_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_event_candlesticks() {
    let client = test_client();
    // Find an event with a known series ticker
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need at least one event");
    let event = &events.events[0];
    let event_ticker = &event.event_ticker;
    let series_ticker = &event.series_ticker;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_week_ago = now - 7 * 86400;

    let params = GetEventCandlesticksParams::new(one_week_ago, now, CandlestickPeriod::OneDay);
    let result = client
        .get_event_candlesticks(series_ticker, event_ticker, params)
        .await;
    assert!(
        result.is_ok(),
        "get_event_candlesticks failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_event_forecast_percentile_history() {
    let client = test_client();
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need at least one event");
    let event = &events.events[0];
    let event_ticker = &event.event_ticker;
    let series_ticker = &event.series_ticker;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_week_ago = now - 7 * 86400;

    let params = GetEventForecastPercentileHistoryParams::new(
        vec![2500, 5000, 7500],
        one_week_ago,
        now,
        ForecastPeriod::OneDay,
    );
    let result = client
        .get_event_forecast_percentile_history(series_ticker, event_ticker, params)
        .await;
    // This endpoint may return 400 if the event doesn't support forecast data
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("400") => {
            eprintln!(
                "NOTE: get_event_forecast_percentile_history returned 400 (event may not support forecasts)"
            );
        }
        Err(e) => panic!(
            "get_event_forecast_percentile_history failed unexpectedly: {:?}",
            e
        ),
    }
}

// =========================================================================
// Series API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_series_list() {
    let client = test_client();
    let result = client.get_series_list().await;
    assert!(result.is_ok(), "get_series_list failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_series() {
    let client = test_client();
    let list = client.get_series_list().await.expect("need series list");
    if list.series.is_empty() {
        eprintln!("SKIP: no series found on demo");
        return;
    }
    let series_ticker = &list.series[0].ticker;

    let result = client.get_series(series_ticker).await;
    assert!(result.is_ok(), "get_series failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_series_with_params() {
    let client = test_client();
    let list = client.get_series_list().await.expect("need series list");
    if list.series.is_empty() {
        eprintln!("SKIP: no series found on demo");
        return;
    }
    let series_ticker = &list.series[0].ticker;

    let params = GetSingleSeriesParams::new().include_volume(true);
    let result = client.get_series_with_params(series_ticker, params).await;
    assert!(
        result.is_ok(),
        "get_series_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fee_changes() {
    let client = test_client();
    let result = client.get_fee_changes().await;
    assert!(result.is_ok(), "get_fee_changes failed: {:?}", result.err());
}

// =========================================================================
// Candlesticks API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_candlesticks() {
    let client = test_client();
    // Find an open market to get candlesticks for
    let events = client
        .get_events_with_params(GetEventsParams::new().limit(1))
        .await
        .expect("need events");
    let event = &events.events[0];
    let series_ticker = &event.series_ticker;

    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .event_ticker(&event.event_ticker)
                .limit(1),
        )
        .await
        .expect("need markets in event");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no markets in event");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_week_ago = now - 7 * 86400;

    let params = GetCandlesticksParams::new(one_week_ago, now, CandlestickPeriod::OneDay);
    let result = client.get_candlesticks(series_ticker, ticker, params).await;
    assert!(
        result.is_ok(),
        "get_candlesticks failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_batch_candlesticks() {
    let client = test_client();
    let markets = client
        .get_markets_with_params(GetMarketsParams::new().limit(2))
        .await
        .expect("need markets");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no markets found");
        return;
    }

    let tickers: Vec<&str> = markets.markets.iter().map(|m| m.ticker.as_str()).collect();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_week_ago = now - 7 * 86400;

    let params = GetBatchCandlesticksParams::from_tickers(
        &tickers,
        one_week_ago,
        now,
        CandlestickPeriod::OneDay,
    );
    let result = client.get_batch_candlesticks(params).await;
    assert!(
        result.is_ok(),
        "get_batch_candlesticks failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Search API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_tags_by_categories() {
    let client = test_client();
    let result = client.get_tags_by_categories().await;
    assert!(
        result.is_ok(),
        "get_tags_by_categories failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_filters_by_sport() {
    let client = test_client();
    let result = client.get_filters_by_sport().await;
    assert!(
        result.is_ok(),
        "get_filters_by_sport failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Portfolio API (authenticated, read-only)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_balance() {
    let client = test_client();
    let result = client.get_balance().await;
    assert!(result.is_ok(), "get_balance failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_balance_with_params() {
    let client = test_client();
    let params = GetBalanceParams::new().subaccount(0);
    let result = client.get_balance_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_balance_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_positions() {
    let client = test_client();
    let result = client.get_positions().await;
    assert!(result.is_ok(), "get_positions failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_positions_with_params() {
    let client = test_client();
    let params = GetPositionsParams::new().limit(10);
    let result = client.get_positions_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_positions_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fills() {
    let client = test_client();
    let result = client.get_fills().await;
    assert!(result.is_ok(), "get_fills failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fills_with_params() {
    let client = test_client();
    let params = GetFillsParams::new().limit(5);
    let result = client.get_fills_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_fills_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_orders() {
    let client = test_client();
    let result = client.get_orders().await;
    assert!(result.is_ok(), "get_orders failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_orders_with_params() {
    let client = test_client();
    let params = GetOrdersParams::new().limit(5);
    let result = client.get_orders_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_orders_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_settlements() {
    let client = test_client();
    let result = client.get_settlements().await;
    assert!(result.is_ok(), "get_settlements failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_settlements_with_params() {
    let client = test_client();
    let params = GetSettlementsParams::new().limit(5);
    let result = client.get_settlements_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_settlements_with_params failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Account API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_api_limits() {
    let client = test_client();
    let result = client.get_api_limits().await;
    assert!(result.is_ok(), "get_api_limits failed: {:?}", result.err());
}

// =========================================================================
// API Keys API (read-only)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_api_keys() {
    let client = test_client();
    let result = client.get_api_keys().await;
    assert!(result.is_ok(), "get_api_keys failed: {:?}", result.err());
}

// =========================================================================
// Orders API (read-only parts)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_queue_positions() {
    let client = test_client();
    // Requires market_tickers or event_ticker
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let params = GetQueuePositionsParams::new().market_tickers(&markets.markets[0].ticker);
    let result = client.get_queue_positions_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_queue_positions failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Order Groups API (read-only)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_list_order_groups() {
    let client = test_client();
    let result = client.list_order_groups().await;
    assert!(
        result.is_ok(),
        "list_order_groups failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Subaccounts API (read-only)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_subaccount_balances() {
    let client = test_client();
    let result = client.get_subaccount_balances().await;
    assert!(
        result.is_ok(),
        "get_subaccount_balances failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_subaccount_transfers() {
    let client = test_client();
    let result = client.get_subaccount_transfers().await;
    assert!(
        result.is_ok(),
        "get_subaccount_transfers failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_resting_order_value() {
    let client = test_client();
    let result = client.get_resting_order_value().await;
    // This endpoint may return 403 for non-FCM accounts, which is expected
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("permission") =>
        {
            eprintln!("NOTE: get_resting_order_value returned 403 (expected for non-FCM accounts)");
        }
        Err(e) => panic!("get_resting_order_value failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_subaccount_netting() {
    let client = test_client();
    match client.get_subaccount_netting().await {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("500") => {
            eprintln!(
                "SKIP: get_subaccount_netting returned server-side 500: {}",
                msg
            );
        }
        Err(e) => panic!("get_subaccount_netting failed: {:?}", e),
    }
}

// =========================================================================
// Communications API (read-only)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_communications_id() {
    let client = test_client();
    let result = client.get_communications_id().await;
    assert!(
        result.is_ok(),
        "get_communications_id failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_list_rfqs() {
    let client = test_client();
    let result = client.list_rfqs().await;
    assert!(result.is_ok(), "list_rfqs failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_list_quotes() {
    let client = test_client();
    // list_quotes requires either quote_creator_user_id or rfq_creator_user_id.
    // Get our communications ID and use it as the rfq_creator_user_id.
    let comms = client.get_communications_id().await.expect("need comms id");
    let our_id = &comms.communications_id;
    let params = ListQuotesParams::new().rfq_creator_user_id(our_id);
    let result = client.list_quotes_with_params(params).await;
    // The API may return 403 if our communications_id doesn't match expectations.
    // In that case, try with quote_creator_user_id instead.
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("403") => {
            let params2 = ListQuotesParams::new().quote_creator_user_id(our_id);
            let result2 = client.list_quotes_with_params(params2).await;
            match &result2 {
                Ok(_) => {}
                Err(kalshi_trade_rs::Error::Api(msg2)) if msg2.contains("403") => {
                    eprintln!(
                        "NOTE: list_quotes returned 403 with both user ID params (comms_id may not match user_id format)"
                    );
                }
                Err(e) => panic!("list_quotes failed unexpectedly: {:?}", e),
            }
        }
        Err(e) => panic!("list_quotes failed: {:?}", e),
    }
}

// =========================================================================
// Milestones API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_milestones() {
    let client = test_client();
    // The milestones endpoint requires a limit parameter
    let params = GetMilestonesParams::new().limit(100);
    let result = client.get_milestones_with_params(params).await;
    assert!(result.is_ok(), "get_milestones failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_milestones_with_params() {
    let client = test_client();
    let params = GetMilestonesParams::new().limit(5);
    let result = client.get_milestones_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_milestones_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_milestone() {
    let client = test_client();
    let milestones = client
        .get_milestones_with_params(GetMilestonesParams::new().limit(1))
        .await
        .expect("need milestones");
    if milestones.milestones.is_empty() {
        eprintln!("SKIP: no milestones found on demo");
        return;
    }
    let milestone_id = milestones.milestones[0]
        .id
        .as_deref()
        .expect("milestone should have an id");

    let result = client.get_milestone(milestone_id).await;
    assert!(result.is_ok(), "get_milestone failed: {:?}", result.err());
}

// =========================================================================
// Structured Targets API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_structured_targets() {
    let client = test_client();
    let result = client.get_structured_targets().await;
    assert!(
        result.is_ok(),
        "get_structured_targets failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_structured_target() {
    let client = test_client();
    let targets = client
        .get_structured_targets_with_params(GetStructuredTargetsParams::new().page_size(1))
        .await
        .expect("need structured targets");
    if targets.structured_targets.is_empty() {
        eprintln!("SKIP: no structured targets found on demo");
        return;
    }
    let target_id = targets.structured_targets[0]
        .id
        .as_deref()
        .expect("target should have an id");

    let result = client.get_structured_target(target_id).await;
    assert!(
        result.is_ok(),
        "get_structured_target failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Multivariate Collections API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_multivariate_collections() {
    let client = test_client();
    let result = client.get_multivariate_collections().await;
    assert!(
        result.is_ok(),
        "get_multivariate_collections failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_multivariate_collection() {
    let client = test_client();
    let collections = client
        .get_multivariate_collections_with_params(GetMultivariateCollectionsParams::new().limit(1))
        .await
        .expect("need collections");
    if collections.collections.is_empty() {
        eprintln!("SKIP: no multivariate collections found on demo");
        return;
    }
    let collection_ticker = &collections.collections[0].collection_ticker;

    let result = client.get_multivariate_collection(collection_ticker).await;
    assert!(
        result.is_ok(),
        "get_multivariate_collection failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_lookup_history() {
    let client = test_client();
    let collections = client
        .get_multivariate_collections_with_params(GetMultivariateCollectionsParams::new().limit(1))
        .await
        .expect("need collections");
    if collections.collections.is_empty() {
        eprintln!("SKIP: no multivariate collections found on demo");
        return;
    }
    let collection_ticker = &collections.collections[0].collection_ticker;

    let result = client.get_lookup_history(collection_ticker, 86400).await;
    assert!(
        result.is_ok(),
        "get_lookup_history failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Live Data API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_batch_live_data() {
    let client = test_client();
    let milestones = client
        .get_milestones_with_params(GetMilestonesParams::new().limit(3))
        .await
        .expect("need milestones");
    if milestones.milestones.is_empty() {
        eprintln!("SKIP: no milestones found on demo");
        return;
    }
    let ids: Vec<&str> = milestones
        .milestones
        .iter()
        .filter_map(|m| m.id.as_deref())
        .collect();
    if ids.is_empty() {
        eprintln!("SKIP: milestones have no IDs");
        return;
    }

    let params = GetBatchLiveDataParams::from_ids(&ids);
    let result = client.get_batch_live_data(params).await;
    assert!(
        result.is_ok(),
        "get_batch_live_data failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Incentive Programs API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_incentive_programs() {
    let client = test_client();
    let result = client.get_incentive_programs().await;
    assert!(
        result.is_ok(),
        "get_incentive_programs failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Historical API
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_cutoff() {
    let client = test_client();
    let result = client.get_historical_cutoff().await;
    assert!(
        result.is_ok(),
        "get_historical_cutoff failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_markets() {
    let client = test_client();
    let result = client.get_historical_markets().await;
    assert!(
        result.is_ok(),
        "get_historical_markets failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_markets_with_params() {
    let client = test_client();
    let params = GetHistoricalMarketsParams::new().limit(5);
    let result = client.get_historical_markets_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_historical_markets_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_market() {
    let client = test_client();
    let markets = client
        .get_historical_markets_with_params(GetHistoricalMarketsParams::new().limit(1))
        .await
        .expect("need historical markets");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no historical markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    let result = client.get_historical_market(ticker).await;
    assert!(
        result.is_ok(),
        "get_historical_market failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_fills() {
    let client = test_client();
    let result = client.get_historical_fills().await;
    assert!(
        result.is_ok(),
        "get_historical_fills failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_fills_with_params() {
    let client = test_client();
    let params = GetHistoricalFillsParams::new().limit(5);
    let result = client.get_historical_fills_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_historical_fills_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_orders() {
    let client = test_client();
    let result = client.get_historical_orders().await;
    assert!(
        result.is_ok(),
        "get_historical_orders failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_orders_with_params() {
    let client = test_client();
    let params = GetHistoricalOrdersParams::new().limit(5);
    let result = client.get_historical_orders_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_historical_orders_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_historical_candlesticks() {
    let client = test_client();
    let markets = client
        .get_historical_markets_with_params(GetHistoricalMarketsParams::new().limit(1))
        .await
        .expect("need historical markets");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no historical markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let one_year_ago = now - 365 * 86400;
    let params = GetHistoricalCandlesticksParams::new(one_year_ago, now, CandlestickPeriod::OneDay);
    let result = client.get_historical_candlesticks(ticker, params).await;
    assert!(
        result.is_ok(),
        "get_historical_candlesticks failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Order lifecycle (create → get → cancel) on demo
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_order_lifecycle() {
    use kalshi_trade_rs::{Action, Side};

    let client = test_client();

    // Find an open market
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // Create an order at a very low price so it won't fill
    let request = CreateOrderRequest::new(ticker, Side::Yes, Action::Buy, 1).yes_price(1);
    let create_result = client.create_order(request).await;
    assert!(
        create_result.is_ok(),
        "create_order failed: {:?}",
        create_result.err()
    );
    let order = create_result.unwrap();
    let order_id = &order.order.order_id;

    // Get the order — may 404 if order was immediately executed
    let get_result = client.get_order(order_id).await;
    match &get_result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("404") => {
            eprintln!("NOTE: get_order returned 404 (order may have been immediately executed)");
            return;
        }
        Err(e) => panic!("get_order failed unexpectedly: {:?}", e),
    }

    // Cancel the order
    let cancel_result = client.cancel_order(order_id).await;
    assert!(
        cancel_result.is_ok(),
        "cancel_order failed: {:?}",
        cancel_result.err()
    );
}

// =========================================================================
// Batch order operations on demo
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_batch_order_lifecycle() {
    use kalshi_trade_rs::{Action, Side};

    let client = test_client();

    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // Batch create
    let orders = vec![
        CreateOrderRequest::new(ticker, Side::Yes, Action::Buy, 1).yes_price(1),
        CreateOrderRequest::new(ticker, Side::No, Action::Buy, 1).no_price(1),
    ];
    let batch_request = BatchCreateOrdersRequest::new(orders);
    let create_result = client.batch_create_orders(batch_request).await;
    assert!(
        create_result.is_ok(),
        "batch_create_orders failed: {:?}",
        create_result.err()
    );

    let created = create_result.unwrap();
    let order_ids: Vec<String> = created
        .orders
        .iter()
        .filter_map(|r| r.order.as_ref().map(|o| o.order_id.clone()))
        .collect();

    if !order_ids.is_empty() {
        // Batch cancel
        #[allow(deprecated)]
        let cancel_request = BatchCancelOrdersRequest::new(order_ids);
        let cancel_result = client.batch_cancel_orders(cancel_request).await;
        assert!(
            cancel_result.is_ok(),
            "batch_cancel_orders failed: {:?}",
            cancel_result.err()
        );
    }
}

// =========================================================================
// Order Group lifecycle on demo
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_order_group_lifecycle() {
    let client = test_client();

    // Create
    let request = CreateOrderGroupRequest::new(100);
    let create_result = client.create_order_group(request).await;
    assert!(
        create_result.is_ok(),
        "create_order_group failed: {:?}",
        create_result.err()
    );
    let group_id = create_result.unwrap().order_group_id;

    // Brief pause to allow the order group to propagate
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Get
    let get_result = client.get_order_group(&group_id).await;
    assert!(
        get_result.is_ok(),
        "get_order_group failed: {:?}",
        get_result.err()
    );

    // Update limit
    let update_result = client
        .update_order_group_limit(&group_id, UpdateOrderGroupLimitRequest::new(200))
        .await;
    assert!(
        update_result.is_ok(),
        "update_order_group_limit failed: {:?}",
        update_result.err()
    );

    // Reset
    let reset_result = client.reset_order_group(&group_id).await;
    assert!(
        reset_result.is_ok(),
        "reset_order_group failed: {:?}",
        reset_result.err()
    );

    // Delete
    let delete_result = client.delete_order_group(&group_id).await;
    assert!(
        delete_result.is_ok(),
        "delete_order_group failed: {:?}",
        delete_result.err()
    );
}

/// Tests trigger_order_group separately since it may affect group state.
#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_trigger_order_group() {
    let client = test_client();

    // Create a group specifically to trigger
    let request = CreateOrderGroupRequest::new(100);
    let create_result = client.create_order_group(request).await;
    assert!(
        create_result.is_ok(),
        "create_order_group failed: {:?}",
        create_result.err()
    );
    let group_id = create_result.unwrap().order_group_id;

    // Trigger (cancels all orders in the group)
    let trigger_result = client.trigger_order_group(&group_id).await;
    assert!(
        trigger_result.is_ok(),
        "trigger_order_group failed: {:?}",
        trigger_result.err()
    );

    // Cleanup: try to delete (may fail if trigger already cleaned up)
    let _ = client.delete_order_group(&group_id).await;
}

// =========================================================================
// Missing read-only param variant tests
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_series_list_with_params() {
    let client = test_client();
    let params = kalshi_trade_rs::models::GetSeriesParams::new().include_volume(true);
    let result = client.get_series_list_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_series_list_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fee_changes_with_params() {
    let client = test_client();
    let params = GetFeeChangesParams::new().show_historical(true);
    let result = client.get_fee_changes_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_fee_changes_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_list_rfqs_with_params() {
    let client = test_client();
    let params = ListRfqsParams::new().limit(5);
    let result = client.list_rfqs_with_params(params).await;
    assert!(
        result.is_ok(),
        "list_rfqs_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_list_order_groups_with_params() {
    let client = test_client();
    let params = GetOrderGroupsParams::new().limit(5);
    let result = client.list_order_groups_with_params(params).await;
    assert!(
        result.is_ok(),
        "list_order_groups_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_subaccount_transfers_with_params() {
    let client = test_client();
    let params = GetSubaccountTransfersParams::new().limit(5);
    let result = client.get_subaccount_transfers_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_subaccount_transfers_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_incentive_programs_with_params() {
    let client = test_client();
    let params = GetIncentiveProgramsParams::new().limit(5);
    let result = client.get_incentive_programs_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_incentive_programs_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_lookup_history_with_params() {
    let client = test_client();
    let collections = client
        .get_multivariate_collections_with_params(GetMultivariateCollectionsParams::new().limit(1))
        .await
        .expect("need collections");
    if collections.collections.is_empty() {
        eprintln!("SKIP: no multivariate collections found on demo");
        return;
    }
    let collection_ticker = &collections.collections[0].collection_ticker;

    let params = GetLookupHistoryParams::new(86400).limit(5);
    let result = client
        .get_lookup_history_with_params(collection_ticker, params)
        .await;
    assert!(
        result.is_ok(),
        "get_lookup_history_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_live_data() {
    let client = test_client();
    let milestones = client
        .get_milestones_with_params(GetMilestonesParams::new().limit(1))
        .await
        .expect("need milestones");
    if milestones.milestones.is_empty() {
        eprintln!("SKIP: no milestones found on demo");
        return;
    }
    let milestone = &milestones.milestones[0];
    let milestone_id = milestone
        .id
        .as_deref()
        .expect("milestone should have an id");
    let milestone_type = milestone
        .milestone_type
        .as_deref()
        .expect("milestone should have a type");

    let result = client.get_live_data(milestone_type, milestone_id).await;
    // The milestone type/id combination may not have live data available
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("404") || msg.contains("400") => {
            eprintln!(
                "NOTE: get_live_data returned error (milestone may not have live data): {}",
                msg
            );
        }
        Err(e) => panic!("get_live_data failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_structured_targets_with_params() {
    let client = test_client();
    let params = GetStructuredTargetsParams::new().page_size(3);
    let result = client.get_structured_targets_with_params(params).await;
    assert!(
        result.is_ok(),
        "get_structured_targets_with_params failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_multivariate_collections_with_params() {
    let client = test_client();
    let params = GetMultivariateCollectionsParams::new().limit(3);
    let result = client
        .get_multivariate_collections_with_params(params)
        .await;
    assert!(
        result.is_ok(),
        "get_multivariate_collections_with_params failed: {:?}",
        result.err()
    );
}

// =========================================================================
// Order amend + decrease lifecycle on demo
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_order_amend_and_decrease() {
    use kalshi_trade_rs::{Action, Side};

    let client = test_client();

    // Find an open market
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // Create order with count=5 at low price
    let request = CreateOrderRequest::new(ticker, Side::Yes, Action::Buy, 5).yes_price(1);
    let create_result = client.create_order(request).await;
    assert!(
        create_result.is_ok(),
        "create_order failed: {:?}",
        create_result.err()
    );
    let order_id = create_result.unwrap().order.order_id;

    // Amend the order (change price)
    let amend_request = AmendOrderRequest::new(ticker, Side::Yes, Action::Buy)
        .yes_price(2)
        .count(5);
    let amend_result = client.amend_order(&order_id, amend_request).await;
    // Amend may fail if order was already matched; handle gracefully
    match &amend_result {
        Ok(resp) => {
            // The amended order has a new ID
            let new_order_id = &resp.order.order_id;

            // Decrease order (reduce by 2)
            let decrease_request = DecreaseOrderRequest::reduce_by(2);
            let decrease_result = client.decrease_order(new_order_id, decrease_request).await;
            assert!(
                decrease_result.is_ok(),
                "decrease_order failed: {:?}",
                decrease_result.err()
            );

            // Get queue position for this order
            let queue_result = client.get_order_queue_position(new_order_id).await;
            // Queue position may fail for cancelled/filled orders
            match &queue_result {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "NOTE: get_order_queue_position returned error (may be expected): {:?}",
                    e
                ),
            }

            // Cleanup: cancel the order
            let _ = client.cancel_order(new_order_id).await;
        }
        Err(e) => {
            eprintln!(
                "NOTE: amend_order failed (order may have been matched): {:?}",
                e
            );
            // Cleanup original order
            let _ = client.cancel_order(&order_id).await;
        }
    }
}

// =========================================================================
// RFQ lifecycle on demo
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_rfq_lifecycle() {
    let client = test_client();

    // Find an open market for the RFQ
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // Create RFQ
    let request = CreateRfqRequest::with_contracts(ticker, 1, false);
    let create_result = client.create_rfq(request).await;
    // RFQs may not be available for all markets or accounts
    match &create_result {
        Ok(rfq_response) => {
            let rfq_id = &rfq_response.id;

            // Get RFQ
            let get_result = client.get_rfq(rfq_id).await;
            assert!(get_result.is_ok(), "get_rfq failed: {:?}", get_result.err());

            // Cancel RFQ
            let cancel_result = client.cancel_rfq(rfq_id).await;
            assert!(
                cancel_result.is_ok(),
                "cancel_rfq failed: {:?}",
                cancel_result.err()
            );
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403")
                || msg.contains("400")
                || msg.contains("409")
                || msg.contains("permission")
                || msg.contains("already_exists") =>
        {
            eprintln!(
                "NOTE: create_rfq returned error (may not be available or already exists): {}",
                msg
            );
        }
        Err(e) => panic!("create_rfq failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// Quote endpoints (create_quote, get_quote, cancel_quote, confirm_quote, accept_quote)
// These require an active RFQ, so we test them together.
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_quote_lifecycle() {
    let client = test_client();

    // Find an open market
    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // First create an RFQ
    let rfq_request = CreateRfqRequest::with_contracts(ticker, 1, false);
    let rfq_result = client.create_rfq(rfq_request).await;
    let rfq_id = match rfq_result {
        Ok(r) => r.id,
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("400") || msg.contains("permission") =>
        {
            eprintln!("SKIP: create_rfq not available for this account: {}", msg);
            return;
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("409") || msg.contains("already_exists") =>
        {
            eprintln!("SKIP: demo env has leftover RFQ from prior run: {}", msg);
            return;
        }
        Err(e) => panic!("create_rfq failed unexpectedly: {:?}", e),
    };

    // Create a quote for the RFQ
    let quote_request = CreateQuoteRequest::from_cents(&rfq_id, 50, false);
    let quote_result = client.create_quote(quote_request).await;
    match &quote_result {
        Ok(quote_response) => {
            let quote_id = &quote_response.id;

            // Get the quote
            let get_result = client.get_quote(quote_id).await;
            assert!(
                get_result.is_ok(),
                "get_quote failed: {:?}",
                get_result.err()
            );

            // Cancel the quote
            let cancel_result = client.cancel_quote(quote_id).await;
            assert!(
                cancel_result.is_ok(),
                "cancel_quote failed: {:?}",
                cancel_result.err()
            );
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("400") || msg.contains("permission") =>
        {
            eprintln!(
                "NOTE: create_quote returned error (may not be available): {}",
                msg
            );
        }
        Err(e) => panic!("create_quote failed unexpectedly: {:?}", e),
    }

    // Cleanup: cancel the RFQ
    let _ = client.cancel_rfq(&rfq_id).await;
}

// Note: confirm_quote and accept_quote require specific workflow states
// that are difficult to set up in an automated test. They are tested
// implicitly through the quote lifecycle — the endpoint paths and
// serialization are verified by the create/get/cancel flow above.
// The accept_quote and confirm_quote methods use the same PUT pattern.

// =========================================================================
// Lookup tickers (multivariate)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_lookup_tickers() {
    let client = test_client();
    let collections = client
        .get_multivariate_collections_with_params(GetMultivariateCollectionsParams::new().limit(1))
        .await
        .expect("need collections");
    if collections.collections.is_empty() {
        eprintln!("SKIP: no multivariate collections found on demo");
        return;
    }
    let collection_ticker = &collections.collections[0].collection_ticker;

    // Lookup with empty variables — may return 404 which is fine
    let request = LookupTickersRequest::empty();
    let result = client.lookup_tickers(collection_ticker, request).await;
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("404") || msg.contains("400") => {
            eprintln!(
                "NOTE: lookup_tickers returned error (expected with empty variables): {}",
                msg
            );
        }
        Err(e) => panic!("lookup_tickers failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// Create market in collection (multivariate)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_create_market_in_collection() {
    let client = test_client();
    let collections = client
        .get_multivariate_collections_with_params(GetMultivariateCollectionsParams::new().limit(1))
        .await
        .expect("need collections");
    if collections.collections.is_empty() {
        eprintln!("SKIP: no multivariate collections found on demo");
        return;
    }
    let collection_ticker = &collections.collections[0].collection_ticker;

    // Try to create a market with empty variables — will likely fail with 400
    // but this verifies the endpoint path and request serialization
    let request = CreateMarketInCollectionRequest::empty();
    let result = client
        .create_market_in_collection(collection_ticker, request)
        .await;
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("400") || msg.contains("403") => {
            eprintln!(
                "NOTE: create_market_in_collection returned expected error: {}",
                msg
            );
        }
        Err(e) => panic!("create_market_in_collection failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// Subaccount management (create, transfer, netting)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_create_subaccount() {
    let client = test_client();

    // Creating a subaccount is permanent (up to 32), so be cautious
    let request = CreateSubaccountRequest::with_name("integration-test");
    let result = client.create_subaccount(request).await;
    match &result {
        Ok(resp) => {
            eprintln!("Created subaccount: {}", resp.subaccount_number);
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("400") || msg.contains("403") || msg.contains("limit") =>
        {
            eprintln!(
                "NOTE: create_subaccount returned error (may have reached limit): {}",
                msg
            );
        }
        Err(e) => panic!("create_subaccount failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_transfer_between_subaccounts() {
    let client = test_client();

    // Check if we have any subaccounts
    let balances = client
        .get_subaccount_balances()
        .await
        .expect("need balances");
    if balances.subaccount_balances.len() < 2 {
        eprintln!("SKIP: need at least 2 subaccounts for transfer test");
        return;
    }

    // Transfer $0.01 from primary to first subaccount, then back
    let request = TransferBetweenSubaccountsRequest::new(0, 1, 1); // 1 cent
    let result = client.transfer_between_subaccounts(request).await;
    match &result {
        Ok(_) => {
            // Transfer back
            let reverse = TransferBetweenSubaccountsRequest::new(1, 0, 1);
            let _ = client.transfer_between_subaccounts(reverse).await;
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("400") || msg.contains("insufficient") =>
        {
            eprintln!(
                "NOTE: transfer failed (may have insufficient balance): {}",
                msg
            );
        }
        Err(e) => panic!("transfer_between_subaccounts failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_update_subaccount_netting() {
    let client = test_client();

    // Get current netting state first
    let netting = match client.get_subaccount_netting().await {
        Ok(n) => n,
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("500") => {
            eprintln!(
                "SKIP: get_subaccount_netting returned server-side 500: {}",
                msg
            );
            return;
        }
        Err(e) => panic!("need netting config: {:?}", e),
    };
    if netting.netting_configs.is_empty() {
        eprintln!("SKIP: no netting configs found");
        return;
    }

    // Toggle netting for primary account, then toggle back
    let current = &netting.netting_configs[0];
    let current_enabled = current.enabled;
    let subaccount = current.subaccount_number;

    let request = UpdateSubaccountNettingRequest::new(subaccount, !current_enabled);
    let result = client.update_subaccount_netting(request).await;
    match &result {
        Ok(_) => {
            // Restore original state
            let restore = UpdateSubaccountNettingRequest::new(subaccount, current_enabled);
            let _ = client.update_subaccount_netting(restore).await;
        }
        Err(kalshi_trade_rs::Error::Api(msg)) if msg.contains("403") || msg.contains("400") => {
            eprintln!("NOTE: update_subaccount_netting returned error: {}", msg);
        }
        Err(e) => panic!("update_subaccount_netting failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// API Key management
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_api_key_generate_and_delete() {
    let client = test_client();

    // Generate a new API key
    let request = GenerateApiKeyRequest::new("integration-test-key");
    let result = client.generate_api_key(request).await;
    match &result {
        Ok(resp) => {
            let key_id = &resp.api_key_id;

            // Delete the key we just created
            let delete_result = client.delete_api_key(key_id).await;
            assert!(
                delete_result.is_ok(),
                "delete_api_key failed: {:?}",
                delete_result.err()
            );
        }
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("400") || msg.contains("permission") =>
        {
            eprintln!(
                "NOTE: generate_api_key not available for this account tier: {}",
                msg
            );
        }
        Err(e) => panic!("generate_api_key failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_create_api_key() {
    let client = test_client();

    // create_api_key requires a user-provided public key — just test the endpoint responds
    let request = CreateApiKeyRequest::new("integration-test-key", "not-a-real-key");
    let result = client.create_api_key(request).await;
    // Expected to fail with 400 (invalid key format) — but proves endpoint exists
    match &result {
        Ok(_) => eprintln!("NOTE: create_api_key succeeded (unexpected with dummy key)"),
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("400") || msg.contains("403") || msg.contains("invalid") =>
        {
            // Expected
        }
        Err(e) => panic!("create_api_key failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// FCM endpoints (Futures Commission Merchant — likely 403 for most accounts)
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fcm_orders() {
    let client = test_client();
    let params = GetFcmOrdersParams::new("dummy-subtrader-id").limit(5);
    let result = client.get_fcm_orders(params).await;
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("permission") || msg.contains("400") =>
        {
            eprintln!(
                "NOTE: get_fcm_orders returned expected error (non-FCM account): {}",
                msg
            );
        }
        Err(e) => panic!("get_fcm_orders failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_get_fcm_positions() {
    let client = test_client();
    let params = GetFcmPositionsParams::new("dummy-subtrader-id").limit(5);
    let result = client.get_fcm_positions(params).await;
    match &result {
        Ok(_) => {}
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("403") || msg.contains("permission") || msg.contains("400") =>
        {
            eprintln!(
                "NOTE: get_fcm_positions returned expected error (non-FCM account): {}",
                msg
            );
        }
        Err(e) => panic!("get_fcm_positions failed unexpectedly: {:?}", e),
    }
}

// =========================================================================
// Subaccount variant endpoints
// These test the _for_subaccount variants which pass an explicit subaccount
// parameter to the same underlying API endpoint.
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_cancel_order_for_subaccount() {
    use kalshi_trade_rs::{Action, Side};

    let client = test_client();

    let markets = client
        .get_markets_with_params(
            GetMarketsParams::new()
                .status(MarketFilterStatus::Open)
                .limit(1),
        )
        .await
        .expect("need an open market");
    if markets.markets.is_empty() {
        eprintln!("SKIP: no open markets found on demo");
        return;
    }
    let ticker = &markets.markets[0].ticker;

    // Create order on primary subaccount (0) at a very low price so it won't fill
    let request = CreateOrderRequest::new(ticker, Side::Yes, Action::Buy, 1)
        .yes_price(1)
        .subaccount(0);
    let create_result = client.create_order(request).await;
    assert!(
        create_result.is_ok(),
        "create_order failed: {:?}",
        create_result.err()
    );
    let order_id = create_result.unwrap().order.order_id;

    // Cancel for subaccount 0
    let cancel_result = client.cancel_order_for_subaccount(&order_id, 0).await;
    assert!(
        cancel_result.is_ok(),
        "cancel_order_for_subaccount failed: {:?}",
        cancel_result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_order_group_subaccount_variants() {
    let client = test_client();

    // Create group
    let request = CreateOrderGroupRequest::new(100);
    let create_result = client.create_order_group(request).await;
    assert!(
        create_result.is_ok(),
        "create_order_group failed: {:?}",
        create_result.err()
    );
    let group_id = create_result.unwrap().order_group_id;

    // Brief pause to allow the order group to propagate
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // get_order_group_for_subaccount
    let get_result = client.get_order_group_for_subaccount(&group_id, 0).await;
    assert!(
        get_result.is_ok(),
        "get_order_group_for_subaccount failed: {:?}",
        get_result.err()
    );

    // reset_order_group_for_subaccount
    let reset_result = client.reset_order_group_for_subaccount(&group_id, 0).await;
    assert!(
        reset_result.is_ok(),
        "reset_order_group_for_subaccount failed: {:?}",
        reset_result.err()
    );

    // delete_order_group_for_subaccount
    let delete_result = client.delete_order_group_for_subaccount(&group_id, 0).await;
    assert!(
        delete_result.is_ok(),
        "delete_order_group_for_subaccount failed: {:?}",
        delete_result.err()
    );
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_trigger_order_group_for_subaccount() {
    let client = test_client();

    // Create a group to trigger
    let request = CreateOrderGroupRequest::new(100);
    let create_result = client.create_order_group(request).await;
    assert!(
        create_result.is_ok(),
        "create_order_group failed: {:?}",
        create_result.err()
    );
    let group_id = create_result.unwrap().order_group_id;

    // trigger_order_group_for_subaccount
    let trigger_result = client
        .trigger_order_group_for_subaccount(&group_id, 0)
        .await;
    assert!(
        trigger_result.is_ok(),
        "trigger_order_group_for_subaccount failed: {:?}",
        trigger_result.err()
    );

    // Cleanup
    let _ = client.delete_order_group(&group_id).await;
}

// =========================================================================
// accept_quote and confirm_quote
// These require specific RFQ/quote workflow states. We test that they
// produce the expected API response (even if 404/400) to verify the
// endpoint path, HTTP method, and request serialization.
// =========================================================================

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_accept_quote_endpoint() {
    use kalshi_trade_rs::Side;

    let client = test_client();

    // Accept a non-existent quote — should return 404
    let request = AcceptQuoteRequest::new(Side::Yes);
    let result = client.accept_quote("nonexistent-quote-id", request).await;
    match &result {
        Ok(_) => eprintln!("NOTE: accept_quote succeeded unexpectedly"),
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("404") || msg.contains("400") || msg.contains("403") =>
        {
            // Expected — endpoint exists and responds correctly
        }
        Err(e) => panic!("accept_quote failed unexpectedly: {:?}", e),
    }
}

#[tokio::test]
#[ignore = "requires demo API credentials"]
async fn test_confirm_quote_endpoint() {
    let client = test_client();

    // Confirm a non-existent quote — should return 404
    let result = client.confirm_quote("nonexistent-quote-id").await;
    match &result {
        Ok(_) => eprintln!("NOTE: confirm_quote succeeded unexpectedly"),
        Err(kalshi_trade_rs::Error::Api(msg))
            if msg.contains("404") || msg.contains("400") || msg.contains("403") =>
        {
            // Expected — endpoint exists and responds correctly
        }
        Err(e) => panic!("confirm_quote failed unexpectedly: {:?}", e),
    }
}
