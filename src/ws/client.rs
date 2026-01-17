//! WebSocket streaming client for Kalshi API.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};

use super::{
    ConnectStrategy,
    channel::Channel,
    command::{StreamCommand, SubscribeResult, UnsubscribeResult, UpdateAction},
    message::StreamUpdate,
    session::KalshiStreamSession,
};

use crate::{
    auth::KalshiConfig,
    error::{Error, Result},
};

/// Default buffer size for the broadcast channel.
const DEFAULT_BUFFER_SIZE: usize = 1024;

/// State of a single channel subscription.
#[derive(Debug, Clone)]
struct SubscriptionState {
    /// Server-assigned subscription ID.
    sid: i64,
    /// Markets currently subscribed to for this channel.
    markets: HashSet<String>,
}

/// Shared subscription state across all handles.
type SharedSubscriptions = Arc<RwLock<HashMap<Channel, SubscriptionState>>>;

/// Owner of the WebSocket connection and actor task.
///
/// This struct owns the WebSocket connection lifetime. When dropped, it will
/// shut down the actor task. Use [`handle()`](Self::handle) to get a cloneable
/// handle for interacting with the stream.
///
/// # Example
///
/// ```no_run
/// use kalshi_trade_rs::auth::KalshiConfig;
/// use kalshi_trade_rs::ws::{Channel, ConnectStrategy, KalshiStreamClient};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = KalshiConfig::from_env()?;
///
/// // Connect with retry for production
/// let client = KalshiStreamClient::connect_with_strategy(
///     &config,
///     ConnectStrategy::Retry,
/// ).await?;
///
/// let mut handle = client.handle();
///
/// // Subscribe to ticker updates for specific markets
/// handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
///
/// // Add another market (automatically uses add_markets under the hood)
/// handle.subscribe(Channel::Ticker, &["KXBTC-25DEC31-100000"]).await?;
///
/// // Process updates
/// while let Ok(update) = handle.update_receiver.recv().await {
///     println!("Update: {:?}", update);
/// }
///
/// // Graceful shutdown
/// client.shutdown().await?;
/// # Ok(())
/// # }
/// ```
pub struct KalshiStreamClient {
    actor_handle: JoinHandle<()>,
    cmd_sender: mpsc::Sender<StreamCommand>,
    update_sender: broadcast::Sender<StreamUpdate>,
    subscriptions: SharedSubscriptions,
}

impl KalshiStreamClient {
    /// Connect to Kalshi's WebSocket API with the default (Simple) strategy.
    ///
    /// This establishes a WebSocket connection and starts the background actor
    /// that manages the connection. Uses `ConnectStrategy::Simple` which fails
    /// fast on connection errors.
    ///
    /// For production use, consider [`connect_with_strategy`](Self::connect_with_strategy)
    /// with `ConnectStrategy::Retry`.
    ///
    /// # Arguments
    ///
    /// * `config` - Kalshi API configuration with credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established.
    pub async fn connect(config: &KalshiConfig) -> Result<Self> {
        Self::connect_with_options(config, ConnectStrategy::Simple, DEFAULT_BUFFER_SIZE).await
    }

