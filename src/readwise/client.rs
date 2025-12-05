//! Readwise API client

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Highlight to save (v2 API)
#[derive(Debug, Clone, Serialize)]
pub struct Highlight {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Document to save (v3 API / Reader)
#[derive(Debug, Clone, Serialize)]
pub struct Document {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Response from save operations
#[derive(Debug, Clone, Deserialize)]
pub struct SaveResponse {
    pub id: Option<String>,
}

/// Trait for Readwise operations (for testability)
#[async_trait]
pub trait ReadwiseClient: Send + Sync {
    /// Save a highlight (v2 API)
    async fn save_highlight(&self, token: &str, highlight: Highlight) -> Result<()>;

    /// Save a document to Reader (v3 API)
    async fn save_document(&self, token: &str, document: Document) -> Result<()>;

    /// Verify a token is valid
    async fn verify_token(&self, token: &str) -> Result<bool>;
}

/// HTTP-based Readwise client
pub struct HttpReadwiseClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpReadwiseClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://readwise.io/api".to_string(),
        }
    }
}

impl Default for HttpReadwiseClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReadwiseClient for HttpReadwiseClient {
    async fn save_highlight(&self, token: &str, highlight: Highlight) -> Result<()> {
        #[derive(Serialize)]
        struct HighlightsPayload {
            highlights: Vec<Highlight>,
        }

        let payload = HighlightsPayload {
            highlights: vec![highlight],
        };

        let response = self
            .client
            .post(format!("{}/v2/highlights/", self.base_url))
            .header("Authorization", format!("Token {}", token))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Readwise API error {}: {}", status, text);
        }

        Ok(())
    }

    async fn save_document(&self, token: &str, document: Document) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/v3/save/", self.base_url))
            .header("Authorization", format!("Token {}", token))
            .json(&document)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Readwise Reader API error {}: {}", status, text);
        }

        Ok(())
    }

    async fn verify_token(&self, token: &str) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/v2/auth/", self.base_url))
            .header("Authorization", format!("Token {}", token))
            .send()
            .await?;

        Ok(response.status().as_u16() == 204)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_serialization() {
        let highlight = Highlight {
            text: "Test text".to_string(),
            title: Some("Test Title".to_string()),
            author: Some("Test Author".to_string()),
            source_url: Some("https://example.com".to_string()),
            category: Some("tweets".to_string()),
            note: None,
        };

        let json = serde_json::to_string(&highlight).unwrap();
        assert!(json.contains("Test text"));
        assert!(json.contains("tweets"));
        assert!(!json.contains("note")); // Should be skipped when None
    }
}
