//! WebSocket protocol message serialization and deserialization.
//!
//! This module provides functions for building and parsing Kalshi WebSocket messages.

use super::{
    Channel,
    command::{CommunicationsSharding, UpdateAction},
};
use serde::Deserialize;
use serde_json::Value as JsonValue;

/// Build a subscribe command message.
///
/// # Arguments
/// * `id` - Message ID for correlation
/// * `channels` - List of channels to subscribe to
/// * `market_tickers` - Market ticker(s) to subscribe to
/// * `market_ids` - Market ID(s) (UUIDs) to subscribe to
/// * `sharding` - Optional sharding config for communications channel
/// * `skip_ticker_ack` - When true, skip market tickers in OK acknowledgements
/// * `send_initial_snapshot` - When true, send initial snapshot on subscribe
///
/// # Returns
/// JSON string ready to send over WebSocket
///
/// # Note
/// The Kalshi API requires at least one market ticker or market ID for most channels.
/// Omitting both will likely result in an error response.
pub fn build_subscribe(
    id: u64,
    channels: &[Channel],
    market_tickers: &[&str],
    market_ids: &[&str],
    sharding: Option<&CommunicationsSharding>,
    skip_ticker_ack: Option<bool>,
    send_initial_snapshot: Option<bool>,
) -> String {
    let channel_strings: Vec<&str> = channels.iter().map(|c| c.as_str()).collect();

    let mut params = serde_json::json!({
        "channels": channel_strings
    });

    // Add market tickers if provided
    if !market_tickers.is_empty() {
        if market_tickers.len() == 1 {
            params["market_ticker"] = serde_json::json!(market_tickers[0]);
        } else {
            params["market_tickers"] = serde_json::json!(market_tickers);
        }
    }

    // Add market IDs if provided
    if !market_ids.is_empty() {
        if market_ids.len() == 1 {
            params["market_id"] = serde_json::json!(market_ids[0]);
        } else {
            params["market_ids"] = serde_json::json!(market_ids);
        }
    }

    // Add sharding parameters if provided (for communications channel)
    if let Some(sharding) = sharding {
        if let Some(shard_factor) = sharding.shard_factor {
            params["shard_factor"] = serde_json::json!(shard_factor);
        }
        if let Some(shard_key) = sharding.shard_key {
            params["shard_key"] = serde_json::json!(shard_key);
        }
    }

    // Add skip_ticker_ack flag if set
    if skip_ticker_ack == Some(true) {
        params["skip_ticker_ack"] = serde_json::json!(true);
    }

    // Add send_initial_snapshot flag if set
    if send_initial_snapshot == Some(true) {
        params["send_initial_snapshot"] = serde_json::json!(true);
    }

    serde_json::json!({
        "id": id,
        "cmd": "subscribe",
        "params": params
    })
    .to_string()
}

/// Build a list_subscriptions command message.
///
/// # Arguments
/// * `id` - Message ID for correlation
///
/// # Returns
/// JSON string ready to send over WebSocket
pub fn build_list_subscriptions(id: u64) -> String {
    serde_json::json!({
        "id": id,
        "cmd": "list_subscriptions"
    })
    .to_string()
}

/// Build an unsubscribe command message.
///
/// # Arguments
/// * `id` - Message ID for correlation
/// * `sids` - Subscription IDs to unsubscribe from
///
/// # Returns
/// JSON string ready to send over WebSocket
pub fn build_unsubscribe(id: u64, sids: &[i64]) -> String {
    serde_json::json!({
        "id": id,
        "cmd": "unsubscribe",
        "params": {
            "sids": sids
        }
    })
    .to_string()
}

