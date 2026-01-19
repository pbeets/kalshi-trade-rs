//! WebSocket stream session for the Kalshi API.
//!
//! This module implements a session that manages the WebSocket connection
//! to Kalshi's streaming API. The session owns the WebSocket connection and handles
//! all communication in a single async task.

use tracing::{debug, error, info, warn};

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
    time::{interval_at, sleep, sleep_until, timeout},
};

use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{self, Message, client::IntoClientRequest, http::HeaderValue},
};

use super::{
    BACKOFF_BASE, CONNECT_TIMEOUT, ConnectStrategy, HealthConfig, MAX_BACKOFF,
    channel::Channel,
    command::{
        ChannelError, ChannelSubscription, StreamCommand, SubscribeResult, UnsubscribeResult,
    },
    message::{StreamMessage, StreamUpdate},
    protocol::{self, IncomingMessage},
};

use crate::{
    auth::KalshiConfig,
    error::{DisconnectReason, Error, Result},
};

/// WebSocket stream type alias for clarity.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// State of a single channel subscription.
#[derive(Debug, Clone)]
pub struct SubscriptionState {
    /// Server-assigned subscription ID.
    pub sid: i64,
    /// Markets currently subscribed to for this channel.
    pub markets: HashSet<String>,
}

/// Shared subscription state across all handles.
pub type SharedSubscriptions = Arc<RwLock<HashMap<Channel, SubscriptionState>>>;

/// Collects multiple responses for a single multi-channel subscribe request.
///
/// When subscribing to N channels, Kalshi sends N responses (all with the same
/// request ID but different `sid` values). This collector accumulates those
/// responses until all expected responses are received.
struct SubscribeCollector {
    /// Number of responses we expect to receive.
    expected: usize,
    /// Successfully subscribed channels.
    successful: Vec<ChannelSubscription>,
    /// Channels that failed to subscribe.
    failed: Vec<ChannelError>,
    /// Channel to send the final result when all responses are collected.
    sender: oneshot::Sender<std::result::Result<SubscribeResult, String>>,
}

impl SubscribeCollector {
    /// Create a new collector expecting `expected` responses.
    fn new(
        expected: usize,
        sender: oneshot::Sender<std::result::Result<SubscribeResult, String>>,
    ) -> Self {
        Self {
            expected,
            successful: Vec::with_capacity(expected),
            failed: Vec::new(),
            sender,
        }
    }

    /// Add a successful subscription response.
    /// Returns `true` if all expected responses have been received.
    fn add_success(&mut self, channel: String, sid: i64) -> bool {
        self.successful.push(ChannelSubscription { channel, sid });
        self.is_complete()
    }

    /// Add a failed subscription response.
    /// Returns `true` if all expected responses have been received.
    fn add_error(&mut self, channel: Option<String>, code: String, message: String) -> bool {
        self.failed.push(ChannelError {
            channel,
            code,
            message,
        });
        self.is_complete()
    }

    /// Check if we've received all expected responses.
    fn is_complete(&self) -> bool {
        self.successful.len() + self.failed.len() >= self.expected
    }

    /// Consume the collector and send the final result.
    fn finish(self) {
        let result = SubscribeResult {
            successful: self.successful,
            failed: self.failed,
        };
        let _ = self.sender.send(Ok(result));
    }
}

/// Collects multiple responses for a single multi-SID unsubscribe request.
///
/// When unsubscribing from N SIDs, Kalshi sends N responses (all with the same
/// request ID but different `sid` values). This collector accumulates those
/// responses until all expected responses are received.
struct UnsubscribeCollector {
    /// Number of responses we expect to receive.
    expected: usize,
    /// SIDs that have been successfully unsubscribed.
    unsubscribed: Vec<i64>,
    /// Channel to send the final result when all responses are collected.
    sender: oneshot::Sender<std::result::Result<UnsubscribeResult, String>>,
}

impl UnsubscribeCollector {
    /// Create a new collector expecting `expected` responses.
    fn new(
        expected: usize,
        sender: oneshot::Sender<std::result::Result<UnsubscribeResult, String>>,
    ) -> Self {
        Self {
            expected,
            unsubscribed: Vec::with_capacity(expected),
            sender,
        }
    }

    /// Record an unsubscribed SID. Returns `true` if all expected responses have been received.
    fn add_unsubscribed(&mut self, sid: i64) -> bool {
        self.unsubscribed.push(sid);
        self.is_complete()
    }

