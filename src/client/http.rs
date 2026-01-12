use reqwest::{Client, Method, RequestBuilder, Response, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    auth::KalshiConfig,
    client::Environment,
    error::{Error, Result},
};

/// HTTP client for making authenticated requests to the Kalshi API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    base_url: String,
    config: KalshiConfig,
}

impl HttpClient {
    /// Create a new authenticated HTTP client.
    ///
    /// # Arguments
    /// * `config` - The Kalshi configuration containing environment and credentials
    pub fn new(config: KalshiConfig) -> Result<Self> {
        let client = Client::builder().build().map_err(Error::Http)?;

        Ok(Self {
            base_url: config.environment.base_url().to_string(),
            client,
            config,
        })
    }

    /// Get the current timestamp in milliseconds.
    fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX epoch")
            .as_millis() as u64
    }

    /// Build authentication headers for a request.
    ///
    /// # Arguments
    /// * `method` - The HTTP method
    /// * `path` - The API path (without base URL, e.g., "/portfolio/balance")
    fn auth_headers(&self, method: &Method, path: &str) -> Result<HeaderMap> {
        let timestamp_ms = Self::current_timestamp_ms();

        // The path for signing includes the API prefix but NOT query parameters
        let path_without_query = path.split('?').next().unwrap_or(path);
        let sign_path = format!(
            "{}{}",
            self.config.environment.api_path_prefix(),
            path_without_query
        );

        let signature = self
            .config
            .sign(timestamp_ms, method.as_str(), &sign_path)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "KALSHI-ACCESS-KEY",
            self.config
                .api_key_id()
                .parse()
                .map_err(|e| Error::InvalidHeaderValue(format!("api_key_id: {}", e)))?,
        );
        headers.insert(
            "KALSHI-ACCESS-TIMESTAMP",
            timestamp_ms
                .to_string()
                .parse()
                .map_err(|e| Error::InvalidHeaderValue(format!("timestamp: {}", e)))?,
        );
        headers.insert(
            "KALSHI-ACCESS-SIGNATURE",
            signature
                .parse()
                .map_err(|e| Error::InvalidHeaderValue(format!("signature: {}", e)))?,
        );

        Ok(headers)
    }

    /// Build a request with authentication headers.
    fn build_request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = format!("{}{}", self.base_url, path);
        let headers = self.auth_headers(&method, path)?;

        Ok(self.client.request(method, &url).headers(headers))
    }

    /// Execute a request and handle the response.
    async fn execute(&self, request: RequestBuilder) -> Result<Response> {
        let response = request.send().await.map_err(Error::Http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(Error::Api(format!("{}: {}", status, body)))
        }
    }

    /// Make a GET request and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path (e.g., "/portfolio/balance")
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let request = self.build_request(Method::GET, path)?;
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a POST request with a JSON body and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path (e.g., "/portfolio/orders")
    /// * `body` - The request body to serialize as JSON
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let request = self.build_request(Method::POST, path)?.json(body);
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a POST request with a JSON body, expecting no response body.
    ///
    /// # Arguments
    /// * `path` - The API path
    /// * `body` - The request body to serialize as JSON
    pub async fn post_no_response<B: Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let request = self.build_request(Method::POST, path)?.json(body);
        self.execute(request).await?;
        Ok(())
    }

    /// Make a DELETE request.
    ///
    /// # Arguments
    /// * `path` - The API path (e.g., "/portfolio/orders/{order_id}")
    pub async fn delete(&self, path: &str) -> Result<()> {
        let request = self.build_request(Method::DELETE, path)?;
        self.execute(request).await?;
        Ok(())
    }

    /// Make a DELETE request and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path
    pub async fn delete_with_response<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let request = self.build_request(Method::DELETE, path)?;
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a DELETE request with a JSON body and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path
    /// * `body` - The request body to serialize as JSON
    pub async fn delete_with_body<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let request = self.build_request(Method::DELETE, path)?.json(body);
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a PUT request with a JSON body and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path
    /// * `body` - The request body to serialize as JSON
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let request = self.build_request(Method::PUT, path)?.json(body);
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a PUT request with an empty body and deserialize the response.
    ///
    /// # Arguments
    /// * `path` - The API path
    pub async fn put_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let request = self.build_request(Method::PUT, path)?;
        let response = self.execute(request).await?;
        response.json::<T>().await.map_err(Error::Http)
    }

    /// Make a PUT request with an empty body, expecting no response body.
    ///
    /// Suitable for endpoints that return 204 No Content.
    ///
    /// # Arguments
    /// * `path` - The API path
    pub async fn put_no_content(&self, path: &str) -> Result<()> {
        let request = self.build_request(Method::PUT, path)?;
        self.execute(request).await?;
        Ok(())
    }

    /// Get the base URL for this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the environment for this client.
    pub fn environment(&self) -> Environment {
        self.config.environment
    }
}
