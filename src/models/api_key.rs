//! API key management models and response types.
//!
//! API keys are used for authenticating with the Kalshi API.
//! These endpoints allow management of API keys programmatically.

use serde::{Deserialize, Serialize};

/// An API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// The API key identifier.
    #[serde(default)]
    pub api_key: Option<String>,
    /// The name/label for the API key.
    #[serde(default)]
    pub name: Option<String>,
    /// When the API key was created (RFC3339 timestamp).
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the API key was last used (RFC3339 timestamp).
    #[serde(default)]
    pub last_used_at: Option<String>,
    /// The RSA public key (PEM format).
    #[serde(default)]
    pub public_key: Option<String>,
}

/// Response from GET /api_keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeysResponse {
    /// The list of API keys.
    pub api_keys: Vec<ApiKey>,
}

/// Request body for POST /api_keys.
///
/// Creates a new API key using a user-provided RSA public key.
/// Available for Premier/Market Maker tier users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    /// The name/label for the API key.
    pub name: String,
    /// The RSA public key in PEM format.
    pub public_key: String,
}

impl CreateApiKeyRequest {
    /// Create a new API key request.
    ///
    /// # Arguments
    ///
    /// * `name` - A label for the API key
    /// * `public_key` - The RSA public key in PEM format
    #[must_use]
    pub fn new(name: impl Into<String>, public_key: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            public_key: public_key.into(),
        }
    }
}

/// Response from POST /api_keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    /// The created API key.
    pub api_key: ApiKey,
}

/// Request body for POST /api_keys/generate.
///
/// Generates a new API key with platform-generated RSA keypair.
/// The private key is only returned once and cannot be retrieved later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateApiKeyRequest {
    /// The name/label for the API key.
    pub name: String,
}

impl GenerateApiKeyRequest {
    /// Create a new generate API key request.
    ///
    /// # Arguments
    ///
    /// * `name` - A label for the API key
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Response from POST /api_keys/generate.
///
/// Contains both the API key and the private key.
/// **Important**: The private key is only returned once and cannot be retrieved later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateApiKeyResponse {
    /// The generated API key.
    pub api_key: ApiKey,
    /// The RSA private key in PEM format.
    ///
    /// **Important**: Store this securely. It cannot be retrieved again.
    pub private_key: String,
}

/// Response from DELETE /api_keys/{api_key}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteApiKeyResponse {
    /// The deleted API key identifier.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Whether the deletion was successful.
    #[serde(default)]
    pub deleted: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request() {
        let request = CreateApiKeyRequest::new("My Key", "-----BEGIN PUBLIC KEY-----...");
        assert_eq!(request.name, "My Key");
        assert!(request.public_key.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn test_generate_request() {
        let request = GenerateApiKeyRequest::new("My Key");
        assert_eq!(request.name, "My Key");
    }

    #[test]
    fn test_deserialize_api_keys_response() {
        let json = r#"{"api_keys": []}"#;
        let response: ApiKeysResponse = serde_json::from_str(json).unwrap();
        assert!(response.api_keys.is_empty());
    }

    #[test]
    fn test_deserialize_api_key() {
        let json = r#"{
            "api_keys": [{
                "api_key": "ak_123",
                "name": "Test Key"
            }]
        }"#;
        let response: ApiKeysResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.api_keys.len(), 1);
        assert_eq!(response.api_keys[0].api_key, Some("ak_123".to_string()));
    }
}
