//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registered user
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub bluesky_did: String,
    pub bluesky_handle: String,
    pub created_at: DateTime<Utc>,
}

/// OAuth tokens for a user (encrypted at rest)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub readwise_token: String,
    pub bookmark_sync_enabled: bool,
    pub extract_links: bool,
    pub last_bookmark_cursor: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// A processed bookmark (for deduplication)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProcessedBookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_uri: String,
    pub processed_at: DateTime<Utc>,
}

/// A processed DM
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProcessedDm {
    pub id: Uuid,
    pub user_id: Uuid,
    pub message_id: String,
    pub post_uri: Option<String>,
    pub status: String,
    pub processed_at: DateTime<Utc>,
}
