//! WebSocket protocol message serialization and deserialization.
//!
//! This module provides functions for building and parsing Kalshi WebSocket messages.

use serde::Deserialize;
use serde_json::Value as JsonValue;

use super::Channel;

/// Build a subscribe command message.
///
/// # Arguments
/// * `id` - Message ID for correlation
/// * `channels` - List of channels to subscribe to
/// * `market_tickers` - Market ticker(s) to subscribe to
///
/// # Returns
/// JSON string ready to send over WebSocket
pub fn build_subscribe(id: u64, channels: &[Channel], market_tickers: &[&str]) -> String {
    let channel_strings: Vec<&str> = channels.iter().map(|c| c.as_str()).collect();

    let params = if market_tickers.len() == 1 {
        serde_json::json!({
            "channels": channel_strings,
            "market_ticker": market_tickers[0]
        })
    } else {
        serde_json::json!({
            "channels": channel_strings,
            "market_tickers": market_tickers
        })
    };

    serde_json::json!({
        "id": id,
        "cmd": "subscribe",
        "params": params
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
        msg: JsonValue,
    },
    /// Error response
    Error {
        id: Option<u64>,
        code: String,
        message: String,
    },
}

/// Internal structure for parsing incoming messages
#[derive(Debug, Deserialize)]
struct RawMessage {
    id: Option<u64>,
    #[serde(rename = "type")]
    msg_type: Option<String>,
    sid: Option<i64>,
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
/// Parsed `IncomingMessage` or a JSON parsing error
pub fn parse_incoming(text: &str) -> Result<IncomingMessage, serde_json::Error> {
    let raw: RawMessage = serde_json::from_str(text)?;

    // Check for error response
    if let Some(error) = raw.error {
        return Ok(IncomingMessage::Error {
            id: raw.id,
            code: error.code.unwrap_or_default(),
            message: error.message.unwrap_or_default(),
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
        });
    }

    // Check if it's an update (has sid but no id)
    if let Some(sid) = raw.sid {
        return Ok(IncomingMessage::Update {
            msg_type: raw.msg_type.unwrap_or_default(),
            sid,
            msg: raw.msg.unwrap_or(JsonValue::Null),
        });
    }

    // Otherwise it's a response (has id)
    if let Some(id) = raw.id {
        return Ok(IncomingMessage::Response {
            id,
            msg_type: raw.msg_type.unwrap_or_default(),
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
        let result = build_subscribe(1, &[Channel::OrderbookDelta], &["AAPL-YES"]);
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
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["params"]["channels"],
            serde_json::json!(["orderbook_delta", "ticker", "trade", "fill"])
        );
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
            IncomingMessage::Update { msg_type, sid, msg } => {
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
            IncomingMessage::Error { id, code, message } => {
                assert_eq!(id, Some(1));
                assert_eq!(code, "invalid_params");
                assert_eq!(message, "Invalid market ticker");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_with_top_level_fields() {
        let json = r#"{"id": 2, "type": "error", "code": "auth_failed", "message": "Authentication required"}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error { id, code, message } => {
                assert_eq!(id, Some(2));
                assert_eq!(code, "auth_failed");
                assert_eq!(message, "Authentication required");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_parse_error_without_id() {
        let json = r#"{"code": "connection_error", "message": "Connection lost"}"#;
        let result = parse_incoming(json).unwrap();

        match result {
            IncomingMessage::Error { id, code, message } => {
                assert_eq!(id, None);
                assert_eq!(code, "connection_error");
                assert_eq!(message, "Connection lost");
            }
            _ => panic!("Expected Error variant"),
        }
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
            IncomingMessage::Update { msg_type, sid, msg } => {
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
            IncomingMessage::Update { msg_type, sid, msg } => {
                assert_eq!(msg_type, "trade");
                assert_eq!(sid, 55);
                assert_eq!(msg["count"], 5);
                assert_eq!(msg["price"], 42);
                assert_eq!(msg["taker_side"], "yes");
            }
            _ => panic!("Expected Update variant"),
        }
    }
}
