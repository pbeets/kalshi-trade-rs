//! Authentication and configuration for the Kalshi API.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use rand_core::OsRng;
use std::{env, fs, path::Path};

use crate::{
    client::Environment,
    error::{Error, Result},
};

use rsa::{
    RsaPrivateKey,
    pkcs1::DecodeRsaPrivateKey,
    pkcs8::DecodePrivateKey,
    pss::BlindedSigningKey,
    sha2::Sha256,
    signature::{RandomizedSigner, SignatureEncoding},
};

/// Configuration for connecting to the Kalshi API.
///
/// Can be constructed directly, via [`from_env`](Self::from_env), or using the
/// [`builder`](Self::builder) pattern.
#[derive(Clone)]
pub struct KalshiConfig {
    /// The environment to connect to (Demo or Prod).
    pub environment: Environment,
    pub api_key_id: String,
    private_key: RsaPrivateKey,
}

impl KalshiConfig {
    /// Create a new configuration with the given parameters.
    ///
    /// # Arguments
    /// * `environment` - Demo or Prod environment
    /// * `api_key_id` - Your Kalshi API key ID
    /// * `private_key_pem` - Your RSA private key in PEM format
    ///
    /// # Errors
    /// Returns an error if the private key PEM is invalid.
    pub fn new(
        environment: Environment,
        api_key_id: impl Into<String>,
        private_key_pem: &str,
    ) -> Result<Self> {
        // Try PKCS#8 format first ("BEGIN PRIVATE KEY"), then PKCS#1 ("BEGIN RSA PRIVATE KEY")
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(private_key_pem))
            .map_err(|e| Error::InvalidPrivateKey(e.to_string()))?;

        Ok(Self {
            environment,
            api_key_id: api_key_id.into(),
            private_key,
        })
    }

    /// Load configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - `KALSHI_ENV`: "demo" or "prod" (defaults to "demo")
    /// - `KALSHI_API_KEY_ID`: Your API key ID (required)
    /// - `KALSHI_PRIVATE_KEY_PATH`: Path to your private key PEM file (required)
    ///
    /// # Errors
    /// Returns an error if required environment variables are missing or if
    /// the private key file cannot be read or parsed.
    pub fn from_env() -> Result<Self> {
        let environment = match env::var("KALSHI_ENV")
            .unwrap_or_else(|_| "demo".to_string())
            .to_lowercase()
            .as_str()
        {
            "prod" | "production" => Environment::Prod,
            _ => Environment::Demo,
        };

        let api_key_id = env::var("KALSHI_API_KEY_ID")
            .map_err(|_| Error::MissingEnvVar("KALSHI_API_KEY_ID".to_string()))?;

        let private_key_path = env::var("KALSHI_PRIVATE_KEY_PATH")
            .map_err(|_| Error::MissingEnvVar("KALSHI_PRIVATE_KEY_PATH".to_string()))?;

        let private_key_pem = fs::read_to_string(&private_key_path)
            .map_err(|e| Error::PrivateKeyFileError(private_key_path, e.to_string()))?;

        Self::new(environment, api_key_id, &private_key_pem)
    }

    /// Create a builder for more flexible configuration.
    pub fn builder() -> KalshiConfigBuilder {
        KalshiConfigBuilder::default()
    }

    /// Generate a signature for an API request.
    ///
    /// # Arguments
    /// * `timestamp_ms` - Current timestamp in milliseconds
    /// * `method` - HTTP method (GET, POST, DELETE, etc.)
    /// * `path` - Request path including /trade-api/v2/ prefix, without query params
    ///
    /// # Returns
    /// Base64-encoded RSA-PSS signature
    pub(crate) fn sign(&self, timestamp_ms: u64, method: &str, path: &str) -> Result<String> {
        // Message format: {timestamp_ms}{METHOD}{path}
        let message = format!("{}{}{}", timestamp_ms, method.to_uppercase(), path);

        let signing_key = BlindedSigningKey::<Sha256>::new(self.private_key.clone());
        let signature = signing_key.sign_with_rng(&mut OsRng, message.as_bytes());

        Ok(BASE64.encode(signature.to_bytes()))
    }

    /// Get the API key ID.
    pub fn api_key_id(&self) -> &str {
        &self.api_key_id
    }
}

