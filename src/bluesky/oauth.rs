//! OAuth flow helpers for AT Protocol
//!
//! Implements the AT Protocol OAuth flow with PKCE and DPoP.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::Client;
use thiserror::Error;
use tokio::sync::RwLock;

use atproto_identity::key::{generate_key, KeyData, KeyType};
use atproto_identity::resolve::SharedIdentityResolver;
use atproto_oauth::resources::{pds_resources, AuthorizationServer};
use atproto_oauth::workflow::{
    oauth_complete, oauth_init, oauth_refresh, OAuthClient, OAuthRequest, OAuthRequestState,
    ParResponse, TokenResponse,
};

/// Errors that can occur during OAuth operations
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    #[error("Failed to resolve identity: {0}")]
    IdentityResolution(String),

    #[error("Failed to discover authorization server: {0}")]
    ServerDiscovery(String),

    #[error("OAuth initialization failed: {0}")]
    InitFailed(String),

    #[error("OAuth completion failed: {0}")]
    CompleteFailed(String),

    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),

    #[error("Invalid state parameter")]
    InvalidState,

    #[error("State expired")]
    StateExpired,

    #[error("Missing authorization code")]
    MissingCode,

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Key generation failed: {0}")]
    KeyGeneration(String),

    #[error("Key serialization failed: {0}")]
    KeySerialization(String),
}

/// Pending OAuth request stored between login and callback
#[derive(Debug, Clone)]
pub struct PendingOAuthRequest {
    pub oauth_request: OAuthRequest,
    pub authorization_server: AuthorizationServer,
    pub created_at: DateTime<Utc>,
}

/// In-memory store for pending OAuth requests
/// Maps state parameter -> PendingOAuthRequest
#[derive(Debug, Default)]
pub struct OAuthStateStore {
    pending: RwLock<HashMap<String, PendingOAuthRequest>>,
}

impl OAuthStateStore {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Store a pending OAuth request
    pub async fn store(&self, state: String, request: PendingOAuthRequest) {
        let mut pending = self.pending.write().await;
        pending.insert(state, request);
    }

    /// Retrieve and remove a pending OAuth request
    pub async fn take(&self, state: &str) -> Option<PendingOAuthRequest> {
        let mut pending = self.pending.write().await;
        pending.remove(state)
    }

    /// Clean up expired requests (older than 10 minutes)
    pub async fn cleanup_expired(&self) {
        let mut pending = self.pending.write().await;
        let cutoff = Utc::now() - chrono::Duration::minutes(10);
        pending.retain(|_, req| req.created_at > cutoff);
    }
}

/// OAuth service for handling AT Protocol authentication
pub struct OAuthService {
    http_client: Client,
    oauth_client: OAuthClient,
    identity_resolver: SharedIdentityResolver,
    state_store: Arc<OAuthStateStore>,
}

impl OAuthService {
    /// Create a new OAuth service
    pub fn new(
        http_client: Client,
        oauth_client: OAuthClient,
        identity_resolver: SharedIdentityResolver,
        state_store: Arc<OAuthStateStore>,
    ) -> Self {
        Self {
            http_client,
            oauth_client,
            identity_resolver,
            state_store,
        }
    }

    /// Initiate the OAuth flow for a user handle
    ///
    /// Returns the authorization URL to redirect the user to
    pub async fn initiate_login(&self, handle: &str) -> Result<String, OAuthError> {
        // 1. Resolve handle to DID and get PDS
        let document = self
            .identity_resolver
            .resolve(handle)
            .await
            .map_err(|e| OAuthError::IdentityResolution(e.to_string()))?;

        // Get PDS endpoint from document
        let pds = document.pds_endpoints().first().cloned().ok_or_else(|| {
            OAuthError::IdentityResolution("No PDS endpoint in DID document".into())
        })?;

        // 2. Discover authorization server from PDS
        let (_protected_resource, authorization_server) = pds_resources(&self.http_client, pds)
            .await
            .map_err(|e| OAuthError::ServerDiscovery(e.to_string()))?;

        // 3. Generate PKCE verifier and challenge
        let (code_verifier, code_challenge) = atproto_oauth::pkce::generate();

        // 4. Generate random state and nonce
        let state = generate_random_string(32);
        let nonce = generate_random_string(32);

        // 5. Create OAuth request state
        let oauth_request_state = OAuthRequestState {
            state: state.clone(),
            nonce: nonce.clone(),
            code_challenge: code_challenge.clone(),
            scope: "atproto transition:generic".to_string(),
        };

        // 6. Generate DPoP key
        let dpop_key_data = generate_dpop_key()?;

        // 7. Call oauth_init to get PAR response
        let par_response = oauth_init(
            &self.http_client,
            &self.oauth_client,
            &dpop_key_data,
            Some(handle),
            &authorization_server,
            &oauth_request_state,
        )
        .await
        .map_err(|e| OAuthError::InitFailed(e.to_string()))?;

        // 8. Create OAuthRequest for storage
        let oauth_request = OAuthRequest {
            oauth_state: state.clone(),
            issuer: authorization_server.issuer.clone(),
            authorization_server: authorization_server.issuer.clone(),
            nonce,
            pkce_verifier: code_verifier,
            signing_public_key: serialize_key(&self.oauth_client.private_signing_key_data),
            dpop_private_key: serialize_key(&dpop_key_data),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(par_response.expires_in as i64),
        };

        // 9. Store pending request
        let pending = PendingOAuthRequest {
            oauth_request,
            authorization_server: authorization_server.clone(),
            created_at: Utc::now(),
        };
        self.state_store.store(state.clone(), pending).await;

        // 10. Build authorization URL
        let auth_url = format!(
            "{}?request_uri={}&client_id={}",
            authorization_server.authorization_endpoint,
            urlencoding::encode(&par_response.request_uri),
            urlencoding::encode(&self.oauth_client.client_id),
        );

        Ok(auth_url)
    }

