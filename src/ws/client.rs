//! WebSocket streaming client for Kalshi API.

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};

use super::{
    ConnectStrategy,
    actor::KalshiStreamSession,
    channel::Channel,
    command::{StreamCommand, SubscribeResult},
    message::StreamUpdate,
};
use crate::{
    auth::KalshiConfig,
    error::{Error, Result},
};

/// Default buffer size for the broadcast channel.
const DEFAULT_BUFFER_SIZE: usize = 1024;

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
/// // Simple connection (fast-fail)
/// let client = KalshiStreamClient::connect(&config).await?;
///
/// // Or with retry for production
/// let client = KalshiStreamClient::connect_with_strategy(
///     &config,
///     ConnectStrategy::Retry,
/// ).await?;
///
/// let mut handle = client.handle();
///
/// // Subscribe to ticker updates
/// let result = handle.subscribe(
///     &[Channel::Ticker],
///     Some(&["INXD-25JAN17-B5955"]),
/// ).await?;
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
        })
    }

    /// Get a cloneable handle for interacting with the stream.
    ///
    /// Handles can be cloned and shared across tasks. Each cloned handle
    /// gets its own broadcast receiver that starts receiving from the
    /// point of subscription.
    pub fn handle(&self) -> KalshiStreamHandle {
        KalshiStreamHandle {
            cmd_sender: self.cmd_sender.clone(),
            update_sender: self.update_sender.clone(),
            update_receiver: self.update_sender.subscribe(),
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
/// maintains its own broadcast receiver for receiving updates.
///
/// # Example
///
/// ```no_run
/// use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle, StreamMessage};
///
/// async fn subscribe_and_process(handle: KalshiStreamHandle) {
///     let mut handle = handle;
///
///     // Subscribe to orderbook updates
///     let result = handle.subscribe(
///         &[Channel::OrderbookDelta],
///         Some(&["BTCUSD-25JAN17"]),
///     ).await.unwrap();
///
///     println!("Subscribed with SIDs: {:?}", result.sids());
///
///     // Process updates - handle disconnection
///     while let Ok(update) = handle.update_receiver.recv().await {
///         match &update.msg {
///             StreamMessage::Closed { reason } => {
///                 println!("Connection closed: {}", reason);
///                 break;
///             }
///             StreamMessage::ConnectionLost { reason } => {
///                 eprintln!("Connection lost: {}", reason);
///                 break; // Implement reconnection logic here
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
}

impl Clone for KalshiStreamHandle {
    fn clone(&self) -> Self {
        Self {
            cmd_sender: self.cmd_sender.clone(),
            update_sender: self.update_sender.clone(),
            update_receiver: self.update_sender.subscribe(),
        }
    }
}

impl KalshiStreamHandle {
    /// Subscribe to channels for specific market tickers.
    ///
    /// # Arguments
    ///
    /// * `channels` - The channels to subscribe to (e.g., Ticker, OrderbookDelta).
    /// * `market_tickers` - List of market tickers to subscribe to. Most channels
    ///   require at least one ticker. Pass `None` or an empty slice only for
    ///   channels that support subscription without tickers (e.g., `fill`,
    ///   `communications`).
    ///
    /// # Returns
    ///
    /// Returns a [`SubscribeResult`] containing the subscription IDs (sids)
    /// assigned by the server, which can be used for unsubscribing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command channel is closed (actor has shut down)
    /// - The server rejects the subscription request
    /// - No market tickers provided for a channel that requires them
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::ws::{Channel, KalshiStreamHandle};
    /// # async fn example(handle: &KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// // Subscribe to ticker and trade channels for specific markets
    /// let result = handle.subscribe(
    ///     &[Channel::Ticker, Channel::Trade],
    ///     Some(&["BTCUSD-25JAN17", "ETHUSD-25JAN17"]),
    /// ).await?;
    ///
    /// println!("Subscription IDs: {:?}", result.sids());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(
        &self,
        channels: &[Channel],
        market_tickers: Option<&[&str]>,
    ) -> Result<SubscribeResult> {
        // Validate: channels requiring market tickers must have at least one
        let has_tickers = market_tickers.is_some_and(|t| !t.is_empty());

        if !has_tickers {
            let requiring: Vec<&str> = channels
                .iter()
                .filter(|c| c.requires_market_ticker())
                .map(|c| c.as_str())
                .collect();

            if !requiring.is_empty() {
                return Err(Error::MissingMarketTickers(requiring.join(", ")));
            }
        }

        let (tx, rx) = oneshot::channel();

        let channel_strings: Vec<String> =
            channels.iter().map(|c| c.as_str().to_string()).collect();

        let ticker_strings: Vec<String> = market_tickers
            .map(|tickers| tickers.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let cmd = StreamCommand::Subscribe {
            channels: channel_strings,
            market_tickers: ticker_strings,
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

    /// Unsubscribe from subscriptions by their IDs.
    ///
    /// # Arguments
    ///
    /// * `sids` - The subscription IDs to unsubscribe from, as returned
    ///   by [`subscribe`](Self::subscribe).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The command channel is closed (actor has shut down)
    /// - The server rejects the unsubscribe request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use kalshi_trade_rs::ws::KalshiStreamHandle;
    /// # async fn example(handle: &KalshiStreamHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// // Unsubscribe from specific subscription IDs
    /// handle.unsubscribe(&[123, 456]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&self, sids: &[i64]) -> Result<()> {
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
            .map_err(Error::Api)?;

        Ok(())
    }

    /// Request graceful close of the connection.
    ///
    /// This sends a close command to the actor but does not wait for
    /// the connection to fully close. For a blocking shutdown, use
    /// [`KalshiStreamClient::shutdown`] instead.
    pub async fn close(&self) {
        let _ = self.cmd_sender.send(StreamCommand::Close).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_buffer_size() {
        assert_eq!(DEFAULT_BUFFER_SIZE, 1024);
    }

    #[tokio::test]
    async fn test_handle_clone_gets_new_receiver() {
        let (cmd_sender, _cmd_receiver) = mpsc::channel(32);
        let (update_sender, _) = broadcast::channel::<StreamUpdate>(16);

        let handle1 = KalshiStreamHandle {
            cmd_sender: cmd_sender.clone(),
            update_sender: update_sender.clone(),
            update_receiver: update_sender.subscribe(),
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