impl std::fmt::Debug for KalshiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiConfig")
            .field("environment", &self.environment)
            .field("api_key_id", &self.api_key_id)
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for [`KalshiConfig`].
#[derive(Default)]
pub struct KalshiConfigBuilder {
    environment: Option<Environment>,
    api_key_id: Option<String>,
    private_key_pem: Option<String>,
    private_key_path: Option<String>,
}

impl KalshiConfigBuilder {
    /// Set the environment (Demo or Prod). Defaults to Demo.
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Set the API key ID.
    pub fn api_key_id(mut self, api_key_id: impl Into<String>) -> Self {
        self.api_key_id = Some(api_key_id.into());
        self
    }

    /// Set the private key directly as a PEM string.
    pub fn private_key_pem(mut self, pem: impl Into<String>) -> Self {
        self.private_key_pem = Some(pem.into());
        self
    }

    /// Set the path to the private key PEM file.
    /// The file will be read when [`build`](Self::build) is called.
    pub fn private_key_path(mut self, path: impl AsRef<Path>) -> Self {
        self.private_key_path = Some(path.as_ref().to_string_lossy().into_owned());
        self
    }

    /// Build the configuration.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `api_key_id` is not set
    /// - Neither `private_key_pem` nor `private_key_path` is set
    /// - The private key file cannot be read (if using `private_key_path`)
    /// - The private key PEM is invalid
    pub fn build(self) -> Result<KalshiConfig> {
        let api_key_id = self
            .api_key_id
            .ok_or_else(|| Error::Auth("api_key_id is required".to_string()))?;

        let private_key_pem = match (self.private_key_pem, self.private_key_path) {
            (Some(pem), _) => pem,
            (None, Some(path)) => fs::read_to_string(&path)
                .map_err(|e| Error::PrivateKeyFileError(path, e.to_string()))?,
            (None, None) => {
                return Err(Error::Auth(
                    "either private_key_pem or private_key_path is required".to_string(),
                ));
            }
        };

        let environment = self.environment.unwrap_or(Environment::Demo);

        KalshiConfig::new(environment, api_key_id, &private_key_pem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test PEM key for testing only - DO NOT USE IN PRODUCTION
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCxVp8iHrhET3Sq
xSGml5zWLlyAEAFBo26Utt2aco0hUBS2epzSzUu+r+s0TenyI/60QOHAwE7d+vkq
emvk+1j3wm0rsioGhkZiGjBV4Z6TzGf1VaR1REaWNwIukTF0MoighuFQ0IcNBmja
hin6vNCBc+Xb6d7P/3IcfgEtBq/QRY9Xc7qe/eMF0B/dgeKLKYTM6mehEDOJmmSs
RQ4nAQVwi1oBGxu9QV/IISuaJ2X2uUPhsP3lcL6CRntuPLmq+E+2Dx7/ltSQSo0H
aa9BX5WPguHZL4zNsG5Iw39Zfuf9upFhJvkqJwXFDaCbqsOEEqwKxB+J1SIPrjec
ELids2ehAgMBAAECggEABIMU4RTBXtRttSouElOjtQc5u8cewaKIECI8QNPshR4S
PfwylaJWfuvxt3Wl5FgxCcvVhy+2j7Ri6TTzZ1LBaI+GF6JqYRrC21M1Ctd9xgOz
yLgsuOvP+T4ZRYGLklMIr4igJ8LXD6ziibmuzImRGPhh+FjogrWlrif53VNzQ6U9
/M2KdZAt1kzkGslYbEaM2BrsvxnehScBGPIesHhaycsQfU5WUg1JYf1hhHNDXAnm
ZJkxCu6ngpzoAj1W6XDTw1+97YVr9eVlOhSSoKFBRbGlWrUSengf9dENT3EHtQ1n
N3pZwg5I/FetNYOyqmU7AwWUwbn0Z1YggJ6OdUFn9QKBgQDmnWrz8xaUgy18ZE0w
v0ezjEyqff6JPzISmsCi8OxYp6ILYkRRGX6PwxSs+xaPMpLV6Lpwc8W6ipjRXSL5
38GiM6vo3De7OAlKi7vdOkZUyfI2lN7sHAIEhxELmyhRFlOhdCXL4mvt39HQrRmp
sm8fGF9m1nZDASnnmxg67443zQKBgQDE2+eKi14aJ8oSMvsI+xyxYJXM1irXZY++
eKdaTdUNqMaRFDb5E5l9tug9RFOwyEnT8+faRUCNvnKexAPjBJy8coCTkWmsV2qL
gVGmkg7mRpQOPiLFgxvIv8rl73KGYE3BhMsqRJUYOg3W7pCL/Wu8aeIVHWReCew8
gEHN7qWzJQKBgHoLVv2xaQLBhUHuZvdkU1LO7gfQU/NYUWyNH2Nb8whb9qLlp1fw
EQ2N5RRCcUbpMdIorvoyGrNFA+jQzGDGrNflVpYObSQUXL5pwssqOuxGT3vZPzxe
+iZhQIEO5MA8+5dXO2Vx90JVD9nKsekfuDURlfN7jeyZ4g5jAui1vGMFAoGAK2w2
TqEfSLWuJQWJyhlZ4uZjJKO5H3oPkvwaLhks/a5U3nuPBLIGEWzfHWSM8Vm8rzwF
0GemLZ3suoiSMuk5iXfYVLSmkpVVsx/7Wrqs/q5iyiF6mgapgkaMAtwmbu2fOSiJ
h/FI0ec8VkSZLcfgk9bnP7EUCoo1ycKgwUP62OECgYAd2X6dPxEfXa+WZb85WhIj
OwzlK5bT9ilefSVJ7EyhqyPx8ZjheGOFnYYg9qBk9NFZtr4s0t8fxdlwBFf30rOu
iynqnXgceG4vGoaVxY2MgFvB61Ktle7WfWGZz4jEn/QyZkQbg5hDKDQzJ2N49JrO
9nBnR2R/e8zsmkh4lClsVA==
-----END PRIVATE KEY-----"#;

    #[test]
    fn test_config_new() {
        let config = KalshiConfig::new(Environment::Demo, "test-key-id", TEST_PRIVATE_KEY_PEM);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.environment, Environment::Demo);
        assert_eq!(config.api_key_id, "test-key-id");
    }

    #[test]
    fn test_config_builder() {
        let config = KalshiConfig::builder()
            .environment(Environment::Prod)
            .api_key_id("builder-key-id")
            .private_key_pem(TEST_PRIVATE_KEY_PEM)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.environment, Environment::Prod);
        assert_eq!(config.api_key_id, "builder-key-id");
    }

    #[test]
    fn test_config_builder_missing_api_key() {
        let result = KalshiConfig::builder()
            .private_key_pem(TEST_PRIVATE_KEY_PEM)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder_missing_private_key() {
        let result = KalshiConfig::builder().api_key_id("test-key").build();

        assert!(result.is_err());
    }

    #[test]
    fn test_sign() {
        let config =
            KalshiConfig::new(Environment::Demo, "test-key-id", TEST_PRIVATE_KEY_PEM).unwrap();

        let signature = config.sign(1703123456789, "GET", "/trade-api/v2/portfolio/balance");
        assert!(signature.is_ok());

        // Signature should be base64 encoded
        let sig = signature.unwrap();
        assert!(BASE64.decode(&sig).is_ok());
    }

    #[test]
    fn test_debug_redacts_private_key() {
        let config =
            KalshiConfig::new(Environment::Demo, "test-key-id", TEST_PRIVATE_KEY_PEM).unwrap();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("BEGIN PRIVATE KEY"));
    }
}
