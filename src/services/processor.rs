//! Post processor
//!
//! Fetches posts, detects threads, and saves to Readwise.

use anyhow::Result;
use tracing::{debug, info, instrument, warn};

use crate::bluesky::{BlueskyClient, PostView, ThreadViewPost};
use crate::content::links::extract_links;
use crate::content::{format_post_as_highlight, format_thread_as_document, is_thread};
use crate::readwise::client::{Document, ReadwiseClient};

/// Options for processing a post
#[derive(Debug, Clone, Default)]
pub struct ProcessOptions {
    /// Extract and save links found in the post
    pub extract_links: bool,
    /// Optional note to attach to the highlight
    pub note: Option<String>,
}

/// Post processor handles fetching posts and saving to Readwise
pub struct PostProcessor<B: BlueskyClient, R: ReadwiseClient> {
    bluesky: B,
    readwise: R,
}

impl<B: BlueskyClient, R: ReadwiseClient> PostProcessor<B, R> {
    /// Create a new post processor
    pub fn new(bluesky: B, readwise: R) -> Self {
        Self { bluesky, readwise }
    }

    /// Process a post URI and save to Readwise
    #[instrument(skip(self, readwise_token))]
    pub async fn process_post(
        &self,
        post_uri: &str,
        readwise_token: &str,
        options: ProcessOptions,
    ) -> Result<()> {
        info!("Processing post: {}", post_uri);

        // Fetch the full thread
        let thread_response = self.bluesky.get_post_thread(post_uri).await?;
        let thread = &thread_response.thread;

        // Determine if this is a thread or single post
        if self.is_part_of_thread(thread) {
            debug!("Post is part of a thread, saving to Reader");
            self.save_thread(thread, readwise_token).await?;
        } else {
            debug!("Single post, saving as highlight");
            self.save_single_post(&thread.post, readwise_token, options.note.as_deref())
                .await?;
        }

        // Optionally extract and save links
        if options.extract_links {
            self.process_links(&thread.post, readwise_token).await?;
        }

        Ok(())
    }

    /// Check if a post is part of a thread
    fn is_part_of_thread(&self, thread: &ThreadViewPost) -> bool {
        // Has parent posts or is the start of a multi-post thread
        thread.parent.is_some()
            || thread
                .replies
                .as_ref()
                .map(|r| !r.is_empty())
                .unwrap_or(false)
            || is_thread(&thread.post)
    }

    /// Save a single post as a Readwise highlight
    async fn save_single_post(
        &self,
        post: &PostView,
        readwise_token: &str,
        note: Option<&str>,
    ) -> Result<()> {
        let highlight = format_post_as_highlight(post, note);
        self.readwise
            .save_highlight(readwise_token, highlight)
            .await?;
        info!("Saved post as highlight");
        Ok(())
    }

    /// Save a thread as a Readwise Reader document
    async fn save_thread(&self, thread: &ThreadViewPost, readwise_token: &str) -> Result<()> {
        let document = format_thread_as_document(thread);
        self.readwise
            .save_document(readwise_token, document)
            .await?;
        info!("Saved thread to Reader");
        Ok(())
    }

    /// Extract links from a post and save them to Reader
    async fn process_links(&self, post: &PostView, readwise_token: &str) -> Result<()> {
        let links = extract_links(&post.record);

        if links.is_empty() {
            debug!("No links found in post");
            return Ok(());
        }

        info!("Found {} links to save", links.len());

        for link in links {
            // Save each link as a Reader document
            let document = Document {
                url: link.clone(),
                html: None,
                title: None,
                author: None,
                tags: Some(vec!["bluesky".to_string(), "extracted-link".to_string()]),
            };

            match self.readwise.save_document(readwise_token, document).await {
                Ok(_) => debug!("Saved link: {}", link),
                Err(e) => warn!("Failed to save link {}: {}", link, e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bluesky::types::*;
    use crate::readwise::client::Highlight;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    // Mock Bluesky client
    struct MockBlueskyClient {
        thread: ThreadResponse,
    }

    #[async_trait]
    impl BlueskyClient for MockBlueskyClient {
        async fn get_bookmarks(&self, _cursor: Option<&str>) -> Result<BookmarkResponse> {
            Ok(BookmarkResponse {
                cursor: None,
                bookmarks: vec![],
            })
        }

        async fn get_post_thread(&self, _uri: &str) -> Result<ThreadResponse> {
            Ok(self.thread.clone())
        }

        async fn send_dm(&self, _convo_id: &str, _text: &str) -> Result<()> {
            Ok(())
        }
    }

    // Mock Readwise client
    struct MockReadwiseClient {
        highlights: Mutex<Vec<Highlight>>,
        documents: Mutex<Vec<Document>>,
    }

    impl MockReadwiseClient {
        fn new() -> Self {
            Self {
                highlights: Mutex::new(vec![]),
                documents: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl ReadwiseClient for MockReadwiseClient {
        async fn save_highlight(&self, _token: &str, highlight: Highlight) -> Result<()> {
            self.highlights.lock().unwrap().push(highlight);
            Ok(())
        }

        async fn save_document(&self, _token: &str, document: Document) -> Result<()> {
            self.documents.lock().unwrap().push(document);
            Ok(())
        }

        async fn verify_token(&self, _token: &str) -> Result<bool> {
            Ok(true)
        }
    }

    fn make_test_post() -> PostView {
        PostView {
            uri: "at://did:plc:test/app.bsky.feed.post/abc123".to_string(),
            cid: "cid123".to_string(),
            author: Author {
                did: "did:plc:test".to_string(),
                handle: "test.bsky.social".to_string(),
                display_name: Some("Test User".to_string()),
            },
            record: PostRecord {
                text: "Hello, world!".to_string(),
                created_at: Utc::now(),
                reply: None,
                facets: None,
            },
            indexed_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_process_single_post() {
        let post = make_test_post();
        let thread = ThreadResponse {
            thread: ThreadViewPost {
                post: post.clone(),
                parent: None,
                replies: None,
            },
        };

        let bluesky = MockBlueskyClient { thread };
        let readwise = MockReadwiseClient::new();
        let processor = PostProcessor::new(bluesky, readwise);

        let result = processor
            .process_post(&post.uri, "test_token", ProcessOptions::default())
            .await;

        assert!(result.is_ok());
    }
}