    /// Check if we've received all expected responses.
    fn is_complete(&self) -> bool {
        self.unsubscribed.len() >= self.expected
    }

    /// Consume the collector and send the result.
    fn finish(self) {
        let result = UnsubscribeResult {
            sids: self.unsubscribed,
        };
        let _ = self.sender.send(Ok(result));
    }
}

/// The WebSocket stream session that manages the connection lifecycle.
///
/// The session owns the WebSocket connection (split into reader and writer),
/// processes commands from clients, and broadcasts updates to subscribers.
pub struct KalshiStreamSession {
    /// Configuration for authentication.
    #[allow(dead_code)]
    config: KalshiConfig,
    /// Receiver for commands from client handles.
    cmd_receiver: mpsc::Receiver<StreamCommand>,
    /// Health monitoring configuration.
    health_config: HealthConfig,
    /// Last time we received a pong response to our ping.
    last_pong: Instant,
    /// Last time we had any activity (sent or received a message).
    /// Used to detect stale connections when Kalshi stops sending heartbeats
    /// during periods of activity (Kalshi only sends pings on idle connections).
    last_activity: Instant,
    /// Next request ID to use for outgoing messages.
    next_request_id: u64,
    /// Pending multi-channel subscribe requests awaiting multiple responses.
    pending_subscriptions: HashMap<u64, SubscribeCollector>,
    /// Pending multi-SID unsubscribe requests awaiting multiple responses.
    pending_unsubscriptions: HashMap<u64, UnsubscribeCollector>,
    /// Pending update_subscription requests awaiting single response.
    /// Returns the updated list of markets for the subscription.
    pending_updates: HashMap<u64, oneshot::Sender<std::result::Result<Vec<String>, String>>>,
    /// Whether we're waiting for a pong response.
    ping_pending: bool,
    /// One-shot sender to signal that the session is ready.
    /// Sent when the session enters its main loop.
    ready_sender: Option<oneshot::Sender<()>>,
    /// Sender for broadcasting updates to subscribers.
    update_sender: broadcast::Sender<StreamUpdate>,
    /// Shared subscription state with client handles.
    /// Used to capture and clear subscriptions on disconnect.
    subscriptions: SharedSubscriptions,
    /// WebSocket reader half.
    ws_reader: SplitStream<WsStream>,
    /// WebSocket writer half.
    ws_writer: SplitSink<WsStream, Message>,
}

impl KalshiStreamSession {
    /// Connect to the Kalshi WebSocket API.
    ///
    /// The `ready_sender` will be notified when the session enters its main loop,
    /// allowing the caller to confirm the session is running before returning.
    ///
    /// # Arguments
    ///
    /// * `config` - The Kalshi configuration with API credentials.
    /// * `strategy` - Connection strategy (Simple or Retry).
    /// * `cmd_receiver` - Receiver for commands from client handles.
    /// * `update_sender` - Sender for broadcasting updates to subscribers.
    /// * `subscriptions` - Shared subscription state with client handles.
    /// * `ready_sender` - One-shot sender to signal session readiness.
    pub async fn connect(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
        cmd_receiver: mpsc::Receiver<StreamCommand>,
        update_sender: broadcast::Sender<StreamUpdate>,
        subscriptions: SharedSubscriptions,
        ready_sender: oneshot::Sender<()>,
    ) -> Result<Self> {
        Self::connect_full(
            config,
            strategy,
            HealthConfig::default(),
            cmd_receiver,
            update_sender,
            subscriptions,
            Some(ready_sender),
        )
        .await
    }

