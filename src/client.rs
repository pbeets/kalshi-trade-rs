mod http;
mod websocket;

pub use http::HttpClient;
pub use websocket::WebSocketClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Demo,
    Prod,
}

pub struct KalshiClient {
    _env: Environment,
}
