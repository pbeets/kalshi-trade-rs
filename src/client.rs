mod http;
mod websocket;

pub use http::HttpClient;
pub use websocket::WebSocketClient;

use crate::{
    api::{events, exchange, markets, orders, portfolio},
    auth::KalshiConfig,
    error::Result,
    models::{
        AmendOrderRequest, AmendOrderResponse, BalanceResponse, BatchCancelOrdersRequest,
        BatchCancelOrdersResponse, BatchCreateOrdersRequest, BatchCreateOrdersResponse,
        CancelOrderResponse, CreateOrderRequest, DecreaseOrderRequest, EventResponse,
        EventsResponse, ExchangeAnnouncementsResponse, ExchangeScheduleResponse,
        ExchangeStatusResponse, FillsResponse, GetEventParams, GetEventsParams, GetFillsParams,
        GetMarketsParams, GetOrderbookParams, GetOrdersParams, GetPositionsParams, GetTradesParams,
        MarketResponse, MarketsResponse, OrderResponse, OrderbookResponse, OrdersResponse,
        PositionsResponse, TradesResponse, UserDataTimestampResponse,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    #[default]
    Demo,
    Prod,
}

impl Environment {
    pub fn base_url(&self) -> &'static str {
        match self {
            Environment::Demo => "https://demo-api.kalshi.co/trade-api/v2",
            Environment::Prod => "https://trading-api.kalshi.com/trade-api/v2",
        }
    }

    pub fn ws_url(&self) -> &'static str {
        match self {
            Environment::Demo => "wss://demo-api.kalshi.co/trade-api/ws/v2",
            Environment::Prod => "wss://trading-api.kalshi.com/trade-api/ws/v2",
        }
    }

    pub fn api_path_prefix(&self) -> &'static str {
        "/trade-api/v2"
    }
}

/// The main Kalshi API client.
///
/// This is the primary entry point for interacting with the Kalshi API.
/// Methods are provided directly on the client, matching the Python SDK style.
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::{KalshiClient, KalshiConfig};
///
/// // Create configuration from environment variables
/// let config = KalshiConfig::from_env()?;
///
/// // Create the client
/// let client = KalshiClient::new(config)?;
///
/// // Get balance
/// let balance = client.get_balance().await?;
/// println!("Balance: ${:.2}", balance.balance as f64 / 100.0);
///
/// // Get positions
/// let positions = client.get_positions().await?;
/// for pos in positions.market_positions {
///     println!("{}: {} contracts", pos.ticker, pos.position);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct KalshiClient {
    http: HttpClient,
}

impl KalshiClient {
    /// Create a new Kalshi client with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The Kalshi configuration containing credentials and environment
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(config: KalshiConfig) -> Result<Self> {
        let http = HttpClient::new(config)?;
        Ok(Self { http })
    }

    /// Get the underlying HTTP client for advanced usage.
    ///
    /// This allows direct access to make custom API calls.
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the current environment (Demo or Prod).
    pub fn environment(&self) -> Environment {
        self.http.environment()
    }

    // =========================================================================
    // Portfolio API
    // =========================================================================

    /// Get the current account balance.
    ///
    /// Returns the available balance and portfolio value in cents.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let balance = client.get_balance().await?;
    /// println!("Balance: ${:.2}", balance.balance as f64 / 100.0);
    /// ```
    pub async fn get_balance(&self) -> Result<BalanceResponse> {
        portfolio::get_balance(&self.http).await
    }

    /// Get all positions with default parameters.
    ///
    /// Returns both market-level and event-level positions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let positions = client.get_positions().await?;
    /// for pos in positions.market_positions {
    ///     println!("{}: {} contracts", pos.ticker, pos.position);
    /// }
    /// ```
    pub async fn get_positions(&self) -> Result<PositionsResponse> {
        self.get_positions_with_params(GetPositionsParams::default())
            .await
    }