/// Build an update_subscription command message.
///
/// # Arguments
/// * `id` - Message ID for correlation
/// * `sid` - Subscription ID to update
/// * `market_tickers` - Market tickers to add or remove
/// * `market_ids` - Market IDs (UUIDs) to add or remove
/// * `action` - Whether to add or delete markets
/// * `send_initial_snapshot` - When true, send initial snapshot for added markets
///
/// # Returns
/// JSON string ready to send over WebSocket
pub fn build_update_subscription(
    id: u64,
    sid: i64,
    market_tickers: &[&str],
    market_ids: &[&str],
    action: UpdateAction,
    send_initial_snapshot: Option<bool>,
) -> String {
    let action_str = match action {
        UpdateAction::AddMarkets => "add_markets",
        UpdateAction::DeleteMarkets => "delete_markets",
    };

    let mut params = serde_json::json!({
        "sids": [sid],
        "action": action_str
    });

    // Add market tickers if provided
    if !market_tickers.is_empty() {
        if market_tickers.len() == 1 {
            params["market_ticker"] = serde_json::json!(market_tickers[0]);
        } else {
            params["market_tickers"] = serde_json::json!(market_tickers);
        }
    }

    // Add market IDs if provided
    if !market_ids.is_empty() {
        if market_ids.len() == 1 {
            params["market_id"] = serde_json::json!(market_ids[0]);
        } else {
            params["market_ids"] = serde_json::json!(market_ids);
        }
    }

    // Add send_initial_snapshot flag if set
    if send_initial_snapshot == Some(true) {
        params["send_initial_snapshot"] = serde_json::json!(true);
    }

    serde_json::json!({
        "id": id,
        "cmd": "update_subscription",
        "params": params
    })
    .to_string()
}

/// Incoming WebSocket message types.
#[derive(Debug, Clone, PartialEq)]
pub enum IncomingMessage {
    /// Response to a command (subscribe, unsubscribe, etc.)
    Response {
        id: u64,
        msg_type: String,
        msg: JsonValue,
    },
    /// Subscription update (orderbook, trade, etc.)
    Update {
        msg_type: String,
        sid: i64,
        seq: Option<i64>,
        msg: JsonValue,
    },
    /// Error response
    Error {
        id: Option<u64>,
        code: String,
        message: String,
        market_ticker: Option<String>,
        market_id: Option<String>,
    },
}

/// Internal structure for parsing incoming messages
#[derive(Debug, Deserialize)]
struct RawMessage {
    id: Option<u64>,
    #[serde(rename = "type")]
    msg_type: Option<String>,
    sid: Option<i64>,
    seq: Option<i64>,
    msg: Option<JsonValue>,
    // Error fields
    code: Option<String>,
    message: Option<String>,
    error: Option<ErrorPayload>,
}

#[derive(Debug, Deserialize)]
struct ErrorPayload {
    code: Option<String>,
    message: Option<String>,
}