    /// Connect with a specific connection strategy.
    ///
    /// # Arguments
    ///
    /// * `config` - Kalshi API configuration with credentials.
    /// * `strategy` - Connection strategy (Simple or Retry).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kalshi_trade_rs::auth::KalshiConfig;
    /// use kalshi_trade_rs::ws::{ConnectStrategy, KalshiStreamClient};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = KalshiConfig::from_env()?;
    ///
    /// // Use Retry for production - will retry indefinitely with backoff
    /// let client = KalshiStreamClient::connect_with_strategy(
    ///     &config,
    ///     ConnectStrategy::Retry,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_strategy(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
    ) -> Result<Self> {
        Self::connect_with_options(config, strategy, DEFAULT_BUFFER_SIZE).await
    }

    /// Connect with full customization options.
    ///
    /// # Arguments
    ///
    /// * `config` - Kalshi API configuration with credentials.
    /// * `strategy` - Connection strategy (Simple or Retry).
    /// * `buffer_size` - Size of the broadcast channel buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established.
    pub async fn connect_with_options(
        config: &KalshiConfig,
        strategy: ConnectStrategy,
        buffer_size: usize,
    ) -> Result<Self> {
        let (cmd_sender, cmd_receiver) = mpsc::channel(32);
        let (update_sender, _) = broadcast::channel(buffer_size);

        let actor =
            KalshiStreamSession::connect(config, strategy, cmd_receiver, update_sender.clone())
                .await?;
        let actor_handle = tokio::spawn(actor.run());

        Ok(Self {
            actor_handle,
            cmd_sender,
            update_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get a cloneable handle for interacting with the stream.
    ///
    /// Handles can be cloned and shared across tasks. Each cloned handle
    /// gets its own broadcast receiver that starts receiving from the
    /// point of subscription. All handles share the same subscription state.
    pub fn handle(&self) -> KalshiStreamHandle {
        KalshiStreamHandle {
            cmd_sender: self.cmd_sender.clone(),
            update_sender: self.update_sender.clone(),
            update_receiver: self.update_sender.subscribe(),
            subscriptions: self.subscriptions.clone(),
        }
    }

    /// Shut down the connection and wait for the actor to exit.
    ///
    /// This sends a close command to the actor and waits for it to
    /// complete its shutdown sequence.
    ///
    /// # Errors
    ///
    /// Returns an error if the actor task panicked.
    pub async fn shutdown(self) -> Result<()> {
        let _ = self.cmd_sender.send(StreamCommand::Close).await;
        self.actor_handle
            .await
            .map_err(|e| Error::Api(e.to_string()))
    }
}

/// Cloneable handle for interacting with a Kalshi WebSocket stream.
///
/// Handles can be freely cloned and shared across tasks. Each handle
/// maintains its own broadcast receiver for receiving updates, but all
/// handles share the same subscription state.
///
/// # Example
///
/// ```no_run
/// use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle, StreamMessage};
///
/// async fn subscribe_and_process(mut handle: KalshiStreamHandle) {
///     // Subscribe to orderbook updates for a market
///     handle.subscribe(Channel::OrderbookDelta, &["INXD-25JAN17-B5955"]).await.unwrap();
///
///     // Add more markets later
///     handle.subscribe(Channel::OrderbookDelta, &["KXBTC-25DEC31-100000"]).await.unwrap();
///
///     // Check what markets we're subscribed to
///     println!("Markets: {:?}", handle.markets(Channel::OrderbookDelta));
///
///     // Process updates
///     while let Ok(update) = handle.update_receiver.recv().await {
///         match &update.msg {
///             StreamMessage::Closed { reason } => {
///                 println!("Connection closed: {}", reason);
///                 break;
///             }
///             StreamMessage::ConnectionLost { reason } => {
///                 eprintln!("Connection lost: {}", reason);
///                 break;
///             }
///             _ => println!("Got update: {:?}", update),
///         }
///     }
/// }
/// ```
pub struct KalshiStreamHandle {
    cmd_sender: mpsc::Sender<StreamCommand>,
    update_sender: broadcast::Sender<StreamUpdate>,
    /// Receiver for stream updates.
    ///
    /// This receives all updates from subscribed channels. Note that
    /// if the receiver falls too far behind, it will start missing
    /// messages (lagged error).
    pub update_receiver: broadcast::Receiver<StreamUpdate>,
    /// Shared subscription state.
    subscriptions: SharedSubscriptions,
}

impl Clone for KalshiStreamHandle {
    fn clone(&self) -> Self {
        Self {
            cmd_sender: self.cmd_sender.clone(),
            update_sender: self.update_sender.clone(),
            update_receiver: self.update_sender.subscribe(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

impl KalshiStreamHandle {
    /// Subscribe to a channel for specific markets.
    ///
    /// If already subscribed to this channel, automatically adds the new markets
    /// to the existing subscription (using `add_markets` under the hood).
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to subscribe to (e.g., Ticker, OrderbookDelta).
    /// * `markets` - Markets to subscribe to. Pass an empty slice for user channels
    ///   like `Fill` or `MarketPositions` that don't require markets.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command channel is closed (actor has shut down)
    /// - The server rejects the subscription request
    /// - Markets are required but not provided
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle};
    /// # async fn example(handle: &mut KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// // Subscribe to ticker for some markets
    /// handle.subscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
    ///
    /// // Add more markets (automatically uses add_markets)
    /// handle.subscribe(Channel::Ticker, &["KXBTC-25DEC31-100000"]).await?;
    ///
    /// // Subscribe to fills (no markets needed)
    /// handle.subscribe(Channel::Fill, &[]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&mut self, channel: Channel, markets: &[&str]) -> Result<()> {
        // Validate: channels requiring markets must have at least one
        if markets.is_empty() && channel.requires_market_ticker() {
            return Err(Error::MissingMarketTickers(channel.as_str().to_string()));
        }

        let markets_set: HashSet<String> = markets.iter().map(|s| s.to_string()).collect();

        // Snapshot current state to decide our action
        let (existing_sid, new_markets) = {
            let subs = self
                .subscriptions
                .read()
                .expect("subscription lock poisoned");
            match subs.get(&channel) {
                Some(state) => {
                    // Already subscribed - find markets we don't have yet
                    let new: Vec<&str> = markets
                        .iter()
                        .filter(|m| !state.markets.contains(**m))
                        .copied()
                        .collect();
                    (Some(state.sid), new)
                }
                None => (None, markets.to_vec()),
            }
        };

        if let Some(sid) = existing_sid {
            // Already subscribed to this channel
            if new_markets.is_empty() {
                // Already have all requested markets
                return Ok(());
            }

            // Add the new markets via update_subscription
            let updated_markets = self
                .update_subscription_internal(sid, &new_markets, UpdateAction::AddMarkets)
                .await?;

            // Update local state, but verify SID still matches (handle concurrent modifications)
            let mut subs = self
                .subscriptions
                .write()
                .expect("subscription lock poisoned");
            if let Some(state) = subs.get_mut(&channel)
                && state.sid == sid
            {
                // SID matches - safe to update
                state.markets = updated_markets.into_iter().collect();
            }
            // If SID changed, another operation modified state; our server-side
            // update succeeded but we defer to the newer local state
        } else {
            // New subscription
            let result = self.subscribe_raw(&[channel], markets).await?;

            // Check for failures first
            if !result.failed.is_empty() {
                let errors: Vec<String> = result
                    .failed
                    .iter()
                    .map(|e| format!("{}: {}", e.code, e.message))
                    .collect();
                return Err(Error::Api(errors.join("; ")));
            }

            // Extract the SID and update state
            if let Some(sub) = result.successful.first() {
                let mut subs = self
                    .subscriptions
                    .write()
                    .expect("subscription lock poisoned");
                // Use entry to handle race: if another task subscribed first, keep theirs
                subs.entry(channel).or_insert(SubscriptionState {
                    sid: sub.sid,
                    markets: markets_set,
                });
            }
        }

        Ok(())
    }

    /// Remove specific markets from a channel subscription.
    ///
    /// This removes the specified markets from an existing subscription. If you want
    /// to unsubscribe from an entire channel, use [`unsubscribe_all`](Self::unsubscribe_all).
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to modify.
    /// * `markets` - Markets to remove from the subscription.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if:
    /// - The markets were successfully removed
    /// - The specified markets weren't subscribed (no-op)
    /// - An empty slice was passed (no-op)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command channel is closed (actor has shut down)
    /// - Not currently subscribed to this channel
    /// - The server rejects the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle};
    /// # async fn example(handle: &mut KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// // Remove specific markets from ticker subscription
    /// handle.unsubscribe(Channel::Ticker, &["INXD-25JAN17-B5955"]).await?;
    ///
    /// // To unsubscribe from entire channel, use unsubscribe_all:
    /// handle.unsubscribe_all(Channel::Ticker).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&mut self, channel: Channel, markets: &[&str]) -> Result<()> {
        // Empty markets = no-op (use unsubscribe_all for full unsubscribe)
        if markets.is_empty() {
            return Ok(());
        }

        let state = {
            let subs = self
                .subscriptions
                .read()
                .expect("subscription lock poisoned");
            subs.get(&channel).cloned()
        };

        let state = state.ok_or_else(|| {
            Error::Api(format!("Not subscribed to channel: {}", channel.as_str()))
        })?;

        // Check if removing these markets would leave us with none
        let markets_to_remove: HashSet<&str> = markets.iter().copied().collect();
        let remaining_count = state
            .markets
            .iter()
            .filter(|m| !markets_to_remove.contains(m.as_str()))
            .count();

        if remaining_count == 0 {
            // Would remove all markets - do a full unsubscribe instead
            self.unsubscribe_raw(&[state.sid]).await?;

            let mut subs = self
                .subscriptions
                .write()
                .expect("subscription lock poisoned");
            // Only remove if SID still matches
            if subs.get(&channel).is_some_and(|s| s.sid == state.sid) {
                subs.remove(&channel);
            }
        } else {
            // Partial unsubscribe - remove specific markets
            let updated_markets = self
                .update_subscription_internal(state.sid, markets, UpdateAction::DeleteMarkets)
                .await?;

            // Update local state, verifying SID still matches
            let mut subs = self
                .subscriptions
                .write()
                .expect("subscription lock poisoned");
            if let Some(current) = subs.get_mut(&channel)
                && current.sid == state.sid
            {
                current.markets = updated_markets.into_iter().collect();
            }
        }

        Ok(())
    }

    /// Unsubscribe from an entire channel.
    ///
    /// This completely closes the subscription for a channel, removing all markets.
    /// Use [`unsubscribe`](Self::unsubscribe) if you only want to remove specific markets.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to unsubscribe from.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command channel is closed (actor has shut down)
    /// - Not currently subscribed to this channel
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle};
    /// # async fn example(handle: &mut KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// // Unsubscribe from entire ticker channel
    /// handle.unsubscribe_all(Channel::Ticker).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe_all(&mut self, channel: Channel) -> Result<()> {
        let state = {
            let subs = self
                .subscriptions
                .read()
                .expect("subscription lock poisoned");
            subs.get(&channel).cloned()
        };

        let state = state.ok_or_else(|| {
            Error::Api(format!("Not subscribed to channel: {}", channel.as_str()))
        })?;

        self.unsubscribe_raw(&[state.sid]).await?;

        let mut subs = self
            .subscriptions
            .write()
            .expect("subscription lock poisoned");
        // Only remove if SID still matches (handle concurrent modifications)
        if subs.get(&channel).is_some_and(|s| s.sid == state.sid) {
            subs.remove(&channel);
        }

        Ok(())
    }

    /// Get the markets currently subscribed for a channel.
    ///
    /// Returns an empty vector if not subscribed to this channel.
    pub fn markets(&self, channel: Channel) -> Vec<String> {
        let subs = self
            .subscriptions
            .read()
            .expect("subscription lock poisoned");
        subs.get(&channel)
            .map(|s| s.markets.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all active subscriptions.
    ///
    /// Returns a map of channel to list of subscribed markets.
    pub fn subscriptions(&self) -> HashMap<Channel, Vec<String>> {
        let subs = self
            .subscriptions
            .read()
            .expect("subscription lock poisoned");
        subs.iter()
            .map(|(ch, state)| (*ch, state.markets.iter().cloned().collect()))
            .collect()
    }

    /// Check if subscribed to a channel.
    pub fn is_subscribed(&self, channel: Channel) -> bool {
        let subs = self
            .subscriptions
            .read()
            .expect("subscription lock poisoned");
        subs.contains_key(&channel)
    }

    /// Get the subscription ID for a channel, if subscribed.
    pub fn sid(&self, channel: Channel) -> Option<i64> {
        let subs = self
            .subscriptions
            .read()
            .expect("subscription lock poisoned");
        subs.get(&channel).map(|s| s.sid)
    }

    /// Request graceful close of the connection.
    ///
    /// This sends a close command to the actor but does not wait for
    /// the connection to fully close. For a blocking shutdown, use
    /// [`KalshiStreamClient::shutdown`] instead.
    pub async fn close(&self) {
        let _ = self.cmd_sender.send(StreamCommand::Close).await;
    }

    // --- Internal methods ---

    /// Raw subscribe without local state management.
    async fn subscribe_raw(
        &self,
        channels: &[Channel],
        markets: &[&str],
    ) -> Result<SubscribeResult> {
        let (tx, rx) = oneshot::channel();

        let channel_strings: Vec<String> =
            channels.iter().map(|c| c.as_str().to_string()).collect();

        let market_strings: Vec<String> = markets.iter().map(|s| s.to_string()).collect();

        let cmd = StreamCommand::Subscribe {
            channels: channel_strings,
            market_tickers: market_strings,
            response: tx,
        };

        self.cmd_sender
            .send(cmd)
            .await
            .map_err(|_| Error::Api("Actor channel closed".to_string()))?;

        rx.await
            .map_err(|_| Error::Api("Response channel closed".to_string()))?
            .map_err(Error::Api)
    }

    /// Raw unsubscribe without local state management.
    async fn unsubscribe_raw(&self, sids: &[i64]) -> Result<UnsubscribeResult> {
        let (tx, rx) = oneshot::channel();

        let cmd = StreamCommand::Unsubscribe {
            sids: sids.to_vec(),
            response: tx,
        };

        self.cmd_sender
            .send(cmd)
            .await
            .map_err(|_| Error::Api("Actor channel closed".to_string()))?;

        rx.await
            .map_err(|_| Error::Api("Response channel closed".to_string()))?
            .map_err(Error::Api)
    }

    /// Internal update_subscription call.
    ///
    /// Returns the updated list of markets for the subscription.
    async fn update_subscription_internal(
        &self,
        sid: i64,
        markets: &[&str],
        action: UpdateAction,
    ) -> Result<Vec<String>> {
        let (tx, rx) = oneshot::channel();

        let cmd = StreamCommand::UpdateSubscription {
            sid,
            markets: markets.iter().map(|s| s.to_string()).collect(),
            action,
            response: tx,
        };

        self.cmd_sender
            .send(cmd)
            .await
            .map_err(|_| Error::Api("Actor channel closed".to_string()))?;

        rx.await
            .map_err(|_| Error::Api("Response channel closed".to_string()))?
            .map_err(Error::Api)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test handle with empty subscriptions.
    fn create_test_handle() -> (KalshiStreamHandle, mpsc::Receiver<StreamCommand>) {
        let (cmd_sender, cmd_receiver) = mpsc::channel(32);
        let (update_sender, _) = broadcast::channel::<StreamUpdate>(16);
        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        let handle = KalshiStreamHandle {
            cmd_sender,
            update_sender: update_sender.clone(),
            update_receiver: update_sender.subscribe(),
            subscriptions,
        };

        (handle, cmd_receiver)
    }

    /// Helper to pre-populate subscription state for testing.
    fn add_subscription(handle: &KalshiStreamHandle, channel: Channel, sid: i64, markets: &[&str]) {
        let mut subs = handle
            .subscriptions
            .write()
            .expect("subscription lock poisoned");
        subs.insert(
            channel,
            SubscriptionState {
                sid,
                markets: markets.iter().map(|s| s.to_string()).collect(),
            },
        );
    }

    // ========== Channel Validation Tests ==========

    #[tokio::test]
    async fn test_subscribe_validates_missing_markets_for_market_channels() {
        let (mut handle, _) = create_test_handle();

        // All market-scoped channels should require markets
        for channel in [
            Channel::Ticker,
            Channel::Trade,
            Channel::OrderbookDelta,
            Channel::MarketLifecycle,
        ] {
            let result = handle.subscribe(channel, &[]).await;
            assert!(result.is_err(), "{:?} should require markets", channel);
        }
    }

    #[tokio::test]
    async fn test_subscribe_allows_empty_markets_for_user_channels() {
        // User-scoped channels don't require markets - test all of them
        for (channel, channel_str) in [
            (Channel::Fill, "fill"),
            (Channel::MarketPositions, "market_positions"),
            (Channel::Communications, "communications"),
            (Channel::Multivariate, "multivariate"),
        ] {
            let (mut handle, mut cmd_receiver) = create_test_handle();

            let channel_str_owned = channel_str.to_string();
            let responder = tokio::spawn(async move {
                if let Some(StreamCommand::Subscribe { response, .. }) = cmd_receiver.recv().await {
                    let _ = response.send(Ok(SubscribeResult {
                        successful: vec![super::super::command::ChannelSubscription {
                            channel: channel_str_owned,
                            sid: 1,
                        }],
                        failed: vec![],
                    }));
                }
            });

            let result = handle.subscribe(channel, &[]).await;
            assert!(result.is_ok(), "{:?} should allow empty markets", channel);

            responder.await.unwrap();
        }
    }

    // ========== Subscribe Command Routing Tests ==========

    #[tokio::test]
    async fn test_subscribe_sends_subscribe_command_for_new_channel() {
        let (mut handle, mut cmd_receiver) = create_test_handle();

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::Subscribe {
                    channels,
                    market_tickers,
                    response,
                }) => {
                    // Verify correct command type and params
                    assert_eq!(channels, vec!["ticker"]);
                    assert_eq!(market_tickers, vec!["MARKET-A"]);

                    let _ = response.send(Ok(SubscribeResult {
                        successful: vec![super::super::command::ChannelSubscription {
                            channel: "ticker".to_string(),
                            sid: 100,
                        }],
                        failed: vec![],
                    }));
                }
                other => panic!("Expected Subscribe command, got {:?}", other),
            }
        });

        handle
            .subscribe(Channel::Ticker, &["MARKET-A"])
            .await
            .unwrap();
        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_sends_update_command_for_existing_channel() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["EXISTING"]);

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::UpdateSubscription {
                    sid,
                    markets,
                    action,
                    response,
                }) => {
                    // Must use UpdateSubscription, not Subscribe
                    assert_eq!(sid, 100);
                    assert_eq!(markets, vec!["NEW-MARKET"]);
                    assert_eq!(action, UpdateAction::AddMarkets);

                    let _ =
                        response.send(Ok(vec!["EXISTING".to_string(), "NEW-MARKET".to_string()]));
                }
                other => panic!("Expected UpdateSubscription command, got {:?}", other),
            }
        });

        handle
            .subscribe(Channel::Ticker, &["NEW-MARKET"])
            .await
            .unwrap();
        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_filters_already_subscribed_markets() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["EXISTING-A", "EXISTING-B"]);

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::UpdateSubscription {
                    markets, response, ..
                }) => {
                    // Should only contain the NEW market, not existing ones
                    assert_eq!(markets, vec!["NEW-MARKET"]);
                    assert!(!markets.contains(&"EXISTING-A".to_string()));

                    let _ = response.send(Ok(vec![
                        "EXISTING-A".to_string(),
                        "EXISTING-B".to_string(),
                        "NEW-MARKET".to_string(),
                    ]));
                }
                other => panic!("Expected UpdateSubscription, got {:?}", other),
            }
        });

        // Request includes both existing and new markets
        handle
            .subscribe(Channel::Ticker, &["EXISTING-A", "NEW-MARKET"])
            .await
            .unwrap();
        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_skips_command_when_all_markets_exist() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["MARKET-A", "MARKET-B"]);

        // Subscribe to markets we already have
        handle
            .subscribe(Channel::Ticker, &["MARKET-A"])
            .await
            .unwrap();

        // No command should have been sent
        assert!(
            cmd_receiver.try_recv().is_err(),
            "Should not send command when all markets already subscribed"
        );
    }

    // ========== Unsubscribe Command Routing Tests ==========

    #[tokio::test]
    async fn test_unsubscribe_empty_slice_is_noop() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["MARKET-A"]);

        handle.unsubscribe(Channel::Ticker, &[]).await.unwrap();

        // No command sent, state unchanged
        assert!(cmd_receiver.try_recv().is_err());
        assert!(handle.is_subscribed(Channel::Ticker));
    }

    #[tokio::test]
    async fn test_unsubscribe_partial_sends_update_command() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["KEEP", "REMOVE"]);

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::UpdateSubscription {
                    sid,
                    markets,
                    action,
                    response,
                }) => {
                    assert_eq!(sid, 100);
                    assert_eq!(markets, vec!["REMOVE"]);
                    assert_eq!(action, UpdateAction::DeleteMarkets);

                    let _ = response.send(Ok(vec!["KEEP".to_string()]));
                }
                other => panic!("Expected UpdateSubscription, got {:?}", other),
            }
        });

        handle
            .unsubscribe(Channel::Ticker, &["REMOVE"])
            .await
            .unwrap();

        // Should still be subscribed with remaining market
        assert!(handle.is_subscribed(Channel::Ticker));
        assert_eq!(handle.markets(Channel::Ticker), vec!["KEEP".to_string()]);

        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_unsubscribe_last_market_sends_full_unsubscribe() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["ONLY-MARKET"]);

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::Unsubscribe { sids, response }) => {
                    // Should be Unsubscribe, not UpdateSubscription
                    assert_eq!(sids, vec![100]);
                    let _ = response.send(Ok(super::super::command::UnsubscribeResult { sids }));
                }
                other => panic!("Expected Unsubscribe command, got {:?}", other),
            }
        });

        handle
            .unsubscribe(Channel::Ticker, &["ONLY-MARKET"])
            .await
            .unwrap();

        // Channel should be fully removed
        assert!(!handle.is_subscribed(Channel::Ticker));

        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_unsubscribe_all_sends_unsubscribe_command() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 200, &["A", "B", "C"]);

        let responder = tokio::spawn(async move {
            match cmd_receiver.recv().await {
                Some(StreamCommand::Unsubscribe { sids, response }) => {
                    assert_eq!(sids, vec![200]);
                    let _ = response.send(Ok(super::super::command::UnsubscribeResult { sids }));
                }
                other => panic!("Expected Unsubscribe, got {:?}", other),
            }
        });

        handle.unsubscribe_all(Channel::Ticker).await.unwrap();
        assert!(!handle.is_subscribed(Channel::Ticker));

        responder.await.unwrap();
    }

    // ========== Error Handling Tests ==========

    #[tokio::test]
    async fn test_unsubscribe_not_subscribed_returns_error() {
        let (mut handle, _) = create_test_handle();

        let result = handle.unsubscribe(Channel::Ticker, &["ANY"]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not subscribed"));
    }

    #[tokio::test]
    async fn test_unsubscribe_all_not_subscribed_returns_error() {
        let (mut handle, _) = create_test_handle();

        let result = handle.unsubscribe_all(Channel::Ticker).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_subscribe_propagates_server_errors() {
        let (mut handle, mut cmd_receiver) = create_test_handle();

        let responder = tokio::spawn(async move {
            if let Some(StreamCommand::Subscribe { response, .. }) = cmd_receiver.recv().await {
                let _ = response.send(Ok(SubscribeResult {
                    successful: vec![],
                    failed: vec![super::super::command::ChannelError {
                        channel: Some("ticker".to_string()),
                        code: "invalid_ticker".to_string(),
                        message: "Market not found".to_string(),
                    }],
                }));
            }
        });

        let result = handle.subscribe(Channel::Ticker, &["INVALID"]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Market not found"));

        responder.await.unwrap();
    }

    // ========== State Consistency Tests ==========

    #[tokio::test]
    async fn test_state_uses_server_returned_markets() {
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["OLD"]);

        let responder = tokio::spawn(async move {
            if let Some(StreamCommand::UpdateSubscription { response, .. }) =
                cmd_receiver.recv().await
            {
                // Server returns different list than expected
                let _ = response.send(Ok(vec![
                    "OLD".to_string(),
                    "NEW".to_string(),
                    "BONUS".to_string(), // Server added extra market
                ]));
            }
        });

        handle.subscribe(Channel::Ticker, &["NEW"]).await.unwrap();

        // State should reflect what server returned, including the bonus
        let markets = handle.markets(Channel::Ticker);
        assert_eq!(markets.len(), 3);
        assert!(markets.contains(&"BONUS".to_string()));

        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_subscribe_uses_entry_api() {
        // This tests that we use or_insert to avoid overwriting concurrent subscriptions
        let (mut handle, mut cmd_receiver) = create_test_handle();

        // Simulate another task having subscribed while we were waiting
        let subscriptions = handle.subscriptions.clone();
        let responder = tokio::spawn(async move {
            if let Some(StreamCommand::Subscribe { response, .. }) = cmd_receiver.recv().await {
                // Before responding, simulate concurrent subscription
                {
                    let mut subs = subscriptions.write().unwrap();
                    subs.insert(
                        Channel::Ticker,
                        SubscriptionState {
                            sid: 999, // Different SID - concurrent subscription
                            markets: HashSet::from(["CONCURRENT".to_string()]),
                        },
                    );
                }

                // Now respond with our SID
                let _ = response.send(Ok(SubscribeResult {
                    successful: vec![super::super::command::ChannelSubscription {
                        channel: "ticker".to_string(),
                        sid: 100,
                    }],
                    failed: vec![],
                }));
            }
        });

        handle
            .subscribe(Channel::Ticker, &["MARKET"])
            .await
            .unwrap();

        // Should keep the concurrent subscription (999), not overwrite with ours (100)
        assert_eq!(handle.sid(Channel::Ticker), Some(999));

        responder.await.unwrap();
    }

    #[tokio::test]
    async fn test_sid_mismatch_skips_state_update() {
        // When SID changes between read and write, we should skip the update
        let (mut handle, mut cmd_receiver) = create_test_handle();
        add_subscription(&handle, Channel::Ticker, 100, &["EXISTING"]);

        let subscriptions = handle.subscriptions.clone();
        let responder = tokio::spawn(async move {
            if let Some(StreamCommand::UpdateSubscription { response, .. }) =
                cmd_receiver.recv().await
            {
                // Simulate SID change (e.g., unsubscribe/resubscribe by another task)
                {
                    let mut subs = subscriptions.write().unwrap();
                    if let Some(state) = subs.get_mut(&Channel::Ticker) {
                        state.sid = 999; // Changed!
                        state.markets = HashSet::from(["DIFFERENT".to_string()]);
                    }
                }

                // Return update for old SID 100
                let _ = response.send(Ok(vec!["EXISTING".to_string(), "NEW".to_string()]));
            }
        });

        handle.subscribe(Channel::Ticker, &["NEW"]).await.unwrap();

        // Should NOT have updated state because SID changed
        // State should still be what the "concurrent" operation set
        assert_eq!(handle.sid(Channel::Ticker), Some(999));
        let markets = handle.markets(Channel::Ticker);
        assert!(markets.contains(&"DIFFERENT".to_string()));
        assert!(!markets.contains(&"NEW".to_string()));

        responder.await.unwrap();
    }

    // ========== Original Tests ==========

    #[test]
    fn test_default_buffer_size() {
        assert_eq!(DEFAULT_BUFFER_SIZE, 1024);
    }

    #[tokio::test]
    async fn test_handle_clone_shares_subscriptions() {
        let (cmd_sender, _cmd_receiver) = mpsc::channel(32);
        let (update_sender, _) = broadcast::channel::<StreamUpdate>(16);
        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        let handle1 = KalshiStreamHandle {
            cmd_sender: cmd_sender.clone(),
            update_sender: update_sender.clone(),
            update_receiver: update_sender.subscribe(),
            subscriptions: subscriptions.clone(),
        };

        let handle2 = handle1.clone();

        // Modify subscriptions via handle1
        {
            let mut subs = handle1
                .subscriptions
                .write()
                .expect("subscription lock poisoned");
            subs.insert(
                Channel::Ticker,
                SubscriptionState {
                    sid: 123,
                    markets: HashSet::from(["MARKET-A".to_string()]),
                },
            );
        }

        // handle2 should see the same state
        assert!(handle2.is_subscribed(Channel::Ticker));
        assert_eq!(handle2.sid(Channel::Ticker), Some(123));
        assert_eq!(
            handle2.markets(Channel::Ticker),
            vec!["MARKET-A".to_string()]
        );
    }

    #[tokio::test]
    async fn test_handle_clone_gets_new_receiver() {
        let (cmd_sender, _cmd_receiver) = mpsc::channel(32);
        let (update_sender, _) = broadcast::channel::<StreamUpdate>(16);
        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        let handle1 = KalshiStreamHandle {
            cmd_sender: cmd_sender.clone(),
            update_sender: update_sender.clone(),
            update_receiver: update_sender.subscribe(),
            subscriptions,
        };

        let handle2 = handle1.clone();

        // Both handles should be able to receive the same message
        let test_update = StreamUpdate {
            channel: "ticker".to_string(),
            sid: 1,
            seq: Some(1),
            msg: super::super::message::StreamMessage::Ticker(super::super::message::TickerData {
                market_ticker: "TEST".to_string(),
                price: 50,
                yes_bid: 49,
                yes_ask: 51,
                volume: 100,
                open_interest: 50,
                dollar_volume: 5000,
                dollar_open_interest: 2500,
                ts: 1234567890,
                price_dollars: None,
                yes_bid_dollars: None,
                no_bid_dollars: None,
            }),
        };

        // Send a message
        update_sender.send(test_update.clone()).unwrap();

        // Both receivers should get it
        let mut handle1 = handle1;
        let mut handle2 = handle2;

        let received1 = handle1.update_receiver.recv().await.unwrap();
        let received2 = handle2.update_receiver.recv().await.unwrap();

        assert_eq!(received1.sid, 1);
        assert_eq!(received2.sid, 1);
    }
}