    /// Get positions with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetPositionsParams::new()
    ///     .ticker("AAPL-25JAN")
    ///     .limit(50);
    /// let positions = client.get_positions_with_params(params).await?;
    /// ```
    pub async fn get_positions_with_params(
        &self,
        params: GetPositionsParams,
    ) -> Result<PositionsResponse> {
        portfolio::get_positions(&self.http, params).await
    }

    /// Get all fills with default parameters.
    ///
    /// A fill represents a matched trade execution.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let fills = client.get_fills().await?;
    /// for fill in fills.fills {
    ///     println!("Fill {}: {} @ {} cents", fill.fill_id, fill.count, fill.yes_price);
    /// }
    /// ```
    pub async fn get_fills(&self) -> Result<FillsResponse> {
        self.get_fills_with_params(GetFillsParams::default()).await
    }

    /// Get fills with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetFillsParams::new()
    ///     .ticker("AAPL-25JAN")
    ///     .limit(100);
    /// let fills = client.get_fills_with_params(params).await?;
    /// ```
    pub async fn get_fills_with_params(&self, params: GetFillsParams) -> Result<FillsResponse> {
        portfolio::get_fills(&self.http, params).await
    }

    /// Get all orders with default parameters.
    ///
    /// Returns orders in all states (resting, executed, canceled).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let orders = client.get_orders().await?;
    /// for order in orders.orders {
    ///     println!("Order {}: {:?}", order.order_id, order.status);
    /// }
    /// ```
    pub async fn get_orders(&self) -> Result<OrdersResponse> {
        self.get_orders_with_params(GetOrdersParams::default())
            .await
    }

    /// Get orders with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::OrderStatus;
    ///
    /// let params = GetOrdersParams::new()
    ///     .status(OrderStatus::Resting)
    ///     .limit(50);
    /// let orders = client.get_orders_with_params(params).await?;
    /// ```
    pub async fn get_orders_with_params(&self, params: GetOrdersParams) -> Result<OrdersResponse> {
        portfolio::get_orders(&self.http, params).await
    }

    // =========================================================================
    // Exchange API
    // =========================================================================

    /// Get the current exchange status.
    ///
    /// Returns whether the exchange and trading are currently active.
    /// This is useful for checking if the exchange is in maintenance mode
    /// or if trading is paused.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = client.get_exchange_status().await?;
    /// if status.trading_active {
    ///     println!("Trading is open!");
    /// } else {
    ///     println!("Trading is closed");
    ///     if let Some(resume_time) = status.exchange_estimated_resume_time {
    ///         println!("Estimated resume: {}", resume_time);
    ///     }
    /// }
    /// ```
    pub async fn get_exchange_status(&self) -> Result<ExchangeStatusResponse> {
        exchange::get_exchange_status(&self.http).await
    }

    /// Get the exchange schedule.
    ///
    /// Returns the weekly trading schedule and any scheduled maintenance windows.
    /// All times are in Eastern Time (ET).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let schedule = client.get_exchange_schedule().await?;
    /// for period in &schedule.schedule.standard_hours {
    ///     for session in &period.monday {
    ///         println!("Monday: {} - {}", session.open_time, session.close_time);
    ///     }
    /// }
    /// ```
    pub async fn get_exchange_schedule(&self) -> Result<ExchangeScheduleResponse> {
        exchange::get_exchange_schedule(&self.http).await
    }

    /// Get exchange announcements.
    ///
    /// Returns all exchange-wide announcements including info, warning, and error messages.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::AnnouncementStatus;
    ///
    /// let announcements = client.get_exchange_announcements().await?;
    /// for ann in announcements.announcements {
    ///     if ann.status == AnnouncementStatus::Active {
    ///         println!("[{:?}] {}", ann.announcement_type, ann.message);
    ///     }
    /// }
    /// ```
    pub async fn get_exchange_announcements(&self) -> Result<ExchangeAnnouncementsResponse> {
        exchange::get_exchange_announcements(&self.http).await
    }

