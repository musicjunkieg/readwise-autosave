//! DM bot service
//!
//! Polls bot account DMs and processes save requests.

use anyhow::{anyhow, Result};
use regex::Regex;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::bluesky::BlueskyClient;
use crate::readwise::client::ReadwiseClient;
use crate::services::processor::{PostProcessor, ProcessOptions};

/// DM bot configuration
pub struct DmBotConfig {
    /// Polling interval
    pub poll_interval: Duration,
}

impl Default for DmBotConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(10),
        }
    }
}

/// Parsed DM command
#[derive(Debug, Clone, PartialEq)]
pub enum DmCommand {
    /// Save a post (with optional note and flags)
    SavePost {
        post_url: String,
        note: Option<String>,
        extract_links: bool,
    },
    /// Register with a Readwise token (DM-only registration)
    Register { readwise_token: String },
    /// Request help
    Help,
    /// Request settings link
    Settings,
    /// Unknown command
    Unknown(String),
}

/// DM bot service
pub struct DmBotService<B: BlueskyClient, R: ReadwiseClient> {
    processor: PostProcessor<B, R>,
    bluesky: B,
    config: DmBotConfig,
}

impl<B: BlueskyClient + Clone, R: ReadwiseClient + Clone> DmBotService<B, R> {
    /// Create a new DM bot service
    pub fn new(bluesky: B, readwise: R, config: DmBotConfig) -> Self {
        Self {
            processor: PostProcessor::new(bluesky.clone(), readwise),
            bluesky,
            config,
        }
    }

    /// Start the DM polling loop
    pub async fn run(&self) -> Result<()> {
        let mut ticker = interval(self.config.poll_interval);

        info!("Starting DM bot");

        loop {
            ticker.tick().await;

            match self.poll_dms().await {
                Ok(count) => {
                    if count > 0 {
                        info!("Processed {} DMs", count);
                    } else {
                        debug!("No new DMs");
                    }
                }
                Err(e) => {
                    error!("Error polling DMs: {}", e);
                }
            }
        }
    }

    /// Poll for new DMs and process them
    async fn poll_dms(&self) -> Result<usize> {
        // TODO: Implement actual DM polling using chat.bsky.convo.listConvos
        // TODO: Track processed message IDs to avoid duplicates
        // For now, this is a stub
        Ok(0)
    }

    /// Process a single DM message
    pub async fn process_message(
        &self,
        convo_id: &str,
        message_text: &str,
        readwise_token: &str,
    ) -> Result<String> {
        let command = Self::parse_message(message_text);

        match command {
            DmCommand::SavePost {
                post_url,
                note,
                extract_links,
            } => {
                // Convert URL to AT-URI
                let post_uri = Self::url_to_at_uri(&post_url)?;

                let options = ProcessOptions {
                    extract_links,
                    note,
                };

                self.processor
                    .process_post(&post_uri, readwise_token, options)
                    .await?;

                Ok("✅ Saved to Readwise!".to_string())
            }
            DmCommand::Register { readwise_token: _ } => {
                // TODO: Save the Readwise token for this user
                Ok("✅ Registered! You can now DM me post URLs to save them.".to_string())
            }
            DmCommand::Help => Ok(Self::help_message()),
            DmCommand::Settings => {
                // TODO: Return actual settings URL
                Ok("⚙️ Visit https://your-domain.com/dashboard to manage settings".to_string())
            }
            DmCommand::Unknown(text) => {
                warn!("Unknown command: {}", text);
                Ok(format!(
                    "❓ I didn't understand that. {}\n\n{}",
                    text,
                    Self::help_message()
                ))
            }
        }
    }

    /// Parse a DM message into a command
    pub fn parse_message(text: &str) -> DmCommand {
        let text = text.trim();

        // Check for help command
        if text.eq_ignore_ascii_case("help") {
            return DmCommand::Help;
        }

        // Check for settings command
        if text.eq_ignore_ascii_case("settings") {
            return DmCommand::Settings;
        }

        // Check for register command
        if let Some(token) = text.strip_prefix("register ") {
            return DmCommand::Register {
                readwise_token: token.trim().to_string(),
            };
        }

        // Try to extract a Bluesky post URL
        let url_pattern =
            Regex::new(r"https://bsky\.app/profile/[^/]+/post/[a-zA-Z0-9]+").unwrap();

        if let Some(url_match) = url_pattern.find(text) {
            let post_url = url_match.as_str().to_string();

            // Check for +links flag
            let extract_links = text.contains("+links");

            // Extract note (text after URL, excluding flags)
            let after_url = text[url_match.end()..].trim();
            let note = after_url
                .replace("+links", "")
                .trim()
                .to_string();
            let note = if note.is_empty() { None } else { Some(note) };

            return DmCommand::SavePost {
                post_url,
                note,
                extract_links,
            };
        }

        DmCommand::Unknown(text.to_string())
    }

