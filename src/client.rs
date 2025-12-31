mod http;
mod websocket;

pub use http::HttpClient;
pub use websocket::WebSocketClient;

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
