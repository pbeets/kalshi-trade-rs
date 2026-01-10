//! WebSocket stream actor for the Kalshi API.
//!
//! This module implements the actor pattern for managing the WebSocket connection
//! to Kalshi's streaming API. The actor owns the WebSocket connection and handles
//! all communication in a single async task.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde_json::Value as JsonValue;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
    time::{interval_at, sleep, timeout},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{self, Message, client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error, info, warn};

use super::{
    BACKOFF_BASE, CONNECT_TIMEOUT, ConnectStrategy, HealthConfig, MAX_BACKOFF,
    channel::Channel,
    command::{StreamCommand, SubscribeResult},
    message::{StreamMessage, StreamUpdate},
    protocol::{self, IncomingMessage},
    request_handler::RequestHandler,
};
use crate::{
    auth::KalshiConfig,
    error::{Error, Result},
};

/// WebSocket stream type alias for clarity.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The WebSocket stream actor that manages the connection lifecycle.
///
/// The actor owns the WebSocket connection (split into reader and writer),
/// processes commands from clients, and broadcasts updates to subscribers.
pub struct StreamActor {
    /// Configuration for authentication.
    #[allow(dead_code)]
    config: KalshiConfig,
    /// Receiver for commands from client handles.
    cmd_receiver: mpsc::Receiver<StreamCommand>,
    /// Sender for broadcasting updates to subscribers.
    update_sender: broadcast::Sender<StreamUpdate>,
    /// WebSocket reader half.
    ws_reader: SplitStream<WsStream>,
    /// WebSocket writer half.
    ws_writer: SplitSink<WsStream, Message>,
    /// Handler for mapping request IDs to response channels.
    request_handler: RequestHandler,
    /// Next request ID to use for outgoing messages.
    next_request_id: u64,
    /// Health monitoring configuration.
    health_config: HealthConfig,
    /// Last time we received a pong response.
    last_pong: Instant,
    /// Whether we're waiting for a pong response.
    ping_pending: bool,
}

impl StreamActor {
    /// Connect to the Kalshi WebSocket API with the specified strategy.
    ///
    /// # Arguments
    ///
    /// * `config` - The Kalshi configuration with API credentials.
    /// * `strategy` - Connection strategy (Simple or Retry).
    /// * `cmd_receiver` - Receiver for commands from client handles.
    /// * `update_sender` - Sender for broadcasting updates to subscribers.
    pub async fn connect(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
        cmd_receiver: mpsc::Receiver<StreamCommand>,
        update_sender: broadcast::Sender<StreamUpdate>,
    ) -> Result<Self> {
        Self::connect_with_health(
            config,
            strategy,
            HealthConfig::default(),
            cmd_receiver,
            update_sender,
        )
        .await
    }

    /// Connect with custom health monitoring configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Kalshi configuration with API credentials.
    /// * `strategy` - Connection strategy (Simple or Retry).
    /// * `health_config` - Health monitoring configuration.
    /// * `cmd_receiver` - Receiver for commands from client handles.
    /// * `update_sender` - Sender for broadcasting updates to subscribers.
    pub async fn connect_with_health(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
        health_config: HealthConfig,
        cmd_receiver: mpsc::Receiver<StreamCommand>,
        update_sender: broadcast::Sender<StreamUpdate>,
    ) -> Result<Self> {
        let ws_url = config.environment.ws_url();
        let ws_stream = Self::connect_with_strategy(config, ws_url, strategy).await?;

        let (ws_writer, ws_reader) = ws_stream.split();

        Ok(Self {
            config: config.clone(),
            cmd_receiver,
            update_sender,
            ws_reader,
            ws_writer,
            request_handler: RequestHandler::new(),
            next_request_id: 1,
            health_config,
            last_pong: Instant::now(),
            ping_pending: false,
        })
    }

    /// Connect using the specified strategy.
    async fn connect_with_strategy(
        config: &KalshiConfig,
        ws_url: &str,
        strategy: ConnectStrategy,
    ) -> Result<WsStream> {
        match strategy {
            ConnectStrategy::Simple => Self::connect_with_auth(config, ws_url).await,
            ConnectStrategy::Retry => Self::connect_with_retry(config, ws_url).await,
        }
    }

