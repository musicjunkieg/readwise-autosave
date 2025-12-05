//! Bookmark sync service
//!
//! Polls user bookmarks and saves new ones to Readwise.

use anyhow::Result;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::bluesky::BlueskyClient;
use crate::db::models::{User, UserSettings};
use crate::readwise::client::ReadwiseClient;
use crate::services::processor::{PostProcessor, ProcessOptions};

/// Bookmark sync service configuration
pub struct BookmarkSyncConfig {
    /// Polling interval
    pub poll_interval: Duration,
}

impl Default for BookmarkSyncConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(30),
        }
    }
}

/// Bookmark sync service
pub struct BookmarkSyncService<B: BlueskyClient, R: ReadwiseClient> {
    processor: PostProcessor<B, R>,
    config: BookmarkSyncConfig,
}

impl<B: BlueskyClient + Clone, R: ReadwiseClient + Clone> BookmarkSyncService<B, R> {
    /// Create a new bookmark sync service
    pub fn new(bluesky: B, readwise: R, config: BookmarkSyncConfig) -> Self {
        Self {
            processor: PostProcessor::new(bluesky, readwise),
            config,
        }
    }

    /// Start the bookmark sync loop for a user
    /// This should be spawned as a tokio task
    pub async fn run_for_user(
        &self,
        _user: User,
        _settings: UserSettings,
        bluesky_client: B,
    ) -> Result<()> {
        let mut ticker = interval(self.config.poll_interval);

        info!("Starting bookmark sync");

        loop {
            ticker.tick().await;

            match self.poll_bookmarks(&bluesky_client, &_settings).await {
                Ok(count) => {
                    if count > 0 {
                        info!("Processed {} new bookmarks", count);
                    } else {
                        debug!("No new bookmarks");
                    }
                }
                Err(e) => {
                    error!("Error polling bookmarks: {}", e);
                }
            }
        }
    }

    /// Poll bookmarks and process new ones
    async fn poll_bookmarks(&self, bluesky: &B, settings: &UserSettings) -> Result<usize> {
        // Get bookmarks starting from the last cursor
        let cursor = settings.last_bookmark_cursor.as_deref();
        let response = bluesky.get_bookmarks(cursor).await?;

        let mut processed_count = 0;

        for bookmark in &response.bookmarks {
            let post_uri = &bookmark.subject.uri;

            // TODO: Check if already processed in database
            // For now, just process all bookmarks in the response

            let options = ProcessOptions {
                extract_links: settings.extract_links,
                note: None,
            };

            match self
                .processor
                .process_post(post_uri, &settings.readwise_token, options)
                .await
            {
                Ok(_) => {
                    processed_count += 1;
                    // TODO: Mark as processed in database
                }
                Err(e) => {
                    warn!("Failed to process bookmark {}: {}", post_uri, e);
                }
            }
        }

        // TODO: Update last_bookmark_cursor in database
        if let Some(new_cursor) = &response.cursor {
            debug!("New cursor: {}", new_cursor);
        }

        Ok(processed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BookmarkSyncConfig::default();
        assert_eq!(config.poll_interval, Duration::from_secs(30));
    }
}