    /// Get the user data timestamp.
    ///
    /// Returns an approximate indication of when user portfolio data
    /// (balance, orders, fills, positions) was last validated.
    /// Useful for determining data freshness when there may be delays
    /// in reflecting recent trades.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let timestamp = client.get_user_data_timestamp().await?;
    /// println!("Data as of: {}", timestamp.as_of_time);
    /// ```
    pub async fn get_user_data_timestamp(&self) -> Result<UserDataTimestampResponse> {
        exchange::get_user_data_timestamp(&self.http).await
    }

    // =========================================================================
    // Markets API
    // =========================================================================

    /// Get a list of markets with default parameters.
    ///
    /// Returns up to 100 markets. Use `get_markets_with_params` for filtering
    /// and pagination.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let markets = client.get_markets().await?;
    /// for market in markets.markets {
    ///     println!("{}: {}", market.ticker, market.title.unwrap_or_default());
    /// }
    /// ```
    pub async fn get_markets(&self) -> Result<MarketsResponse> {
        self.get_markets_with_params(GetMarketsParams::default())
            .await
    }

    /// Get a list of markets with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::MarketFilterStatus;
    ///
    /// let params = GetMarketsParams::new()
    ///     .status(MarketFilterStatus::Open)
    ///     .limit(50);
    /// let markets = client.get_markets_with_params(params).await?;
    /// ```
    pub async fn get_markets_with_params(
        &self,
        params: GetMarketsParams,
    ) -> Result<MarketsResponse> {
        markets::get_markets(&self.http, params).await
    }

    /// Get details for a specific market by ticker.
    ///
    /// # Arguments
    ///
    /// * `ticker` - The market ticker (e.g., "KXBTC-25JAN10-B50000")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let market = client.get_market("KXBTC-25JAN10-B50000").await?;
    /// println!("Status: {:?}", market.market.status);
    /// println!("Last price: {}", market.market.last_price_dollars.unwrap_or_default());
    /// ```
    pub async fn get_market(&self, ticker: &str) -> Result<MarketResponse> {
        markets::get_market(&self.http, ticker).await
    }

    /// Get the orderbook for a market with default depth.
    ///
    /// Returns all price levels in the orderbook.
    ///
    /// # Arguments
    ///
    /// * `ticker` - The market ticker
    ///
    /// # Example
    ///
    /// ```ignore
    /// let orderbook = client.get_orderbook("KXBTC-25JAN10-B50000").await?;
    /// for level in &orderbook.orderbook.yes {
    ///     println!("YES: {} @ {} cents", level[1], level[0]);
    /// }
    /// ```
    pub async fn get_orderbook(&self, ticker: &str) -> Result<OrderbookResponse> {
        self.get_orderbook_with_params(ticker, GetOrderbookParams::default())
            .await
    }

    /// Get the orderbook for a market with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `ticker` - The market ticker
    /// * `params` - Query parameters (e.g., depth)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetOrderbookParams::new().depth(10);
    /// let orderbook = client.get_orderbook_with_params("KXBTC-25JAN10-B50000", params).await?;
    /// ```
    pub async fn get_orderbook_with_params(
        &self,
        ticker: &str,
        params: GetOrderbookParams,
    ) -> Result<OrderbookResponse> {
        markets::get_orderbook(&self.http, ticker, params).await
    }

    /// Get trades with default parameters.
    ///
    /// Returns the most recent trades on the exchange.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let trades = client.get_trades().await?;
    /// for trade in trades.trades {
    ///     println!("{}: {} contracts @ {} cents ({:?})",
    ///         trade.ticker, trade.count, trade.yes_price, trade.taker_side);
    /// }
    /// ```
    pub async fn get_trades(&self) -> Result<TradesResponse> {
        self.get_trades_with_params(GetTradesParams::default())
            .await
    }

