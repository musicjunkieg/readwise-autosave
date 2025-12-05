//! AT Protocol client
//!
//! Wraps atproto-client for our specific needs.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use super::types::*;

/// Trait for Bluesky API operations (for testability)
#[async_trait]
pub trait BlueskyClient: Send + Sync {
    /// Get user's bookmarks
    async fn get_bookmarks(&self, cursor: Option<&str>) -> Result<BookmarkResponse>;

    /// Get a post thread
    async fn get_post_thread(&self, uri: &str) -> Result<ThreadResponse>;

    /// Send a DM
    async fn send_dm(&self, convo_id: &str, text: &str) -> Result<()>;
}

/// Bluesky public data service base URL
const BSKY_PUBLIC_API: &str = "https://public.api.bsky.app";

/// Bluesky authenticated API base URL
const BSKY_API: &str = "https://bsky.social";

/// Bluesky chat API proxy header value
const BSKY_CHAT_PROXY: &str = "did:web:api.bsky.chat#bsky_chat";

/// Concrete HTTP client for Bluesky API
pub struct HttpBlueskyClient {
    http: Client,
    /// Access token for authenticated requests
    access_token: Option<String>,
    /// DID of the authenticated user
    did: Option<String>,
}

impl HttpBlueskyClient {
    /// Create a new unauthenticated client (for public API only)
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            access_token: None,
            did: None,
        }
    }

    /// Create a new authenticated client
    pub fn with_auth(access_token: String, did: String) -> Self {
        Self {
            http: Client::new(),
            access_token: Some(access_token),
            did: Some(did),
        }
    }

    /// Update the access token (e.g., after refresh)
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Make an authenticated GET request
    async fn auth_get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow!("Authentication required"))?;

        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, body));
        }

        Ok(response.json().await?)
    }

    /// Make an authenticated POST request to the chat API
    async fn chat_post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow!("Authentication required"))?;

        let url = format!("{}/xrpc/{}", BSKY_API, endpoint);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("atproto-proxy", BSKY_CHAT_PROXY)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Chat API error {}: {}", status, body));
        }

        Ok(response.json().await?)
    }
}

impl Default for HttpBlueskyClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BlueskyClient for HttpBlueskyClient {
    #[instrument(skip(self))]
    async fn get_bookmarks(&self, cursor: Option<&str>) -> Result<BookmarkResponse> {
        let mut url = format!("{}/xrpc/app.bsky.bookmark.getBookmarks?limit=50", BSKY_API);
        if let Some(c) = cursor {
            url.push_str(&format!("&cursor={}", c));
        }

        debug!("Fetching bookmarks");
        self.auth_get(&url).await
    }

    #[instrument(skip(self))]
    async fn get_post_thread(&self, uri: &str) -> Result<ThreadResponse> {
        // Public API doesn't require auth for public posts
        let url = format!(
            "{}/xrpc/app.bsky.feed.getPostThread?uri={}&depth=100&parentHeight=100",
            BSKY_PUBLIC_API,
            urlencoding::encode(uri)
        );

        debug!("Fetching post thread");
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, body));
        }

        Ok(response.json().await?)
    }

    #[instrument(skip(self))]
    async fn send_dm(&self, convo_id: &str, text: &str) -> Result<()> {
        #[derive(Serialize)]
        struct SendMessageInput {
            #[serde(rename = "convoId")]
            convo_id: String,
            message: MessageInput,
        }

        #[derive(Serialize)]
        struct MessageInput {
            text: String,
        }

        #[derive(Deserialize)]
        struct SendMessageOutput {
            // We don't need the response data
            #[allow(dead_code)]
            id: String,
        }

        let input = SendMessageInput {
            convo_id: convo_id.to_string(),
            message: MessageInput {
                text: text.to_string(),
            },
        };

        debug!("Sending DM to conversation {}", convo_id);
        let _: SendMessageOutput = self
            .chat_post("chat.bsky.convo.sendMessage", &input)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpBlueskyClient::new();
        assert!(client.access_token.is_none());
        assert!(client.did.is_none());
    }

    #[test]
    fn test_client_with_auth() {
        let client =
            HttpBlueskyClient::with_auth("test_token".to_string(), "did:plc:test".to_string());
        assert_eq!(client.access_token.as_deref(), Some("test_token"));
        assert_eq!(client.did.as_deref(), Some("did:plc:test"));
    }
}
