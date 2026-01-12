//! API Keys Management endpoints.
//!
//! This module provides functions for managing API keys programmatically.
//! Note: API keys are typically managed via the Kalshi web UI, but these
//! endpoints allow programmatic management for advanced use cases.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        ApiKeysResponse, CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyResponse,
        GenerateApiKeyRequest, GenerateApiKeyResponse,
    },
};

/// URL-encode a string for use in path segments.
fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// List all API keys for the authenticated user.
///
/// Returns all API keys associated with the account.
///
/// # Example
///
/// ```ignore
/// let response = client.get_api_keys().await?;
/// for key in response.api_keys {
///     println!("{}: {:?}", key.api_key.unwrap_or_default(), key.name);
/// }
/// ```
pub async fn get_api_keys(http: &HttpClient) -> Result<ApiKeysResponse> {
    http.get("/api_keys").await
}

/// Create a new API key with a user-provided public key.
///
/// Creates an API key using the provided RSA public key.
/// Available for Premier/Market Maker tier users.
///
/// # Arguments
///
/// * `request` - The API key creation request containing name and public key
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::CreateApiKeyRequest;
///
/// let public_key = std::fs::read_to_string("my_public_key.pem")?;
/// let request = CreateApiKeyRequest::new("My Trading Bot", public_key);
/// let response = client.create_api_key(request).await?;
/// println!("Created API key: {:?}", response.api_key.api_key);
/// ```
pub async fn create_api_key(
    http: &HttpClient,
    request: CreateApiKeyRequest,
) -> Result<CreateApiKeyResponse> {
    http.post("/api_keys", &request).await
}

/// Generate a new API key with platform-generated keypair.
///
/// Creates an API key with a platform-generated RSA keypair.
/// The private key is returned only once and cannot be retrieved later.
///
/// **Important**: Store the returned private key securely. It will not
/// be available again after this call.
///
/// # Arguments
///
/// * `request` - The API key generation request containing the key name
///
/// # Example
///
/// ```ignore
/// use kalshi_trade_rs::GenerateApiKeyRequest;
///
/// let request = GenerateApiKeyRequest::new("My Trading Bot");
/// let response = client.generate_api_key(request).await?;
/// println!("API Key: {:?}", response.api_key.api_key);
/// // IMPORTANT: Save the private key securely!
/// std::fs::write("private_key.pem", &response.private_key)?;
/// ```
pub async fn generate_api_key(
    http: &HttpClient,
    request: GenerateApiKeyRequest,
) -> Result<GenerateApiKeyResponse> {
    http.post("/api_keys/generate", &request).await
}

/// Delete an API key.
///
/// Permanently deletes the specified API key. This action cannot be undone.
///
/// # Arguments
///
/// * `api_key_id` - The API key identifier to delete
///
/// # Example
///
/// ```ignore
/// let response = client.delete_api_key("ak_123").await?;
/// if response.deleted.unwrap_or(false) {
///     println!("API key deleted successfully");
/// }
/// ```
pub async fn delete_api_key(
    http: &HttpClient,
    api_key_id: &str,
) -> Result<DeleteApiKeyResponse> {
    let path = format!("/api_keys/{}", encode_path_segment(api_key_id));
    http.delete_with_response(&path).await
}
