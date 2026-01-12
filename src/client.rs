mod http;
mod websocket;

pub use http::HttpClient;
pub use websocket::WebSocketClient;

use crate::{
    api::{
        communications, events, exchange, markets, order_groups, orders, portfolio, search, series,
    },
    auth::KalshiConfig,
    error::Result,
    models::{
        AcceptQuoteRequest, AmendOrderRequest, AmendOrderResponse, BalanceResponse,
        BatchCancelOrdersRequest, BatchCancelOrdersResponse, BatchCandlesticksResponse,
        BatchCreateOrdersRequest, BatchCreateOrdersResponse, CancelOrderResponse,
        CandlesticksResponse, CreateOrderGroupRequest, CreateOrderRequest, CreateQuoteRequest,
        CreateRfqRequest, DecreaseOrderRequest, EventResponse, EventsResponse,
        ExchangeAnnouncementsResponse, ExchangeScheduleResponse, ExchangeStatusResponse,
        FillsResponse, FiltersBySportResponse, GetBatchCandlesticksParams, GetCandlesticksParams,
        GetEventParams, GetEventsParams, GetFillsParams, GetMarketsParams, GetOrderbookParams,
        GetOrdersParams, GetPositionsParams, GetQueuePositionsParams, GetQuoteResponse,
        GetRfqResponse, GetSettlementsParams, GetTradesParams, ListQuotesParams,
        ListQuotesResponse, ListRfqsParams, ListRfqsResponse, MarketResponse, MarketsResponse,
        OrderGroupResponse, OrderQueuePositionResponse, OrderResponse, OrderbookResponse,
        OrdersResponse, PositionsResponse, QueuePositionsResponse, QuoteResponse, RfqResponse,
        SeriesListResponse, SeriesResponse, SettlementsResponse, TagsByCategoriesResponse,
        TradesResponse, UpdateOrderGroupRequest, UserDataTimestampResponse,
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

    /// Get queue positions for all resting orders.
    ///
    /// Queue position represents the number of contracts that need to be matched
    /// before an order receives a partial or full match, determined using
    /// price-time priority.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let positions = client.get_queue_positions().await?;
    /// for pos in positions.queue_positions {
    ///     println!("Order {} on {}: {} contracts ahead",
    ///         pos.order_id, pos.market_ticker, pos.queue_position);
    /// }
    /// ```
    pub async fn get_queue_positions(&self) -> Result<QueuePositionsResponse> {
        self.get_queue_positions_with_params(GetQueuePositionsParams::default())
            .await
    }

    /// Get queue positions for resting orders with filtering.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering by market or event ticker
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetQueuePositionsParams::new()
    ///     .event_ticker("KXBTC-25JAN");
    /// let positions = client.get_queue_positions_with_params(params).await?;
    /// ```
    pub async fn get_queue_positions_with_params(
        &self,
        params: GetQueuePositionsParams,
    ) -> Result<QueuePositionsResponse> {
        orders::get_queue_positions(&self.http, params).await
    }

    /// Get queue position for a specific order.
    ///
    /// Returns the number of contracts ahead of this order in the queue.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_order_queue_position("abc123").await?;
    /// println!("Contracts ahead: {}", response.queue_position);
    /// ```
    pub async fn get_order_queue_position(
        &self,
        order_id: &str,
    ) -> Result<OrderQueuePositionResponse> {
        orders::get_order_queue_position(&self.http, order_id).await
    }

    // =========================================================================
    // Order Groups API
    // =========================================================================

    /// Create a new order group.
    ///
    /// Creates multiple orders atomically as a group.
    ///
    /// # Arguments
    ///
    /// * `request` - The order group creation request
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{CreateOrderGroupRequest, CreateOrderGroupOrder, Side, Action};
    ///
    /// let orders = vec![
    ///     CreateOrderGroupOrder::new("KXBTC-25JAN", Side::Yes, Action::Buy, 10).yes_price(50),
    ///     CreateOrderGroupOrder::new("KXETH-25JAN", Side::Yes, Action::Buy, 5).yes_price(40),
    /// ];
    /// let request = CreateOrderGroupRequest { orders };
    /// let response = client.create_order_group(request).await?;
    /// ```
    pub async fn create_order_group(
        &self,
        request: CreateOrderGroupRequest,
    ) -> Result<OrderGroupResponse> {
        order_groups::create_order_group(&self.http, request).await
    }

    /// Get an order group by ID.
    ///
    /// # Arguments
    ///
    /// * `order_group_id` - The ID of the order group to retrieve
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_order_group("og_123").await?;
    /// ```
    pub async fn get_order_group(&self, order_group_id: &str) -> Result<OrderGroupResponse> {
        order_groups::get_order_group(&self.http, order_group_id).await
    }

    /// Update an existing order group.
    ///
    /// # Arguments
    ///
    /// * `order_group_id` - The ID of the order group to update
    /// * `request` - The update request containing orders to update or add
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{UpdateOrderGroupRequest, UpdateOrderGroupOrder, Side, Action};
    ///
    /// let orders = vec![
    ///     UpdateOrderGroupOrder::new("KXBTC-25JAN", Side::Yes, Action::Buy, 20)
    ///         .order_id("order_123")
    ///         .yes_price(55),
    /// ];
    /// let request = UpdateOrderGroupRequest { orders };
    /// let response = client.update_order_group("og_123", request).await?;
    /// ```
    pub async fn update_order_group(
        &self,
        order_group_id: &str,
        request: UpdateOrderGroupRequest,
    ) -> Result<OrderGroupResponse> {
        order_groups::update_order_group(&self.http, order_group_id, request).await
    }

    // =========================================================================
    // Candlesticks API
    // =========================================================================

    /// Get candlestick (OHLCV) data for a specific market.
    ///
    /// Returns historical price data in candlestick format.
    ///
    /// # Arguments
    ///
    /// * `series_ticker` - The series ticker containing the market
    /// * `ticker` - The market ticker
    /// * `params` - Query parameters including time range and interval
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{GetCandlesticksParams, CandlestickPeriod};
    ///
    /// let now = std::time::SystemTime::now()
    ///     .duration_since(std::time::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs() as i64;
    /// let one_day_ago = now - 86400;
    ///
    /// let params = GetCandlesticksParams::new(one_day_ago, now, CandlestickPeriod::OneHour);
    /// let candles = client.get_candlesticks("KXBTC", "KXBTC-25JAN10-B50000", params).await?;
    /// for candle in candles.candlesticks {
    ///     if let Some(price) = &candle.price {
    ///         println!("Close: {:?}", price.close_dollars);
    ///     }
    /// }
    /// ```
    pub async fn get_candlesticks(
        &self,
        series_ticker: &str,
        ticker: &str,
        params: GetCandlesticksParams,
    ) -> Result<CandlesticksResponse> {
        markets::get_candlesticks(&self.http, series_ticker, ticker, params).await
    }

    /// Get candlestick data for multiple markets in a single request.
    ///
    /// Supports up to 100 market tickers per request.
    /// Returns up to 10,000 candlesticks total across all markets.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters including tickers, time range, and interval
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{GetBatchCandlesticksParams, CandlestickPeriod};
    ///
    /// let now = std::time::SystemTime::now()
    ///     .duration_since(std::time::UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs() as i64;
    /// let one_day_ago = now - 86400;
    ///
    /// let params = GetBatchCandlesticksParams::from_tickers(
    ///     &["TICKER-1", "TICKER-2"],
    ///     one_day_ago,
    ///     now,
    ///     CandlestickPeriod::OneHour,
    /// );
    /// let response = client.get_batch_candlesticks(params).await?;
    /// for market in response.markets {
    ///     println!("{}: {} candlesticks", market.market_ticker, market.candlesticks.len());
    /// }
    /// ```
    pub async fn get_batch_candlesticks(
        &self,
        params: GetBatchCandlesticksParams,
    ) -> Result<BatchCandlesticksResponse> {
        markets::get_batch_candlesticks(&self.http, params).await
    }

    // =========================================================================
    // Settlements API
    // =========================================================================

    /// Get settlement history with default parameters.
    ///
    /// Returns historical settlement records showing P&L from settled markets.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let settlements = client.get_settlements().await?;
    /// for settlement in settlements.settlements {
    ///     println!("{}: revenue={} cents", settlement.ticker, settlement.revenue);
    /// }
    /// ```
    pub async fn get_settlements(&self) -> Result<SettlementsResponse> {
        self.get_settlements_with_params(GetSettlementsParams::default())
            .await
    }

    /// Get settlement history with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// let params = GetSettlementsParams::new()
    ///     .limit(50)
    ///     .event_ticker("KXBTC-25JAN");
    /// let settlements = client.get_settlements_with_params(params).await?;
    /// ```
    pub async fn get_settlements_with_params(
        &self,
        params: GetSettlementsParams,
    ) -> Result<SettlementsResponse> {
        portfolio::get_settlements(&self.http, params).await
    }

    // =========================================================================
    // Search API
    // =========================================================================

    /// Get tags organized by series categories.
    ///
    /// Returns a mapping of series categories to their associated tags,
    /// which can be used for filtering and search functionality.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tags = client.get_tags_by_categories().await?;
    /// for (category, category_tags) in &tags.tags_by_categories {
    ///     println!("{}: {:?}", category, category_tags);
    /// }
    /// ```
    pub async fn get_tags_by_categories(&self) -> Result<TagsByCategoriesResponse> {
        search::get_tags_by_categories(&self.http).await
    }

    /// Get filtering options organized by sport.
    ///
    /// Returns available scopes and competitions for each sport,
    /// along with a recommended display order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let filters = client.get_filters_by_sport().await?;
    /// for sport in &filters.sport_ordering {
    ///     if let Some(filter) = filters.filters_by_sports.get(sport) {
    ///         println!("{}: scopes={:?}", sport, filter.scopes);
    ///     }
    /// }
    /// ```
    pub async fn get_filters_by_sport(&self) -> Result<FiltersBySportResponse> {
        search::get_filters_by_sport(&self.http).await
    }

    // =========================================================================
    // Series API
    // =========================================================================

    /// Get details for a specific series by ticker.
    ///
    /// # Arguments
    ///
    /// * `series_ticker` - The series ticker (e.g., "KXBTC")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_series("KXBTC").await?;
    /// println!("Series: {}", response.series.title);
    /// ```
    pub async fn get_series(&self, series_ticker: &str) -> Result<SeriesResponse> {
        series::get_series(&self.http, series_ticker).await
    }

    /// Get a list of series with default parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_series_list().await?;
    /// for s in response.series {
    ///     println!("{} ({})", s.title, s.ticker);
    /// }
    /// ```
    pub async fn get_series_list(&self) -> Result<SeriesListResponse> {
        self.get_series_list_with_params(crate::models::GetSeriesParams::default())
            .await
    }

    /// Get a list of series with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::GetSeriesParams;
    ///
    /// let params = GetSeriesParams::new().category("crypto").include_volume(true);
    /// let response = client.get_series_list_with_params(params).await?;
    /// ```
    pub async fn get_series_list_with_params(
        &self,
        params: crate::models::GetSeriesParams,
    ) -> Result<SeriesListResponse> {
        series::get_series_list(&self.http, params).await
    }

    // =========================================================================
    // Communications API (RFQs and Quotes)
    // =========================================================================

    /// Create a new RFQ (Request for Quote).
    ///
    /// Submits an RFQ to the exchange to request quotes from market makers.
    ///
    /// # Arguments
    ///
    /// * `request` - The RFQ creation request containing ticker, size, and side
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::{CreateRfqRequest, Side};
    ///
    /// let request = CreateRfqRequest::new("KXBTC-25JAN", 100, Side::Yes);
    /// let response = client.create_rfq(request).await?;
    /// println!("RFQ ID: {}", response.rfq_id);
    /// ```
    pub async fn create_rfq(&self, request: CreateRfqRequest) -> Result<RfqResponse> {
        communications::create_rfq(&self.http, request).await
    }

    /// Create a new quote for an RFQ.
    ///
    /// Submits a quote in response to an existing RFQ.
    ///
    /// # Arguments
    ///
    /// * `request` - The quote creation request containing rfq_id, ticker, price, and count
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::CreateQuoteRequest;
    ///
    /// let request = CreateQuoteRequest::new("rfq_123", "KXBTC-25JAN", 50, 100);
    /// let response = client.create_quote(request).await?;
    /// println!("Quote ID: {}", response.quote_id);
    /// ```
    pub async fn create_quote(&self, request: CreateQuoteRequest) -> Result<QuoteResponse> {
        communications::create_quote(&self.http, request).await
    }

    /// Accept a quote.
    ///
    /// Accepts a specific quote for an RFQ, executing the trade.
    ///
    /// # Arguments
    ///
    /// * `quote_id` - The ID of the quote to accept
    /// * `request` - The acceptance request containing price and count
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::AcceptQuoteRequest;
    ///
    /// let request = AcceptQuoteRequest::new(50, 100);
    /// let response = client.accept_quote("quote_123", request).await?;
    /// ```
    pub async fn accept_quote(
        &self,
        quote_id: &str,
        request: AcceptQuoteRequest,
    ) -> Result<QuoteResponse> {
        communications::accept_quote(&self.http, quote_id, request).await
    }

    /// Cancel an RFQ.
    ///
    /// Cancels an existing RFQ that has not yet been filled.
    ///
    /// # Arguments
    ///
    /// * `rfq_id` - The ID of the RFQ to cancel
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.cancel_rfq("rfq_123").await?;
    /// ```
    pub async fn cancel_rfq(&self, rfq_id: &str) -> Result<RfqResponse> {
        communications::cancel_rfq(&self.http, rfq_id).await
    }

    /// Cancel a quote.
    ///
    /// Cancels an existing quote that has not yet been accepted.
    ///
    /// # Arguments
    ///
    /// * `quote_id` - The ID of the quote to cancel
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.cancel_quote("quote_123").await?;
    /// ```
    pub async fn cancel_quote(&self, quote_id: &str) -> Result<QuoteResponse> {
        communications::cancel_quote(&self.http, quote_id).await
    }

    /// Get details of an RFQ.
    ///
    /// # Arguments
    ///
    /// * `rfq_id` - The ID of the RFQ to retrieve
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_rfq("rfq_123").await?;
    /// println!("RFQ market: {}", response.rfq.market_ticker);
    /// ```
    pub async fn get_rfq(&self, rfq_id: &str) -> Result<GetRfqResponse> {
        communications::get_rfq(&self.http, rfq_id).await
    }

    /// Get details of a quote.
    ///
    /// # Arguments
    ///
    /// * `quote_id` - The ID of the quote to retrieve
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.get_quote("quote_123").await?;
    /// println!("Quote yes bid: {}", response.quote.yes_bid);
    /// ```
    pub async fn get_quote(&self, quote_id: &str) -> Result<GetQuoteResponse> {
        communications::get_quote(&self.http, quote_id).await
    }

    /// List RFQs with default parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.list_rfqs().await?;
    /// for rfq in response.rfqs {
    ///     println!("{}: {}", rfq.id, rfq.market_ticker);
    /// }
    /// ```
    pub async fn list_rfqs(&self) -> Result<ListRfqsResponse> {
        self.list_rfqs_with_params(ListRfqsParams::default()).await
    }

    /// List RFQs with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::ListRfqsParams;
    ///
    /// let params = ListRfqsParams::new()
    ///     .market_ticker("KXBTC-25JAN")
    ///     .limit(50);
    /// let response = client.list_rfqs_with_params(params).await?;
    /// ```
    pub async fn list_rfqs_with_params(&self, params: ListRfqsParams) -> Result<ListRfqsResponse> {
        communications::list_rfqs(&self.http, params).await
    }

    /// List quotes with default parameters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let response = client.list_quotes().await?;
    /// for quote in response.quotes {
    ///     println!("{}: {} @ {}", quote.quote_id, quote.market_ticker, quote.yes_bid);
    /// }
    /// ```
    pub async fn list_quotes(&self) -> Result<ListQuotesResponse> {
        self.list_quotes_with_params(ListQuotesParams::default())
            .await
    }

    /// List quotes with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters for filtering and pagination
    ///
    /// # Example
    ///
    /// ```ignore
    /// use kalshi_trade_rs::ListQuotesParams;
    ///
    /// let params = ListQuotesParams::new()
    ///     .rfq_id("rfq_123")
    ///     .limit(100);
    /// let response = client.list_quotes_with_params(params).await?;
    /// ```
    pub async fn list_quotes_with_params(
        &self,
        params: ListQuotesParams,
    ) -> Result<ListQuotesResponse> {
        communications::list_quotes(&self.http, params).await
    }
}