    /// Get trades with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetTradesParams::new()
    ///     .ticker("KXBTC-25JAN10-B50000")
    ///     .limit(100);
    /// let trades = client.get_trades_with_params(params).await?;
    /// ```
    pub async fn get_trades_with_params(&self, params: GetTradesParams) -> Result<TradesResponse> {
        markets::get_trades(&self.http, params).await
    }

    // =========================================================================
    // Events API
    // =========================================================================

    /// Get a list of events with default parameters.
    ///
    /// Returns up to 200 events. Use `get_events_with_params` for filtering
    /// and pagination. Note: This excludes multivariate events.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = client.get_events().await?;
    /// for event in events.events {
    ///     println!("{}: {}", event.event_ticker, event.title);
    /// }
    /// ```
    pub async fn get_events(&self) -> Result<EventsResponse> {
        self.get_events_with_params(GetEventsParams::default())
            .await
    }

    /// Get a list of events with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::EventStatus;
    ///
    /// let params = GetEventsParams::new()
    ///     .status(EventStatus::Open)
    ///     .with_nested_markets(true)
    ///     .limit(50);
    /// let events = client.get_events_with_params(params).await?;
    /// ```
    pub async fn get_events_with_params(&self, params: GetEventsParams) -> Result<EventsResponse> {
        events::get_events(&self.http, params).await
    }

    /// Get details for a specific event by ticker.
    ///
    /// # Arguments
    ///
    /// * `event_ticker` - The event ticker
    ///
    /// # Example
    ///
    /// ```ignore
    /// let event = client.get_event("KXBTC-25JAN").await?;
    /// println!("{}: {}", event.event.event_ticker, event.event.title);
    /// ```
    pub async fn get_event(&self, event_ticker: &str) -> Result<EventResponse> {
        self.get_event_with_params(event_ticker, GetEventParams::default())
            .await
    }

    /// Get details for a specific event with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `event_ticker` - The event ticker
    /// * `params` - Query parameters (e.g., with_nested_markets)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetEventParams::new().with_nested_markets(true);
    /// let event = client.get_event_with_params("KXBTC-25JAN", params).await?;
    /// if let Some(markets) = &event.event.markets {
    ///     for market in markets {
    ///         println!("  Market: {}", market.ticker);
    ///     }
    /// }
    /// ```
    pub async fn get_event_with_params(
        &self,
        event_ticker: &str,
        params: GetEventParams,
    ) -> Result<EventResponse> {
        events::get_event(&self.http, event_ticker, params).await
    }

    // =========================================================================
    // Orders API
    // =========================================================================

    /// Create a new order.
    ///
    /// Submits an order to the exchange. Use the builder pattern on
    /// `CreateOrderRequest` to set optional fields.
    ///
    /// # Arguments
    ///
    /// * `request` - The order creation request
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{CreateOrderRequest, Side, Action};
    ///
    /// let request = CreateOrderRequest::new("KXBTC-25JAN10-B50000", Side::Yes, Action::Buy, 10)
    ///     .yes_price(50)
    ///     .post_only(true);
    /// let response = client.create_order(request).await?;
    /// println!("Order created: {}", response.order.order_id);
    /// ```
    pub async fn create_order(&self, request: CreateOrderRequest) -> Result<OrderResponse> {
        orders::create_order(&self.http, request).await
    }

    /// Get a specific order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID
    ///
    /// # Example
    ///
    /// ```ignore
    /// let order = client.get_order("abc123").await?;
    /// println!("Order status: {:?}", order.order.status);
    /// ```
    pub async fn get_order(&self, order_id: &str) -> Result<OrderResponse> {
        orders::get_order(&self.http, order_id).await
    }

    /// Cancel an order by ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to cancel
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.cancel_order("abc123").await?;
    /// println!("Canceled {} contracts", response.reduced_by.unwrap_or(0));
    /// ```
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse> {
        orders::cancel_order(&self.http, order_id).await
    }