    /// Full connect with all options.
    async fn connect_full(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
        health_config: HealthConfig,
        cmd_receiver: mpsc::Receiver<StreamCommand>,
        update_sender: broadcast::Sender<StreamUpdate>,
        subscriptions: SharedSubscriptions,
        ready_sender: Option<oneshot::Sender<()>>,
    ) -> Result<Self> {
        let ws_url = config.environment.ws_url();
        let ws_stream = Self::connect_with_strategy(config, ws_url, strategy).await?;

        let (ws_writer, ws_reader) = ws_stream.split();

        Ok(Self {
            config: config.clone(),
            cmd_receiver,
            health_config,
            last_pong: Instant::now(),
            last_activity: Instant::now(), // Connection establishment is activity
            next_request_id: 1,
            pending_subscriptions: HashMap::new(),
            pending_unsubscriptions: HashMap::new(),
            pending_updates: HashMap::new(),
            ping_pending: false,
            ready_sender,
            subscriptions,
            update_sender,
            ws_reader,
            ws_writer,
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

    /// Run the session's main event loop.
    ///
    /// This method processes:
    /// 1. Commands from client handles via `cmd_receiver`
    /// 2. Incoming WebSocket messages from `ws_reader`
    /// 3. Periodic ping messages for health monitoring
    ///
    /// The loop continues until the connection is closed or an unrecoverable
    /// error occurs. On disconnection, either `StreamMessage::Closed` (for clean
    /// shutdowns) or `StreamMessage::ConnectionLost` (for errors) is broadcast
    /// to all subscribers.
    pub async fn run(mut self) {
        info!("KalshiStreamSession starting main loop");

        // Signal that the session is ready (if a ready_sender was provided)
        if let Some(ready_sender) = self.ready_sender.take() {
            let _ = ready_sender.send(());
        }

        // Set up ping interval for health monitoring
        let ping_start = Instant::now() + self.health_config.ping_interval;
        let mut ping_interval = interval_at(ping_start.into(), self.health_config.ping_interval);

        // Track disconnect reason and whether it's a clean close
        // (reason, is_clean_close)
        let disconnect_info: Option<(DisconnectReason, bool)>;

        loop {
            tokio::select! {
                // Handle commands from client handles
                Some(command) = self.cmd_receiver.recv() => {
                    if self.handle_command(command).await {
                        info!("KalshiStreamSession received close command, shutting down");
                        disconnect_info = Some((DisconnectReason::ClientClosed, true));
                        break;
                    }
                }

                // Handle incoming WebSocket messages
                Some(message) = self.ws_reader.next() => {
                    match self.handle_ws_message(message).await {
                        Ok(false) => {} // Continue
                        Ok(true) => {
                            // Clean close from server
                            disconnect_info = Some((DisconnectReason::ServerClosed, true));
                            break;
                        }
                        Err(reason) => {
                            // Error close
                            disconnect_info = Some((reason, false));
                            break;
                        }
                    }
                }

                // Send periodic ping for health monitoring (client -> server)
                _ = ping_interval.tick() => {
                    if self.ping_pending {
                        // Check if we've timed out waiting for pong
                        let elapsed = self.last_pong.elapsed();
                        if elapsed > self.health_config.ping_timeout {
                            error!("Ping timeout: no pong received in {:?}", elapsed);
                            disconnect_info = Some((DisconnectReason::PingTimeout, false));
                            break;
                        }
                    } else {
                        // Send a ping
                        let ping_data = b"health".to_vec();
                        if let Err(e) = self.ws_writer.send(Message::Ping(ping_data.into())).await {
                            error!("Failed to send ping: {}", e);
                            disconnect_info = Some((DisconnectReason::HealthCheckFailed(format!("send ping: {}", e)), false));
                            break;
                        }
                        self.last_activity = Instant::now();
                        self.ping_pending = true;
                        debug!("Sent health ping");
                    }
                }

                // Check for activity timeout (no messages sent or received)
                // Kalshi only sends heartbeat pings on idle connections, so we track
                // all activity (incoming and outgoing messages) to detect stale connections.
                _ = sleep_until((self.last_activity + self.health_config.server_ping_timeout).into()) => {
                    let elapsed = self.last_activity.elapsed();
                    if elapsed > self.health_config.server_ping_timeout {
                        error!("Activity timeout: no messages sent or received in {:?}", elapsed);
                        disconnect_info = Some((DisconnectReason::ServerHeartbeatTimeout, false));
                        break;
                    }
                }

                // All channels closed
                else => {
                    info!("KalshiStreamSession all channels closed, shutting down");
                    disconnect_info = Some((DisconnectReason::AllChannelsClosed, true));
                    break;
                }
            }
        }

        // Capture and clear subscriptions before broadcasting disconnect
        let lost_subscriptions: Vec<(Channel, Vec<String>)> = {
            let mut subs = self
                .subscriptions
                .write()
                .expect("subscription lock poisoned");

            let captured: Vec<(Channel, Vec<String>)> = subs
                .drain()
                .map(|(channel, state)| (channel, state.markets.into_iter().collect()))
                .collect();

            captured
        };

        // Broadcast disconnection event
        if let Some((reason, is_clean)) = disconnect_info {
            let msg = if is_clean {
                StreamMessage::Closed {
                    reason: reason.clone(),
                }
            } else {
                StreamMessage::ConnectionLost {
                    reason: reason.clone(),
                    subscriptions: lost_subscriptions,
                }
            };

            let disconnect_update = StreamUpdate {
                channel: "system".to_string(),
                sid: 0,
                seq: None,
                msg,
            };
            let _ = self.update_sender.send(disconnect_update);
            info!(
                "Broadcast disconnect event: {} (clean: {})",
                reason, is_clean
            );
        }

        // Clean up: drop pending requests (their senders will be dropped, causing recv errors)
        self.pending_subscriptions.clear();
        self.pending_unsubscriptions.clear();
        self.pending_updates.clear();
        let _ = self.ws_writer.close().await;
        info!("KalshiStreamSession shutdown complete");
    }

    /// Handle a command from a client handle.
    ///
    /// Returns `true` if the session should shut down.
    async fn handle_command(&mut self, command: StreamCommand) -> bool {
        match command {
            StreamCommand::Subscribe {
                channels,
                market_tickers,
                sharding,
                response,
            } => {
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

                if channels.is_empty() {
                    let _ = response.send(Err("No valid channels specified".to_string()));
                    return false;
                }

                // Validate that channels requiring market tickers have at least one
                if market_tickers.is_empty() {
                    let channels_requiring_tickers: Vec<&str> = channels
                        .iter()
                        .filter(|c| c.requires_market_ticker())
                        .map(|c| c.as_str())
                        .collect();

                    if !channels_requiring_tickers.is_empty() {
                        let _ = response.send(Err(format!(
                            "Market tickers required for channels: {}",
                            channels_requiring_tickers.join(", ")
                        )));
                        return false;
                    }
                }

                let tickers: Vec<&str> = market_tickers.iter().map(|s| s.as_str()).collect();
                let num_channels = channels.len();

                // Get request ID for this subscribe request
                let request_id = self.next_request_id;
                self.next_request_id += 1;

                // Build ONE subscribe message with ALL channels
                // Kalshi will respond with N separate responses (one per channel),
                // all sharing the same request ID but with different sids.
                let msg =
                    protocol::build_subscribe(request_id, &channels, &tickers, sharding.as_ref());
                debug!(
                    "Sending subscribe request {} for {} channels: {}",
                    request_id, num_channels, msg
                );

                // Send the subscribe message
                if let Err(e) = self.ws_writer.send(Message::Text(msg.into())).await {
                    error!("Failed to send subscribe message: {}", e);
                    let _ = response.send(Err(format!("WebSocket send error: {}", e)));
                    return false;
                }
                self.last_activity = Instant::now();

                // Register a collector to accumulate all N responses
                let collector = SubscribeCollector::new(num_channels, response);
                self.pending_subscriptions.insert(request_id, collector);
            }

            StreamCommand::Unsubscribe { sids, response } => {
                let request_id = self.next_request_id;
                self.next_request_id += 1;

                let num_sids = sids.len();
                let msg = protocol::build_unsubscribe(request_id, &sids);

                debug!(
                    "Sending unsubscribe request {} for {} SIDs: {}",
                    request_id, num_sids, msg
                );

                // Send the unsubscribe message
                if let Err(e) = self.ws_writer.send(Message::Text(msg.into())).await {
                    error!("Failed to send unsubscribe message: {}", e);
                    let _ = response.send(Err(format!("WebSocket send error: {}", e)));
                    return false;
                }
                self.last_activity = Instant::now();

                // Register a collector to accumulate all N responses
                let collector = UnsubscribeCollector::new(num_sids, response);
                self.pending_unsubscriptions.insert(request_id, collector);
            }

            StreamCommand::UpdateSubscription {
                sid,
                markets,
                action,
                response,
            } => {
                let request_id = self.next_request_id;
                self.next_request_id += 1;

                let market_strs: Vec<&str> = markets.iter().map(|s| s.as_str()).collect();
                let msg =
                    protocol::build_update_subscription(request_id, sid, &market_strs, action);

                debug!(
                    "Sending update_subscription request {} for SID {}: {}",
                    request_id, sid, msg
                );

                // Send the update_subscription message
                if let Err(e) = self.ws_writer.send(Message::Text(msg.into())).await {
                    error!("Failed to send update_subscription message: {}", e);
                    let _ = response.send(Err(format!("WebSocket send error: {}", e)));
                    return false;
                }
                self.last_activity = Instant::now();

                // Register pending request (single response expected)
                self.pending_updates.insert(request_id, response);
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
    /// - `Ok(true)` for clean shutdown (server sent close frame)
    /// - `Err(reason)` for error shutdown
    async fn handle_ws_message(
        &mut self,
        message: std::result::Result<Message, tungstenite::Error>,
    ) -> std::result::Result<bool, DisconnectReason> {
        // Update activity timestamp for any successfully received message
        if message.is_ok() {
            self.last_activity = Instant::now();
        }

        match message {
            Ok(Message::Text(text)) => {
                self.handle_text_message(&text).await;
                Ok(false)
            }

            Ok(Message::Ping(data)) => {
                info!("Received server ping: {:?}", String::from_utf8_lossy(&data));
                // Respond with pong containing the same data
                if let Err(e) = self.ws_writer.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                    return Err(DisconnectReason::HealthCheckFailed(format!(
                        "send pong: {}",
                        e
                    )));
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
                Err(DisconnectReason::IoError(e.to_string()))
            }

            Err(e) => {
                error!("WebSocket error: {}", e);
                Err(DisconnectReason::ProtocolError(e.to_string()))
            }
        }
    }

    /// Handle an incoming text message from the WebSocket.
    async fn handle_text_message(&mut self, text: &str) {
        debug!("Received message: {}", text);

        match protocol::parse_incoming(text) {
            Ok(IncomingMessage::Response { id, msg_type, msg }) => {
                debug!("Response for request {}: type={}", id, msg_type);

                // Check if this is a response to a pending multi-channel subscription
                if msg_type == "subscribed"
                    && let Some(collector) = self.pending_subscriptions.get_mut(&id)
                {
                    // Extract channel and sid from the response
                    let channel = msg
                        .get("channel")
                        .and_then(|c| c.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let sid = msg.get("sid").and_then(|s| s.as_i64()).unwrap_or(-1);

                    debug!(
                        "Subscribe response for request {}: channel={}, sid={}",
                        id, channel, sid
                    );

                    let is_complete = collector.add_success(channel, sid);
                    if is_complete {
                        // All responses received, send the final result
                        if let Some(collector) = self.pending_subscriptions.remove(&id) {
                            collector.finish();
                        }
                    }
                    return;
                }

                if msg_type == "unsubscribed" {
                    // Also broadcast this as an update to let the client know the stream is closed
                    let sid = msg.get("sid").and_then(|s| s.as_i64());
                    if let Some(sid) = sid {
                        let update = StreamUpdate {
                            channel: msg_type.clone(),
                            sid,
                            seq: None,
                            msg: StreamMessage::Unsubscribed,
                        };

                        if let Err(e) = self.update_sender.send(update) {
                            debug!("No update receivers for unsubscribed event: {}", e);
                        }
                    }

                    // Check for pending unsubscribe collector
                    if let Some(collector) = self.pending_unsubscriptions.get_mut(&id) {
                        let sid = sid.unwrap_or(-1);
                        debug!("Unsubscribe response for request {}: sid={}", id, sid);
                        if collector.add_unsubscribed(sid) {
                            // All responses received, finish the collector
                            if let Some(collector) = self.pending_unsubscriptions.remove(&id) {
                                collector.finish();
                            }
                        }
                    }
                    return;
                }

                // Check if this is a response to a pending update_subscription
                if msg_type == "ok"
                    && let Some(response) = self.pending_updates.remove(&id)
                {
                    let markets: Vec<String> = msg
                        .get("market_tickers")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    debug!(
                        "Update subscription response for request {}: markets={:?}",
                        id, markets
                    );

                    let _ = response.send(Ok(markets));
                    return;
                }

                // Unexpected response type with no handler
                warn!("Unexpected response id {} type {}", id, msg_type);
            }

            Ok(IncomingMessage::Update { msg_type, sid, msg }) => {
                debug!("Update on sid {}: type={}", sid, msg_type);

                // Handle "unsubscribed" updates specially
                if msg_type == "unsubscribed" {
                    // This is a confirmation of unsubscription for this sid
                    let update = StreamUpdate {
                        channel: msg_type,
                        sid,
                        seq: None,
                        msg: StreamMessage::Unsubscribed,
                    };

                    if let Err(e) = self.update_sender.send(update) {
                        debug!("No update receivers for unsubscribed event: {}", e);
                    }
                    return;
                }

                // Parse the message using type-based routing
                match StreamMessage::from_type_and_value(&msg_type, msg) {
                    Ok(stream_msg) => {
                        let update = StreamUpdate {
                            channel: msg_type,
                            sid,
                            seq: None,
                            msg: stream_msg,
                        };
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

                // Check if this is an error for a pending multi-channel subscription
                if let Some(request_id) = id {
                    if let Some(collector) = self.pending_subscriptions.get_mut(&request_id) {
                        debug!(
                            "Subscribe error for request {}: code={}, message={}",
                            request_id, code, message
                        );

                        let is_complete = collector.add_error(None, code, message.clone());
                        if is_complete {
                            // All responses received, send the final result
                            if let Some(collector) = self.pending_subscriptions.remove(&request_id)
                            {
                                collector.finish();
                            }
                        }
                        return;
                    }

                    // Check if this is an error for a pending unsubscribe
                    // Note: unsubscribe errors are rare, but we should handle them
                    if self.pending_unsubscriptions.contains_key(&request_id) {
                        warn!(
                            "Unsubscribe error for request {}: code={}, message={}",
                            request_id, code, message
                        );
                        // Remove the collector and let the sender error out
                        self.pending_unsubscriptions.remove(&request_id);
                        return;
                    }

                    // Check if this is an error for a pending update_subscription
                    if let Some(response) = self.pending_updates.remove(&request_id) {
                        warn!(
                            "Update subscription error for request {}: code={}, message={}",
                            request_id, code, message
                        );
                        let _ = response.send(Err(format!("{}: {}", code, message)));
                        return;
                    }

                    warn!(
                        "Unhandled error response id {}: code={}, message={}",
                        request_id, code, message
                    );
                }
            }

            Err(e) => {
                error!("Failed to parse incoming message: {}", e);
            }
        }
    }
}

impl std::fmt::Debug for KalshiStreamSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiStreamSession")
            .field("next_request_id", &self.next_request_id)
            .field("pending_subscriptions", &self.pending_subscriptions.len())
            .field(
                "pending_unsubscriptions",
                &self.pending_unsubscriptions.len(),
            )
            .field("pending_updates", &self.pending_updates.len())
            .field("ping_pending", &self.ping_pending)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value as JsonValue;

    /// Extract subscription IDs from a subscribe response.
    fn extract_sids(response: &JsonValue) -> Vec<i64> {
        // Kalshi response format: {"id": N, "type": "subscribed", "msg": {"channel": "...", "sid": 1}}
        // Note: Kalshi returns a singular "sid" field, not a "sids" array.
        // When subscribing to multiple channels, Kalshi sends multiple responses.

        // First try the singular "sid" field (Kalshi's actual format)
        if let Some(sid) = response
            .get("msg")
            .and_then(|msg| msg.get("sid"))
            .and_then(|s| s.as_i64())
        {
            return vec![sid];
        }

        // Fallback: try "sids" array format (in case Kalshi ever changes)
        response
            .get("msg")
            .and_then(|msg| msg.get("sids"))
            .and_then(|sids| sids.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default()
    }

    #[test]
    fn test_extract_sids_singular() {
        // Kalshi's actual response format
        let response = serde_json::json!({
            "id": 1,
            "type": "subscribed",
            "msg": {
                "channel": "orderbook_delta",
                "sid": 42
            }
        });

        let sids = extract_sids(&response);
        assert_eq!(sids, vec![42]);
    }

    #[test]
    fn test_extract_sids_array_fallback() {
        // Fallback format in case Kalshi ever changes to array format
        let response = serde_json::json!({
            "id": 1,
            "type": "subscribed",
            "msg": {
                "sids": [42, 43, 44]
            }
        });

        let sids = extract_sids(&response);
        assert_eq!(sids, vec![42, 43, 44]);
    }

    #[test]
    fn test_extract_sids_empty() {
        let response = serde_json::json!({
            "id": 1,
            "type": "subscribed",
            "msg": {}
        });

        let sids = extract_sids(&response);
        assert!(sids.is_empty());
    }

    #[test]
    fn test_extract_sids_missing_msg() {
        let response = serde_json::json!({
            "id": 1,
            "type": "error"
        });

        let sids = extract_sids(&response);
        assert!(sids.is_empty());
    }
}