/// Parse an incoming WebSocket message.
///
/// # Arguments
/// * `text` - Raw JSON text received from WebSocket
///
/// # Returns
/// * Parsed `IncomingMessage` or a JSON parsing error
pub fn parse_incoming(text: &str) -> Result<IncomingMessage, serde_json::Error> {
    let raw: RawMessage = serde_json::from_str(text)?;

    // Check for spec-format error: {"type": "error", "msg": {"code": int, "msg": string, ...}}
    if raw.msg_type.as_deref() == Some("error")
        && let Some(ref msg_val) = raw.msg
        && let Some(msg_obj) = msg_val.as_object()
        && msg_obj.contains_key("code")
    {
        let code = msg_obj
            .get("code")
            .and_then(|v| v.as_u64())
            .map(|n| n.to_string())
            .or_else(|| {
                msg_obj
                    .get("code")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_default();
        let message = msg_obj
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let market_ticker = msg_obj
            .get("market_ticker")
            .and_then(|v| v.as_str())
            .map(String::from);
        let market_id = msg_obj
            .get("market_id")
            .and_then(|v| v.as_str())
            .map(String::from);
        return Ok(IncomingMessage::Error {
            id: raw.id,
            code,
            message,
            market_ticker,
            market_id,
        });
    }

    // Check for error response with nested error object
    if let Some(error) = raw.error {
        return Ok(IncomingMessage::Error {
            id: raw.id,
            code: error.code.unwrap_or_default(),
            message: error.message.unwrap_or_default(),
            market_ticker: None,
            market_id: None,
        });
    }

    // Check for top-level error fields
    if (raw.code.is_some() || raw.message.is_some())
        && (raw.msg_type.as_deref() == Some("error") || raw.code.is_some())
    {
        return Ok(IncomingMessage::Error {
            id: raw.id,
            code: raw.code.unwrap_or_default(),
            message: raw.message.unwrap_or_default(),
            market_ticker: None,
            market_id: None,
        });
    }

    // Check if type indicates a known response type (id is optional per spec)
    let is_response_type = matches!(
        raw.msg_type.as_deref(),
        Some("subscribed") | Some("unsubscribed") | Some("ok") | Some("list_subscriptions")
    );

    // Route as Response if id is present OR type is a known response type
    if is_response_type || raw.id.is_some() {
        let id = raw.id.unwrap_or(0);
        // If it's a response but has sid at top level (like unsubscribed),
        // ensure sid is preserved in msg if msg is otherwise null
        let msg = raw.msg.unwrap_or_else(|| {
            if let Some(sid) = raw.sid {
                serde_json::json!({ "sid": sid })
            } else {
                JsonValue::Null
            }
        });

        return Ok(IncomingMessage::Response {
            id,
            msg_type: raw.msg_type.unwrap_or_default(),
            msg,
        });
    }

    // Check if it's an update (has sid but no id)
    if let Some(sid) = raw.sid {
        return Ok(IncomingMessage::Update {
            msg_type: raw.msg_type.unwrap_or_default(),
            sid,
            seq: raw.seq,
            msg: raw.msg.unwrap_or(JsonValue::Null),
        });
    }

    // Fallback: treat as response with id 0
    Ok(IncomingMessage::Response {
        id: 0,
        msg_type: raw.msg_type.unwrap_or_default(),
        msg: raw.msg.unwrap_or(JsonValue::Null),
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_build_subscribe_single_ticker() {
        let result = build_subscribe(
            1,
            &[Channel::OrderbookDelta],
            &["AAPL-YES"],
            &[],
            None,
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(
            parsed["params"]["channels"],
            serde_json::json!(["orderbook_delta"])
        );
        assert_eq!(parsed["params"]["market_ticker"], "AAPL-YES");
        assert!(parsed["params"].get("market_tickers").is_none());
    }

    #[test]
    fn test_build_subscribe_multiple_tickers() {
        let result = build_subscribe(
            2,
            &[Channel::OrderbookDelta, Channel::Ticker],
            &["AAPL-YES", "GOOG-NO"],
            &[],
            None,
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 2);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(
            parsed["params"]["channels"],
            serde_json::json!(["orderbook_delta", "ticker"])
        );
        assert_eq!(
            parsed["params"]["market_tickers"],
            serde_json::json!(["AAPL-YES", "GOOG-NO"])
        );
        assert!(parsed["params"].get("market_ticker").is_none());
    }

    #[test]
    fn test_build_subscribe_no_tickers() {
        // Subscribe to all markets by omitting market_ticker(s)
        let result = build_subscribe(3, &[Channel::Ticker], &[], &[], None, None, None);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 3);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(parsed["params"]["channels"], serde_json::json!(["ticker"]));
        // Neither market_ticker nor market_tickers should be present
        assert!(parsed["params"].get("market_ticker").is_none());
        assert!(parsed["params"].get("market_tickers").is_none());
    }

    #[test]
    fn test_build_subscribe_all_channels() {
        let result = build_subscribe(
            3,
            &[
                Channel::OrderbookDelta,
                Channel::Ticker,
                Channel::Trade,
                Channel::Fill,
            ],
            &["TEST"],
            &[],
            None,
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["params"]["channels"],
            serde_json::json!(["orderbook_delta", "ticker", "trade", "fill"])
        );
    }

    #[test]
    fn test_build_subscribe_with_sharding() {
        let sharding = CommunicationsSharding::new(4, 2);
        let result = build_subscribe(
            5,
            &[Channel::Communications],
            &[],
            &[],
            Some(&sharding),
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 5);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(
            parsed["params"]["channels"],
            serde_json::json!(["communications"])
        );
        assert_eq!(parsed["params"]["shard_factor"], 4);
        assert_eq!(parsed["params"]["shard_key"], 2);
    }

    #[test]
    fn test_build_subscribe_with_skip_ticker_ack() {
        let result = build_subscribe(
            6,
            &[Channel::Ticker],
            &["MARKET-A"],
            &[],
            None,
            Some(true),
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 6);
        assert_eq!(parsed["cmd"], "subscribe");
        assert_eq!(parsed["params"]["channels"], serde_json::json!(["ticker"]));
        assert_eq!(parsed["params"]["skip_ticker_ack"], true);
    }

    #[test]
    fn test_build_subscribe_without_skip_ticker_ack() {
        let result = build_subscribe(7, &[Channel::Ticker], &["MARKET-A"], &[], None, None, None);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["params"].get("skip_ticker_ack").is_none());
    }

    #[test]
    fn test_build_subscribe_skip_ticker_ack_false() {
        let result = build_subscribe(
            8,
            &[Channel::Ticker],
            &["MARKET-A"],
            &[],
            None,
            Some(false),
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // skip_ticker_ack=false should not add the field
        assert!(parsed["params"].get("skip_ticker_ack").is_none());
    }

    #[test]
    fn test_build_subscribe_with_send_initial_snapshot() {
        let result = build_subscribe(
            9,
            &[Channel::Ticker],
            &["MARKET-A"],
            &[],
            None,
            None,
            Some(true),
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 9);
        assert_eq!(parsed["params"]["send_initial_snapshot"], true);
    }

    #[test]
    fn test_build_subscribe_with_market_id() {
        let result = build_subscribe(
            10,
            &[Channel::OrderbookDelta],
            &[],
            &["abc-123-uuid"],
            None,
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 10);
        assert_eq!(parsed["params"]["market_id"], "abc-123-uuid");
        assert!(parsed["params"].get("market_ids").is_none());
        assert!(parsed["params"].get("market_ticker").is_none());
    }

    #[test]
    fn test_build_subscribe_with_market_ids() {
        let result = build_subscribe(
            11,
            &[Channel::OrderbookDelta],
            &[],
            &["uuid-1", "uuid-2"],
            None,
            None,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 11);
        assert_eq!(
            parsed["params"]["market_ids"],
            serde_json::json!(["uuid-1", "uuid-2"])
        );
        assert!(parsed["params"].get("market_id").is_none());
    }

    #[test]
    fn test_build_unsubscribe_single_sid() {
        let result = build_unsubscribe(5, &[123]);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 5);
        assert_eq!(parsed["cmd"], "unsubscribe");
        assert_eq!(parsed["params"]["sids"], serde_json::json!([123]));
    }

    #[test]
    fn test_build_unsubscribe_multiple_sids() {
        let result = build_unsubscribe(6, &[100, 200, 300]);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 6);
        assert_eq!(parsed["cmd"], "unsubscribe");
        assert_eq!(parsed["params"]["sids"], serde_json::json!([100, 200, 300]));
    }

    #[test]
    fn test_parse_response() {
        let json = r#"{"id": 1, "type": "subscribed", "msg": {"sid": 42}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 1);
                assert_eq!(msg_type, "subscribed");
                assert_eq!(msg["sid"], 42);
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[test]
    fn test_parse_update() {
        let json = r#"{"type": "orderbook_delta", "sid": 42, "msg": {"yes": [[50, 100]]}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Update { msg_type, sid, msg, .. } => {
                assert_eq!(msg_type, "orderbook_delta");
                assert_eq!(sid, 42);
                assert_eq!(msg["yes"], serde_json::json!([[50, 100]]));
            }
            _ => panic!("Expected Update variant"),
        }
    }

    #[test]
    fn test_parse_error_with_nested_error() {
        let json =
            r#"{"id": 1, "error": {"code": "invalid_params", "message": "Invalid market ticker"}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error {
                id,
                code,
                message,
                market_ticker,
                market_id,
            } => {
                assert_eq!(id, Some(1));
                assert_eq!(code, "invalid_params");
                assert_eq!(message, "Invalid market ticker");
                assert_eq!(market_ticker, None);
                assert_eq!(market_id, None);
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_with_top_level_fields() {
        let json = r#"{"id": 2, "type": "error", "code": "auth_failed", "message": "Authentication required"}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error {
                id,
                code,
                message,
                market_ticker,
                market_id,
            } => {
                assert_eq!(id, Some(2));
                assert_eq!(code, "auth_failed");
                assert_eq!(message, "Authentication required");
                assert_eq!(market_ticker, None);
                assert_eq!(market_id, None);
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_without_id() {
        let json = r#"{"code": "connection_error", "message": "Connection lost"}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error {
                id,
                code,
                message,
                market_ticker,
                market_id,
            } => {
                assert_eq!(id, None);
                assert_eq!(code, "connection_error");
                assert_eq!(message, "Connection lost");
                assert_eq!(market_ticker, None);
                assert_eq!(market_id, None);
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_spec_format() {
        let json = r#"{"id": 127, "type": "error", "msg": {"code": 16, "msg": "Market not found", "market_ticker": "INVALID-MARKET"}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error {
                id,
                code,
                message,
                market_ticker,
                market_id,
            } => {
                assert_eq!(id, Some(127));
                assert_eq!(code, "16");
                assert_eq!(message, "Market not found");
                assert_eq!(market_ticker, Some("INVALID-MARKET".to_string()));
                assert_eq!(market_id, None);
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_spec_format_with_market_id() {
        let json = r#"{"type": "error", "msg": {"code": 5, "msg": "Access denied", "market_id": "abc-123"}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error {
                id,
                code,
                message,
                market_ticker,
                market_id,
            } => {
                assert_eq!(id, None);
                assert_eq!(code, "5");
                assert_eq!(message, "Access denied");
                assert_eq!(market_ticker, None);
                assert_eq!(market_id, Some("abc-123".to_string()));
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_build_list_subscriptions() {
        let result = build_list_subscriptions(3);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 3);
        assert_eq!(parsed["cmd"], "list_subscriptions");
        assert!(parsed.get("params").is_none());
    }

    #[test]
    fn test_parse_response_with_null_msg() {
        let json = r#"{"id": 10, "type": "ack"}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 10);
                assert_eq!(msg_type, "ack");
                assert_eq!(msg, JsonValue::Null);
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "not valid json";
        let result = parse_incoming(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ticker_update() {
        let json = r#"{"type": "ticker", "sid": 99, "msg": {"price": 65, "volume": 1000}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Update { msg_type, sid, msg, .. } => {
                assert_eq!(msg_type, "ticker");
                assert_eq!(sid, 99);
                assert_eq!(msg["price"], 65);
                assert_eq!(msg["volume"], 1000);
            }
            _ => panic!("Expected Update variant"),
        }
    }

    #[test]
    fn test_parse_trade_update() {
        let json = r#"{"type": "trade", "sid": 55, "msg": {"count": 5, "price": 42, "taker_side": "yes"}}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Update { msg_type, sid, msg, .. } => {
                assert_eq!(msg_type, "trade");
                assert_eq!(sid, 55);
                assert_eq!(msg["count"], 5);
                assert_eq!(msg["price"], 42);
                assert_eq!(msg["taker_side"], "yes");
            }
            _ => panic!("Expected Update variant"),
        }
    }

    #[test]
    fn test_parse_unsubscribed_message() {
        // Log example: {"type":"unsubscribed","id":2,"sid":2,"seq":707}
        let json = r#"{"type":"unsubscribed","id":2,"sid":2,"seq":707}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 2);
                assert_eq!(msg_type, "unsubscribed");
                // The parser should synthesize a msg object with sid if msg is null
                assert_eq!(msg["sid"], 2);
            }
            _ => panic!("Expected Response variant"),
        }
    }

    #[test]
    fn test_build_update_subscription_add_markets() {
        let result = build_update_subscription(
            7,
            123,
            &["MARKET-A", "MARKET-B"],
            &[],
            UpdateAction::AddMarkets,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 7);
        assert_eq!(parsed["cmd"], "update_subscription");
        assert_eq!(parsed["params"]["sids"], serde_json::json!([123]));
        assert_eq!(
            parsed["params"]["market_tickers"],
            serde_json::json!(["MARKET-A", "MARKET-B"])
        );
        assert_eq!(parsed["params"]["action"], "add_markets");
    }

    #[test]
    fn test_build_update_subscription_delete_markets() {
        let result = build_update_subscription(
            8,
            456,
            &["MARKET-X"],
            &[],
            UpdateAction::DeleteMarkets,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 8);
        assert_eq!(parsed["cmd"], "update_subscription");
        assert_eq!(parsed["params"]["sids"], serde_json::json!([456]));
        assert_eq!(parsed["params"]["market_ticker"], "MARKET-X");
        assert_eq!(parsed["params"]["action"], "delete_markets");
    }

    #[test]
    fn test_build_update_subscription_with_market_ids() {
        let result = build_update_subscription(
            9,
            789,
            &[],
            &["uuid-abc", "uuid-def"],
            UpdateAction::AddMarkets,
            None,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 9);
        assert_eq!(parsed["cmd"], "update_subscription");
        assert_eq!(parsed["params"]["sids"], serde_json::json!([789]));
        assert_eq!(
            parsed["params"]["market_ids"],
            serde_json::json!(["uuid-abc", "uuid-def"])
        );
        assert!(parsed["params"].get("market_tickers").is_none());
        assert_eq!(parsed["params"]["action"], "add_markets");
    }

    #[test]
    fn test_build_update_subscription_with_send_initial_snapshot() {
        let result = build_update_subscription(
            10,
            100,
            &["MARKET-A"],
            &[],
            UpdateAction::AddMarkets,
            Some(true),
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["id"], 10);
        assert_eq!(parsed["params"]["send_initial_snapshot"], true);
        assert_eq!(parsed["params"]["action"], "add_markets");
    }

    #[test]
    fn test_parse_subscribed_without_id() {
        let json = r#"{"type":"subscribed","sid":5}"#;
        let result = parse_incoming(json).unwrap();
        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 0);
                assert_eq!(msg_type, "subscribed");
                assert_eq!(msg["sid"], 5);
            }
            _ => panic!("Expected Response variant, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_unsubscribed_without_id() {
        let json = r#"{"type":"unsubscribed","sid":3,"seq":100}"#;
        let result = parse_incoming(json).unwrap();
        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 0);
                assert_eq!(msg_type, "unsubscribed");
                assert_eq!(msg["sid"], 3);
            }
            _ => panic!("Expected Response variant, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_ok_response_without_id() {
        let json = r#"{"type":"ok"}"#;
        let result = parse_incoming(json).unwrap();
        match result {
            IncomingMessage::Response { id, msg_type, .. } => {
                assert_eq!(id, 0);
                assert_eq!(msg_type, "ok");
            }
            _ => panic!("Expected Response variant, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_list_subscriptions_without_id() {
        let json = r#"{"type":"list_subscriptions","msg":{"subs":[]}}"#;
        let result = parse_incoming(json).unwrap();
        match result {
            IncomingMessage::Response { id, msg_type, msg } => {
                assert_eq!(id, 0);
                assert_eq!(msg_type, "list_subscriptions");
                assert!(msg["subs"].is_array());
            }
            _ => panic!("Expected Response variant, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_update_with_sid_not_confused_as_response() {
        let json = r#"{"type":"ticker","sid":10,"msg":{"price":50}}"#;
        let result = parse_incoming(json).unwrap();
        match result {
            IncomingMessage::Update { msg_type, sid, msg } => {
                assert_eq!(msg_type, "ticker");
                assert_eq!(sid, 10);
                assert_eq!(msg["price"], 50);
            }
            _ => panic!("Expected Update variant, got {:?}", result),
        }
    }
}
