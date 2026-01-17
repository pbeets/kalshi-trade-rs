//! API Keys Management endpoints.
//!
//! Programmatic API key management. Typically managed via the Kalshi web UI,
//! but these endpoints allow programmatic access for advanced use cases.

use url::form_urlencoded;

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        ApiKeysResponse, CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyResponse,
        GenerateApiKeyRequest, GenerateApiKeyResponse,
    },
};

fn encode_path_segment(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/// Returns all API keys associated with the account.
pub async fn get_api_keys(http: &HttpClient) -> Result<ApiKeysResponse> {
    http.get("/api_keys").await
}

/// Creates an API key using the provided RSA public key (Premier/Market Maker tier).
pub async fn create_api_key(
    http: &HttpClient,
    request: CreateApiKeyRequest,
) -> Result<CreateApiKeyResponse> {
    http.post("/api_keys", &request).await
}

/// Creates an API key with platform-generated RSA keypair.
///
/// **Important**: The private key is returned only once. Store it securely.
pub async fn generate_api_key(
    http: &HttpClient,
    request: GenerateApiKeyRequest,
) -> Result<GenerateApiKeyResponse> {
    http.post("/api_keys/generate", &request).await
}

/// Permanently deletes the specified API key. This action cannot be undone.
pub async fn delete_api_key(http: &HttpClient, api_key_id: &str) -> Result<DeleteApiKeyResponse> {
    let path = format!("/api_keys/{}", encode_path_segment(api_key_id));
    http.delete_with_response(&path).await
}