    /// Connect with exponential backoff retry.
    async fn connect_with_retry(config: &KalshiConfig, ws_url: &str) -> Result<WsStream> {
        let mut attempt: u64 = 1;

        loop {
            info!("Connection attempt {} to {}", attempt, ws_url);

            match timeout(CONNECT_TIMEOUT, Self::connect_with_auth(config, ws_url)).await {
                Ok(Ok(ws_stream)) => return Ok(ws_stream),
                Ok(Err(e)) => warn!("Connection failed: {}", e),
                Err(_) => warn!("Connection timed out after {:?}", CONNECT_TIMEOUT),
            }

            let backoff = (BACKOFF_BASE * attempt as u32).min(MAX_BACKOFF);
            info!("Retrying in {:?}", backoff);
            sleep(backoff).await;
            attempt += 1;
        }
    }

    /// Establish a WebSocket connection with authentication headers.
    ///
    /// Kalshi WebSocket requires the following headers for authentication:
    /// - KALSHI-ACCESS-KEY: The API key ID
    /// - KALSHI-ACCESS-SIGNATURE: RSA-PSS signature of the message
    /// - KALSHI-ACCESS-TIMESTAMP: Timestamp in milliseconds
    ///
    /// The signature message format is: `{timestamp}GET/trade-api/ws/v2`
    async fn connect_with_auth(config: &KalshiConfig, ws_url: &str) -> Result<WsStream> {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        // The signature message format for WebSocket: {timestamp}GET/trade-api/ws/v2
        let signature = config.sign(timestamp_ms, "GET", "/trade-api/ws/v2")?;

        // Build the request with auth headers
        let mut request = ws_url
            .into_client_request()
            .map_err(|e| Error::WebSocket(Box::new(e)))?;

        let headers = request.headers_mut();
        headers.insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(config.api_key_id())
                .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
        );
        headers.insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&signature)
                .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
        );
        headers.insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp_ms.to_string())
                .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
        );

        info!("Connecting to Kalshi WebSocket at {}", ws_url);

        let (ws_stream, response) = tokio_tungstenite::connect_async(request).await?;

        info!(
            "Connected to Kalshi WebSocket (status: {})",
            response.status()
        );

        Ok(ws_stream)
    }

    /// Run the actor's main event loop.
    ///
    /// This method processes:
    /// 1. Commands from client handles via `cmd_receiver`
    /// 2. Incoming WebSocket messages from `ws_reader`
    /// 3. Periodic ping messages for health monitoring
    ///
    /// The loop continues until the connection is closed or an unrecoverable
    /// error occurs. On disconnection, a `StreamMessage::Disconnected` event
    /// is broadcast to all subscribers.
    pub async fn run(mut self) {
        info!("StreamActor starting main loop");

        // Set up ping interval for health monitoring
        let ping_start = Instant::now() + self.health_config.ping_interval;
        let mut ping_interval = interval_at(ping_start.into(), self.health_config.ping_interval);

        let disconnect_reason: Option<(String, bool)>;

        loop {
            tokio::select! {
                // Handle commands from client handles
                Some(command) = self.cmd_receiver.recv() => {
                    if self.handle_command(command).await {
                        info!("StreamActor received close command, shutting down");
                        disconnect_reason = Some(("Client requested close".to_string(), true));
                        break;
                    }
                }

                // Handle incoming WebSocket messages
                Some(message) = self.ws_reader.next() => {
                    match self.handle_ws_message(message).await {
                        Ok(false) => {} // Continue
                        Ok(true) => {
                            // Clean close from server
                            disconnect_reason = Some(("Server closed connection".to_string(), true));
                            break;
                        }
                        Err(reason) => {
                            // Error close
                            disconnect_reason = Some((reason, false));
                            break;
                        }
                    }
                }

                // Send periodic ping for health monitoring
                _ = ping_interval.tick() => {
                    if self.ping_pending {
                        // Check if we've timed out waiting for pong
                        let elapsed = self.last_pong.elapsed();
                        if elapsed > self.health_config.ping_timeout {
                            error!("Ping timeout: no pong received in {:?}", elapsed);
                            disconnect_reason = Some(("Ping timeout".to_string(), false));
                            break;
                        }
                    } else {
                        // Send a ping
                        let ping_data = b"health".to_vec();
                        if let Err(e) = self.ws_writer.send(Message::Ping(ping_data)).await {
                            error!("Failed to send ping: {}", e);
                            disconnect_reason = Some((format!("Failed to send ping: {}", e), false));
                            break;
                        }
                        self.ping_pending = true;
                        debug!("Sent health ping");
                    }
                }

                // All channels closed
                else => {
                    info!("StreamActor all channels closed, shutting down");
                    disconnect_reason = Some(("All channels closed".to_string(), true));
                    break;
                }
            }
        }

        // Broadcast disconnection event
        if let Some((reason, was_clean)) = disconnect_reason {
            let disconnect_update = StreamUpdate {
                channel: "system".to_string(),
                sid: 0,
                seq: None,
                msg: StreamMessage::Disconnected {
                    reason: reason.clone(),
                    was_clean,
                },
            };
            let _ = self.update_sender.send(disconnect_update);
            info!(
                "Broadcast disconnect event: {} (clean: {})",
                reason, was_clean
            );
        }

        // Clean up: cancel pending requests and close connection
        self.request_handler.cancel_all();
        let _ = self.ws_writer.close().await;
        info!("StreamActor shutdown complete");
    }

    /// Handle a command from a client handle.
    ///
    /// Returns `true` if the actor should shut down.
    async fn handle_command(&mut self, command: StreamCommand) -> bool {
        match command {
            StreamCommand::Subscribe {
                channels,
                market_tickers,
                response,
            } => {
                let request_id = self.next_request_id;
                self.next_request_id += 1;

                // Convert channel strings to Channel enums for protocol
                let channels: Vec<Channel> = channels
                    .iter()
                    .filter_map(|s| match s.as_str() {
                        "orderbook_delta" => Some(Channel::OrderbookDelta),
                        "ticker" => Some(Channel::Ticker),
                        "trade" => Some(Channel::Trade),
                        "fill" => Some(Channel::Fill),
                        "market_positions" => Some(Channel::MarketPositions),
                        "communications" => Some(Channel::Communications),
                        "market_lifecycle_v2" => Some(Channel::MarketLifecycle),
                        "multivariate" => Some(Channel::Multivariate),
                        _ => {
                            warn!("Unknown channel: {}", s);
                            None
                        }
                    })
                    .collect();

                let tickers: Vec<&str> = market_tickers.iter().map(|s| s.as_str()).collect();
                let msg = protocol::build_subscribe(request_id, &channels, &tickers);

                debug!("Sending subscribe request {}: {}", request_id, msg);

                // Register the response handler
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.request_handler.register(request_id, tx);

                // Send the subscribe message
                if let Err(e) = self.ws_writer.send(Message::Text(msg)).await {
                    error!("Failed to send subscribe message: {}", e);
                    let _ = response.send(Err(format!("WebSocket send error: {}", e)));
                    return false;
                }

                // Wait for response and forward it
                tokio::spawn(async move {
                    match rx.await {
                        Ok(json) => {
                            // Parse subscription IDs from response
                            let sids = Self::extract_sids(&json);
                            let result = SubscribeResult {
                                sids,
                                response: json,
                            };
                            let _ = response.send(Ok(result));
                        }
                        Err(_) => {
                            let _ = response.send(Err("Request cancelled".to_string()));
                        }
                    }
                });
            }

            StreamCommand::Unsubscribe { sids, response } => {
                let request_id = self.next_request_id;
                self.next_request_id += 1;

                let msg = protocol::build_unsubscribe(request_id, &sids);

                debug!("Sending unsubscribe request {}: {}", request_id, msg);

                // Register the response handler
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.request_handler.register(request_id, tx);

                // Send the unsubscribe message
                if let Err(e) = self.ws_writer.send(Message::Text(msg)).await {
                    error!("Failed to send unsubscribe message: {}", e);
                    let _ = response.send(Err(format!("WebSocket send error: {}", e)));
                    return false;
                }

                // Wait for response and forward it
                tokio::spawn(async move {
                    match rx.await {
                        Ok(json) => {
                            let _ = response.send(Ok(json));
                        }
                        Err(_) => {
                            let _ = response.send(Err("Request cancelled".to_string()));
                        }
                    }
                });
            }

            StreamCommand::Close => {
                info!("Received close command");
                return true;
            }
        }

        false
    }

    /// Handle an incoming WebSocket message.
    ///
    /// Returns:
    /// - `Ok(false)` to continue processing
    /// - `Ok(true)` for clean shutdown
    /// - `Err(reason)` for error shutdown
    async fn handle_ws_message(
        &mut self,
        message: std::result::Result<Message, tungstenite::Error>,
    ) -> std::result::Result<bool, String> {
        match message {
            Ok(Message::Text(text)) => {
                self.handle_text_message(&text).await;
                Ok(false)
            }

            Ok(Message::Ping(data)) => {
                debug!("Received ping: {:?}", String::from_utf8_lossy(&data));
                // Kalshi sends ping with body "heartbeat" every 10 seconds
                // Respond with pong containing the same data
                if let Err(e) = self.ws_writer.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                    return Err(format!("Failed to send pong: {}", e));
                }
                Ok(false)
            }

            Ok(Message::Pong(data)) => {
                debug!("Received pong: {:?}", String::from_utf8_lossy(&data));
                self.last_pong = Instant::now();
                self.ping_pending = false;
                Ok(false)
            }

            Ok(Message::Close(frame)) => {
                info!("Received close frame: {:?}", frame);
                Ok(true)
            }

            Ok(Message::Binary(data)) => {
                warn!("Received unexpected binary message: {} bytes", data.len());
                Ok(false)
            }

            Ok(Message::Frame(_)) => {
                // Raw frame, typically not received in normal operation
                Ok(false)
            }

            Err(tungstenite::Error::ConnectionClosed) => {
                info!("WebSocket connection closed");
                Ok(true)
            }

            Err(tungstenite::Error::AlreadyClosed) => {
                info!("WebSocket already closed");
                Ok(true)
            }

            Err(tungstenite::Error::Io(ref e)) => {
                error!("WebSocket I/O error: {}", e);
                Err(format!("I/O error: {}", e))
            }

            Err(e) => {
                error!("WebSocket error: {}", e);
                Err(format!("WebSocket error: {}", e))
            }
        }
    }

    /// Handle an incoming text message from the WebSocket.
    async fn handle_text_message(&mut self, text: &str) {
        debug!("Received message: {}", text);

        match protocol::parse_incoming(text) {
            Ok(IncomingMessage::Response { id, msg_type, msg }) => {
                debug!("Response for request {}: type={}", id, msg_type);

                // Build the full response JSON including type and msg
                let full_response = serde_json::json!({
                    "id": id,
                    "type": msg_type,
                    "msg": msg,
                });

                if !self.request_handler.handle_response(id, full_response) {
                    warn!("No handler for response id {}", id);
                }
            }

            Ok(IncomingMessage::Update { msg_type, sid, msg }) => {
                debug!("Update on sid {}: type={}", sid, msg_type);

                // Try to parse as StreamUpdate
                match serde_json::from_value::<StreamUpdate>(serde_json::json!({
                    "type": msg_type,
                    "sid": sid,
                    "msg": msg,
                })) {
                    Ok(update) => {
                        if let Err(e) = self.update_sender.send(update) {
                            // No receivers - this is okay, they might subscribe later
                            debug!("No update receivers: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse update: {}", e);
                    }
                }
            }

            Ok(IncomingMessage::Error { id, code, message }) => {
                error!("Error response: code={}, message={}", code, message);

                if let Some(request_id) = id {
                    let error_response = serde_json::json!({
                        "id": request_id,
                        "error": {
                            "code": code,
                            "message": message,
                        }
                    });
                    if !self
                        .request_handler
                        .handle_response(request_id, error_response)
                    {
                        warn!("No handler for error response id {}", request_id);
                    }
                }
            }

            Err(e) => {
                error!("Failed to parse incoming message: {}", e);
            }
        }
    }

    /// Extract subscription IDs from a subscribe response.
    fn extract_sids(response: &JsonValue) -> Vec<i64> {
        // Response format: {"id": N, "type": "subscribed", "msg": {"sids": [1, 2, 3]}}
        response
            .get("msg")
            .and_then(|msg| msg.get("sids"))
            .and_then(|sids| sids.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default()
    }
}

impl std::fmt::Debug for StreamActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamActor")
            .field("next_request_id", &self.next_request_id)
            .field("pending_requests", &self.request_handler.pending_count())
            .field("ping_pending", &self.ping_pending)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sids() {
        let response = serde_json::json!({
            "id": 1,
            "type": "subscribed",
            "msg": {
                "sids": [42, 43, 44]
            }
        });

        let sids = StreamActor::extract_sids(&response);
        assert_eq!(sids, vec![42, 43, 44]);
    }

    #[test]
    fn test_extract_sids_empty() {
        let response = serde_json::json!({
            "id": 1,
            "type": "subscribed",
            "msg": {}
        });

        let sids = StreamActor::extract_sids(&response);
        assert!(sids.is_empty());
    }

    #[test]
    fn test_extract_sids_missing_msg() {
        let response = serde_json::json!({
            "id": 1,
            "type": "error"
        });

        let sids = StreamActor::extract_sids(&response);
        assert!(sids.is_empty());
    }
}