    /// Complete the OAuth flow with the callback code
    pub async fn complete_login(
        &self,
        state: &str,
        code: &str,
    ) -> Result<TokenResponse, OAuthError> {
        // 1. Retrieve pending request
        let pending = self
            .state_store
            .take(state)
            .await
            .ok_or(OAuthError::InvalidState)?;

        // 2. Check if expired
        if pending.oauth_request.expires_at < Utc::now() {
            return Err(OAuthError::StateExpired);
        }

        // 3. Deserialize DPoP key
        let dpop_key_data = deserialize_key(&pending.oauth_request.dpop_private_key)?;

        // 4. Exchange code for tokens
        let token_response = oauth_complete(
            &self.http_client,
            &self.oauth_client,
            &dpop_key_data,
            code,
            &pending.oauth_request,
            &pending.authorization_server,
        )
        .await
        .map_err(|e| OAuthError::CompleteFailed(e.to_string()))?;

        Ok(token_response)
    }

    /// Refresh an access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
        did: &str,
    ) -> Result<TokenResponse, OAuthError> {
        // Resolve DID to get document for PDS endpoint
        let document = self
            .identity_resolver
            .resolve(did)
            .await
            .map_err(|e| OAuthError::IdentityResolution(e.to_string()))?;

        // Generate new DPoP key for refresh
        let dpop_key_data = generate_dpop_key()?;

        // Refresh tokens
        let token_response = oauth_refresh(
            &self.http_client,
            &self.oauth_client,
            &dpop_key_data,
            refresh_token,
            &document,
        )
        .await
        .map_err(|e| OAuthError::RefreshFailed(e.to_string()))?;

        Ok(token_response)
    }
}

/// Generate a cryptographically secure random string
fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a DPoP key pair (P-256/ES256)
fn generate_dpop_key() -> Result<KeyData, OAuthError> {
    // Generate a P-256 private key for DPoP proofs
    generate_key(KeyType::P256Private).map_err(|e| OAuthError::KeyGeneration(e.to_string()))
}

/// Serialize a key for storage
fn serialize_key(key: &KeyData) -> String {
    use base64::prelude::*;
    let (key_type, bytes) = (key.key_type(), key.bytes());
    let type_byte = match key_type {
        KeyType::P256Private => 0u8,
        KeyType::P256Public => 1u8,
        KeyType::P384Private => 2u8,
        KeyType::P384Public => 3u8,
        KeyType::K256Private => 4u8,
        KeyType::K256Public => 5u8,
    };
    let mut data = vec![type_byte];
    data.extend_from_slice(bytes);
    BASE64_STANDARD.encode(&data)
}

/// Deserialize a key from storage
fn deserialize_key(serialized: &str) -> Result<KeyData, OAuthError> {
    use base64::prelude::*;
    let data = BASE64_STANDARD
        .decode(serialized)
        .map_err(|e| OAuthError::KeySerialization(e.to_string()))?;

    if data.is_empty() {
        return Err(OAuthError::KeySerialization("Empty key data".into()));
    }

    let key_type = match data[0] {
        0 => KeyType::P256Private,
        1 => KeyType::P256Public,
        2 => KeyType::P384Private,
        3 => KeyType::P384Public,
        4 => KeyType::K256Private,
        5 => KeyType::K256Public,
        _ => return Err(OAuthError::KeySerialization("Unknown key type".into())),
    };

    Ok(KeyData::new(key_type, data[1..].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(32);
        let s2 = generate_random_string(32);
        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2); // Should be unique
    }

    #[tokio::test]
    async fn test_state_store() {
        let store = OAuthStateStore::new();

        // Test that we can't retrieve a non-existent state
        assert!(store.take("nonexistent").await.is_none());
    }
}
