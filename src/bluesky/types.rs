//! Bluesky API types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bookmark response from getBookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub cursor: Option<String>,
    pub bookmarks: Vec<BookmarkView>,
}

/// A single bookmark view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkView {
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
    // item can be postView, blockedPost, or notFoundPost
    // We'll handle this as JSON for now
    pub item: serde_json::Value,
}

/// Strong reference to a record (uri + cid)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

/// Thread response from getPostThread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResponse {
    pub thread: ThreadViewPost,
}

/// A post in a thread view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadViewPost {
    pub post: PostView,
    pub parent: Option<Box<ThreadViewPost>>,
    pub replies: Option<Vec<ThreadViewPost>>,
}

/// A post view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: Author,
    pub record: PostRecord,
    pub indexed_at: DateTime<Utc>,
}

/// Post author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
}

/// Post record content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRecord {
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub reply: Option<ReplyRef>,
    pub facets: Option<Vec<Facet>>,
}

/// Reply reference indicating this is part of a thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

/// Rich text facet (mentions, links, hashtags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub index: ByteSlice,
    pub features: Vec<FacetFeature>,
}

/// Byte range for a facet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteSlice {
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Facet feature types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum FacetFeature {
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link { uri: String },
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention { did: String },
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag { tag: String },
}