    /// Amend an existing order.
    ///
    /// Modifies the price and/or quantity of an existing order.
    /// The order is canceled and replaced atomically.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to amend
    /// * `request` - The amendment details
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{AmendOrderRequest, Side, Action};
    ///
    /// let request = AmendOrderRequest::new(
    ///     "KXBTC-25JAN10-B50000",
    ///     Side::Yes,
    ///     Action::Buy,
    ///     "my-order-1",
    ///     "my-order-2",
    /// ).yes_price(55);
    /// let response = client.amend_order("abc123", request).await?;
    /// println!("Amended: old price={}, new price={}",
    ///     response.old_order.yes_price, response.order.yes_price);
    /// ```
    pub async fn amend_order(
        &self,
        order_id: &str,
        request: AmendOrderRequest,
    ) -> Result<AmendOrderResponse> {
        orders::amend_order(&self.http, order_id, request).await
    }

    /// Decrease an order's quantity.
    ///
    /// Reduces the remaining quantity of an order without canceling it entirely.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID
    /// * `request` - The decrease details (reduce_by or reduce_to)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::DecreaseOrderRequest;
    ///
    /// // Reduce by 5 contracts
    /// let request = DecreaseOrderRequest::reduce_by(5);
    /// let response = client.decrease_order("abc123", request).await?;
    /// println!("Remaining: {} contracts", response.order.remaining_count);
    /// ```
    pub async fn decrease_order(
        &self,
        order_id: &str,
        request: DecreaseOrderRequest,
    ) -> Result<OrderResponse> {
        orders::decrease_order(&self.http, order_id, request).await
    }

    /// Create multiple orders in a single request.
    ///
    /// Supports up to 20 orders per batch. Each order in the batch is
    /// processed independently, so some may succeed while others fail.
    ///
    /// # Arguments
    ///
    /// * `request` - The batch create request containing orders
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{BatchCreateOrdersRequest, CreateOrderRequest, Side, Action};
    ///
    /// let orders = vec![
    ///     CreateOrderRequest::new("KXBTC-25JAN10-B50000", Side::Yes, Action::Buy, 5)
    ///         .yes_price(50),
    ///     CreateOrderRequest::new("KXBTC-25JAN10-B55000", Side::No, Action::Buy, 3)
    ///         .no_price(40),
    /// ];
    /// let response = client.batch_create_orders(BatchCreateOrdersRequest::new(orders)).await?;
    /// for result in response.orders {
    ///     if let Some(order) = result.order {
    ///         println!("Created order: {}", order.order_id);
    ///     } else if let Some(error) = result.error {
    ///         println!("Failed: {}", error.message);
    ///     }
    /// }
    /// ```
    pub async fn batch_create_orders(
        &self,
        request: BatchCreateOrdersRequest,
    ) -> Result<BatchCreateOrdersResponse> {
        orders::batch_create_orders(&self.http, request).await
    }

    /// Cancel multiple orders in a single request.
    ///
    /// Supports up to 20 orders per batch. Each order in the batch is
    /// processed independently.
    ///
    /// # Arguments
    ///
    /// * `request` - The batch cancel request containing order IDs
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::BatchCancelOrdersRequest;
    ///
    /// let order_ids = vec!["order1".to_string(), "order2".to_string()];
    /// let response = client.batch_cancel_orders(BatchCancelOrdersRequest::new(order_ids)).await?;
    /// for result in response.orders {
    ///     if let Some(order) = result.order {
    ///         println!("Canceled order: {}", order.order_id);
    ///     }
    /// }
    /// ```
    pub async fn batch_cancel_orders(
        &self,
        request: BatchCancelOrdersRequest,
    ) -> Result<BatchCancelOrdersResponse> {
        orders::batch_cancel_orders(&self.http, request).await
    }
}