    /// Convert a bsky.app URL to an AT-URI
    fn url_to_at_uri(url: &str) -> Result<String> {
        // URL format: https://bsky.app/profile/{handle}/post/{rkey}
        let parts: Vec<&str> = url.split('/').collect();

        if parts.len() < 6 {
            return Err(anyhow!("Invalid Bluesky URL format"));
        }

        let handle = parts[4];
        let rkey = parts[6];

        // TODO: Resolve handle to DID using identity resolution
        // For now, assume handle format for the AT-URI
        // This should be: at://{did}/app.bsky.feed.post/{rkey}
        Ok(format!("at://{}/app.bsky.feed.post/{}", handle, rkey))
    }

    /// Generate help message
    fn help_message() -> String {
        r#"📚 Readwise Autosave Bot

Commands:
• Send a post URL to save it
• URL +links - Also save linked content
• URL Your note here - Add a note
• register <token> - Register with Readwise token
• settings - Get link to settings
• help - Show this message

Examples:
https://bsky.app/profile/user.bsky.social/post/abc123
https://bsky.app/profile/user.bsky.social/post/abc123 +links
https://bsky.app/profile/user.bsky.social/post/abc123 Great thread!"#
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_save_post() {
        let msg = "https://bsky.app/profile/test.bsky.social/post/abc123";
        let cmd = DmBotService::<MockClient, MockClient>::parse_message(msg);

        match cmd {
            DmCommand::SavePost {
                post_url,
                note,
                extract_links,
            } => {
                assert_eq!(post_url, "https://bsky.app/profile/test.bsky.social/post/abc123");
                assert!(note.is_none());
                assert!(!extract_links);
            }
            _ => panic!("Expected SavePost command"),
        }
    }

    #[test]
    fn test_parse_save_post_with_links() {
        let msg = "https://bsky.app/profile/test.bsky.social/post/abc123 +links";
        let cmd = DmBotService::<MockClient, MockClient>::parse_message(msg);

        match cmd {
            DmCommand::SavePost { extract_links, .. } => {
                assert!(extract_links);
            }
            _ => panic!("Expected SavePost command"),
        }
    }

    #[test]
    fn test_parse_save_post_with_note() {
        let msg = "https://bsky.app/profile/test.bsky.social/post/abc123 Great thread!";
        let cmd = DmBotService::<MockClient, MockClient>::parse_message(msg);

        match cmd {
            DmCommand::SavePost { note, .. } => {
                assert_eq!(note, Some("Great thread!".to_string()));
            }
            _ => panic!("Expected SavePost command"),
        }
    }

    #[test]
    fn test_parse_help() {
        let cmd = DmBotService::<MockClient, MockClient>::parse_message("help");
        assert_eq!(cmd, DmCommand::Help);
    }

    #[test]
    fn test_parse_register() {
        let cmd = DmBotService::<MockClient, MockClient>::parse_message("register abc123token");
        match cmd {
            DmCommand::Register { readwise_token } => {
                assert_eq!(readwise_token, "abc123token");
            }
            _ => panic!("Expected Register command"),
        }
    }

    #[test]
    fn test_url_to_at_uri() {
        let url = "https://bsky.app/profile/test.bsky.social/post/abc123";
        let uri = DmBotService::<MockClient, MockClient>::url_to_at_uri(url).unwrap();
        assert_eq!(uri, "at://test.bsky.social/app.bsky.feed.post/abc123");
    }

    // Mock client for tests
    use crate::bluesky::types::{BookmarkResponse, ThreadResponse};
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockClient;

    #[async_trait]
    impl BlueskyClient for MockClient {
        async fn get_bookmarks(&self, _cursor: Option<&str>) -> Result<BookmarkResponse> {
            Ok(BookmarkResponse {
                cursor: None,
                bookmarks: vec![],
            })
        }

        async fn get_post_thread(&self, _uri: &str) -> Result<ThreadResponse> {
            unimplemented!()
        }

        async fn send_dm(&self, _convo_id: &str, _text: &str) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ReadwiseClient for MockClient {
        async fn save_highlight(
            &self,
            _token: &str,
            _highlight: crate::readwise::client::Highlight,
        ) -> Result<()> {
            Ok(())
        }

        async fn save_document(
            &self,
            _token: &str,
            _document: crate::readwise::client::Document,
        ) -> Result<()> {
            Ok(())
        }

        async fn verify_token(&self, _token: &str) -> Result<bool> {
            Ok(true)
        }
    }
}
